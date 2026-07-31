use core::fmt;
use std::sync::Arc;

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, CredentialGeneration,
};
use cloud_sdk::transport::{
    BoundTransport, EndpointIdentity, EndpointIdentityError, ResponseStorageSanitizer,
    ResponseWriter, TransportFailure,
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
            credentials: Arc::new(CredentialStore::new(credential.token)),
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

    /// Installs a refresh only if its captured generation is still current.
    pub fn refresh_bearer_token(
        &self,
        handoff: BearerRefreshHandoff,
        replacement: BearerToken,
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        self.credentials.refresh(handoff, replacement)
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

    async fn send_inner(
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
            .await
            .map_err(|failure| failure.map(TransportError::RawHttp))
    }
}

impl AsyncAuthenticatedTransport for AsyncClient {
    type Error = AuthenticatedTransportFailure;

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
