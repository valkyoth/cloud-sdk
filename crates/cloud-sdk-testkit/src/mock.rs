//! Deterministic no-allocation mock transport.

use core::fmt;
use core::sync::atomic::{AtomicUsize, Ordering};

use cloud_sdk::Method;
use cloud_sdk::transport::{
    AsyncTransport, BlockingTransport, BoundTransport, EndpointIdentity, EndpointIdentityError,
    HeaderSensitivity, RequestHeaders, RequestTarget, ResponseContentType, ResponseHeaders,
    ResponseStorageSanitizer, TransportRequest, TransportResponse,
};

use crate::{FixtureBodyError, ResponseFixture};

/// Expected request fields for one mock exchange.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ExpectedRequest<'a> {
    method: Method,
    target: RequestTarget<'a>,
    body: &'a [u8],
    headers: RequestHeaders<'a>,
}

impl<'a> ExpectedRequest<'a> {
    /// Creates a bodyless expected request.
    #[must_use]
    pub const fn new(method: Method, target: RequestTarget<'a>) -> Self {
        Self {
            method,
            target,
            body: &[],
            headers: RequestHeaders::EMPTY,
        }
    }

    /// Adds the exact expected request body.
    #[must_use]
    pub const fn with_body(mut self, body: &'a [u8]) -> Self {
        self.body = body;
        self
    }

    /// Adds the exact expected ordered request headers.
    #[must_use]
    pub const fn with_headers(mut self, headers: RequestHeaders<'a>) -> Self {
        self.headers = headers;
        self
    }

    const fn method(self) -> Method {
        self.method
    }

    const fn target(self) -> RequestTarget<'a> {
        self.target
    }

    const fn body(self) -> &'a [u8] {
        self.body
    }

    const fn headers(self) -> RequestHeaders<'a> {
        self.headers
    }
}

impl fmt::Debug for ExpectedRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ExpectedRequest")
            .field("method", &self.method)
            .field("target", &"[redacted]")
            .field("body", &"[redacted]")
            .field("headers", &self.headers)
            .finish()
    }
}

/// One expected request and deterministic response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MockExchange<'a> {
    request: ExpectedRequest<'a>,
    response: ResponseFixture<'a>,
}

impl<'a> MockExchange<'a> {
    /// Creates one mock exchange.
    #[must_use]
    pub const fn new(request: ExpectedRequest<'a>, response: ResponseFixture<'a>) -> Self {
        Self { request, response }
    }
}

/// Deterministic mock transport failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MockError {
    /// No expected exchange remains.
    Exhausted,
    /// HTTP method differs from the next expectation.
    MethodMismatch,
    /// Request target differs from the next expectation.
    TargetMismatch,
    /// Request body differs from the next expectation.
    BodyMismatch,
    /// Request headers differ from the next expectation.
    HeadersMismatch,
    /// Caller response buffer cannot hold the complete fixture body.
    ResponseBufferTooSmall,
    /// Internal cursor arithmetic failed closed.
    CursorOverflow,
    /// Another request changed the ordered cursor during this exchange.
    ConcurrentRequest,
    /// Fixture metadata could not be represented by the core transport.
    InvalidFixtureMetadata,
}

impl_static_error!(MockError,
    Self::Exhausted => "mock transport has no expected exchange remaining",
    Self::MethodMismatch => "mock request method differs from expectation",
    Self::TargetMismatch => "mock request target differs from expectation",
    Self::BodyMismatch => "mock request body differs from expectation",
    Self::HeadersMismatch => "mock request headers differ from expectation",
    Self::ResponseBufferTooSmall => "mock response buffer is too small",
    Self::CursorOverflow => "mock transport cursor overflowed",
    Self::ConcurrentRequest => "mock transport cursor changed concurrently",
    Self::InvalidFixtureMetadata => "mock fixture metadata is invalid",
);

/// Ordered no-allocation mock implementation of [`BlockingTransport`].
pub struct MockTransport<'a> {
    exchanges: &'a [MockExchange<'a>],
    cursor: AtomicUsize,
    endpoint: Option<EndpointIdentity<'a>>,
}

