use core::fmt;
use std::sync::Arc;

use cloud_sdk::authentication::{AsyncAuthenticatedTransport, AuthenticatedRequest};
use cloud_sdk::transport::{
    BoundTransport, EndpointIdentity, EndpointIdentityError, ResponseStorageSanitizer,
    ResponseWriter,
};
use cloud_sdk_sanitization::sanitize_bytes;
use reqwest::Client;

use crate::shared::{
    BasicCredential, HttpsEndpoint, TransportError, map_authentication_error,
    validate_basic_authentication,
};

use super::client::execute;

/// Hardened provider-neutral reqwest asynchronous Basic-auth transport.
#[derive(Clone)]
pub struct AsyncBasicClient {
    client: Client,
    endpoint: HttpsEndpoint,
    credential: Arc<BasicCredential>,
    allow_insecure_loopback: bool,
}

impl AsyncBasicClient {
    pub(super) fn new(
        client: Client,
        endpoint: HttpsEndpoint,
        credential: BasicCredential,
        allow_insecure_loopback: bool,
    ) -> Self {
        Self {
            client,
            endpoint,
            credential: Arc::new(credential),
            allow_insecure_loopback,
        }
    }

    async fn send_inner(
        &self,
        authenticated: AuthenticatedRequest<'_, '_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), TransportError> {
        let endpoint = self
            .endpoint
            .identity()
            .map_err(|_| TransportError::AuthenticationEndpointMismatch)?;
        validate_basic_authentication(
            endpoint,
            self.credential.scope(),
            authenticated.policy(),
            self.allow_insecure_loopback,
        )
        .map_err(map_authentication_error)?;
        let authorization = self
            .credential
            .header_value()
            .map_err(|_| TransportError::HeaderRejected)?;
        execute(
            &self.client,
            &self.endpoint,
            authorization,
            authenticated,
            response,
        )
        .await
    }
}

impl AsyncAuthenticatedTransport for AsyncBasicClient {
    type Error = TransportError;

    async fn send_authenticated<'transport, 'request, 'policy, 'writer>(
        &'transport self,
        request: AuthenticatedRequest<'request, 'policy>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
    {
        self.send_inner(request, response).await
    }
}

impl ResponseStorageSanitizer for AsyncBasicClient {
    fn sanitize_response_storage(&self, response_storage: &mut [u8]) {
        sanitize_bytes(response_storage);
    }
}

impl BoundTransport for AsyncBasicClient {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.endpoint.identity()
    }
}

impl fmt::Debug for AsyncBasicClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AsyncBasicClient")
            .field("endpoint", &"[redacted]")
            .field("credential", &"[redacted]")
            .finish_non_exhaustive()
    }
}
