//! Allocation-free dynamic response selection for multi-request scenarios.

use core::fmt;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use cloud_sdk::Method;
use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, BlockingAuthenticatedTransport,
};
use cloud_sdk::transport::{
    AsyncResponseStaging, AsyncTransport, BlockingTransport, BoundTransport, EndpointIdentity,
    EndpointIdentityError, RequestHeaders, RequestTarget, ResponseCompletion, ResponseWriter,
    TransportRequest,
};

use crate::mock::{MockResponseSink, stage_response};
use crate::{MAX_DYNAMIC_RECORDS, MockError, RequestRecordSlot, ResponseFixture};

/// Read-only request view supplied to dynamic fixture builders.
#[derive(Clone, Copy)]
pub struct DynamicRequest<'request> {
    sequence: usize,
    request: TransportRequest<'request>,
}

impl<'request> DynamicRequest<'request> {
    const fn new(sequence: usize, request: TransportRequest<'request>) -> Self {
        Self { sequence, request }
    }

    /// Returns the zero-based successful scenario sequence.
    #[must_use]
    pub const fn sequence(self) -> usize {
        self.sequence
    }

    /// Returns the request method.
    #[must_use]
    pub const fn method(self) -> Method {
        self.request.method()
    }

    /// Returns the validated request target.
    #[must_use]
    pub const fn target(self) -> RequestTarget<'request> {
        self.request.target()
    }

    /// Returns the borrowed request body.
    #[must_use]
    pub const fn body(self) -> &'request [u8] {
        self.request.body()
    }

    /// Returns ordered request headers.
    #[must_use]
    pub const fn headers(self) -> RequestHeaders<'request> {
        self.request.headers()
    }
}

impl fmt::Debug for DynamicRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DynamicRequest")
            .field("sequence", &self.sequence)
            .field("method", &self.request.method())
            .field("target", &"[redacted]")
            .field("body", &"[redacted]")
            .field("headers", &self.request.headers())
            .finish()
    }
}

/// Provider-neutral contract for choosing one deterministic response fixture.
pub trait ProviderFixtureBuilder<'fixture> {
    /// Builder-specific mismatch or fixture-selection failure.
    type Error;

    /// Selects the response for one request without consuming scenario state.
    fn build<'request>(
        &self,
        request: DynamicRequest<'request>,
    ) -> Result<&'fixture ResponseFixture<'fixture>, Self::Error>;
}

/// Closure adapter for [`ProviderFixtureBuilder`].
pub struct DynamicResponder<F> {
    responder: F,
}

impl<F> DynamicResponder<F> {
    /// Wraps a closure as a dynamic fixture builder.
    #[must_use]
    pub const fn new(responder: F) -> Self {
        Self { responder }
    }
}

impl<'fixture, E, F> ProviderFixtureBuilder<'fixture> for DynamicResponder<F>
where
    F: for<'request> Fn(DynamicRequest<'request>) -> Result<&'fixture ResponseFixture<'fixture>, E>,
{
    type Error = E;

    fn build<'request>(
        &self,
        request: DynamicRequest<'request>,
    ) -> Result<&'fixture ResponseFixture<'fixture>, Self::Error> {
        (self.responder)(request)
    }
}

/// Invalid dynamic mock configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DynamicMockConfigError {
    /// At least one caller-owned record slot is required.
    NoRecordSlots,
    /// The configured recording capacity exceeds the hard bound.
    TooManyRecordSlots,
    /// Record slots must be empty when attached to a scenario.
    DirtyRecordSlot,
}

impl_static_error!(DynamicMockConfigError,
    Self::NoRecordSlots => "dynamic mock requires at least one record slot",
    Self::TooManyRecordSlots => "dynamic mock recording capacity exceeds the limit",
    Self::DirtyRecordSlot => "dynamic mock record slots are not empty",
);

/// Dynamic mock failure. Builder details are never rendered by `Debug` or `Display`.
pub enum DynamicMockError<E> {
    /// The caller-owned bounded record capacity is exhausted.
    Exhausted,
    /// A request is already selecting or staging the current scenario step.
    ConcurrentRequest,
    /// The provider fixture builder rejected the request.
    Builder(E),
    /// The selected fixture could not be staged.
    Fixture(MockError),
    /// Internal successful-sequence arithmetic failed closed.
    CursorOverflow,
}

impl<E> fmt::Debug for DynamicMockError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Exhausted => "DynamicMockError::Exhausted",
            Self::ConcurrentRequest => "DynamicMockError::ConcurrentRequest",
            Self::Builder(_) => "DynamicMockError::Builder([redacted])",
            Self::Fixture(_) => "DynamicMockError::Fixture([redacted])",
            Self::CursorOverflow => "DynamicMockError::CursorOverflow",
        })
    }
}

impl<E> fmt::Display for DynamicMockError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Exhausted => "dynamic mock recording capacity is exhausted",
            Self::ConcurrentRequest => "dynamic mock request overlaps another request",
            Self::Builder(_) => "dynamic fixture builder rejected the request",
            Self::Fixture(_) => "dynamic response fixture could not be staged",
            Self::CursorOverflow => "dynamic mock cursor overflowed",
        })
    }
}

impl<E> core::error::Error for DynamicMockError<E> {}

/// Bounded allocation-free dynamic mock transport.
pub struct DynamicMockTransport<'fixture, 'records, B> {
    builder: B,
    records: &'records [RequestRecordSlot],
    cursor: AtomicUsize,
    in_flight: AtomicBool,
    endpoint: Option<EndpointIdentity<'fixture>>,
}

