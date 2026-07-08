//! Response-domain primitives.

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
}
