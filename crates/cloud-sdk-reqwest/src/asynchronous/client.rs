use core::fmt;
use std::sync::Arc;

use cloud_sdk::Method;
use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, CredentialGeneration, RefreshHandoff,
};
use cloud_sdk::transport::{
    BoundTransport, EndpointIdentity, EndpointIdentityError, ResponseMetadata,
    ResponseStorageSanitizer, ResponseWriter, StatusCode,
};
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};
use reqwest::header::{AUTHORIZATION, HeaderName, HeaderValue};
use reqwest::{Body, Client};

use crate::shared::{
    AuthenticationValidationError, BearerCredential, BearerCredentialScope,
    BearerCredentialSnapshot, BearerToken, CredentialStateError, CredentialStore,
    CredentialUpdateError, HttpsEndpoint, TokenRefreshError, TokenRotationError, TransportError,
    capture_response_headers, parse_rate_limit, parse_response_content_type,
    validate_bearer_authentication,
};

use super::body::SanitizedBuffer;

/// Hardened provider-neutral reqwest asynchronous bearer transport.
///
/// The adapter uses reqwest's Tokio-based execution internally but does not
/// install or own a runtime. Callers must poll it from a compatible executor.
#[derive(Clone)]
pub struct AsyncClient {
    client: Client,
    endpoint: HttpsEndpoint,
    scope: Arc<BearerCredentialScope>,
    credentials: Arc<CredentialStore>,
    allow_insecure_loopback: bool,
}

impl AsyncClient {
    pub(super) fn new(
        client: Client,
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
        handoff: RefreshHandoff,
        replacement: BearerToken,
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        self.credentials.refresh(handoff, replacement)
    }

    /// Validates refreshed mutable bytes, clears them, and rejects stale work.
    pub fn refresh_bearer_token_from_mut_bytes(
        &self,
        handoff: RefreshHandoff,
        source: &mut [u8],
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        self.credentials.refresh_from_mut_bytes(handoff, source)
    }

    /// Consumes guarded refreshed storage and rejects stale work.
    pub fn refresh_bearer_token_from_secret_buffer(
        &self,
        handoff: RefreshHandoff,
        source: SecretBuffer<'_>,
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        self.credentials.refresh_from_secret_buffer(handoff, source)
    }

    async fn send_inner(
        &self,
        authenticated: AuthenticatedRequest<'_, '_>,
        response_writer: &mut ResponseWriter<'_>,
    ) -> Result<(), TransportError> {
        if response_writer.is_committed() {
            return Err(TransportError::ResponseCommitFailed);
        }
        let mut response_writer = response_writer
            .begin_attempt()
            .map_err(|_| TransportError::ResponseCommitFailed)?;
        let endpoint_identity = self
            .endpoint
            .identity()
            .map_err(|_| TransportError::AuthenticationEndpointMismatch)?;
        validate_bearer_authentication(
            endpoint_identity,
            &self.scope,
            authenticated.policy(),
            self.allow_insecure_loopback,
        )
        .map_err(map_authentication_error)?;
        let request = authenticated.transport_request();
        let url = self
            .endpoint
            .compose(request.target())
            .map_err(|_| TransportError::TargetRejected)?;
        let token_snapshot = self
            .credentials
            .snapshot()
            .map_err(|_| TransportError::CredentialStateUnavailable)?;
        let authorization = token_snapshot
            .header_value()
            .map_err(|_| TransportError::HeaderRejected)?;
        let mut outbound = self
            .client
            .request(map_method(request.method())?, url)
            .header(AUTHORIZATION, authorization);

        for header in request.headers().as_slice() {
            let name = HeaderName::from_bytes(header.name().as_str().as_bytes())
                .map_err(|_| TransportError::HeaderRejected)?;
            let mut value = HeaderValue::from_str(header.value().as_str())
                .map_err(|_| TransportError::HeaderRejected)?;
            value.set_sensitive(matches!(
                header.sensitivity(),
                cloud_sdk::transport::HeaderSensitivity::Sensitive
            ));
            outbound = outbound.header(name, value);
        }
        if !request.body().is_empty() && request.headers().get("content-type").is_none() {
            return Err(TransportError::MissingContentType);
        }
        if !request.body().is_empty() {
            let body = SanitizedBuffer::copy_from(request.body())
                .map_err(|_| TransportError::RequestBodyAllocationFailed)?;
            let _ = u64::try_from(request.body().len())
                .map_err(|_| TransportError::RequestBodyTooLarge)?;
            outbound = outbound.body(Body::from(body.into_bytes()));
        }

        let mut response = outbound.send().await.map_err(classify_reqwest_error)?;
        self.endpoint
            .verify_origin(response.url())
            .map_err(|_| TransportError::ResponseOriginChanged)?;
        if response.content_length().is_some_and(|length| {
            u64::try_from(response_writer.body_capacity()).map_or(true, |cap| length > cap)
        }) {
            return Err(TransportError::ResponseTooLarge);
        }
        let status =
            StatusCode::new(response.status().as_u16()).ok_or(TransportError::InvalidStatus)?;
        let buffered = read_response(&mut response, response_writer.body_capacity()).await?;
        capture_response_headers(
            response.headers(),
            response_writer
                .headers_mut()
                .map_err(|_| TransportError::ResponseCommitFailed)?,
        )?;
        let rate_limit = parse_rate_limit(response_writer.headers())?;
        parse_response_content_type(response_writer.headers())?;
        let body_len = buffered.len();
        let initialized = response_writer
            .body_mut()
            .map_err(|_| TransportError::ResponseCommitFailed)?
            .get_mut(..body_len)
            .ok_or(TransportError::ResponseReadFailed)?;
        initialized.copy_from_slice(buffered.as_ref());
        let mut metadata = ResponseMetadata::EMPTY;
        if let Some(value) = rate_limit {
            metadata = metadata.with_rate_limit(value);
        }
        drop(token_snapshot);
        response_writer
            .commit(status, body_len, metadata)
            .map_err(|_| TransportError::ResponseCommitFailed)
    }
}

impl AsyncAuthenticatedTransport for AsyncClient {
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

fn map_authentication_error(error: AuthenticationValidationError) -> TransportError {
    match error {
        AuthenticationValidationError::InsecureEndpoint => {
            TransportError::InsecureAuthenticationEndpoint
        }
        AuthenticationValidationError::EndpointMismatch => {
            TransportError::AuthenticationEndpointMismatch
        }
        AuthenticationValidationError::ScopeRejected => TransportError::AuthenticationScopeRejected,
    }
}

async fn read_response(
    response: &mut reqwest::Response,
    limit: usize,
) -> Result<SanitizedBuffer, TransportError> {
    let mut buffered = SanitizedBuffer::with_capacity(limit)
        .map_err(|_| TransportError::ResponseBodyAllocationFailed)?;
    loop {
        let chunk = response
            .chunk()
            .await
            .map_err(|_| TransportError::ResponseReadFailed)?;
        let Some(chunk) = chunk else { break };
        buffered
            .extend_bounded(&chunk, limit)
            .map_err(|_| TransportError::ResponseTooLarge)?;
    }
    Ok(buffered)
}

fn map_method(method: Method) -> Result<reqwest::Method, TransportError> {
    reqwest::Method::from_bytes(method.as_str().as_bytes())
        .map_err(|_| TransportError::MethodRejected)
}

fn classify_reqwest_error(error: reqwest::Error) -> TransportError {
    if error.is_timeout() {
        TransportError::TimedOut
    } else if error.is_connect() {
        TransportError::ConnectFailed
    } else {
        TransportError::RequestFailed
    }
}
