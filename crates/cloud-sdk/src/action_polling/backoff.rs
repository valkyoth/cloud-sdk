use crate::retry::MonotonicDuration;

use super::{PollContext, ProgressChange};

/// Largest admitted exponential multiplier.
pub const MAX_BACKOFF_MULTIPLIER: u8 = 16;

/// Caller-owned action polling backoff without sleep or clock access.
pub trait PollBackoff {
    /// Policy-specific failure. Driver diagnostics always redact this value.
    type Error;

    /// Chooses one requested delay from validated nonsensitive context.
    fn delay(&mut self, context: PollContext) -> Result<MonotonicDuration, Self::Error>;
}

/// Invalid exponential backoff configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExponentialBackoffError {
    /// Initial and maximum delays must be nonzero.
    ZeroDelay,
    /// Initial delay exceeds the maximum.
    InitialExceedsMaximum,
    /// Multiplier is zero or exceeds its hard bound.
    InvalidMultiplier,
}

impl_static_error!(ExponentialBackoffError,
    Self::ZeroDelay => "poll backoff delays must be nonzero",
    Self::InitialExceedsMaximum => "initial poll backoff exceeds its maximum",
    Self::InvalidMultiplier => "poll backoff multiplier is invalid",
);

/// Allocation-free exponential backoff reset by provider progress.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExponentialBackoff {
    initial: MonotonicDuration,
    maximum: MonotonicDuration,
    multiplier: u8,
    next: MonotonicDuration,
}

impl ExponentialBackoff {
    /// Creates bounded deterministic backoff.
    pub const fn new(
        initial: MonotonicDuration,
        maximum: MonotonicDuration,
        multiplier: u8,
    ) -> Result<Self, ExponentialBackoffError> {
        if initial.get() == 0 || maximum.get() == 0 {
            return Err(ExponentialBackoffError::ZeroDelay);
        }
        if initial.get() > maximum.get() {
            return Err(ExponentialBackoffError::InitialExceedsMaximum);
        }
        if multiplier == 0 || multiplier > MAX_BACKOFF_MULTIPLIER {
            return Err(ExponentialBackoffError::InvalidMultiplier);
        }
        Ok(Self {
            initial,
            maximum,
            multiplier,
            next: initial,
        })
    }

    /// Returns the next delay before progress policy is applied.
    #[must_use]
    pub const fn next_delay(self) -> MonotonicDuration {
        self.next
    }
}

impl PollBackoff for ExponentialBackoff {
    type Error = core::convert::Infallible;

    fn delay(&mut self, context: PollContext) -> Result<MonotonicDuration, Self::Error> {
        if matches!(
            context.progress_change(),
            ProgressChange::Initial | ProgressChange::Advanced | ProgressChange::Reset
        ) {
            self.next = self.initial;
        }
        let selected = self.next;
        let multiplied = selected
            .get()
            .saturating_mul(u64::from(self.multiplier))
            .min(self.maximum.get());
        self.next = MonotonicDuration::new(multiplied);
        Ok(selected)
    }
}
