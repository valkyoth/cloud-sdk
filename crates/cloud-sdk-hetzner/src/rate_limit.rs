//! Rate-limit response metadata domains.

pub use cloud_sdk::rate_limit::{RateLimit, RateLimitError};

/// Known rate-limit response header names.
pub const RATE_LIMIT_HEADERS: &[&str] =
    &["ratelimit-limit", "ratelimit-remaining", "ratelimit-reset"];

#[cfg(test)]
mod tests {
    use super::{RATE_LIMIT_HEADERS, RateLimit, RateLimitError};

    #[test]
    fn rejects_impossible_remaining_count() {
        assert_eq!(
            RateLimit::new(10, 11, 42),
            Err(RateLimitError::RemainingExceedsLimit)
        );
        assert_eq!(RateLimit::new(10, 9, 42).map(RateLimit::remaining), Ok(9));
        assert_eq!(RATE_LIMIT_HEADERS.len(), 3);
    }
}
