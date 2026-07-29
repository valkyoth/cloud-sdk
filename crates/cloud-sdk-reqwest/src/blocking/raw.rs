use core::fmt;

use cloud_sdk::transport::{
    BlockingRawHttpExecutor, BoundTransport, EndpointIdentity, EndpointIdentityError,
    RawResponsePolicy, ResponseStorageSanitizer, ResponseWriter, TransportFailure,
    TransportRequest,
};
use cloud_sdk_sanitization::sanitize_bytes;

use crate::shared::{HttpsEndpoint, RawHttpError, RawHyperClient, RawTransportFailure};

/// Raw blocking HTTP executor with no implicit authentication or provider policy.
#[derive(Clone)]
pub struct RawBlockingClient {
    inner: RawHyperClient,
    endpoint: HttpsEndpoint,
}

impl RawBlockingClient {
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
        runtime.block_on(self.inner.execute(request, policy, response_writer))
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

impl fmt::Debug for RawBlockingClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RawBlockingClient")
            .field("endpoint", &"[redacted]")
            .finish_non_exhaustive()
    }
}
