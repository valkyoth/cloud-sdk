use core::fmt;
use std::sync::Arc;

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, CredentialGeneration, CredentialLifetime,
};
use cloud_sdk::transport::{
    AsyncResponseStaging, BoundTransport, EndpointIdentity, EndpointIdentityError,
    ResponseCompletion, ResponseStorageSanitizer, TransportFailure,
};
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};

use crate::shared::{
    AuthenticatedTransportFailure, BearerCredential, BearerCredentialScope,
    BearerCredentialSnapshot, BearerRefreshHandoff, BearerToken, CredentialStateError,
    CredentialStore, CredentialUpdateError, HttpsEndpoint, TokenRefreshError, TokenRotationError,
    TransportError, map_authentication_error, validate_bearer_authentication,
};

use super::RawAsyncClient;

/// Hardened provider-neutral reqwest asynchronous bearer transport.
///
/// The adapter uses reqwest's Tokio-based execution internally but does not
/// install or own a runtime. Callers must poll it from a compatible executor.
#[derive(Clone)]
pub struct AsyncClient {
    client: RawAsyncClient,
    endpoint: HttpsEndpoint,
    scope: Arc<BearerCredentialScope>,
    credentials: Arc<CredentialStore>,
    allow_insecure_loopback: bool,
}

impl AsyncClient {
    pub(super) fn new(
        client: RawAsyncClient,
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

    /// Atomically replaces the token while retaining immutable scope.
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

    async fn send_inner<'writer, 'buffer>(
        &self,
        authenticated: AuthenticatedRequest<'_, '_>,
        response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, AuthenticatedTransportFailure> {
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
                response,
            )
            .await
            .map_err(|failure| failure.map(TransportError::RawHttp))
    }
}

impl AsyncAuthenticatedTransport for AsyncClient {
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

impl ResponseStorageSanitizer for AsyncClient {
    fn sanitize_response_storage(&self, response_storage: &mut [u8]) {
        sanitize_bytes(response_storage);
    }
}

impl BoundTransport for AsyncClient {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.endpoint.identity()
    }
}

impl fmt::Debug for AsyncClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AsyncClient")
            .field("endpoint", &"[redacted]")
            .field("scope", &"[redacted]")
            .field("credentials", &"[redacted]")
            .finish_non_exhaustive()
    }
}
