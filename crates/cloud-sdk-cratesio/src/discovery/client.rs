use super::checked::CheckedGet;
use super::{DiscoveryError, DiscoveryRequest, DiscoveryResponse, MAX_DISCOVERY_BYTES};
use crate::{
    endpoint::OfficialCratesIoEndpoint,
    query::MAX_TARGET_BYTES,
    wire::{
        CratesIoWireError, IdentifyingUserAgent, JsonResponsePolicy, OfficialApiAttempt,
        OfficialApiGate, ScheduleError,
    },
};
use cloud_sdk::{
    Method,
    rate_limit::{RetryAfter, WallClockTimestamp},
    transport::{
        BoundTransport, BoundUserAgent, HeaderName, MediaType, RawResponsePolicy, RequestHeader,
        RequestHeaders, ResponseBuffer, ResponseMediaPolicy, StatusCode, TransportRequest,
    },
};
use core::fmt;

#[cfg(feature = "async")]
mod asynchronous;
#[cfg(all(test, feature = "blocking", feature = "async"))]
mod tests;

/// Anonymous client over a trusted, fixed-origin raw executor. Each call makes
/// exactly one exchange, with no credentials, redirects or retries. All client
/// instances share the process API gate. Callers coordinate separate processes.
pub struct DiscoveryClient<'a, T: ?Sized> {
    pub(crate) executor: &'a T,
    pub(crate) endpoint: OfficialCratesIoEndpoint,
    pub(crate) gate: OfficialApiGate<'a>,
    pub(crate) maximum: usize,
}
impl<T: ?Sized> fmt::Debug for DiscoveryClient<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DiscoveryClient([redacted])")
    }
}
impl<'a, T: BoundTransport + BoundUserAgent + ?Sized> DiscoveryClient<'a, T> {
    /// Selects the fixed production API and explicit identifying user agent.
    pub fn production(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum_response_bytes: usize,
    ) -> Result<Self, DiscoveryError> {
        Self::new(
            executor,
            identity,
            maximum_response_bytes,
            OfficialCratesIoEndpoint::production_api(),
        )
    }
    /// Selects the fixed staging API, without sharing credentials with production.
    pub fn staging(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum_response_bytes: usize,
    ) -> Result<Self, DiscoveryError> {
        Self::new(
            executor,
            identity,
            maximum_response_bytes,
            OfficialCratesIoEndpoint::staging_api(),
        )
    }
    fn new(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
        endpoint: OfficialCratesIoEndpoint,
    ) -> Result<Self, DiscoveryError> {
        if maximum == 0 || maximum > MAX_DISCOVERY_BYTES {
            return Err(DiscoveryError::Limit);
        }
        endpoint
            .verify_transport(executor)
            .map_err(|_| DiscoveryError::Binding)?;
        if executor.configured_user_agent() != identity.as_str().as_bytes() {
            return Err(DiscoveryError::Binding);
        }
        Ok(Self {
            executor,
            endpoint,
            gate: OfficialApiGate::new(identity),
            maximum,
        })
    }
    pub(crate) fn policy(&self) -> Result<RawResponsePolicy<'static>, DiscoveryError> {
        RawResponsePolicy::new(
            self.maximum,
            self.maximum,
            ResponseMediaPolicy::Required(&[MediaType::JSON]),
            ResponseMediaPolicy::Required(&[MediaType::JSON]),
            &[HeaderName::new("retry-after").map_err(|_| DiscoveryError::Value)?],
            8,
        )
        .map_err(|_| DiscoveryError::Value)
    }
    pub(crate) fn verify(&self) -> Result<(), DiscoveryError> {
        if self.executor.configured_user_agent() != self.gate.user_agent().as_str().as_bytes() {
            return Err(DiscoveryError::Binding);
        }
        self.endpoint
            .verify_transport(self.executor)
            .map_err(|_| DiscoveryError::Binding)
    }
    pub(crate) fn headers(&self) -> Result<[RequestHeader<'_>; 2], DiscoveryError> {
        Ok([
            RequestHeader::accept(MediaType::JSON),
            RequestHeader::new("accept-encoding", "identity").map_err(|_| DiscoveryError::Value)?,
        ])
    }
    pub(crate) fn decode<E, R: CheckedGet>(
        &self,
        request: R,
        response: ResponseBuffer<'_>,
        attempt: &mut OfficialApiAttempt,
    ) -> Result<R::Response, DiscoveryExecutionError<E>> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| DiscoveryExecutionError::Model(DiscoveryError::Value))?;
        let policy = JsonResponsePolicy::new(StatusCode::OK, self.maximum)
            .map_err(DiscoveryExecutionError::Wire)?;
        let result = policy.admit(response, WallClockTimestamp::new(now.as_secs()));
        let delay = match &result {
            Ok(success) => success.retry_after(),
            Err(CratesIoWireError::Provider(provider)) => provider.retry_after(),
            _ => None,
        };
        if let Some(delay) = delay {
            let seconds = match delay {
                RetryAfter::Delay(value) => value.get(),
                RetryAfter::HttpDate(date) => u64::try_from(date.epoch_seconds())
                    .unwrap_or(0)
                    .saturating_sub(now.as_secs()),
            };
            attempt
                .defer(core::time::Duration::from_secs(seconds))
                .map_err(DiscoveryExecutionError::Schedule)?;
        }
        request
            .decode(
                self.endpoint,
                result.map_err(DiscoveryExecutionError::Wire)?,
            )
            .map_err(DiscoveryExecutionError::Model)
    }
}

