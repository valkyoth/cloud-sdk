use core::fmt;

use cloud_sdk::transport::{
    BlockingRawHttpExecutor, BoundTransport, EndpointIdentity, EndpointIdentityError,
    RawResponsePolicy, ResponseStorageSanitizer, ResponseWriter, TransportFailure,
    TransportRequest,
};
use cloud_sdk_sanitization::sanitize_bytes;
use http::header::HeaderValue;

use crate::shared::{HttpsEndpoint, RawHttpError, RawHyperClient, RawTransportFailure};

/// Raw blocking HTTP executor with no implicit authentication or provider policy.
#[derive(Clone)]
pub struct RawBlockingClient {
    inner: RawHyperClient,
    endpoint: HttpsEndpoint,
}

impl RawBlockingClient {
    /// Opens one anonymous finite GET and returns a live bounded source. The
    /// body is not preloaded; redirects and transformed responses are rejected.
    pub fn open_stream(
        &self,
        request: TransportRequest<'_>,
        maximum_body_bytes: u64,
    ) -> Result<super::BlockingStreamingResponse, RawTransportFailure> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(TransportFailure::not_sent(
                RawHttpError::BlockingRuntimeContext,
            ));
        }
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| TransportFailure::not_sent(RawHttpError::RuntimeInitializationFailed))?;
        let response = runtime.block_on(self.inner.open_stream(request, maximum_body_bytes))?;
        Ok(super::BlockingStreamingResponse {
            response,
            runtime: Some(runtime),
        })
    }

    pub(super) const fn new(inner: RawHyperClient, endpoint: HttpsEndpoint) -> Self {
        Self { inner, endpoint }
    }

    fn execute_inner(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        response_writer: &mut ResponseWriter<'_>,
    ) -> Result<(), RawTransportFailure> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(TransportFailure::not_sent(
                RawHttpError::BlockingRuntimeContext,
            ));
        }
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| TransportFailure::not_sent(RawHttpError::RuntimeInitializationFailed))?;
        let mut attempt = response_writer
            .begin_attempt()
            .map_err(|_| TransportFailure::not_sent(RawHttpError::ResponseAlreadyCommitted))?;
        let completion = runtime.block_on(self.inner.execute(request, policy, &mut attempt))?;
        let status = completion.status();
        attempt.commit_completion(completion).map_err(|_| {
            TransportFailure::response_started_with_status(
                status,
                RawHttpError::ResponseCommitFailed,
            )
        })
    }

    pub(crate) fn execute_authenticated(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        authorization: HeaderValue,
        response_writer: &mut ResponseWriter<'_>,
    ) -> Result<(), RawTransportFailure> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(TransportFailure::not_sent(
                RawHttpError::BlockingRuntimeContext,
            ));
        }
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| TransportFailure::not_sent(RawHttpError::RuntimeInitializationFailed))?;
        let mut attempt = response_writer
            .begin_attempt()
            .map_err(|_| TransportFailure::not_sent(RawHttpError::ResponseAlreadyCommitted))?;
        let completion = runtime.block_on(self.inner.execute_authenticated(
            request,
            policy,
            authorization,
            &mut attempt,
        ))?;
        let status = completion.status();
        attempt.commit_completion(completion).map_err(|_| {
            TransportFailure::response_started_with_status(
                status,
                RawHttpError::ResponseCommitFailed,
            )
        })
    }
}

impl BlockingRawHttpExecutor for RawBlockingClient {
    type Error = RawTransportFailure;

    fn execute(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        self.execute_inner(request, policy, response)
    }
}

impl cloud_sdk::transport::BlockingAuthorizedRawHttpExecutor for RawBlockingClient {
    fn execute_authorized(
        &self,
        expected: EndpointIdentity<'_>,
        authorization: cloud_sdk::transport::HeaderValue<'_>,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        if self.endpoint_identity().ok() != Some(expected) {
            return Err(TransportFailure::not_sent(RawHttpError::TargetRejected));
        }
        let authorization =
            crate::shared::sensitive_header_value(authorization.as_str().as_bytes())
                .map_err(|_| TransportFailure::not_sent(RawHttpError::HeaderRejected))?;
        self.execute_authenticated(request, policy, authorization, response)
    }
}

impl ResponseStorageSanitizer for RawBlockingClient {
    fn sanitize_response_storage(&self, response_storage: &mut [u8]) {
        sanitize_bytes(response_storage);
    }
}

impl BoundTransport for RawBlockingClient {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.endpoint.identity()
    }
}

impl cloud_sdk::transport::BoundUserAgent for RawBlockingClient {
    fn configured_user_agent(&self) -> &[u8] {
        self.inner.configured_user_agent()
    }
}

impl fmt::Debug for RawBlockingClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RawBlockingClient")
            .field("endpoint", &"[redacted]")
            .finish_non_exhaustive()
    }
}
