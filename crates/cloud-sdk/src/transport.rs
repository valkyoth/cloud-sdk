//! Provider-neutral blocking and asynchronous transport contracts.

mod asynchronous;
mod content_type;
mod endpoint;
mod header;
mod request_target;
mod response;

pub use asynchronous::AsyncTransport;
pub use content_type::{
    ContentType, ContentTypeError, MAX_CONTENT_TYPE_BYTES, MediaType, ResponseContentType,
};
pub use endpoint::{
    AcknowledgedCustomEndpoint, BoundTransport, CustomEndpointAcknowledgement, EndpointIdentity,
    EndpointIdentityError, EndpointPolicy, EndpointPolicyError, EndpointPolicyKind, EndpointScheme,
    MAX_ENDPOINT_BASE_PATH_BYTES, MAX_ENDPOINT_HOST_BYTES, MAX_ENDPOINT_REGION_BYTES,
    MAX_OFFICIAL_ENDPOINTS, RegionEndpoint,
};
pub use header::{
    HeaderError, HeaderName, HeaderSensitivity, HeaderValue, MAX_HEADER_NAME_BYTES,
    MAX_HEADER_VALUE_BYTES, MAX_REQUEST_HEADER_BYTES, MAX_REQUEST_HEADERS,
    MAX_RESPONSE_HEADER_BYTES, MAX_RESPONSE_HEADERS, RequestHeader, RequestHeaders, ResponseHeader,
    ResponseHeaders,
};
pub use request_target::{
    CanonicalQuery, FormQuery, MAX_REQUEST_TARGET_BYTES, QueryPair, QueryPairs, RequestPath,
    RequestPathError, RequestQuery, RequestTarget, RequestTargetError, StructuredQueryError,
};
pub use response::{
    ResponseBuffer, ResponseMetadata, ResponseWriter, ResponseWriterError, TransportResponse,
};

use core::fmt;

use crate::Method;

/// Explicit cleanup contract for caller-owned response storage.
///
/// Prepared execution invokes this for the complete supplied buffer before
/// endpoint verification or response-capacity admission. Production
/// implementations must use a cleanup primitive that cannot be removed as a
/// dead store. This remains separate from [`BlockingTransport`] and
/// [`AsyncTransport`] so direct transport implementations cannot silently
/// acquire a weaker cleanup promise.
pub trait ResponseStorageSanitizer {
    /// Clears the complete caller-owned response buffer.
    fn sanitize_response_storage(&self, response_storage: &mut [u8]);
}

/// Provider-neutral request passed to a blocking transport.
#[derive(Clone, Copy)]
pub struct TransportRequest<'a> {
    method: Method,
    target: RequestTarget<'a>,
    body: &'a [u8],
    headers: RequestHeaders<'a>,
}

impl<'a> TransportRequest<'a> {
    /// Creates a bodyless request.
    #[must_use]
    pub const fn new(method: Method, target: RequestTarget<'a>) -> Self {
        Self {
            method,
            target,
            body: &[],
            headers: RequestHeaders::EMPTY,
        }
    }

    /// Adds a borrowed request body.
    #[must_use]
    pub const fn with_body(mut self, body: &'a [u8]) -> Self {
        self.body = body;
        self
    }

    /// Adds a complete validated request-header block.
    #[must_use]
    pub const fn with_headers(mut self, headers: RequestHeaders<'a>) -> Self {
        self.headers = headers;
        self
    }

    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        self.method
    }

    /// Returns the validated origin-form target.
    #[must_use]
    pub const fn target(self) -> RequestTarget<'a> {
        self.target
    }

    /// Returns the borrowed body bytes.
    #[must_use]
    pub const fn body(self) -> &'a [u8] {
        self.body
    }

    /// Returns the complete ordered request-header block.
    #[must_use]
    pub const fn headers(self) -> RequestHeaders<'a> {
        self.headers
    }
}

impl fmt::Debug for TransportRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportRequest")
            .field("method", &self.method)
            .field("target", &self.target)
            .field("body", &"[redacted]")
            .field("headers", &self.headers)
            .finish()
    }
}

/// Valid HTTP response status code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StatusCode(u16);

impl StatusCode {
    /// `200 OK`.
    pub const OK: Self = Self(200);
    /// `201 Created`.
    pub const CREATED: Self = Self(201);
    /// `202 Accepted`.
    pub const ACCEPTED: Self = Self(202);
    /// `204 No Content`.
    pub const NO_CONTENT: Self = Self(204);
    /// `429 Too Many Requests`.
    pub const TOO_MANY_REQUESTS: Self = Self(429);

    /// Creates a status code in the HTTP `100..=599` range.
    #[must_use]
    pub const fn new(value: u16) -> Option<Self> {
        if value < 100 || value > 599 {
            return None;
        }
        Some(Self(value))
    }

    /// Returns the numeric status code.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// Reports whether this is a success status.
    #[must_use]
    pub const fn is_success(self) -> bool {
        self.0 >= 200 && self.0 <= 299
    }

    /// Reports whether this is a client or server error status.
    #[must_use]
    pub const fn is_error(self) -> bool {
        self.0 >= 400
    }
}

/// Synchronous transport over caller-owned request and response buffers.
///
/// Authentication, base URLs, headers, timeouts, TLS, and retry policy belong
/// to adapters and are intentionally outside this minimal contract.
/// The shared receiver does not itself promise concurrency: callers may issue
/// overlapping requests only when the concrete implementation satisfies their
/// required [`Sync`] and [`Send`] bounds. Sequential implementations may use
/// safe interior mutability without becoming `Sync`.
pub trait BlockingTransport {
    /// Transport-specific failure.
    type Error;

    /// Sends one request and writes the response body into the caller buffer.
    fn send(
        &self,
        request: TransportRequest<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests;
