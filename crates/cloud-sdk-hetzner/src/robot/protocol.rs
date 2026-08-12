//! Bounded Robot error, quota, and maintenance response decoding.

use alloc::vec::Vec;
use core::fmt;

use cloud_sdk::rate_limit::{DelaySeconds, QuotaBucket, QuotaBucketId, QuotaError, QuotaReset};
use cloud_sdk::transport::{DeliveryClassified, DeliveryPhase};

use crate::serde::SensitiveText;

use super::MAX_ROBOT_ERROR_BODY_BYTES;

mod decode;
pub use decode::decode_robot_failure;
pub(crate) use decode::decode_robot_failure_with;

/// Maximum missing or invalid field names retained from one invalid-input error.
pub const MAX_ROBOT_INPUT_FIELDS: usize = 256;
const MAX_ROBOT_ERROR_CODE_BYTES: usize = 128;
const MAX_ROBOT_ERROR_MESSAGE_BYTES: usize = 16_384;
const MAX_ROBOT_INPUT_FIELD_BYTES: usize = 1_024;
const ROBOT_QUOTA_BUCKET_ID: &[u8] = b"robot-global";

/// Stable classification for one Robot failure.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotFailureCategory {
    /// HTTP Basic credentials were rejected.
    AuthenticationRejected,
    /// Required or supplied form input was invalid.
    InvalidInput,
    /// The source-locked Robot request quota was exhausted.
    QuotaExceeded,
    /// Robot reported service maintenance.
    Maintenance,
    /// A source-locked provider error was returned.
    Provider,
    /// A separately supplied transport failure was classified as transient.
    TransientTransport,
}

/// Caller-policy retry disposition for one classified Robot failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotRetryDisposition {
    /// Retry is forbidden for this response classification.
    Never,
    /// Retry requires an explicit caller policy.
    ExplicitPolicy,
    /// Retry requires an explicit caller policy and at least this delay.
    ExplicitAfter(DelaySeconds),
}

/// Source-locked Robot provider error codes admitted before endpoint families.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotProviderErrorCode {
    /// The requested server was not found.
    ServerNotFound,
    /// No resources were found for a list operation.
    NotFound,
    /// The requested subnet was not found.
    SubnetNotFound,
    /// Separate MAC assignment is unavailable for the subnet.
    MacNotAvailable,
    /// Robot could not update the subnet traffic-warning policy.
    TrafficWarningUpdateFailed,
    /// Robot could not apply or restore the subnet MAC.
    MacFailed,
    /// The server does not provide any reset capability.
    ResetNotAvailable,
    /// A manually performed reset is already active.
    ResetManualActive,
    /// Robot could not execute the selected reset.
    ResetFailed,
}

/// Protected details from an `INVALID_INPUT` response.
#[derive(Eq, PartialEq)]
pub struct RobotInvalidInput {
    message: SensitiveText,
    missing: Vec<SensitiveText>,
    invalid: Vec<SensitiveText>,
}

impl RobotInvalidInput {
    /// Runs a closure with temporary access to the provider message.
    pub fn try_with_message<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.message.try_with_secret(inspect)
    }

    /// Returns the number of missing input names.
    #[must_use]
    pub fn missing_len(&self) -> usize {
        self.missing.len()
    }

    /// Returns the number of invalid input names.
    #[must_use]
    pub fn invalid_len(&self) -> usize {
        self.invalid.len()
    }

    /// Runs a closure with one protected missing input name.
    pub fn try_with_missing<R>(
        &self,
        index: usize,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<Option<R>, core::str::Utf8Error> {
        self.missing
            .get(index)
            .map(|value| value.try_with_secret(inspect))
            .transpose()
    }

    /// Runs a closure with one protected invalid input name.
    pub fn try_with_invalid<R>(
        &self,
        index: usize,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<Option<R>, core::str::Utf8Error> {
        self.invalid
            .get(index)
            .map(|value| value.try_with_secret(inspect))
            .transpose()
    }
}

impl fmt::Debug for RobotInvalidInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotInvalidInput")
            .field("message", &"[redacted]")
            .field("missing_len", &self.missing.len())
            .field("invalid_len", &self.invalid.len())
            .field("fields", &"[redacted]")
            .finish()
    }
}

/// Validated Robot quota exhaustion metadata.
#[derive(Eq, PartialEq)]
pub struct RobotQuota {
    message: SensitiveText,
    max_requests: u64,
    interval: DelaySeconds,
}

impl RobotQuota {
    /// Runs a closure with temporary access to the provider message.
    pub fn try_with_message<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.message.try_with_secret(inspect)
    }

    /// Returns the source-locked maximum request count.
    #[must_use]
    pub const fn max_requests(&self) -> u64 {
        self.max_requests
    }

    /// Returns the source-locked quota interval.
    #[must_use]
    pub const fn interval(&self) -> DelaySeconds {
        self.interval
    }

    /// Creates the provider-neutral exhausted quota bucket.
    pub fn quota_bucket(&self) -> Result<QuotaBucket, QuotaError> {
        QuotaBucket::new(
            QuotaBucketId::new(ROBOT_QUOTA_BUCKET_ID)?,
            self.max_requests,
            0,
            QuotaReset::After(self.interval),
        )
    }
}

impl fmt::Debug for RobotQuota {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotQuota")
            .field("message", &"[redacted]")
            .field("max_requests", &self.max_requests)
            .field("interval", &self.interval)
            .finish()
    }
}

