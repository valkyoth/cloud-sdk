use core::fmt;
use core::time::Duration;

/// Source-locked crates.io API interval. Static downloads have a separate policy.
pub const API_REQUEST_INTERVAL: Duration = Duration::from_secs(1);
/// SDK ceiling for a provider Retry-After delay in official client execution.
/// Larger values fail the current call and impose this bounded quiet period.
pub const MAX_PROVIDER_DELAY: Duration = Duration::from_secs(86_400);

/// Clock-free, non-cloneable admission state shared by all of a caller's API work.
///
/// Supply elapsed time from one trusted monotonic clock and synchronize access
/// externally. Do not create a scheduler per request, credential, or worker.
/// Admission consumes the interval even if transport subsequently fails.
#[derive(Debug, Default)]
pub struct ApiSchedule {
    last_observed: Option<Duration>,
    next: Duration,
}

impl ApiSchedule {
    /// Creates an unused schedule with no implicit clock or sleeper.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            last_observed: None,
            next: Duration::ZERO,
        }
    }

    /// Admits one immediate attempt, or returns the required wait. No reservations
    /// accumulate, so delayed workers cannot redeem old permits in a burst.
    pub fn try_start(&mut self, now: Duration) -> Result<(), ScheduleError> {
        if self.last_observed.is_some_and(|last| now < last) {
            return Err(ScheduleError::ClockRollback);
        }
        self.last_observed = Some(now);
        if now < self.next {
            return Err(ScheduleError::Wait(self.next.saturating_sub(now)));
        }
        self.next = now
            .checked_add(API_REQUEST_INTERVAL)
            .ok_or(ScheduleError::Overflow)?;
        Ok(())
    }

    /// Extends the interval after a completed attempt or a caller-approved server
    /// delay. Never shortens a previously imposed delay. The caller owns sleeping.
    pub fn defer_until(&mut self, deadline: Duration) {
        self.next = self.next.max(deadline);
    }
}

/// Payload-free scheduling failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleError {
    /// No request may start until this monotonic delay has elapsed.
    Wait(Duration),
    /// The trusted monotonic clock moved backwards.
    ClockRollback,
    /// A deadline cannot be represented or a response delay exceeds SDK policy.
    Overflow,
    /// The process-wide request gate is busy or was poisoned by a panic.
    Unavailable,
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Wait(_) => "crates.io request must wait for its scheduled interval",
            Self::ClockRollback => "crates.io scheduling clock moved backwards",
            Self::Overflow => "crates.io scheduling delay exceeds supported bounds",
            Self::Unavailable => "crates.io request gate is unavailable",
        })
    }
}

impl core::error::Error for ScheduleError {}
