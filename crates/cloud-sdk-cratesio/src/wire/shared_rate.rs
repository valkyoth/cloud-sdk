use core::fmt;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

use super::{API_REQUEST_INTERVAL, ApiSchedule, IdentifyingUserAgent, ScheduleError};

static SCHEDULE: Mutex<ApiSchedule> = Mutex::new(ApiSchedule::new());
static EPOCH: OnceLock<Instant> = OnceLock::new();

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

    /// Tries one complete blocking exchange without sleeping or retrying.
    /// Concurrent attempts return Unavailable; early attempts return Wait.
    /// Callback panics poison the gate, failing all later calls closed.
    pub fn try_call<T, E>(
        &self,
        send: impl FnOnce(IdentifyingUserAgent<'_>) -> Result<T, E>,
    ) -> Result<T, OfficialCallError<E>> {
        let mut schedule = SCHEDULE
            .try_lock()
            .map_err(|_| OfficialCallError::Schedule(ScheduleError::Unavailable))?;
        let epoch = EPOCH.get_or_init(Instant::now);
        schedule
            .try_start(epoch.elapsed())
            .map_err(OfficialCallError::Schedule)?;
        let result = send(self.user_agent);
        let deadline = epoch
            .elapsed()
            .checked_add(API_REQUEST_INTERVAL)
            .ok_or(OfficialCallError::Schedule(ScheduleError::Overflow))?;
        schedule.defer_until(deadline);
        result.map_err(OfficialCallError::Transport)
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
