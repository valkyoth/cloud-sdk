/// Provider progress supplied with one running observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgressObservation {
    /// The provider does not expose progress.
    Unavailable,
    /// Ordinary provider progress in `0..=100`.
    Percent(u8),
    /// Provider-declared phase reset in `0..=100`.
    Reset(u8),
}

/// Provider-specific progress transition policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgressPolicy {
    /// Ordinary progress must never move backwards and resets are forbidden.
    Nondecreasing,
    /// Explicit resets are admitted up to this hard count.
    ExplicitResets {
        /// Maximum accepted explicit phase resets.
        max_resets: u16,
    },
    /// Percent values remain validated but ordering is provider-defined.
    Unordered,
}

/// Validated progress transition exposed to backoff policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProgressChange {
    /// No progress value was supplied.
    Unavailable,
    /// This is the first tracked progress value.
    Initial,
    /// Progress did not change.
    Unchanged,
    /// Progress moved forward.
    Advanced,
    /// The provider explicitly reset progress for a new phase.
    Reset,
    /// Ordering is intentionally provider-defined.
    Unordered,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProgressError {
    Invalid,
    Regressed,
    ResetForbidden,
    ResetLimit,
}

pub(crate) struct ProgressTracker {
    policy: ProgressPolicy,
    last: Option<u8>,
    resets: u16,
}

impl ProgressTracker {
    pub(crate) const fn new(policy: ProgressPolicy) -> Self {
        Self {
            policy,
            last: None,
            resets: 0,
        }
    }

    pub(crate) fn observe(
        &mut self,
        observation: ProgressObservation,
    ) -> Result<ProgressChange, ProgressError> {
        let (value, explicit_reset) = match observation {
            ProgressObservation::Unavailable => return Ok(ProgressChange::Unavailable),
            ProgressObservation::Percent(value) => (value, false),
            ProgressObservation::Reset(value) => (value, true),
        };
        if value > 100 {
            return Err(ProgressError::Invalid);
        }
        if self.policy == ProgressPolicy::Unordered {
            self.last = Some(value);
            return Ok(ProgressChange::Unordered);
        }
        if explicit_reset {
            let ProgressPolicy::ExplicitResets { max_resets } = self.policy else {
                return Err(ProgressError::ResetForbidden);
            };
            let resets = self
                .resets
                .checked_add(1)
                .ok_or(ProgressError::ResetLimit)?;
            if resets > max_resets {
                return Err(ProgressError::ResetLimit);
            }
            self.resets = resets;
            self.last = Some(value);
            return Ok(ProgressChange::Reset);
        }
        let change = match self.last {
            None => ProgressChange::Initial,
            Some(last) if value < last => return Err(ProgressError::Regressed),
            Some(last) if value == last => ProgressChange::Unchanged,
            Some(_) => ProgressChange::Advanced,
        };
        self.last = Some(value);
        Ok(change)
    }
}
