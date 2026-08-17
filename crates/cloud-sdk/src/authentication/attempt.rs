use core::sync::atomic::{AtomicU32, Ordering};

const REJECTED: u32 = 1;
const DISPATCHING: u32 = 1 << 1;
const GENERATION_SHIFT: u32 = 2;
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
///
/// Owner identity is deliberately not hashable because exposing it to a
/// caller-supplied hasher could disclose a process address.
///
/// ```compile_fail
/// use cloud_sdk::authentication::CredentialAttempt;
/// fn require_hash<T: core::hash::Hash>() {}
/// require_hash::<CredentialAttempt<'static>>();
/// ```
#[derive(Clone, Copy)]
pub struct CredentialAttempt<'a> {
    owner: &'a SharedCredentialAttemptState,
    generation: CredentialAttemptGeneration,
}

impl CredentialAttempt<'_> {
    /// Returns the generation used by this attempt.
    #[must_use]
    pub const fn generation(self) -> CredentialAttemptGeneration {
        self.generation
    }
}

impl core::fmt::Debug for CredentialAttempt<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CredentialAttempt")
            .field("owner", &"[bound]")
            .field("generation", &self.generation)
            .finish()
    }
}

impl PartialEq for CredentialAttempt<'_> {
    fn eq(&self, other: &Self) -> bool {
        core::ptr::eq(self.owner, other.owner) && self.generation == other.generation
    }
}

impl Eq for CredentialAttempt<'_> {}

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
    /// The attempt belongs to another credential lifecycle.
    ForeignState,
    /// Authentication rejection closed the current generation.
    GenerationRejected,
    /// The supplied generation is no longer current.
    StaleGeneration,
    /// Explicit reconfirmation is valid only after authentication rejection.
    ReconfirmationNotRequired,
    /// Another request is already using this credential generation.
    DispatchBusy,
    /// The bounded monotonic generation cannot advance without wrapping.
    GenerationExhausted,
}

impl_static_error!(CredentialAttemptError,
    Self::ForeignState => "credential attempt belongs to another state",
    Self::GenerationRejected => "credential attempt generation was rejected",
    Self::StaleGeneration => "credential attempt generation is stale",
    Self::ReconfirmationNotRequired => "credential attempt generation is still open",
    Self::DispatchBusy => "credential attempt generation already has an in-flight request",
    Self::GenerationExhausted => "credential attempt generation is exhausted",
);

/// Caller-owned concurrent lockout state for one credential lifecycle.
///
/// Cloned clients share this object by reference. Multiple attempt proofs may
/// begin on one open generation, but [`Self::reserve_dispatch`] admits only one
/// in-flight request. The first authentication rejection closes that
/// generation for every later execution. Only replacement credentials or an
/// explicit [`CredentialReconfirmation`] advance to a new open generation.
pub struct SharedCredentialAttemptState {
    packed: AtomicU32,
}

/// Exclusive admission for one in-flight credential-generation dispatch.
///
/// [`Self::complete`] releases admission after classification. Dropping an
/// unclassified guard rejects the generation, including async cancellation.
/// Authentication rejection must be recorded through [`Self::reject`] while
/// the guard is held.
#[must_use]
pub struct CredentialDispatchGuard<'a> {
    state: &'a SharedCredentialAttemptState,
    generation: CredentialAttemptGeneration,
    classified: bool,
}

impl CredentialDispatchGuard<'_> {
    /// Closes the guarded generation after authentication rejection.
    pub fn reject(&self) -> Result<(), CredentialAttemptError> {
        self.state.reject_dispatched_generation(self.generation)
    }

    /// Returns the exclusively admitted generation.
    #[must_use]
    pub const fn generation(&self) -> CredentialAttemptGeneration {
        self.generation
    }

    /// Marks response classification complete and releases dispatch admission.
    pub fn complete(mut self) {
        self.state.release_dispatch(self.generation, false);
        self.classified = true;
    }
}

impl core::fmt::Debug for CredentialDispatchGuard<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CredentialDispatchGuard")
            .field("owner", &"[bound]")
            .field("generation", &self.generation)
            .finish()
    }
}

impl Drop for CredentialDispatchGuard<'_> {
    fn drop(&mut self) {
        if !self.classified {
            self.state.release_dispatch(self.generation, true);
        }
    }
}

