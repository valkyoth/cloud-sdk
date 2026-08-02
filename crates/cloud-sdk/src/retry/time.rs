//! Monotonic retry-budget values, distinct from wall-clock observations.

/// Caller-observed monotonic duration in implementation-defined ticks.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct MonotonicDuration(u64);

impl MonotonicDuration {
    /// Creates a duration in the caller's consistent monotonic tick unit.
    #[must_use]
    pub const fn new(ticks: u64) -> Self {
        Self(ticks)
    }

    /// Returns the caller-defined monotonic ticks.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Caller-observed monotonic instant in implementation-defined ticks.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MonotonicInstant(u64);

impl MonotonicInstant {
    /// Creates an instant in the same tick domain used by retry durations.
    #[must_use]
    pub const fn new(ticks: u64) -> Self {
        Self(ticks)
    }

    /// Returns the caller-defined monotonic ticks.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    pub(crate) const fn checked_duration_since(self, earlier: Self) -> Option<MonotonicDuration> {
        match self.0.checked_sub(earlier.0) {
            Some(value) => Some(MonotonicDuration::new(value)),
            None => None,
        }
    }

    pub(crate) const fn checked_add(self, duration: MonotonicDuration) -> Option<Self> {
        match self.0.checked_add(duration.get()) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}
