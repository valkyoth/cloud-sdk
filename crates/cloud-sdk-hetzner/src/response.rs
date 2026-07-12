//! Response-domain primitives.

use core::fmt;

/// Error category for API failures.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorCategory {
    /// Client-side request error.
    Request,
    /// Authentication or authorization failure.
    Authentication,
    /// Rate limit was exceeded.
    RateLimit,
    /// Server-side API failure.
    Server,
    /// API returned an error code that is not classified yet.
    Unknown,
}

/// Known API error code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ApiErrorCode {
    /// Invalid request input.
    InvalidInput,
    /// Authentication failed.
    Unauthorized,
    /// Authorization failed.
    Forbidden,
    /// Resource was not found.
    NotFound,
    /// A uniqueness constraint failed.
    UniquenessError,
    /// Resource protection blocked the operation.
    Protected,
    /// API rate limit was exceeded.
    RateLimitExceeded,
    /// API service error.
    ServiceError,
    /// Unknown error code.
    Unknown,
}

impl ApiErrorCode {
    /// Parses a Hetzner error code.
    #[must_use]
    pub const fn from_api_str(value: &str) -> Self {
        match value.as_bytes() {
            b"invalid_input" => Self::InvalidInput,
            b"unauthorized" => Self::Unauthorized,
            b"forbidden" => Self::Forbidden,
            b"not_found" => Self::NotFound,
            b"uniqueness_error" => Self::UniquenessError,
            b"protected" => Self::Protected,
            b"rate_limit_exceeded" => Self::RateLimitExceeded,
            b"service_error" => Self::ServiceError,
            _ => Self::Unknown,
        }
    }

    /// Returns the broad category for the error code.
    #[must_use]
    pub const fn category(self) -> ErrorCategory {
        match self {
            Self::Unauthorized | Self::Forbidden => ErrorCategory::Authentication,
            Self::RateLimitExceeded => ErrorCategory::RateLimit,
            Self::ServiceError => ErrorCategory::Server,
            Self::Unknown => ErrorCategory::Unknown,
            _ => ErrorCategory::Request,
        }
    }
}

/// Borrowed API error envelope.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ApiError<'a> {
    code: ApiErrorCode,
    message: &'a str,
}

impl fmt::Debug for ApiError<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApiError")
            .field("code", &self.code)
            .field("message", &"[redacted]")
            .finish()
    }
}

impl<'a> ApiError<'a> {
    /// Creates a borrowed API error envelope.
    #[must_use]
    pub const fn new(code: ApiErrorCode, message: &'a str) -> Self {
        Self { code, message }
    }

    /// Returns the error code.
    #[must_use]
    pub const fn code(self) -> ApiErrorCode {
        self.code
    }

    /// Returns the error message.
    #[must_use]
    pub const fn message(self) -> &'a str {
        self.message
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiError, ApiErrorCode, ErrorCategory};
    use core::fmt;
    use core::fmt::Write;

    #[test]
    fn classifies_known_error_codes() {
        assert_eq!(
            ApiErrorCode::from_api_str("rate_limit_exceeded").category(),
            ErrorCategory::RateLimit
        );
        assert_eq!(
            ApiErrorCode::from_api_str("not_from_spec"),
            ApiErrorCode::Unknown
        );
    }

    #[test]
    fn carries_borrowed_error_envelope() {
        let error = ApiError::new(ApiErrorCode::InvalidInput, "bad input");
        assert_eq!(error.code(), ApiErrorCode::InvalidInput);
        assert_eq!(error.message(), "bad input");
    }

    #[test]
    fn api_error_debug_redacts_provider_message() {
        let error = ApiError::new(ApiErrorCode::InvalidInput, "reflected-secret");
        let mut output = DebugBuffer::new();
        assert!(write!(&mut output, "{error:?}").is_ok());
        assert!(output.as_str().contains("[redacted]"));
        assert!(!output.as_str().contains("reflected-secret"));
    }

    struct DebugBuffer {
        bytes: [u8; 96],
        len: usize,
    }

    impl DebugBuffer {
        const fn new() -> Self {
            Self {
                bytes: [0; 96],
                len: 0,
            }
        }

        fn as_str(&self) -> &str {
            core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
        }
    }

    impl Write for DebugBuffer {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            let end = self.len.checked_add(value.len()).ok_or(fmt::Error)?;
            let target = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
            target.copy_from_slice(value.as_bytes());
            self.len = end;
            Ok(())
        }
    }
}