#[cfg(feature = "blocking")]
impl<T: BoundTransport + BoundUserAgent + cloud_sdk::transport::BlockingRawHttpExecutor + ?Sized>
    DiscoveryClient<'_, T>
{
    /// Executes one admitted anonymous GET and clears all response storage on
    /// success or failure. Schedule errors never sleep or dispatch a request.
    pub fn execute(
        &self,
        request: DiscoveryRequest<'_>,
        storage: &mut [u8],
        header_storage: &mut [u8],
    ) -> Result<DiscoveryResponse, DiscoveryExecutionError<T::Error>> {
        self.execute_get(request, storage, header_storage)
    }

    pub(crate) fn execute_get<R: CheckedGet>(
        &self,
        request: R,
        storage: &mut [u8],
        header_storage: &mut [u8],
    ) -> Result<R::Response, DiscoveryExecutionError<T::Error>> {
        let mut response = ResponseBuffer::new(storage, self.maximum, header_storage);
        request
            .anonymous()
            .map_err(DiscoveryExecutionError::Model)?;
        self.verify().map_err(DiscoveryExecutionError::Model)?;
        let mut target = [0; MAX_TARGET_BYTES];
        let target = request
            .target(&mut target)
            .map_err(|_| DiscoveryExecutionError::Model(DiscoveryError::Binding))?;
        let headers = self.headers().map_err(DiscoveryExecutionError::Model)?;
        let wire = TransportRequest::new(Method::Get, target.as_request_target()).with_headers(
            RequestHeaders::new(&headers)
                .map_err(|_| DiscoveryExecutionError::Model(DiscoveryError::Value))?,
        );
        let policy = self.policy().map_err(DiscoveryExecutionError::Model)?;
        let mut attempt = self
            .gate
            .begin()
            .map_err(DiscoveryExecutionError::Schedule)?;
        self.executor
            .execute(wire, policy, response.writer())
            .map_err(DiscoveryExecutionError::Transport)?;
        self.decode(request, response, &mut attempt)
    }
}

/// Payload-free error chain. The transport error is available by pattern
/// matching, but is never formatted or exposed through `Error::source`.
pub enum DiscoveryExecutionError<E> {
    /// No attempt was admitted.
    Schedule(ScheduleError),
    /// Trusted raw executor failure.
    Transport(E),
    /// Response staging failure.
    Staging,
    /// Shared response-wire rejection or provider error.
    Wire(CratesIoWireError),
    /// Model, endpoint or bounded storage failure.
    Model(DiscoveryError),
}
impl<E> fmt::Debug for DiscoveryExecutionError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Schedule(e) => f.debug_tuple("Schedule").field(e).finish(),
            Self::Wire(e) => f.debug_tuple("Wire").field(e).finish(),
            Self::Model(e) => f.debug_tuple("Model").field(e).finish(),
            Self::Staging => f.write_str("Staging"),
            Self::Transport(_) => f.write_str("Transport([redacted])"),
        }
    }
}
impl<E> fmt::Display for DiscoveryExecutionError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("crates.io discovery request failed")
    }
}
impl<E> core::error::Error for DiscoveryExecutionError<E> {}