impl SharedCredentialAttemptState {
    /// Creates one open initial credential generation.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            packed: AtomicU32::new(pack(CredentialAttemptGeneration::INITIAL, false, false)),
        }
    }

    /// Returns the current generation and status.
    #[must_use]
    pub fn observe(&self) -> (CredentialAttemptGeneration, CredentialAttemptStatus) {
        unpack(self.packed.load(Ordering::Acquire))
    }

    /// Begins execution only when the current generation remains open.
    pub fn begin(&self) -> Result<CredentialAttempt<'_>, CredentialAttemptError> {
        let (generation, status) = self.observe();
        if status == CredentialAttemptStatus::Rejected {
            return Err(CredentialAttemptError::GenerationRejected);
        }
        Ok(CredentialAttempt {
            owner: self,
            generation,
        })
    }

    /// Revalidates an attempt immediately before credential use.
    pub fn validate(&self, attempt: CredentialAttempt<'_>) -> Result<(), CredentialAttemptError> {
        self.validate_owner(attempt)?;
        self.validate_generation(attempt.generation)
    }

    /// Exclusively admits one dispatch for the supplied open generation.
    pub fn reserve_dispatch(
        &self,
        attempt: CredentialAttempt<'_>,
    ) -> Result<CredentialDispatchGuard<'_>, CredentialAttemptError> {
        self.validate_owner(attempt)?;
        self.reserve_generation(attempt.generation)
    }

    pub(crate) fn reserve_generation(
        &self,
        expected: CredentialAttemptGeneration,
    ) -> Result<CredentialDispatchGuard<'_>, CredentialAttemptError> {
        loop {
            let current = self.packed.load(Ordering::Acquire);
            let (generation, status) = unpack(current);
            if generation != expected {
                return Err(CredentialAttemptError::StaleGeneration);
            }
            if status == CredentialAttemptStatus::Rejected {
                return Err(CredentialAttemptError::GenerationRejected);
            }
            if current & DISPATCHING != 0 {
                return Err(CredentialAttemptError::DispatchBusy);
            }
            if self
                .packed
                .compare_exchange(
                    current,
                    current | DISPATCHING,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                return Ok(CredentialDispatchGuard {
                    state: self,
                    generation,
                    classified: false,
                });
            }
        }
    }

    pub(crate) fn validate_generation(
        &self,
        expected: CredentialAttemptGeneration,
    ) -> Result<(), CredentialAttemptError> {
        let (generation, status) = self.observe();
        if generation != expected {
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
    pub fn reject(&self, attempt: CredentialAttempt<'_>) -> Result<(), CredentialAttemptError> {
        self.validate_owner(attempt)?;
        self.reject_generation(attempt.generation)
    }

    pub(crate) fn reject_generation(
        &self,
        expected: CredentialAttemptGeneration,
    ) -> Result<(), CredentialAttemptError> {
        loop {
            let current = self.packed.load(Ordering::Acquire);
            let (generation, status) = unpack(current);
            if generation != expected {
                return Err(CredentialAttemptError::StaleGeneration);
            }
            if status == CredentialAttemptStatus::Rejected {
                return Ok(());
            }
            if current & DISPATCHING != 0 {
                return Err(CredentialAttemptError::DispatchBusy);
            }
            let next = pack(generation, true, false);
            if self
                .packed
                .compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(());
            }
        }
    }

    fn reject_dispatched_generation(
        &self,
        expected: CredentialAttemptGeneration,
    ) -> Result<(), CredentialAttemptError> {
        loop {
            let current = self.packed.load(Ordering::Acquire);
            let (generation, status) = unpack(current);
            if generation != expected {
                return Err(CredentialAttemptError::StaleGeneration);
            }
            if current & DISPATCHING == 0 {
                return Err(CredentialAttemptError::DispatchBusy);
            }
            if status == CredentialAttemptStatus::Rejected {
                return Ok(());
            }
            let next = current | REJECTED;
            if self
                .packed
                .compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(());
            }
        }
    }

    fn release_dispatch(&self, expected: CredentialAttemptGeneration, reject: bool) {
        loop {
            let current = self.packed.load(Ordering::Acquire);
            let (generation, _) = unpack(current);
            if generation != expected || current & DISPATCHING == 0 {
                return;
            }
            let mut next = current & !DISPATCHING;
            if reject {
                next |= REJECTED;
            }
            if self
                .packed
                .compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return;
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
            if current & DISPATCHING != 0 {
                return Err(CredentialAttemptError::DispatchBusy);
            }
            let next_generation = checked_next(generation)?;
            let next = pack(next_generation, false, false);
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
            if current & DISPATCHING != 0 {
                return Err(CredentialAttemptError::DispatchBusy);
            }
            let next_generation = checked_next(generation)?;
            let next = pack(next_generation, false, false);
            if self
                .packed
                .compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return Ok(next_generation);
            }
        }
    }

    fn validate_owner(&self, attempt: CredentialAttempt<'_>) -> Result<(), CredentialAttemptError> {
        if !core::ptr::eq(self, attempt.owner) {
            return Err(CredentialAttemptError::ForeignState);
        }
        Ok(())
    }

    #[cfg(test)]
    fn set_generation_for_test(&mut self, generation: u32, rejected: bool) {
        *self.packed.get_mut() = pack(CredentialAttemptGeneration(generation), rejected, false);
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

const fn pack(generation: CredentialAttemptGeneration, rejected: bool, dispatching: bool) -> u32 {
    (generation.0 << GENERATION_SHIFT) | (rejected as u32) | ((dispatching as u32) << 1)
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
#[path = "attempt/tests.rs"]
mod tests;
