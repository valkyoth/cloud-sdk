//! Provider-neutral rate-limit response metadata.

/// Rate-limit metadata validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RateLimitError {
    /// A reported request limit must be nonzero.
    LimitZero,
    /// Remaining requests must not exceed the reported limit.
    RemainingExceedsLimit,
}

impl_static_error!(RateLimitError,
    Self::LimitZero => "rate limit must be nonzero",
    Self::RemainingExceedsLimit => "remaining requests exceed the rate limit",
);

/// Validated rate-limit metadata returned by a transport.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RateLimit {
    limit: u64,
    remaining: u64,
    reset_epoch_seconds: u64,
}

impl RateLimit {
    /// Creates coherent rate-limit metadata.
    pub const fn new(
        limit: u64,
        remaining: u64,
        reset_epoch_seconds: u64,
    ) -> Result<Self, RateLimitError> {
        if limit == 0 {
            return Err(RateLimitError::LimitZero);
        }
        if remaining > limit {
            return Err(RateLimitError::RemainingExceedsLimit);
        }
        Ok(Self {
            limit,
            remaining,
            reset_epoch_seconds,
        })
    }

    /// Returns the total request limit for the provider's time frame.
    #[must_use]
    pub const fn limit(self) -> u64 {
        self.limit
    }

    /// Returns the number of requests remaining in the current time frame.
    #[must_use]
    pub const fn remaining(self) -> u64 {
        self.remaining
    }

    /// Returns the provider reset time as Unix epoch seconds.
    #[must_use]
    pub const fn reset_epoch_seconds(self) -> u64 {
        self.reset_epoch_seconds
    }
}

#[cfg(test)]
mod tests {
    use super::{RateLimit, RateLimitError};

    #[test]
    fn rejects_incoherent_metadata() {
        assert_eq!(RateLimit::new(0, 0, 42), Err(RateLimitError::LimitZero));
        assert_eq!(
            RateLimit::new(10, 11, 42),
            Err(RateLimitError::RemainingExceedsLimit)
        );
    }

    #[test]
    fn exposes_coherent_metadata() {
        let rate_limit = RateLimit::new(3600, 3599, 42);
        assert!(rate_limit.is_ok());
        let Ok(rate_limit) = rate_limit else { return };
        assert_eq!(rate_limit.limit(), 3600);
        assert_eq!(rate_limit.remaining(), 3599);
        assert_eq!(rate_limit.reset_epoch_seconds(), 42);
    }
}
