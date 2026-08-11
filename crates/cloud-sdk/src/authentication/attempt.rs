use core::sync::atomic::{AtomicU32, Ordering};

const REJECTED: u32 = 1;
const GENERATION_SHIFT: u32 = 1;
const MAX_GENERATION: u32 = u32::MAX >> GENERATION_SHIFT;

/// Monotonic identity of one credential-attempt generation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CredentialAttemptGeneration(u32);

impl CredentialAttemptGeneration {
    /// Initial generation assigned to newly admitted credentials.
    pub const INITIAL: Self = Self(1);

    /// Returns the generation as a nonzero integer.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Proof that one credential generation was open when execution began.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CredentialAttempt {
    generation: CredentialAttemptGeneration,
}

impl CredentialAttempt {
    /// Returns the generation used by this attempt.
    #[must_use]
    pub const fn generation(self) -> CredentialAttemptGeneration {
        self.generation
    }
}

/// Explicit caller acknowledgement for retrying unchanged credentials.
///
/// Constructing this token must be an operator-level decision. Automatic
/// retry, pagination, polling, and client policy must not create it.
#[derive(Debug)]
pub struct CredentialReconfirmation {
    _private: (),
}

impl CredentialReconfirmation {
    /// Explicitly acknowledges reuse of the same credential material.
    #[must_use]
    pub const fn acknowledge_same_credentials() -> Self {
        Self { _private: () }
    }
}

/// Observable credential-attempt lifecycle state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialAttemptStatus {
    /// The current generation may begin new executions.
    Open,
    /// Authentication rejection closed the current generation.
    Rejected,
}

/// Credential-attempt lifecycle transition failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialAttemptError {
    /// Authentication rejection closed the current generation.
    GenerationRejected,
    /// The supplied generation is no longer current.
    StaleGeneration,
    /// Explicit reconfirmation is valid only after authentication rejection.
    ReconfirmationNotRequired,
    /// The bounded monotonic generation cannot advance without wrapping.
    GenerationExhausted,
}

impl_static_error!(CredentialAttemptError,
    Self::GenerationRejected => "credential attempt generation was rejected",
    Self::StaleGeneration => "credential attempt generation is stale",
    Self::ReconfirmationNotRequired => "credential attempt generation is still open",
    Self::GenerationExhausted => "credential attempt generation is exhausted",
);

/// Caller-owned concurrent lockout state for one credential lifecycle.
///
/// Cloned clients share this object by reference. Multiple attempts may begin
/// on one open generation, but the first authentication rejection closes that
/// generation for every later execution. Only replacement credentials or an
/// explicit [`CredentialReconfirmation`] advance to a new open generation.
pub struct SharedCredentialAttemptState {
    packed: AtomicU32,
}

impl SharedCredentialAttemptState {
    /// Creates one open initial credential generation.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            packed: AtomicU32::new(pack(CredentialAttemptGeneration::INITIAL, false)),
        }
    }

    /// Returns the current generation and status.
    #[must_use]
    pub fn observe(&self) -> (CredentialAttemptGeneration, CredentialAttemptStatus) {
        unpack(self.packed.load(Ordering::Acquire))
    }

    /// Begins execution only when the current generation remains open.
    pub fn begin(&self) -> Result<CredentialAttempt, CredentialAttemptError> {
        let (generation, status) = self.observe();
        if status == CredentialAttemptStatus::Rejected {
            return Err(CredentialAttemptError::GenerationRejected);
        }
        Ok(CredentialAttempt { generation })
    }

    /// Revalidates an attempt immediately before credential use.
    pub fn validate(&self, attempt: CredentialAttempt) -> Result<(), CredentialAttemptError> {
        let (generation, status) = self.observe();
        if generation != attempt.generation {
            return Err(CredentialAttemptError::StaleGeneration);
        }
        if status == CredentialAttemptStatus::Rejected {
            return Err(CredentialAttemptError::GenerationRejected);
        }
        Ok(())
    }

    /// Closes the exact generation that received authentication rejection.
    ///
    /// Repeated concurrent rejection reports for the same generation are
    /// idempotent. A stale report cannot close replacement credentials.
    pub fn reject(&self, attempt: CredentialAttempt) -> Result<(), CredentialAttemptError> {
        loop {
            let current = self.packed.load(Ordering::Acquire);
            let (generation, status) = unpack(current);
            if generation != attempt.generation {
                return Err(CredentialAttemptError::StaleGeneration);
            }
            if status == CredentialAttemptStatus::Rejected {
                return Ok(());
            }
            let next = pack(generation, true);
            if self
                .packed
                .compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(());
            }
        }
    }

    /// Opens a new generation after replacement credentials were admitted.
    pub fn replace(
        &self,
        expected: CredentialAttemptGeneration,
    ) -> Result<CredentialAttemptGeneration, CredentialAttemptError> {
        self.advance(expected)
    }

    /// Opens a new generation after explicit unchanged-credential confirmation.
    pub fn reconfirm(
        &self,
        expected: CredentialAttemptGeneration,
        _acknowledgement: CredentialReconfirmation,
    ) -> Result<CredentialAttemptGeneration, CredentialAttemptError> {
        loop {
            let current = self.packed.load(Ordering::Acquire);
            let (generation, status) = unpack(current);
            if generation != expected {
                return Err(CredentialAttemptError::StaleGeneration);
            }
            if status != CredentialAttemptStatus::Rejected {
                return Err(CredentialAttemptError::ReconfirmationNotRequired);
            }
            let next_generation = checked_next(generation)?;
            let next = pack(next_generation, false);
            if self
                .packed
                .compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(next_generation);
            }
        }
    }

    fn advance(
        &self,
        expected: CredentialAttemptGeneration,
    ) -> Result<CredentialAttemptGeneration, CredentialAttemptError> {
        loop {
            let current = self.packed.load(Ordering::Acquire);
            let (generation, _) = unpack(current);
            if generation != expected {
                return Err(CredentialAttemptError::StaleGeneration);
            }
            let next_generation = checked_next(generation)?;
            let next = pack(next_generation, false);
            if self
                .packed
                .compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(next_generation);
            }
        }
    }

    #[cfg(test)]
    fn set_generation_for_test(&mut self, generation: u32, rejected: bool) {
        *self.packed.get_mut() = pack(CredentialAttemptGeneration(generation), rejected);
    }
}

