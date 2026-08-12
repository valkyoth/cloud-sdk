use core::fmt;
use std::sync::Arc;

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, BoundCredentialTransport, CredentialBinding,
};
use cloud_sdk::transport::{
    AsyncResponseStaging, BoundTransport, EndpointIdentity, EndpointIdentityError,
    ResponseCompletion, ResponseStorageSanitizer, TransportFailure,
};
use cloud_sdk_sanitization::sanitize_bytes;

use crate::shared::{
    AuthenticatedTransportFailure, BasicCredential, HttpsEndpoint, TransportError,
    map_authentication_error, validate_basic_authentication,
};

use super::RawAsyncClient;

/// Hardened provider-neutral reqwest asynchronous Basic-auth transport.
#[derive(Clone)]
pub struct AsyncBasicClient {
    client: RawAsyncClient,
    endpoint: HttpsEndpoint,
    credential: Arc<BasicCredential>,
    allow_insecure_loopback: bool,
}

impl AsyncBasicClient {
    pub(super) fn new(
        client: RawAsyncClient,
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

    async fn send_inner<'writer, 'buffer>(
        &self,
        authenticated: AuthenticatedRequest<'_, '_>,
        response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, AuthenticatedTransportFailure> {
        let endpoint = self.endpoint.identity().map_err(|_| {
            TransportFailure::not_sent(TransportError::AuthenticationEndpointMismatch)
        })?;
        validate_basic_authentication(
            endpoint,
            self.credential.scope(),
            authenticated.policy(),
            self.allow_insecure_loopback,
        )
        .map_err(|error| TransportFailure::not_sent(map_authentication_error(error)))?;
        let authorization = self
            .credential
            .header_value()
            .map_err(|_| TransportFailure::not_sent(TransportError::HeaderRejected))?;
        self.client
            .execute_authenticated(
                authenticated.transport_request(),
                authenticated.response_policy(),
                authorization,
                response,
            )
            .await
            .map_err(|failure| failure.map(TransportError::RawHttp))
    }
}

impl AsyncAuthenticatedTransport for AsyncBasicClient {
    type Error = AuthenticatedTransportFailure;

    async fn send_authenticated<'transport, 'request, 'policy, 'writer, 'buffer>(
        &'transport self,
        request: AuthenticatedRequest<'request, 'policy>,
        response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
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

impl BoundCredentialTransport for AsyncBasicClient {
    fn credential_binding(&self) -> CredentialBinding {
        self.credential.binding()
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
