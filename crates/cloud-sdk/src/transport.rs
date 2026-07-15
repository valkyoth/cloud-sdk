//! Provider-neutral blocking and asynchronous transport contracts.

mod asynchronous;
mod content_type;
mod endpoint;

pub use asynchronous::AsyncTransport;
pub use content_type::{
    ContentType, ContentTypeError, MAX_CONTENT_TYPE_BYTES, MediaType, ResponseContentType,
};
pub use endpoint::{
    BoundTransport, EndpointIdentity, EndpointIdentityError, EndpointScheme,
    MAX_ENDPOINT_BASE_PATH_BYTES, MAX_ENDPOINT_HOST_BYTES,
};

use core::fmt;

use crate::Method;
use crate::rate_limit::RateLimit;

/// Maximum origin-form request-target length admitted by the core contract.
pub const MAX_REQUEST_TARGET_BYTES: usize = 8192;

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

/// Request-target validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestTargetError {
    /// Request targets must not be empty.
    Empty,
    /// Request targets must start with `/`.
    NotOriginForm,
    /// Request targets exceed [`MAX_REQUEST_TARGET_BYTES`].
    TooLong,
    /// Request targets contain a control, space, non-ASCII, fragment, or
    /// backslash byte.
    InvalidByte,
}

impl_static_error!(RequestTargetError,
    Self::Empty => "request target is empty",
    Self::NotOriginForm => "request target is not in origin form",
    Self::TooLong => "request target exceeds the length limit",
    Self::InvalidByte => "request target contains a forbidden byte",
);

/// Validated origin-form HTTP request target.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RequestTarget<'a> {
    value: &'a str,
}

impl<'a> RequestTarget<'a> {
    /// Validates a `/path?query` request target.
    pub fn new(value: &'a str) -> Result<Self, RequestTargetError> {
        if value.is_empty() {
            return Err(RequestTargetError::Empty);
        }
        if value.len() > MAX_REQUEST_TARGET_BYTES {
            return Err(RequestTargetError::TooLong);
        }
        if !value.starts_with('/') || value.starts_with("//") {
            return Err(RequestTargetError::NotOriginForm);
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b'#' && byte != b'\\')
        {
            return Err(RequestTargetError::InvalidByte);
        }
        Ok(Self { value })
    }

    /// Returns the validated request target.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

impl fmt::Debug for RequestTarget<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RequestTarget([redacted])")
    }
}

/// Provider-neutral request passed to a blocking transport.
#[derive(Clone, Copy)]
pub struct TransportRequest<'a> {
    method: Method,
    target: RequestTarget<'a>,
    body: &'a [u8],
    content_type: Option<ContentType<'a>>,
}

impl<'a> TransportRequest<'a> {
    /// Creates a bodyless request.
    #[must_use]
    pub const fn new(method: Method, target: RequestTarget<'a>) -> Self {
        Self {
            method,
            target,
            body: &[],
            content_type: None,
        }
    }

    /// Adds a borrowed request body.
    #[must_use]
    pub const fn with_body(mut self, body: &'a [u8]) -> Self {
        self.body = body;
        self
    }

    /// Adds an explicit content type for the borrowed request body.
    #[must_use]
    pub const fn with_content_type(mut self, content_type: ContentType<'a>) -> Self {
        self.content_type = Some(content_type);
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

    /// Returns the explicit request-body content type, when configured.
    #[must_use]
    pub const fn content_type(self) -> Option<ContentType<'a>> {
        self.content_type
    }
}

impl fmt::Debug for TransportRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportRequest")
            .field("method", &self.method)
            .field("target", &self.target)
            .field("body", &"[redacted]")
            .field("content_type", &self.content_type)
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

/// Response returned after a transport initializes part of a caller buffer.
///
/// The body is structurally bounded by the borrowed initialized slice. A
/// transport cannot report a numeric length independently of the caller's
/// response buffer.
///
/// ```compile_fail
/// use cloud_sdk::transport::{StatusCode, TransportResponse};
///
/// let _ = TransportResponse::new(StatusCode::OK, 1024_usize);
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct TransportResponse<'buffer> {
    status: StatusCode,
    body: &'buffer [u8],
    content_type: Option<ResponseContentType>,
    rate_limit: Option<RateLimit>,
}

impl<'buffer> TransportResponse<'buffer> {
    /// Creates a response over the initialized body bytes.
    #[must_use]
    pub const fn new(status: StatusCode, body: &'buffer [u8]) -> Self {
        Self {
            status,
            body,
            content_type: None,
            rate_limit: None,
        }
    }

    /// Adds a validated response content type captured by the transport.
    #[must_use]
    pub const fn with_content_type(mut self, content_type: ResponseContentType) -> Self {
        self.content_type = Some(content_type);
        self
    }

    /// Adds validated rate-limit metadata captured by the transport.
    #[must_use]
    pub const fn with_rate_limit(mut self, rate_limit: RateLimit) -> Self {
        self.rate_limit = Some(rate_limit);
        self
    }

    /// Returns the status code.
    #[must_use]
    pub const fn status(&self) -> StatusCode {
        self.status
    }

    /// Returns the initialized response body bytes.
    #[must_use]
    pub const fn body(&self) -> &'buffer [u8] {
        self.body
    }

    /// Returns the validated response content type when supplied.
    #[must_use]
    pub const fn content_type(&self) -> Option<ResponseContentType> {
        self.content_type
    }

    /// Returns validated rate-limit metadata when the response supplied it.
    #[must_use]
    pub const fn rate_limit(&self) -> Option<RateLimit> {
        self.rate_limit
    }
}

impl fmt::Debug for TransportResponse<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportResponse")
            .field("status", &self.status)
            .field("body_len", &self.body.len())
            .field("body", &"[redacted]")
            .field("content_type", &self.content_type)
            .field("rate_limit", &self.rate_limit)
            .finish()
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
    fn send<'buffer>(
        &self,
        request: TransportRequest<'_>,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, Self::Error>;
}

#[cfg(test)]
mod tests;
