use crate::rate_limit::{RateLimit, WallClockTimestamp};
use crate::retry::MonotonicDuration;

use super::{ProgressChange, ProgressObservation};

/// Caller decision made independently from delay policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PollControl {
    /// Continue the bounded workflow.
    Continue,
    /// Cancel before another request is admitted.
    Cancel,
}

/// Provider-owned wall-clock telemetry carried without driving local budgets.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ProviderTimeObservation {
    observed_at: Option<WallClockTimestamp>,
    expires_at: Option<WallClockTimestamp>,
}

impl ProviderTimeObservation {
    /// Creates coherent provider timestamps.
    pub const fn new(
        observed_at: Option<WallClockTimestamp>,
        expires_at: Option<WallClockTimestamp>,
    ) -> Result<Self, ProviderTimeError> {
        if let (Some(observed), Some(expires)) = (observed_at, expires_at)
            && expires.get() < observed.get()
        {
            return Err(ProviderTimeError::ExpiryBeforeObservation);
        }
        Ok(Self {
            observed_at,
            expires_at,
        })
    }

    /// Returns the provider-reported observation timestamp.
    #[must_use]
    pub const fn observed_at(self) -> Option<WallClockTimestamp> {
        self.observed_at
    }

    /// Returns the provider-reported expiry timestamp.
    #[must_use]
    pub const fn expires_at(self) -> Option<WallClockTimestamp> {
        self.expires_at
    }
}

/// Invalid provider wall-clock telemetry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderTimeError {
    /// The provider expiry precedes its observation timestamp.
    ExpiryBeforeObservation,
}

impl_static_error!(ProviderTimeError,
    Self::ExpiryBeforeObservation => "provider expiry precedes its observation timestamp",
);

/// Redacted context supplied to caller-owned backoff policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PollContext {
    pub(crate) observation: u32,
    pub(crate) progress: ProgressObservation,
    pub(crate) progress_change: ProgressChange,
    pub(crate) rate_limit: Option<RateLimit>,
    pub(crate) provider_time: ProviderTimeObservation,
}

impl PollContext {
    /// Returns the one-based accepted observation count.
    #[must_use]
    pub const fn observation(self) -> u32 {
        self.observation
    }

    /// Returns provider progress without inferring missing values.
    #[must_use]
    pub const fn progress(self) -> ProgressObservation {
        self.progress
    }

    /// Returns the validated progress transition.
    #[must_use]
    pub const fn progress_change(self) -> ProgressChange {
        self.progress_change
    }

    /// Returns provider quota telemetry when supplied.
    #[must_use]
    pub const fn rate_limit(self) -> Option<RateLimit> {
        self.rate_limit
    }

    /// Returns provider wall-clock telemetry.
    #[must_use]
    pub const fn provider_time(self) -> ProviderTimeObservation {
        self.provider_time
    }
}

/// Next caller action before another provider request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PollRequestStep {
    /// One provider action request is admitted now.
    Request,
    /// Wait this monotonic duration before asking again.
    Delay(MonotonicDuration),
    /// Caller cancellation made the workflow terminal.
    Cancelled,
    /// The monotonic elapsed budget made the workflow terminal.
    TimedOut,
}
