/// Monotonic generation of one credential state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CredentialGeneration(u64);

impl CredentialGeneration {
    /// Initial generation assigned to a newly admitted credential.
    pub const INITIAL: Self = Self(1);

    /// Returns the generation as a nonzero integer.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Creates an executor-neutral handoff for compare-and-swap refresh.
    #[must_use]
    pub const fn refresh_handoff(self) -> RefreshHandoff {
        RefreshHandoff { expected: self }
    }

    /// Advances the generation without wrapping.
    pub fn checked_next(self) -> Result<Self, CredentialGenerationError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(CredentialGenerationError::Exhausted)
    }

    #[cfg(test)]
    pub(super) const fn from_raw_for_test(value: u64) -> Self {
        Self(value)
    }
}

/// Opaque generation captured before an external refresh operation begins.
///
/// The handoff owns no credential, clock, future, executor, or secret-store
/// handle. A credential store uses it to reject stale refresh completion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RefreshHandoff {
    expected: CredentialGeneration,
}

impl RefreshHandoff {
    /// Returns the generation that must still be current.
    #[must_use]
    pub const fn expected_generation(self) -> CredentialGeneration {
        self.expected
    }
}

/// Credential-generation transition error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialGenerationError {
    /// The monotonic generation cannot advance without wrapping.
    Exhausted,
}

impl_static_error!(CredentialGenerationError,
    Self::Exhausted => "credential generation is exhausted",
);
