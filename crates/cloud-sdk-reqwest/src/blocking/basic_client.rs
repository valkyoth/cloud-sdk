use core::fmt;
use std::sync::Arc;

use cloud_sdk::authentication::{AuthenticatedRequest, BlockingAuthenticatedTransport};
use cloud_sdk::transport::{
    BoundTransport, EndpointIdentity, EndpointIdentityError, ResponseStorageSanitizer,
    ResponseWriter, TransportFailure,
};
use cloud_sdk_sanitization::sanitize_bytes;

use crate::shared::{
    AuthenticatedTransportFailure, BasicCredential, HttpsEndpoint, TransportError,
    map_authentication_error, validate_basic_authentication,
};

use super::RawBlockingClient;

/// Hardened provider-neutral reqwest blocking Basic-auth transport.
#[derive(Clone)]
pub struct BlockingBasicClient {
    client: RawBlockingClient,
    endpoint: HttpsEndpoint,
    credential: Arc<BasicCredential>,
    allow_insecure_loopback: bool,
}

impl BlockingBasicClient {
    pub(super) fn new(
        client: RawBlockingClient,
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

    fn send_inner(
        &self,
        authenticated: AuthenticatedRequest<'_, '_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), AuthenticatedTransportFailure> {
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
            .map_err(|failure| failure.map(TransportError::RawHttp))
    }
}

impl BlockingAuthenticatedTransport for BlockingBasicClient {
    type Error = AuthenticatedTransportFailure;

    fn send_authenticated(
        &self,
        request: AuthenticatedRequest<'_, '_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        self.send_inner(request, response)
    }
}

impl ResponseStorageSanitizer for BlockingBasicClient {
    fn sanitize_response_storage(&self, response_storage: &mut [u8]) {
        sanitize_bytes(response_storage);
    }
}

impl BoundTransport for BlockingBasicClient {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.endpoint.identity()
    }
}

impl fmt::Debug for BlockingBasicClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BlockingBasicClient")
            .field("endpoint", &"[redacted]")
            .field("credential", &"[redacted]")
            .finish_non_exhaustive()
    }
}
