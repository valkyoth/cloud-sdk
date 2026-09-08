use core::fmt;

use cloud_sdk::rate_limit::RetryAfter;
use cloud_sdk::transport::StatusCode;

/// Payload-free response or decoder policy failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CratesIoWireError {
    /// Expected status or byte budget is outside the admitted range.
    InvalidPolicy,
    /// No complete transport response was committed.
    Uncommitted,
    /// Response bytes exceed the caller's admitted ceiling.
    ResponseTooLarge,
    /// The media type is absent, invalid, or not UTF-8 JSON.
    ContentType,
    /// JSON syntax, duplicate keys, or structural limits were rejected.
    Json,
    /// The root or provider-error envelope has an invalid shape.
    Envelope,
    /// Status differs from the operation's exact expected success code.
    UnexpectedStatus,
    /// Present Retry-After metadata is invalid.
    RetryAfter,
    /// A valid provider error, including a Cargo error on HTTP 200.
    Provider(ProviderError),
}

impl fmt::Display for CratesIoWireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidPolicy => "crates.io response policy is invalid",
            Self::Uncommitted => "crates.io response is not committed",
            Self::ResponseTooLarge => "crates.io response exceeds its byte budget",
            Self::ContentType => "crates.io response requires UTF-8 application/json",
            Self::Json => "crates.io JSON admission failed",
            Self::Envelope => "crates.io response envelope is invalid",
            Self::UnexpectedStatus => "crates.io response status is unexpected",
            Self::RetryAfter => "crates.io Retry-After metadata is invalid",
            Self::Provider(_) => "crates.io reported an operation failure",
        })
    }
}

impl core::error::Error for CratesIoWireError {}

/// Classification is advisory, never authorization to retry a mutation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderErrorKind {
    /// HTTP 429.
    RateLimited,
    /// HTTP 503.
    Unavailable,
    /// Cargo-compatible error envelope on a successful HTTP status.
    CargoErrorOnSuccess,
    /// Another provider HTTP error.
    Other,
}

/// Sanitized error-envelope metadata. Provider detail strings are discarded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderError {
    pub(super) status: StatusCode,
    pub(super) count: usize,
    pub(super) retry_after: Option<RetryAfter>,
}

impl ProviderError {
    /// Returns the HTTP status, without response content.
    #[must_use]
    pub const fn status(self) -> StatusCode {
        self.status
    }

    /// Returns the validated number of provider error entries.
    #[must_use]
    pub const fn count(self) -> usize {
        self.count
    }

    /// Returns a validated server delay instruction, not a retry decision.
    #[must_use]
    pub const fn retry_after(self) -> Option<RetryAfter> {
        self.retry_after
    }

    /// Classifies response metadata without initiating retries.
    #[must_use]
    pub fn kind(self) -> ProviderErrorKind {
        match self.status.get() {
            429 => ProviderErrorKind::RateLimited,
            503 => ProviderErrorKind::Unavailable,
            _ if self.status.is_success() => ProviderErrorKind::CargoErrorOnSuccess,
            _ => ProviderErrorKind::Other,
        }
    }
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("crates.io provider error (details discarded)")
    }
}

impl core::error::Error for ProviderError {}
