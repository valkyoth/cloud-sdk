use core::fmt;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use super::{API_REQUEST_INTERVAL, ApiSchedule, IdentifyingUserAgent, ScheduleError};

pub(super) static SCHEDULE: Mutex<SharedSchedule> = Mutex::new(SharedSchedule {
    schedule: ApiSchedule::new(),
    active: false,
    poisoned: false,
});
static EPOCH: OnceLock<Instant> = OnceLock::new();

#[cfg(test)]
pub(crate) static TEST_GATE_LOCK: Mutex<()> = Mutex::new(());
#[cfg(test)]
pub(crate) fn reset_test_gate() {
    let mut state = SCHEDULE
        .lock()
        .unwrap_or_else(|_| unreachable!("test schedule lock"));
    assert!(!state.active, "cannot reset a live attempt");
    *state = SharedSchedule {
        schedule: ApiSchedule::new(),
        active: false,
        poisoned: false,
    };
}

pub(super) struct SharedSchedule {
    schedule: ApiSchedule,
    active: bool,
    poisoned: bool,
}

/// Owned admission across a complete synchronous or asynchronous exchange.
/// No mutex guard crosses an await. Drop imposes a new quiet interval even
/// when a future is cancelled; a panic permanently closes the process gate.
pub(crate) struct OfficialApiAttempt {
    delay: core::time::Duration,
}
impl OfficialApiAttempt {
    #[cfg(any(feature = "blocking", feature = "async"))]
    pub(crate) fn defer(&mut self, delay: core::time::Duration) -> Result<(), ScheduleError> {
        self.delay = self.delay.max(delay.min(super::MAX_PROVIDER_DELAY));
        if delay > super::MAX_PROVIDER_DELAY {
            return Err(ScheduleError::Overflow);
        }
        Ok(())
    }
}
impl Drop for OfficialApiAttempt {
    fn drop(&mut self) {
        let Ok(mut state) = SCHEDULE.lock() else {
            return;
        };
        if std::thread::panicking() {
            state.poisoned = true;
        }
        let now = EPOCH.get_or_init(Instant::now).elapsed();
        match now.checked_add(self.delay.max(API_REQUEST_INTERVAL)) {
            Some(deadline) => state.schedule.defer_until(deadline),
            None => state.poisoned = true,
        }
        state.active = false;
    }
}

/// Process-wide synchronous API gate, shared across all instances and credentials.
///
/// Serializes complete attempts and imposes at least a one-second quiet period
/// after completion. The callback must perform exactly one immediate HTTP
/// exchange with redirects and retries disabled and apply the supplied user
/// agent. It must not return a deferred task or future. This is a trusted
/// blocking adapter boundary, not an async executor or a high-level client.
/// Independent processes must coordinate their shared egress quota externally.
pub struct OfficialApiGate<'a> {
    user_agent: IdentifyingUserAgent<'a>,
}

impl<'a> OfficialApiGate<'a> {
    /// Requires explicit operator identity; there is no anonymous default.
    #[must_use]
    pub const fn new(user_agent: IdentifyingUserAgent<'a>) -> Self {
        Self { user_agent }
    }

    pub(crate) fn begin(&self) -> Result<OfficialApiAttempt, ScheduleError> {
        let mut state = SCHEDULE
            .try_lock()
            .map_err(|_| ScheduleError::Unavailable)?;
        if state.active || state.poisoned {
            return Err(ScheduleError::Unavailable);
        }
        state
            .schedule
            .try_start(EPOCH.get_or_init(Instant::now).elapsed())?;
        state.active = true;
        Ok(OfficialApiAttempt {
            delay: API_REQUEST_INTERVAL,
        })
    }

    #[cfg(any(feature = "blocking", feature = "async"))]
    pub(crate) const fn user_agent(&self) -> IdentifyingUserAgent<'a> {
        self.user_agent
    }

    /// Tries one complete blocking exchange without sleeping or retrying.
    /// Concurrent attempts return Unavailable; early attempts return Wait.
    /// Callback panics poison the gate, failing all later calls closed.
    pub fn try_call<T, E>(
        &self,
        send: impl FnOnce(IdentifyingUserAgent<'_>) -> Result<T, E>,
    ) -> Result<T, OfficialCallError<E>> {
        let _attempt = self.begin().map_err(OfficialCallError::Schedule)?;
        send(self.user_agent).map_err(OfficialCallError::Transport)
    }
}

impl fmt::Debug for OfficialApiGate<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OfficialApiGate([redacted])")
    }
}

/// Scheduling or trusted-adapter error with payload-free error-chain formatting.
pub enum OfficialCallError<E> {
    /// The attempt was not admitted.
    Schedule(ScheduleError),
    /// The attempted exchange failed. This value is not exposed as Error::source.
    Transport(E),
}

impl<E> fmt::Debug for OfficialCallError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Schedule(error) => f
                .debug_tuple("OfficialCallError::Schedule")
                .field(error)
                .finish(),
            Self::Transport(_) => f.write_str("OfficialCallError::Transport([redacted])"),
        }
    }
}

impl<E> fmt::Display for OfficialCallError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Schedule(_) => "crates.io request was not scheduled",
            Self::Transport(_) => "crates.io transport failed",
        })
    }
}

impl<E> core::error::Error for OfficialCallError<E> {}

#[cfg(all(test, any(feature = "blocking", feature = "async")))]
mod tests {
    use super::*;
    use crate::discovery::tests::Fixture as _;
    use core::time::Duration;

    #[test]
    fn provider_delay_bounds_remain_monotonic_and_recover_without_sleep() {
        let _serial = TEST_GATE_LOCK.lock().fixture("gate lock");
        let cap = crate::wire::MAX_PROVIDER_DELAY;
        assert_eq!(cap, Duration::from_secs(86_400));
        let identity = IdentifyingUserAgent::new("test/1 (tests@example.org)").fixture("identity");
        for delay in [
            Duration::ZERO,
            Duration::from_secs(1),
            Duration::from_secs(86_399),
            cap,
            Duration::from_secs(86_401),
            Duration::from_secs(u64::MAX),
            Duration::MAX,
        ] {
            reset_test_gate();
            let mut attempt = OfficialApiGate::new(identity).begin().fixture("attempt");
            let expected = if delay > cap {
                Err(ScheduleError::Overflow)
            } else {
                Ok(())
            };
            assert_eq!(attempt.defer(delay), expected);
            assert_eq!(attempt.delay, delay.min(cap).max(API_REQUEST_INTERVAL));
            assert_eq!(attempt.defer(Duration::ZERO), Ok(()));
            assert_eq!(attempt.delay, delay.min(cap).max(API_REQUEST_INTERVAL));
            let now = EPOCH.get().fixture("epoch").elapsed();
            drop(attempt);
            {
                let mut state = SCHEDULE.lock().fixture("state");
                assert!(!state.active);
                assert!(!state.poisoned);
                let Err(ScheduleError::Wait(wait)) = state.schedule.try_start(now) else {
                    unreachable!("quiet interval must remain active");
                };
                assert!(wait >= delay.min(cap).max(API_REQUEST_INTERVAL));
                // Drive the real scheduler at its exact deadline without a 24h sleep.
                assert_eq!(
                    state
                        .schedule
                        .try_start(now.checked_add(wait).fixture("deadline")),
                    Ok(())
                );
            }
        }
        reset_test_gate();
    }
}
