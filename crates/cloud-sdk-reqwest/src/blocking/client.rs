use core::fmt;
use std::io::Read;
use std::sync::Arc;

use cloud_sdk::Method;
use cloud_sdk::transport::{
    BlockingTransport, BoundTransport, EndpointIdentity, EndpointIdentityError, ResponseMetadata,
    ResponseStorageSanitizer, ResponseWriter, StatusCode, TransportRequest,
};
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};
use reqwest::blocking::{Body, Client};
use reqwest::header::{AUTHORIZATION, HeaderName, HeaderValue};

use super::body::{ReadBodyError, SanitizedRequestBody, read_bounded};
use crate::shared::{
    BearerToken, CredentialStateError, CredentialStore, HttpsEndpoint, TokenRotationError,
    TransportError, capture_response_headers, parse_rate_limit, parse_response_content_type,
};

/// Hardened provider-neutral reqwest blocking transport.
#[derive(Clone)]
pub struct BlockingClient {
    client: Client,
    endpoint: HttpsEndpoint,
    credentials: Arc<CredentialStore>,
}

impl BlockingClient {
    pub(super) fn new(client: Client, endpoint: HttpsEndpoint, token: BearerToken) -> Self {
        Self {
            client,
            endpoint,
            credentials: Arc::new(CredentialStore::new(token)),
        }
    }

    /// Atomically replaces the bearer token used by newly started requests.
    ///
    /// In-flight requests retain their previous snapshot. The retired token is
    /// sanitized after its last request snapshot is dropped.
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

    fn send_inner(
        &self,
        request: TransportRequest<'_>,
        response_writer: &mut ResponseWriter<'_>,
    ) -> Result<(), TransportError> {
        if response_writer.is_committed() {
            return Err(TransportError::ResponseCommitFailed);
        }
        let url = self
            .endpoint
            .compose(request.target())
            .map_err(|_| TransportError::TargetRejected)?;
        let method = map_method(request.method())?;
        let token_snapshot = self
            .credentials
            .snapshot()
            .map_err(|_| TransportError::CredentialStateUnavailable)?;
        let authorization = token_snapshot
            .header_value()
            .map_err(|_| TransportError::HeaderRejected)?;
        let mut outbound = self
            .client
            .request(method, url)
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
            let body = SanitizedRequestBody::new(request.body())
                .map_err(|_| TransportError::RequestBodyAllocationFailed)?;
            let body_len = u64::try_from(request.body().len())
                .map_err(|_| TransportError::RequestBodyTooLarge)?;
            outbound = outbound.body(Body::sized(body, body_len));
        }

        let mut response = outbound.send().map_err(classify_reqwest_error)?;
        self.endpoint
            .verify_origin(response.url())
            .map_err(|_| TransportError::ResponseOriginChanged)?;
        capture_response_headers(
            response.headers(),
            response_writer
                .headers_mut()
                .map_err(|_| TransportError::ResponseCommitFailed)?,
        )?;
        if response.content_length().is_some_and(|length| {
            u64::try_from(response_writer.body_capacity()).map_or(true, |cap| length > cap)
        }) {
            return Err(TransportError::ResponseTooLarge);
        }
        let status =
            StatusCode::new(response.status().as_u16()).ok_or(TransportError::InvalidStatus)?;
        let rate_limit = parse_rate_limit(response_writer.headers())?;
        parse_response_content_type(response_writer.headers())?;
        let body_len = read_response(
            &mut response,
            response_writer
                .body_mut()
                .map_err(|_| TransportError::ResponseCommitFailed)?,
        )?;
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

impl BlockingTransport for BlockingClient {
    type Error = TransportError;

    fn send(
        &self,
        request: TransportRequest<'_>,
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
            .field("credentials", &"[redacted]")
            .finish_non_exhaustive()
    }
}

fn read_response(response: &mut impl Read, output: &mut [u8]) -> Result<usize, TransportError> {
    match read_bounded(response, output) {
        Ok(len) => Ok(len),
        Err(error) => {
            sanitize_bytes(output);
            Err(match error {
                ReadBodyError::TooLarge => TransportError::ResponseTooLarge,
                ReadBodyError::ReadFailed => TransportError::ResponseReadFailed,
            })
        }
    }
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