impl<'fixture, 'records, B> DynamicMockTransport<'fixture, 'records, B> {
    /// Creates a dynamic mock over clean caller-owned record slots.
    pub fn new(
        builder: B,
        records: &'records [RequestRecordSlot],
    ) -> Result<Self, DynamicMockConfigError> {
        if records.is_empty() {
            return Err(DynamicMockConfigError::NoRecordSlots);
        }
        if records.len() > MAX_DYNAMIC_RECORDS {
            return Err(DynamicMockConfigError::TooManyRecordSlots);
        }
        if records.iter().any(|slot| !slot.is_empty()) {
            return Err(DynamicMockConfigError::DirtyRecordSlot);
        }
        Ok(Self {
            builder,
            records,
            cursor: AtomicUsize::new(0),
            in_flight: AtomicBool::new(false),
            endpoint: None,
        })
    }

    /// Binds the mock permanently to one normalized endpoint identity.
    #[must_use]
    pub const fn with_endpoint(mut self, endpoint: EndpointIdentity<'fixture>) -> Self {
        self.endpoint = Some(endpoint);
        self
    }

    /// Returns the number of committed successful requests.
    #[must_use]
    pub fn recorded(&self) -> usize {
        self.cursor.load(Ordering::Acquire)
    }

    /// Returns the fixed caller-owned recording capacity.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.records.len()
    }

    /// Returns one committed payload-free observation.
    #[must_use]
    pub fn record(&self, index: usize) -> Option<crate::RecordedRequest> {
        self.records.get(index).and_then(RequestRecordSlot::get)
    }
}

struct InFlightGuard<'a>(&'a AtomicBool);

impl Drop for InFlightGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl<'fixture, B> DynamicMockTransport<'fixture, '_, B>
where
    B: ProviderFixtureBuilder<'fixture>,
{
    fn stage_inner<'buffer>(
        &self,
        request: TransportRequest<'_>,
        response: &mut impl MockResponseSink<'buffer>,
    ) -> Result<ResponseCompletion, DynamicMockError<B::Error>> {
        self.in_flight
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| DynamicMockError::ConcurrentRequest)?;
        let _guard = InFlightGuard(&self.in_flight);
        let sequence = self.cursor.load(Ordering::Acquire);
        let slot = self
            .records
            .get(sequence)
            .ok_or(DynamicMockError::Exhausted)?;
        let fixture = self
            .builder
            .build(DynamicRequest::new(sequence, request))
            .map_err(DynamicMockError::Builder)?;
        let completion = stage_response(fixture, response).map_err(DynamicMockError::Fixture)?;
        let next = sequence
            .checked_add(1)
            .ok_or(DynamicMockError::CursorOverflow)?;
        slot.commit(sequence, request, fixture.status());
        self.cursor.store(next, Ordering::Release);
        Ok(completion)
    }

    fn send_inner(
        &self,
        request: TransportRequest<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), DynamicMockError<B::Error>> {
        if response.is_committed() {
            return Err(DynamicMockError::Fixture(MockError::ResponseWriterRejected));
        }
        let mut attempt = response
            .begin_attempt()
            .map_err(|_| DynamicMockError::Fixture(MockError::ResponseWriterRejected))?;
        let completion = self.stage_inner(request, &mut attempt)?;
        attempt
            .commit_completion(completion)
            .map_err(|_| DynamicMockError::Fixture(MockError::ResponseWriterRejected))
    }
}

impl<'fixture, B> BlockingTransport for DynamicMockTransport<'fixture, '_, B>
where
    B: ProviderFixtureBuilder<'fixture>,
{
    type Error = DynamicMockError<B::Error>;

    fn send(
        &self,
        request: TransportRequest<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        self.send_inner(request, response)
    }
}

impl<'fixture, B> BlockingAuthenticatedTransport for DynamicMockTransport<'fixture, '_, B>
where
    B: ProviderFixtureBuilder<'fixture>,
{
    type Error = DynamicMockError<B::Error>;

    fn send_authenticated(
        &self,
        request: AuthenticatedRequest<'_, '_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        self.send_inner(request.transport_request(), response)
    }
}

impl<'fixture, B> AsyncTransport for DynamicMockTransport<'fixture, '_, B>
where
    B: ProviderFixtureBuilder<'fixture> + Sync,
{
    type Error = DynamicMockError<B::Error>;

    async fn send<'transport, 'request, 'writer, 'buffer>(
        &'transport self,
        request: TransportRequest<'request>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'buffer: 'writer,
    {
        self.stage_inner(request, &mut response)
    }
}

impl<'fixture, B> AsyncAuthenticatedTransport for DynamicMockTransport<'fixture, '_, B>
where
    B: ProviderFixtureBuilder<'fixture> + Sync,
{
    type Error = DynamicMockError<B::Error>;

    async fn send_authenticated<'transport, 'request, 'policy, 'writer, 'buffer>(
        &'transport self,
        request: AuthenticatedRequest<'request, 'policy>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
    {
        self.stage_inner(request.transport_request(), &mut response)
    }
}

impl<B> BoundTransport for DynamicMockTransport<'_, '_, B> {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.endpoint.ok_or(EndpointIdentityError::UnboundTransport)
    }
}

impl<B> fmt::Debug for DynamicMockTransport<'_, '_, B> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DynamicMockTransport")
            .field("recorded", &self.recorded())
            .field("capacity", &self.capacity())
            .finish_non_exhaustive()
    }
}
