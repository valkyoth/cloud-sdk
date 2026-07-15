use core::fmt;
use std::sync::Arc;

use cloud_sdk::Method;
use cloud_sdk::transport::{
    AsyncTransport, BoundTransport, EndpointIdentity, EndpointIdentityError,
    ResponseStorageSanitizer, StatusCode, TransportRequest, TransportResponse,
};
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderValue};
use reqwest::{Body, Client};

use crate::shared::{
    BearerToken, CredentialStateError, CredentialStore, HttpsEndpoint, TokenRotationError,
    TransportError, parse_rate_limit, parse_response_content_type,
};

use super::body::SanitizedBuffer;

/// Hardened provider-neutral reqwest asynchronous transport.
///
/// The adapter uses reqwest's Tokio-based execution internally but does not
/// install or own a runtime. Callers must poll it from a compatible executor.
#[derive(Clone)]
pub struct AsyncClient {
    client: Client,
    endpoint: HttpsEndpoint,
    credentials: Arc<CredentialStore>,
}

impl AsyncClient {
    pub(super) fn new(client: Client, endpoint: HttpsEndpoint, token: BearerToken) -> Self {
        Self {
            client,
            endpoint,
            credentials: Arc::new(CredentialStore::new(token)),
        }
    }

    /// Atomically replaces the bearer token used by newly started requests.
    ///
    /// In-flight requests retain their previous snapshot. No credential lock
    /// is held across network I/O or `.await`.
    pub fn rotate_bearer_token(
        &self,
        replacement: BearerToken,
    ) -> Result<(), CredentialStateError> {
        self.credentials.rotate(replacement)
    }

    /// Validates and rotates from mutable bytes, clearing the complete source
    /// on success or failure. Rejected input leaves the active token unchanged.
    pub fn rotate_bearer_token_from_mut_bytes(
        &self,
        source: &mut [u8],
    ) -> Result<(), TokenRotationError> {
        self.credentials.rotate_from_mut_bytes(source)
    }

    /// Validates and rotates from guarded storage. Dropping the consumed guard
    /// clears the complete source on success or failure.
    pub fn rotate_bearer_token_from_secret_buffer(
        &self,
        source: SecretBuffer<'_>,
    ) -> Result<(), TokenRotationError> {
        self.credentials.rotate_from_secret_buffer(source)
    }

    async fn send_inner<'buffer>(
        &self,
        request: TransportRequest<'_>,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, TransportError> {
        sanitize_bytes(response_body);
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
            .request(map_method(request.method()), url)
            .header(AUTHORIZATION, authorization)
            .header(ACCEPT, HeaderValue::from_static("application/json"));

        if let Some(content_type) = request.content_type() {
            let value = HeaderValue::from_str(content_type.as_str())
                .map_err(|_| TransportError::HeaderRejected)?;
            outbound = outbound.header(CONTENT_TYPE, value);
        } else if !request.body().is_empty() {
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
            u64::try_from(response_body.len()).map_or(true, |cap| length > cap)
        }) {
            return Err(TransportError::ResponseTooLarge);
        }
        let status =
            StatusCode::new(response.status().as_u16()).ok_or(TransportError::InvalidStatus)?;
        let rate_limit = parse_rate_limit(response.headers())?;
        let content_type = parse_response_content_type(response.headers())?;
        let buffered = read_response(&mut response, response_body.len()).await?;
        let body_len = buffered.len();
        let initialized = response_body
            .get_mut(..body_len)
            .ok_or(TransportError::ResponseReadFailed)?;
        initialized.copy_from_slice(buffered.as_ref());
        let response = TransportResponse::new(status, initialized);
        let response = content_type.map_or(response, |value| response.with_content_type(value));
        drop(token_snapshot);
        Ok(rate_limit.map_or(response, |value| response.with_rate_limit(value)))
    }
}

impl AsyncTransport for AsyncClient {
    type Error = TransportError;

    async fn send<'transport, 'request, 'buffer>(
        &'transport self,
        request: TransportRequest<'request>,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, Self::Error>
    where
        'request: 'transport,
        'buffer: 'transport,
    {
        self.send_inner(request, response_body).await
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
            .field("credentials", &"[redacted]")
            .finish_non_exhaustive()
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

const fn map_method(method: Method) -> reqwest::Method {
    match method {
        Method::Get => reqwest::Method::GET,
        Method::Post => reqwest::Method::POST,
        Method::Put => reqwest::Method::PUT,
        Method::Delete => reqwest::Method::DELETE,
    }
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
