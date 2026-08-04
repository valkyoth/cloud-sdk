//! Caller-owned time observations at permit dispatch.

use super::PermitTimestamp;

/// Time source sampled inside permit execution immediately before dispatch.
///
/// The SDK owns no clock. Implementations must provide trustworthy Unix time
/// and must not move backward. Send-async execution additionally requires a
/// `Sync` clock because the returned future may move between threads.
pub trait PermitClock {
    /// Returns the current caller-observed time.
    fn now(&self) -> PermitTimestamp;
}

impl<F> PermitClock for F
where
    F: Fn() -> PermitTimestamp,
{
    fn now(&self) -> PermitTimestamp {
        self()
    }
}
