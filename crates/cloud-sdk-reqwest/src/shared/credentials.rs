use core::fmt;
use std::sync::{Arc, RwLock};

use cloud_sdk::authentication::{CredentialGeneration, CredentialGenerationError, RefreshHandoff};
use cloud_sdk_sanitization::SecretBuffer;

use super::{BearerToken, BearerTokenError};

/// Credential-state access failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialStateError {
    /// The short-lived credential-state lock could not be recovered.
    Unavailable,
}

impl_static_error!(CredentialStateError,
    Self::Unavailable => "credential state is unavailable",
);

/// Validated token rotation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialUpdateError {
    /// The credential state could not be changed.
    StateUnavailable,
    /// The monotonic credential generation cannot advance.
    GenerationExhausted,
}

impl_static_error!(CredentialUpdateError,
    Self::StateUnavailable => "credential state is unavailable",
    Self::GenerationExhausted => "credential generation is exhausted",
);

/// Bearer-token validation or rotation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenRotationError {
    /// The replacement bearer token was rejected before state changed.
    TokenRejected(BearerTokenError),
    /// The credential state could not be changed.
    StateUnavailable,
    /// The monotonic credential generation cannot advance.
    GenerationExhausted,
}

impl fmt::Display for TokenRotationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::TokenRejected(_) => "replacement bearer token was rejected",
            Self::StateUnavailable => "credential state is unavailable",
            Self::GenerationExhausted => "credential generation is exhausted",
        })
    }
}

impl core::error::Error for TokenRotationError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::TokenRejected(error) => Some(error),
            Self::StateUnavailable | Self::GenerationExhausted => None,
        }
    }
}

/// Compare-and-swap bearer refresh failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenRefreshError {
    /// The replacement bearer token was rejected before state changed.
    TokenRejected(BearerTokenError),
    /// A newer rotation or refresh superseded this handoff.
    StaleGeneration,
    /// The refresh handoff belongs to a different credential lifecycle.
    CredentialMismatch,
    /// The credential state could not be changed.
    StateUnavailable,
    /// The monotonic credential generation cannot advance.
    GenerationExhausted,
}

impl fmt::Display for TokenRefreshError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::TokenRejected(_) => "refreshed bearer token was rejected",
            Self::StaleGeneration => "credential refresh generation is stale",
            Self::CredentialMismatch => "credential refresh handoff belongs to another credential",
            Self::StateUnavailable => "credential state is unavailable",
            Self::GenerationExhausted => "credential generation is exhausted",
        })
    }
}

impl core::error::Error for TokenRefreshError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::TokenRejected(error) => Some(error),
            Self::StaleGeneration
            | Self::CredentialMismatch
            | Self::StateUnavailable
            | Self::GenerationExhausted => None,
        }
    }
}

struct VersionedToken {
    generation: CredentialGeneration,
    token: BearerToken,
}

struct CredentialLineage;

/// Store-bound handoff captured before external bearer refresh work.
///
/// The lineage is opaque and redacted. A handoff can update only the exact
/// credential lifecycle whose snapshot created it.
#[derive(Clone)]
pub struct BearerRefreshHandoff {
    lineage: Arc<CredentialLineage>,
    expected: RefreshHandoff,
}

impl BearerRefreshHandoff {
    /// Returns the generation that must still be current.
    #[must_use]
    pub fn expected_generation(&self) -> CredentialGeneration {
        self.expected.expected_generation()
    }
}

impl fmt::Debug for BearerRefreshHandoff {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BearerRefreshHandoff")
            .field("generation", &self.expected_generation())
            .field("lineage", &"[redacted]")
            .finish()
    }
}

/// Redacted snapshot metadata for an in-flight credential generation.
///
/// The internal token remains alive until this snapshot and all transport
/// copies are dropped, but no secret bytes are exposed through this API.
pub struct BearerCredentialSnapshot {
    lineage: Arc<CredentialLineage>,
    current: Arc<VersionedToken>,
}

impl BearerCredentialSnapshot {
    /// Returns the immutable generation captured by this snapshot.
    #[must_use]
    pub fn generation(&self) -> CredentialGeneration {
        self.current.generation
    }

    /// Creates a refresh handoff tied to this exact snapshot generation.
    #[must_use]
    pub fn refresh_handoff(&self) -> BearerRefreshHandoff {
        BearerRefreshHandoff {
            lineage: Arc::clone(&self.lineage),
            expected: self.generation().refresh_handoff(),
        }
    }

    pub(crate) fn header_value(&self) -> Result<reqwest::header::HeaderValue, ()> {
        self.current.token.header_value()
    }

    #[cfg(test)]
    pub(crate) fn owned_bytes(&self) -> &[u8] {
        self.current.token.owned_bytes()
    }
}