/// Protected source-locked Robot provider error.
#[derive(Eq, PartialEq)]
pub struct RobotProviderError {
    code: RobotProviderErrorCode,
    message: SensitiveText,
}

impl RobotProviderError {
    /// Returns the finite provider error code.
    #[must_use]
    pub const fn code(&self) -> RobotProviderErrorCode {
        self.code
    }

    /// Runs a closure with temporary access to the provider message.
    pub fn try_with_message<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.message.try_with_secret(inspect)
    }
}

impl fmt::Debug for RobotProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotProviderError")
            .field("code", &self.code)
            .field("message", &"[redacted]")
            .finish()
    }
}

/// Explicit transient transport classification, separate from provider data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RobotTransientTransport {
    phase: DeliveryPhase,
}

impl RobotTransientTransport {
    /// Classifies an adapter failure only after its delivery phase is known.
    #[must_use]
    pub fn from_failure(failure: &impl DeliveryClassified) -> Self {
        Self {
            phase: failure.delivery_phase(),
        }
    }

    /// Returns the conservative transport delivery phase.
    #[must_use]
    pub const fn delivery_phase(self) -> DeliveryPhase {
        self.phase
    }
}

/// Typed Robot failure. Unknown provider status or code never enters this enum.
#[derive(Debug, Eq, PartialEq)]
pub enum RobotFailure {
    /// HTTP Basic credentials were rejected without a body.
    AuthenticationRejected,
    /// A bounded `INVALID_INPUT` envelope was returned.
    InvalidInput(RobotInvalidInput),
    /// A bounded `RATE_LIMIT_EXCEEDED` envelope was returned.
    QuotaExceeded(RobotQuota),
    /// Robot reported maintenance without a body.
    Maintenance,
    /// A finite source-locked provider error was returned.
    Provider(RobotProviderError),
    /// A transport adapter explicitly supplied a transient failure.
    TransientTransport(RobotTransientTransport),
}

impl RobotFailure {
    /// Creates a transport classification without accepting provider bytes.
    #[must_use]
    pub fn transient_transport(failure: &impl DeliveryClassified) -> Self {
        Self::TransientTransport(RobotTransientTransport::from_failure(failure))
    }

    /// Returns the finite failure category.
    #[must_use]
    pub const fn category(&self) -> RobotFailureCategory {
        match self {
            Self::AuthenticationRejected => RobotFailureCategory::AuthenticationRejected,
            Self::InvalidInput(_) => RobotFailureCategory::InvalidInput,
            Self::QuotaExceeded(_) => RobotFailureCategory::QuotaExceeded,
            Self::Maintenance => RobotFailureCategory::Maintenance,
            Self::Provider(_) => RobotFailureCategory::Provider,
            Self::TransientTransport(_) => RobotFailureCategory::TransientTransport,
        }
    }

    /// Returns the required caller-owned retry disposition.
    #[must_use]
    pub const fn retry_disposition(&self) -> RobotRetryDisposition {
        match self {
            Self::QuotaExceeded(quota) => RobotRetryDisposition::ExplicitAfter(quota.interval()),
            Self::Maintenance | Self::TransientTransport(_) => {
                RobotRetryDisposition::ExplicitPolicy
            }
            Self::AuthenticationRejected | Self::InvalidInput(_) | Self::Provider(_) => {
                RobotRetryDisposition::Never
            }
        }
    }

    /// Reports whether an SDK path may retry without caller policy.
    #[must_use]
    pub const fn allows_automatic_retry(&self) -> bool {
        false
    }
}

/// Failure to admit and decode a Robot error response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotDecodeError {
    /// A success response was passed to the error decoder.
    UnexpectedSuccessStatus,
    /// The status has no source-locked Robot error classification.
    UnsupportedStatus,
    /// A body was required but absent.
    MissingBody,
    /// A body was supplied for a bodyless status.
    UnexpectedBody,
    /// The response content type was absent, malformed, or not JSON.
    InvalidContentType,
    /// The response exceeded the Robot error-body bound.
    ResponseTooLarge,
    /// JSON syntax, UTF-8, nesting, duplicates, or parser bounds were invalid.
    MalformedPayload,
    /// The error envelope had missing, extra, or wrongly typed fields.
    InvalidEnvelope,
    /// The envelope status did not match the HTTP status.
    StatusMismatch,
    /// The provider error code was not source-locked for this status.
    UnknownCode,
    /// Quota values were zero, incoherent, or not representable.
    InvalidQuota,
    /// Bounded result storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotDecodeError,
    Self::UnexpectedSuccessStatus => "Robot success status cannot be decoded as an error",
    Self::UnsupportedStatus => "Robot error status is not source-locked",
    Self::MissingBody => "Robot error response body is missing",
    Self::UnexpectedBody => "Robot bodyless error status included a body",
    Self::InvalidContentType => "Robot error response content type is invalid",
    Self::ResponseTooLarge => "Robot error response exceeds its size limit",
    Self::MalformedPayload => "Robot error response JSON is malformed",
    Self::InvalidEnvelope => "Robot error response envelope is invalid",
    Self::StatusMismatch => "Robot envelope status does not match the HTTP status",
    Self::UnknownCode => "Robot error code is not source-locked",
    Self::InvalidQuota => "Robot quota response is invalid",
    Self::Allocation => "Robot error result allocation failed",
);

#[cfg(test)]
mod tests;
