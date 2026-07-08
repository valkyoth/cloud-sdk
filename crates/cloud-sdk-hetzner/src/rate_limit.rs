//! Rate-limit response metadata domains.

/// Known rate-limit response header names.
pub const RATE_LIMIT_HEADERS: &[&str] =
    &["ratelimit-limit", "ratelimit-remaining", "ratelimit-reset"];
