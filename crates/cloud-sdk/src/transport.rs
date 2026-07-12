//! Provider-neutral blocking transport contracts.

use core::fmt;

use crate::Method;

/// Maximum origin-form request-target length admitted by the core contract.
pub const MAX_REQUEST_TARGET_BYTES: usize = 8192;

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
}

impl<'a> TransportRequest<'a> {
    /// Creates a bodyless request.
    #[must_use]
    pub const fn new(method: Method, target: RequestTarget<'a>) -> Self {
        Self {
            method,
            target,
            body: &[],
        }
    }

    /// Adds a borrowed request body.
    #[must_use]
    pub const fn with_body(mut self, body: &'a [u8]) -> Self {
        self.body = body;
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
}

impl fmt::Debug for TransportRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportRequest")
            .field("method", &self.method)
            .field("target", &self.target)
            .field("body", &"[redacted]")
            .finish()
    }
}

/// Valid HTTP response status code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StatusCode(u16);

impl StatusCode {
    /// `200 OK`.
    pub const OK: Self = Self(200);
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
#[derive(Clone, Copy)]
pub struct TransportResponse<'buffer> {
    status: StatusCode,
    body: &'buffer [u8],
}

impl<'buffer> TransportResponse<'buffer> {
    /// Creates a response over the initialized body bytes.
    #[must_use]
    pub const fn new(status: StatusCode, body: &'buffer [u8]) -> Self {
        Self { status, body }
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
}

impl fmt::Debug for TransportResponse<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportResponse")
            .field("status", &self.status)
            .field("body_len", &self.body.len())
            .field("body", &"[redacted]")
            .finish()
    }
}

/// Synchronous transport over caller-owned request and response buffers.
///
/// Authentication, base URLs, headers, timeouts, TLS, and retry policy belong
/// to adapters and are intentionally outside this minimal contract.
pub trait BlockingTransport {
    /// Transport-specific failure.
    type Error;

    /// Sends one request and writes the response body into the caller buffer.
    fn send<'buffer>(
        &mut self,
        request: TransportRequest<'_>,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::{
        RequestTarget, RequestTargetError, StatusCode, TransportRequest, TransportResponse,
    };
    use crate::Method;
    use core::fmt::Write;

    #[test]
    fn request_targets_are_origin_form_and_bounded() {
        let target = RequestTarget::new("/servers?page=2");
        assert_eq!(target.map(RequestTarget::as_str), Ok("/servers?page=2"));
        assert_eq!(
            RequestTarget::new("https://example.invalid/servers"),
            Err(RequestTargetError::NotOriginForm)
        );
        assert_eq!(
            RequestTarget::new("//evil.example/steal"),
            Err(RequestTargetError::NotOriginForm)
        );
        assert_eq!(
            RequestTarget::new("///evil.example/steal"),
            Err(RequestTargetError::NotOriginForm)
        );
        assert_eq!(
            RequestTarget::new("/servers#fragment"),
            Err(RequestTargetError::InvalidByte)
        );
        assert_eq!(
            RequestTarget::new("/\\evil"),
            Err(RequestTargetError::InvalidByte)
        );
        assert_eq!(RequestTarget::new(""), Err(RequestTargetError::Empty));
        assert_eq!(
            RequestTarget::new("/servers bad"),
            Err(RequestTargetError::InvalidByte)
        );
        let mut accepted = [b'a'; super::MAX_REQUEST_TARGET_BYTES];
        if let Some(first) = accepted.first_mut() {
            *first = b'/';
        }
        let accepted = core::str::from_utf8(&accepted);
        assert!(accepted.is_ok());
        if let Ok(accepted) = accepted {
            assert!(RequestTarget::new(accepted).is_ok());
        }
        let mut rejected = [b'a'; super::MAX_REQUEST_TARGET_BYTES + 1];
        if let Some(first) = rejected.first_mut() {
            *first = b'/';
        }
        let rejected = core::str::from_utf8(&rejected);
        assert!(rejected.is_ok());
        if let Ok(rejected) = rejected {
            assert_eq!(
                RequestTarget::new(rejected),
                Err(RequestTargetError::TooLong)
            );
        }
    }

    #[test]
    fn transport_request_debug_redacts_target_and_body() {
        let target = RequestTarget::new("/servers?token=secret");
        if let Ok(target) = target {
            let request = TransportRequest::new(Method::Post, target).with_body(b"secret-body");
            let mut debug = DebugBuffer::new();
            assert!(write!(&mut debug, "{request:?}").is_ok());
            let debug = debug.as_str();
            assert!(debug.contains("[redacted]"));
            assert!(!debug.contains("secret"));
        }
    }

    #[test]
    fn status_codes_are_bounded_and_classified() {
        assert_eq!(StatusCode::new(99), None);
        assert!(StatusCode::new(204).is_some_and(StatusCode::is_success));
        assert!(StatusCode::new(429).is_some_and(StatusCode::is_error));
        assert_eq!(StatusCode::new(600), None);
    }

    #[test]
    fn transport_response_borrows_exact_initialized_body_and_redacts_debug() {
        let output = b"secret-response-trailing-capacity";
        let body = output.get(..15).unwrap_or_default();
        let response = TransportResponse::new(StatusCode::OK, body);

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.body(), b"secret-response");

        let mut debug = DebugBuffer::new();
        assert!(write!(&mut debug, "{response:?}").is_ok());
        let debug = debug.as_str();
        assert!(debug.contains("body_len: 15"));
        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains("secret-response"));
    }

    struct DebugBuffer {
        bytes: [u8; 192],
        len: usize,
    }

    impl DebugBuffer {
        const fn new() -> Self {
            Self {
                bytes: [0; 192],
                len: 0,
            }
        }

        fn as_str(&self) -> &str {
            core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
        }
    }

    impl Write for DebugBuffer {
        fn write_str(&mut self, value: &str) -> core::fmt::Result {
            let end = self.len.checked_add(value.len()).ok_or(core::fmt::Error)?;
            let target = self.bytes.get_mut(self.len..end).ok_or(core::fmt::Error)?;
            target.copy_from_slice(value.as_bytes());
            self.len = end;
            Ok(())
        }
    }
}
