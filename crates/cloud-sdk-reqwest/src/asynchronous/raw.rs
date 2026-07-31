use core::fmt;

use cloud_sdk::transport::{
    AsyncRawHttpExecutor, BoundTransport, EndpointIdentity, EndpointIdentityError,
    RawResponsePolicy, ResponseStorageSanitizer, ResponseWriter, TransportRequest,
};
use cloud_sdk_sanitization::sanitize_bytes;
use http::header::HeaderValue;

use crate::shared::{HttpsEndpoint, RawHyperClient, RawTransportFailure};

/// Raw async HTTP executor with no implicit authentication or provider policy.
#[derive(Clone)]
pub struct RawAsyncClient {
    inner: RawHyperClient,
    endpoint: HttpsEndpoint,
}

impl RawAsyncClient {
    pub(super) const fn new(inner: RawHyperClient, endpoint: HttpsEndpoint) -> Self {
        Self { inner, endpoint }
    }

    pub(crate) async fn execute_authenticated(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        authorization: HeaderValue,
        response_writer: &mut ResponseWriter<'_>,
    ) -> Result<(), RawTransportFailure> {
        self.inner
            .execute_authenticated(request, policy, authorization, response_writer)
            .await
    }
}

impl AsyncRawHttpExecutor for RawAsyncClient {
    type Error = RawTransportFailure;

    async fn execute<'executor, 'request, 'policy, 'writer>(
        &'executor self,
        request: TransportRequest<'request>,
        policy: RawResponsePolicy<'policy>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>
    where
        'executor: 'writer,
        'request: 'writer,
        'policy: 'writer,
    {
        self.inner.execute(request, policy, response).await
    }
}

impl ResponseStorageSanitizer for RawAsyncClient {
    fn sanitize_response_storage(&self, response_storage: &mut [u8]) {
        sanitize_bytes(response_storage);
    }
}

impl BoundTransport for RawAsyncClient {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.endpoint.identity()
    }
}

impl fmt::Debug for RawAsyncClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RawAsyncClient")
            .field("endpoint", &"[redacted]")
            .finish_non_exhaustive()
    }
}