impl Clone for BearerCredentialSnapshot {
    fn clone(&self) -> Self {
        Self {
            lineage: Arc::clone(&self.lineage),
            current: Arc::clone(&self.current),
        }
    }
}

impl fmt::Debug for BearerCredentialSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BearerCredentialSnapshot")
            .field("generation", &self.generation())
            .field("credential", &"[redacted]")
            .finish()
    }
}

pub(crate) struct CredentialStore {
    lineage: Arc<CredentialLineage>,
    current: RwLock<Arc<VersionedToken>>,
}

impl CredentialStore {
    pub(crate) fn new(token: BearerToken) -> Self {
        Self {
            lineage: Arc::new(CredentialLineage),
            current: RwLock::new(Arc::new(VersionedToken {
                generation: CredentialGeneration::INITIAL,
                token,
            })),
        }
    }

    pub(crate) fn snapshot(&self) -> Result<BearerCredentialSnapshot, CredentialStateError> {
        let current = match self.current.read() {
            Ok(current) => current,
            Err(poisoned) => {
                self.current.clear_poison();
                poisoned.into_inner()
            }
        };
        Ok(BearerCredentialSnapshot {
            lineage: Arc::clone(&self.lineage),
            current: Arc::clone(&current),
        })
    }

    pub(crate) fn rotate(
        &self,
        token: BearerToken,
    ) -> Result<CredentialGeneration, CredentialUpdateError> {
        let retired = {
            let mut current = self.write_current();
            replace_current(&mut current, token)
                .map_err(|_| CredentialUpdateError::GenerationExhausted)?
        };
        let (retired, generation) = retired;
        drop(retired);
        Ok(generation)
    }

    pub(crate) fn refresh(
        &self,
        handoff: BearerRefreshHandoff,
        token: BearerToken,
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        if !Arc::ptr_eq(&self.lineage, &handoff.lineage) {
            return Err(TokenRefreshError::CredentialMismatch);
        }
        let retired = {
            let mut current = self.write_current();
            if handoff.expected_generation() != current.generation {
                return Err(TokenRefreshError::StaleGeneration);
            }
            replace_current(&mut current, token)
                .map_err(|_| TokenRefreshError::GenerationExhausted)?
        };
        let (retired, generation) = retired;
        drop(retired);
        Ok(generation)
    }

    fn write_current(&self) -> std::sync::RwLockWriteGuard<'_, Arc<VersionedToken>> {
        match self.current.write() {
            Ok(current) => current,
            Err(poisoned) => {
                self.current.clear_poison();
                poisoned.into_inner()
            }
        }
    }

    pub(crate) fn rotate_from_mut_bytes(
        &self,
        source: &mut [u8],
    ) -> Result<CredentialGeneration, TokenRotationError> {
        let token =
            BearerToken::from_mut_bytes(source).map_err(TokenRotationError::TokenRejected)?;
        self.rotate(token).map_err(map_rotation_update)
    }

    pub(crate) fn rotate_from_secret_buffer(
        &self,
        source: SecretBuffer<'_>,
    ) -> Result<CredentialGeneration, TokenRotationError> {
        let token =
            BearerToken::from_secret_buffer(source).map_err(TokenRotationError::TokenRejected)?;
        self.rotate(token).map_err(map_rotation_update)
    }

    pub(crate) fn refresh_from_mut_bytes(
        &self,
        handoff: BearerRefreshHandoff,
        source: &mut [u8],
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        let token =
            BearerToken::from_mut_bytes(source).map_err(TokenRefreshError::TokenRejected)?;
        self.refresh(handoff, token)
    }

    pub(crate) fn refresh_from_secret_buffer(
        &self,
        handoff: BearerRefreshHandoff,
        source: SecretBuffer<'_>,
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        let token =
            BearerToken::from_secret_buffer(source).map_err(TokenRefreshError::TokenRejected)?;
        self.refresh(handoff, token)
    }
}

fn map_rotation_update(error: CredentialUpdateError) -> TokenRotationError {
    match error {
        CredentialUpdateError::StateUnavailable => TokenRotationError::StateUnavailable,
        CredentialUpdateError::GenerationExhausted => TokenRotationError::GenerationExhausted,
    }
}

impl fmt::Debug for CredentialStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CredentialStore([redacted])")
    }
}

fn replace_current(
    current: &mut Arc<VersionedToken>,
    token: BearerToken,
) -> Result<(Arc<VersionedToken>, CredentialGeneration), CredentialGenerationError> {
    let generation = current.generation.checked_next()?;
    let replacement = Arc::new(VersionedToken { generation, token });
    Ok((core::mem::replace(current, replacement), generation))
}

#[cfg(test)]
mod tests;
