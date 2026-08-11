use alloc::sync::Arc;

use super::attempt::{
    CredentialAttemptError, CredentialAttemptGeneration, CredentialAttemptStatus,
    CredentialReconfirmation, SharedCredentialAttemptState,
};

/// Owned proof that one credential generation was open when execution began.
///
/// The proof retains opaque owner identity without exposing an address. It can
/// cross task boundaries without borrowing the credential owner.
pub struct OwnedCredentialAttempt {
    owner: Arc<SharedCredentialAttemptState>,
    generation: CredentialAttemptGeneration,
}

impl OwnedCredentialAttempt {
    /// Returns the generation used by this attempt.
    #[must_use]
    pub const fn generation(&self) -> CredentialAttemptGeneration {
        self.generation
    }
}

impl core::fmt::Debug for OwnedCredentialAttempt {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("OwnedCredentialAttempt")
            .field("owner", &"[bound]")
            .field("generation", &self.generation)
            .finish()
    }
}

impl PartialEq for OwnedCredentialAttempt {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.owner, &other.owner) && self.generation == other.generation
    }
}

impl Eq for OwnedCredentialAttempt {}

/// Allocation-backed credential lifecycle for attempts that cross task boundaries.
///
/// Creating the state allocates one shared lineage. Beginning an attempt only
/// clones that lineage and performs no new allocation.
pub struct OwnedCredentialAttemptState {
    state: Arc<SharedCredentialAttemptState>,
}

impl OwnedCredentialAttemptState {
    /// Creates one open initial credential generation and owned lineage.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: Arc::new(SharedCredentialAttemptState::new()),
        }
    }

    /// Returns the current generation and status.
    #[must_use]
    pub fn observe(&self) -> (CredentialAttemptGeneration, CredentialAttemptStatus) {
        self.state.observe()
    }

    /// Begins an owned attempt only when the current generation remains open.
    pub fn begin(&self) -> Result<OwnedCredentialAttempt, CredentialAttemptError> {
        let attempt = self.state.begin()?;
        Ok(OwnedCredentialAttempt {
            owner: Arc::clone(&self.state),
            generation: attempt.generation(),
        })
    }

    /// Revalidates owner identity and generation immediately before use.
    pub fn validate(&self, attempt: &OwnedCredentialAttempt) -> Result<(), CredentialAttemptError> {
        self.validate_owner(attempt)?;
        self.state.validate_generation(attempt.generation)
    }

    /// Closes the exact owned attempt generation after authentication rejection.
    pub fn reject(&self, attempt: &OwnedCredentialAttempt) -> Result<(), CredentialAttemptError> {
        self.validate_owner(attempt)?;
        self.state.reject_generation(attempt.generation)
    }

    /// Opens a new generation after replacement credentials were admitted.
    pub fn replace(
        &self,
        expected: CredentialAttemptGeneration,
    ) -> Result<CredentialAttemptGeneration, CredentialAttemptError> {
        self.state.replace(expected)
    }

    /// Opens a new generation after explicit unchanged-credential confirmation.
    pub fn reconfirm(
        &self,
        expected: CredentialAttemptGeneration,
        acknowledgement: CredentialReconfirmation,
    ) -> Result<CredentialAttemptGeneration, CredentialAttemptError> {
        self.state.reconfirm(expected, acknowledgement)
    }

    fn validate_owner(
        &self,
        attempt: &OwnedCredentialAttempt,
    ) -> Result<(), CredentialAttemptError> {
        if !Arc::ptr_eq(&self.state, &attempt.owner) {
            return Err(CredentialAttemptError::ForeignState);
        }
        Ok(())
    }
}

impl Default for OwnedCredentialAttemptState {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for OwnedCredentialAttemptState {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (generation, status) = self.observe();
        formatter
            .debug_struct("OwnedCredentialAttemptState")
            .field("owner", &"[bound]")
            .field("generation", &generation)
            .field("status", &status)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::OwnedCredentialAttemptState;
    use crate::authentication::{CredentialAttemptError, CredentialAttemptGeneration};

    #[test]
    fn owned_attempts_reject_foreign_and_stale_lineages() {
        let owner_a = OwnedCredentialAttemptState::new();
        let owner_b = OwnedCredentialAttemptState::new();
        let attempt = owner_a
            .begin()
            .unwrap_or_else(|_| unreachable!("initial owned attempt was rejected"));

        assert_eq!(
            owner_b.validate(&attempt),
            Err(CredentialAttemptError::ForeignState)
        );
        assert_eq!(
            owner_b.reject(&attempt),
            Err(CredentialAttemptError::ForeignState)
        );
        let replacement = owner_a
            .replace(CredentialAttemptGeneration::INITIAL)
            .unwrap_or_else(|_| unreachable!("owned replacement was rejected"));
        assert_eq!(replacement.get(), 2);
        assert_eq!(
            owner_a.reject(&attempt),
            Err(CredentialAttemptError::StaleGeneration)
        );
    }
}
