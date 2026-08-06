use core::fmt;
use std::sync::Arc;

use cloud_sdk::authentication::{
    AuthenticatedRequest, BlockingAuthenticatedTransport, CredentialGeneration, CredentialLifetime,
};
use cloud_sdk::transport::{
    BoundTransport, EndpointIdentity, EndpointIdentityError, ResponseStorageSanitizer,
    ResponseWriter, TransportFailure,
};
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};

use super::RawBlockingClient;
use crate::shared::{
    AuthenticatedTransportFailure, BearerCredential, BearerCredentialScope,
    BearerCredentialSnapshot, BearerRefreshHandoff, BearerToken, CredentialStateError,
    CredentialStore, CredentialUpdateError, HttpsEndpoint, TokenRefreshError, TokenRotationError,
    TransportError, map_authentication_error, validate_bearer_authentication,
};

/// Hardened provider-neutral reqwest blocking bearer transport.
#[derive(Clone)]
pub struct BlockingClient {
    client: RawBlockingClient,
    endpoint: HttpsEndpoint,
    scope: Arc<BearerCredentialScope>,
    credentials: Arc<CredentialStore>,
    allow_insecure_loopback: bool,
}

impl BlockingClient {
    pub(super) fn new(
        client: RawBlockingClient,
        endpoint: HttpsEndpoint,
        credential: BearerCredential,
        allow_insecure_loopback: bool,
    ) -> Self {
        Self {
            client,
            endpoint,
            scope: Arc::new(credential.scope),
            credentials: Arc::new(CredentialStore::new(credential.token, credential.lifetime)),
            allow_insecure_loopback,
        }
    }

    /// Captures the current generation without exposing token bytes.
    pub fn credential_snapshot(&self) -> Result<BearerCredentialSnapshot, CredentialStateError> {
        self.credentials.snapshot()
    }

    /// Atomically replaces the bearer token used by newly started requests.
    ///
    /// In-flight requests retain their previous snapshot. The immutable scope
    /// cannot change during rotation.
    pub fn rotate_bearer_token(
        &self,
        replacement: BearerToken,
    ) -> Result<CredentialGeneration, CredentialUpdateError> {
        self.credentials.rotate(replacement)
    }

    /// Atomically replaces an expiring token and its complete lifetime.
    pub fn rotate_bearer_token_with_lifetime(
        &self,
        replacement: BearerToken,
        lifetime: CredentialLifetime,
    ) -> Result<CredentialGeneration, CredentialUpdateError> {
        self.credentials.rotate_with_lifetime(replacement, lifetime)
    }

    /// Validates and rotates mutable bytes, clearing the complete source.
    pub fn rotate_bearer_token_from_mut_bytes(
        &self,
        source: &mut [u8],
    ) -> Result<CredentialGeneration, TokenRotationError> {
        self.credentials.rotate_from_mut_bytes(source)
    }

    /// Validates and rotates guarded storage, which clears on return.
    pub fn rotate_bearer_token_from_secret_buffer(
        &self,
        source: SecretBuffer<'_>,
    ) -> Result<CredentialGeneration, TokenRotationError> {
        self.credentials.rotate_from_secret_buffer(source)
    }

    /// Clears mutable input and atomically installs an expiring replacement.
    pub fn rotate_bearer_token_from_mut_bytes_with_lifetime(
        &self,
        source: &mut [u8],
        lifetime: CredentialLifetime,
    ) -> Result<CredentialGeneration, TokenRotationError> {
        self.credentials
            .rotate_from_mut_bytes_with_lifetime(source, lifetime)
    }

    /// Consumes guarded input and atomically installs an expiring replacement.
    pub fn rotate_bearer_token_from_secret_buffer_with_lifetime(
        &self,
        source: SecretBuffer<'_>,
        lifetime: CredentialLifetime,
    ) -> Result<CredentialGeneration, TokenRotationError> {
        self.credentials
            .rotate_from_secret_buffer_with_lifetime(source, lifetime)
    }