impl<'a> MockTransport<'a> {
    /// Creates a mock over an ordered exchange slice.
    #[must_use]
    pub const fn new(exchanges: &'a [MockExchange<'a>]) -> Self {
        Self {
            exchanges,
            cursor: AtomicUsize::new(0),
            endpoint: None,
        }
    }

    /// Binds the mock permanently to one normalized endpoint identity.
    #[must_use]
    pub const fn with_endpoint(mut self, endpoint: EndpointIdentity<'a>) -> Self {
        self.endpoint = Some(endpoint);
        self
    }

    /// Returns the number of exchanges not yet consumed.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.exchanges
            .len()
            .saturating_sub(self.cursor.load(Ordering::Acquire))
    }

    /// Reports whether every expected exchange was consumed.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.remaining() == 0
    }

    fn send_inner<'buffer>(
        &self,
        request: TransportRequest<'_>,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, MockError> {
        let cursor = self.cursor.load(Ordering::Acquire);
        let exchange = self.exchanges.get(cursor).ok_or(MockError::Exhausted)?;
        if request.method() != exchange.request.method() {
            return Err(MockError::MethodMismatch);
        }
        if request.target() != exchange.request.target() {
            return Err(MockError::TargetMismatch);
        }
        if request.body() != exchange.request.body() {
            return Err(MockError::BodyMismatch);
        }
        if request.headers() != exchange.request.headers() {
            return Err(MockError::HeadersMismatch);
        }
        let next_cursor = cursor.checked_add(1).ok_or(MockError::CursorOverflow)?;
        let content_type = exchange
            .response
            .content_type()
            .map(ResponseContentType::new)
            .transpose()
            .map_err(|_| MockError::InvalidFixtureMetadata)?;
        let rate_limit = exchange
            .response
            .rate_limit()
            .map(|value| value.into_rate_limit())
            .transpose()
            .map_err(|_| MockError::InvalidFixtureMetadata)?;
        let mut response_headers = exchange.response.headers();
        if let Some(value) = exchange.response.content_type() {
            response_headers
                .try_push("content-type", value.as_bytes(), HeaderSensitivity::Public)
                .map_err(|_| MockError::InvalidFixtureMetadata)?;
        }
        if let Some(value) = rate_limit {
            push_rate_limit_headers(&mut response_headers, value)
                .map_err(|_| MockError::InvalidFixtureMetadata)?;
        }
        let body_len =
            exchange
                .response
                .body()
                .write_to(response_body)
                .map_err(|error| match error {
                    FixtureBodyError::OutputTooSmall | FixtureBodyError::TooLarge => {
                        MockError::ResponseBufferTooSmall
                    }
                })?;
        let initialized = response_body
            .get(..body_len)
            .ok_or(MockError::ResponseBufferTooSmall)?;
        let response = TransportResponse::new(exchange.response.status(), initialized)
            .with_headers(response_headers);
        let response = content_type.map_or(response, |value| response.with_content_type(value));
        let response = rate_limit.map_or(response, |value| response.with_rate_limit(value));
        self.cursor
            .compare_exchange(cursor, next_cursor, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| MockError::ConcurrentRequest)?;
        Ok(response)
    }
}

fn push_rate_limit_headers(
    headers: &mut ResponseHeaders,
    rate_limit: cloud_sdk::rate_limit::RateLimit,
) -> Result<(), ()> {
    let mut storage = [0_u8; 20];
    for (name, value) in [
        ("ratelimit-limit", rate_limit.limit()),
        ("ratelimit-remaining", rate_limit.remaining()),
        ("ratelimit-reset", rate_limit.reset_epoch_seconds()),
    ] {
        let text = write_decimal(value, &mut storage).ok_or(())?;
        headers
            .try_push(name, text.as_bytes(), HeaderSensitivity::Public)
            .map_err(|_| ())?;
    }
    Ok(())
}

fn write_decimal(value: u64, output: &mut [u8; 20]) -> Option<&str> {
    let mut value = value;
    let mut cursor = output.len();
    loop {
        cursor = cursor.checked_sub(1)?;
        let digit = u8::try_from(value % 10).ok()?;
        *output.get_mut(cursor)? = b'0'.checked_add(digit)?;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    core::str::from_utf8(output.get(cursor..)?).ok()
}

impl BlockingTransport for MockTransport<'_> {
    type Error = MockError;

    fn send<'buffer>(
        &self,
        request: TransportRequest<'_>,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, Self::Error> {
        self.send_inner(request, response_body)
    }
}

impl AsyncTransport for MockTransport<'_> {
    type Error = MockError;

    async fn send<'transport, 'request, 'buffer>(
        &'transport self,
        request: TransportRequest<'request>,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, Self::Error>
    where
        'request: 'transport,
        'buffer: 'transport,
    {
        self.send_inner(request, response_body)
    }
}

impl ResponseStorageSanitizer for MockTransport<'_> {
    fn sanitize_response_storage(&self, response_storage: &mut [u8]) {
        response_storage.fill(0);
    }
}

impl BoundTransport for MockTransport<'_> {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.endpoint.ok_or(EndpointIdentityError::UnboundTransport)
    }
}

impl fmt::Debug for MockTransport<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MockTransport")
            .field("remaining", &self.remaining())
            .finish_non_exhaustive()
    }
}
