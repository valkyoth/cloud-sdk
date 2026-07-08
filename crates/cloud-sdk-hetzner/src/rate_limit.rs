//! Rate-limit response metadata domains.

/// Known rate-limit response header names.
pub const RATE_LIMIT_HEADERS: &[&str] =
    &["ratelimit-limit", "ratelimit-remaining", "ratelimit-reset"];

/// Rate-limit metadata validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RateLimitError {
    /// Remaining requests must not exceed the limit.
    RemainingExceedsLimit,
}

/// Response rate-limit metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RateLimit {
    limit: u32,
    remaining: u32,
    reset_epoch_seconds: Option<u64>,
}

impl RateLimit {
    /// Creates validated rate-limit metadata.
    pub const fn new(
        limit: u32,
        remaining: u32,
        reset_epoch_seconds: Option<u64>,
    ) -> Result<Self, RateLimitError> {
        if remaining > limit {
            return Err(RateLimitError::RemainingExceedsLimit);
        }
        Ok(Self {
            limit,
            remaining,
            reset_epoch_seconds,
        })
    }

    /// Returns the request limit.
    #[must_use]
    pub const fn limit(self) -> u32 {
        self.limit
    }

    /// Returns remaining requests.
    #[must_use]
    pub const fn remaining(self) -> u32 {
        self.remaining
    }

    /// Returns reset time as Unix epoch seconds when provided.
    #[must_use]
    pub const fn reset_epoch_seconds(self) -> Option<u64> {
        self.reset_epoch_seconds
    }
}

#[cfg(test)]
mod tests {
    use super::{RateLimit, RateLimitError};

    #[test]
    fn rejects_impossible_remaining_count() {
        assert_eq!(
            RateLimit::new(10, 11, None),
            Err(RateLimitError::RemainingExceedsLimit)
        );
        assert_eq!(
            RateLimit::new(10, 9, Some(42)).map(RateLimit::remaining),
            Ok(9)
        );
    }
}