    /// Installs a refresh only if its captured generation is still current.
    pub fn refresh_bearer_token(
        &self,
        handoff: BearerRefreshHandoff,
        replacement: BearerToken,
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        self.credentials.refresh(handoff, replacement)
    }

    /// Installs an expiring refresh if its time-qualified handoff is current.
    pub fn refresh_bearer_token_with_lifetime(
        &self,
        handoff: BearerRefreshHandoff,
        replacement: BearerToken,
        lifetime: CredentialLifetime,
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        self.credentials
            .refresh_with_lifetime(handoff, replacement, lifetime)
    }

    /// Validates refreshed mutable bytes, clears them, and rejects stale work.
    pub fn refresh_bearer_token_from_mut_bytes(
        &self,
        handoff: BearerRefreshHandoff,
        source: &mut [u8],
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        self.credentials.refresh_from_mut_bytes(handoff, source)
    }

    /// Consumes guarded refreshed storage and rejects stale work.
    pub fn refresh_bearer_token_from_secret_buffer(
        &self,
        handoff: BearerRefreshHandoff,
        source: SecretBuffer<'_>,
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        self.credentials.refresh_from_secret_buffer(handoff, source)
    }

    /// Clears mutable refresh input and atomically installs its lifetime.
    pub fn refresh_bearer_token_from_mut_bytes_with_lifetime(
        &self,
        handoff: BearerRefreshHandoff,
        source: &mut [u8],
        lifetime: CredentialLifetime,
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        self.credentials
            .refresh_from_mut_bytes_with_lifetime(handoff, source, lifetime)
    }

    /// Consumes guarded refresh input and atomically installs its lifetime.
    pub fn refresh_bearer_token_from_secret_buffer_with_lifetime(
        &self,
        handoff: BearerRefreshHandoff,
        source: SecretBuffer<'_>,
        lifetime: CredentialLifetime,
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        self.credentials
            .refresh_from_secret_buffer_with_lifetime(handoff, source, lifetime)
    }

    fn send_inner(
        &self,
        authenticated: AuthenticatedRequest<'_, '_>,
        response_writer: &mut ResponseWriter<'_>,
    ) -> Result<(), AuthenticatedTransportFailure> {
        let endpoint_identity = self.endpoint.identity().map_err(|_| {
            TransportFailure::not_sent(TransportError::AuthenticationEndpointMismatch)
        })?;
        validate_bearer_authentication(
            endpoint_identity,
            &self.scope,
            authenticated.policy(),
            self.allow_insecure_loopback,
        )
        .map_err(|error| TransportFailure::not_sent(map_authentication_error(error)))?;
        let token_snapshot = self
            .credentials
            .snapshot()
            .map_err(|_| TransportFailure::not_sent(TransportError::CredentialStateUnavailable))?;
        let authorization = token_snapshot
            .header_value()
            .map_err(|_| TransportFailure::not_sent(TransportError::HeaderRejected))?;
        drop(token_snapshot);
        self.client
            .execute_authenticated(
                authenticated.transport_request(),
                authenticated.response_policy(),
                authorization,
                response_writer,
            )
            .map_err(|failure| failure.map(TransportError::RawHttp))
    }
}

impl BlockingAuthenticatedTransport for BlockingClient {
    type Error = AuthenticatedTransportFailure;

    fn send_authenticated(
        &self,
        request: AuthenticatedRequest<'_, '_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        self.send_inner(request, response)
    }
}

impl ResponseStorageSanitizer for BlockingClient {
    fn sanitize_response_storage(&self, response_storage: &mut [u8]) {
        sanitize_bytes(response_storage);
    }
}

impl BoundTransport for BlockingClient {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.endpoint.identity()
    }
}

impl fmt::Debug for BlockingClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BlockingClient")
            .field("endpoint", &"[redacted]")
            .field("scope", &"[redacted]")
            .field("credentials", &"[redacted]")
            .finish_non_exhaustive()
    }
}
