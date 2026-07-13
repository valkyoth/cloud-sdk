use core::fmt;
use std::time::Duration;

use reqwest::header::HeaderValue;

/// Maximum configured timeout accepted by the adapter.
pub const MAX_TIMEOUT_SECONDS: u64 = 300;

/// Timeout policy validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeoutError {
    /// Every timeout must be nonzero.
    Zero,
    /// Every timeout is capped at [`MAX_TIMEOUT_SECONDS`].
    TooLong,
    /// The connect timeout must not exceed the total timeout.
    ExceedsTotal,
}

/// Explicit total-request and connection timeout policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestTimeouts {
    total: Duration,
    connect: Duration,
}

impl RequestTimeouts {
    /// Validates all timeout dimensions.
    pub fn new(total: Duration, connect: Duration) -> Result<Self, TimeoutError> {
        if total.is_zero() || connect.is_zero() {
            return Err(TimeoutError::Zero);
        }
        let maximum = Duration::from_secs(MAX_TIMEOUT_SECONDS);
        if total > maximum || connect > maximum {
            return Err(TimeoutError::TooLong);
        }
        if connect > total {
            return Err(TimeoutError::ExceedsTotal);
        }
        Ok(Self { total, connect })
    }

    pub(crate) const fn total(self) -> Duration {
        self.total
    }

    pub(crate) const fn connect(self) -> Duration {
        self.connect
    }
}

/// User-agent validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserAgentError {
    /// User agents must not be empty.
    Empty,
    /// User agents are capped at 256 bytes.
    TooLong,
    /// The value is not a valid HTTP header value.
    Invalid,
}

/// Validated, non-secret user-agent header value.
#[derive(Clone)]
pub struct UserAgent {
    pub(crate) value: HeaderValue,
}

impl UserAgent {
    /// Validates a user-agent value.
    pub fn new(value: &str) -> Result<Self, UserAgentError> {
        if value.is_empty() {
            return Err(UserAgentError::Empty);
        }
        if value.len() > 256 {
            return Err(UserAgentError::TooLong);
        }
        let value = HeaderValue::from_str(value).map_err(|_| UserAgentError::Invalid)?;
        Ok(Self { value })
    }
}

impl fmt::Debug for UserAgent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("UserAgent([validated])")
    }
}