fn checked_next(
    generation: CredentialAttemptGeneration,
) -> Result<CredentialAttemptGeneration, CredentialAttemptError> {
    generation
        .0
        .checked_add(1)
        .filter(|candidate| *candidate <= MAX_GENERATION)
        .map(CredentialAttemptGeneration)
        .ok_or(CredentialAttemptError::GenerationExhausted)
}

impl Default for SharedCredentialAttemptState {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for SharedCredentialAttemptState {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (generation, status) = self.observe();
        formatter
            .debug_struct("SharedCredentialAttemptState")
            .field("generation", &generation)
            .field("status", &status)
            .finish()
    }
}

const fn pack(generation: CredentialAttemptGeneration, rejected: bool) -> u32 {
    (generation.0 << GENERATION_SHIFT) | (rejected as u32)
}

const fn unpack(value: u32) -> (CredentialAttemptGeneration, CredentialAttemptStatus) {
    let generation = CredentialAttemptGeneration(value >> GENERATION_SHIFT);
    let status = if value & REJECTED == 0 {
        CredentialAttemptStatus::Open
    } else {
        CredentialAttemptStatus::Rejected
    };
    (generation, status)
}

#[cfg(test)]
mod tests {
    use super::{
        CredentialAttemptError, CredentialAttemptGeneration, CredentialAttemptStatus,
        CredentialReconfirmation, MAX_GENERATION, SharedCredentialAttemptState,
    };

    #[test]
    fn rejection_closes_one_generation_until_replaced_or_reconfirmed() {
        let state = SharedCredentialAttemptState::new();
        let first = state
            .begin()
            .unwrap_or_else(|_| unreachable!("initial credential generation was closed"));
        assert_eq!(first.generation(), CredentialAttemptGeneration::INITIAL);
        assert_eq!(
            state.reconfirm(
                first.generation(),
                CredentialReconfirmation::acknowledge_same_credentials(),
            ),
            Err(CredentialAttemptError::ReconfirmationNotRequired)
        );
        assert_eq!(state.reject(first), Ok(()));
        assert_eq!(
            state.validate(first),
            Err(CredentialAttemptError::GenerationRejected)
        );
        assert_eq!(state.reject(first), Ok(()));
        assert_eq!(
            state.begin(),
            Err(CredentialAttemptError::GenerationRejected)
        );

        let second = state
            .reconfirm(
                first.generation(),
                CredentialReconfirmation::acknowledge_same_credentials(),
            )
            .unwrap_or_else(|_| unreachable!("explicit reconfirmation was rejected"));
        assert_eq!(second.get(), 2);
        assert!(state.begin().is_ok());

        let third = state
            .replace(second)
            .unwrap_or_else(|_| unreachable!("replacement generation was rejected"));
        assert_eq!(third.get(), 3);
        assert!(state.begin().is_ok());
    }

    #[test]
    fn stale_transitions_cannot_close_or_reopen_replacement_credentials() {
        let state = SharedCredentialAttemptState::new();
        let stale = state
            .begin()
            .unwrap_or_else(|_| unreachable!("initial credential generation was closed"));
        let current = state
            .replace(stale.generation())
            .unwrap_or_else(|_| unreachable!("replacement generation was rejected"));
        assert_eq!(
            state.reject(stale),
            Err(CredentialAttemptError::StaleGeneration)
        );
        assert_eq!(
            state.validate(stale),
            Err(CredentialAttemptError::StaleGeneration)
        );
        assert_eq!(
            state.replace(stale.generation()),
            Err(CredentialAttemptError::StaleGeneration)
        );
        assert_eq!(
            state.reconfirm(
                stale.generation(),
                CredentialReconfirmation::acknowledge_same_credentials(),
            ),
            Err(CredentialAttemptError::StaleGeneration)
        );
        assert_eq!(state.observe(), (current, CredentialAttemptStatus::Open));
    }

    #[test]
    fn generation_exhaustion_fails_closed_without_wrapping() {
        let mut state = SharedCredentialAttemptState::new();
        state.set_generation_for_test(MAX_GENERATION, true);
        let generation = state.observe().0;
        assert_eq!(
            state.replace(generation),
            Err(CredentialAttemptError::GenerationExhausted)
        );
        assert_eq!(
            state.reconfirm(
                generation,
                CredentialReconfirmation::acknowledge_same_credentials(),
            ),
            Err(CredentialAttemptError::GenerationExhausted)
        );
        assert_eq!(
            state.observe(),
            (generation, CredentialAttemptStatus::Rejected)
        );
    }
}
