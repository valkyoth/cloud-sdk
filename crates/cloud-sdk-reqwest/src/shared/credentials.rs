use core::fmt;
use std::sync::{Arc, RwLock};

use cloud_sdk::authentication::{
    CredentialGeneration, CredentialGenerationError, CredentialLifetime, CredentialLifetimeState,
    CredentialTimestamp, RefreshHandoff,
};
use cloud_sdk_sanitization::SecretBuffer;

use super::BearerToken;

mod error;

pub use error::{
    CredentialStateError, CredentialUpdateError, RefreshHandoffError, TokenRefreshError,
    TokenRotationError,
};

struct VersionedToken {
    generation: CredentialGeneration,
    token: BearerToken,
    lifetime: Option<CredentialLifetime>,
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
    pub fn refresh_handoff(&self) -> Result<BearerRefreshHandoff, RefreshHandoffError> {
        if self.current.lifetime.is_some() {
            return Err(RefreshHandoffError::ExplicitTimeRequired);
        }
        Ok(self.new_handoff())
    }

    /// Creates a refresh handoff only inside an expiring token's refresh window.
    pub fn refresh_handoff_at(
        &self,
        now: CredentialTimestamp,
    ) -> Result<BearerRefreshHandoff, RefreshHandoffError> {
        let lifetime = self
            .current
            .lifetime
            .ok_or(RefreshHandoffError::LifetimeNotConfigured)?;
        match lifetime.state_at(now) {
            CredentialLifetimeState::ClockRollback => Err(RefreshHandoffError::ClockRollback),
            CredentialLifetimeState::Fresh => Err(RefreshHandoffError::RefreshNotRequired),
            CredentialLifetimeState::RefreshRequired => Ok(self.new_handoff()),
            CredentialLifetimeState::Expired => Err(RefreshHandoffError::CredentialExpired),
        }
    }

    /// Returns the caller-clock lifetime for an expiring credential.
    #[must_use]
    pub fn lifetime(&self) -> Option<CredentialLifetime> {
        self.current.lifetime
    }

    fn new_handoff(&self) -> BearerRefreshHandoff {
        BearerRefreshHandoff {
            lineage: Arc::clone(&self.lineage),
            expected: self.generation().refresh_handoff(),
        }
    }

    pub(crate) fn header_value(&self) -> Result<reqwest::header::HeaderValue, ()> {
        self.current.token.header_value()
    }

    #[cfg(test)]
    pub(crate) fn header_value_with_drop_probe(&self) -> Result<reqwest::header::HeaderValue, ()> {
        self.current.token.header_value_with_drop_probe()
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
    pub(crate) fn new(token: BearerToken, lifetime: Option<CredentialLifetime>) -> Self {
        Self {
            lineage: Arc::new(CredentialLineage),
            current: RwLock::new(Arc::new(VersionedToken {
                generation: CredentialGeneration::INITIAL,
                token,
                lifetime,
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
            replace_current(&mut current, token, None).map_err(map_update_failure)?
        };
        let (retired, generation) = retired;
        drop(retired);
        Ok(generation)
    }

    pub(crate) fn rotate_with_lifetime(
        &self,
        token: BearerToken,
        lifetime: CredentialLifetime,
    ) -> Result<CredentialGeneration, CredentialUpdateError> {
        let retired = {
            let mut current = self.write_current();
            replace_current(&mut current, token, Some(lifetime)).map_err(map_update_failure)?
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
            replace_current(&mut current, token, None).map_err(map_refresh_update)?
        };
        let (retired, generation) = retired;
        drop(retired);
        Ok(generation)
    }

    pub(crate) fn refresh_with_lifetime(
        &self,
        handoff: BearerRefreshHandoff,
        token: BearerToken,
        lifetime: CredentialLifetime,
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        if !Arc::ptr_eq(&self.lineage, &handoff.lineage) {
            return Err(TokenRefreshError::CredentialMismatch);
        }
        let retired = {
            let mut current = self.write_current();
            if handoff.expected_generation() != current.generation {
                return Err(TokenRefreshError::StaleGeneration);
            }
            replace_current(&mut current, token, Some(lifetime)).map_err(map_refresh_update)?
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

    pub(crate) fn rotate_from_mut_bytes_with_lifetime(
        &self,
        source: &mut [u8],
        lifetime: CredentialLifetime,
    ) -> Result<CredentialGeneration, TokenRotationError> {
        let token =
            BearerToken::from_mut_bytes(source).map_err(TokenRotationError::TokenRejected)?;
        self.rotate_with_lifetime(token, lifetime)
            .map_err(map_rotation_update)
    }

    pub(crate) fn rotate_from_secret_buffer_with_lifetime(
        &self,
        source: SecretBuffer<'_>,
        lifetime: CredentialLifetime,
    ) -> Result<CredentialGeneration, TokenRotationError> {
        let token =
            BearerToken::from_secret_buffer(source).map_err(TokenRotationError::TokenRejected)?;
        self.rotate_with_lifetime(token, lifetime)
            .map_err(map_rotation_update)
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

    pub(crate) fn refresh_from_mut_bytes_with_lifetime(
        &self,
        handoff: BearerRefreshHandoff,
        source: &mut [u8],
        lifetime: CredentialLifetime,
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        let token =
            BearerToken::from_mut_bytes(source).map_err(TokenRefreshError::TokenRejected)?;
        self.refresh_with_lifetime(handoff, token, lifetime)
    }

    pub(crate) fn refresh_from_secret_buffer_with_lifetime(
        &self,
        handoff: BearerRefreshHandoff,
        source: SecretBuffer<'_>,
        lifetime: CredentialLifetime,
    ) -> Result<CredentialGeneration, TokenRefreshError> {
        let token =
            BearerToken::from_secret_buffer(source).map_err(TokenRefreshError::TokenRejected)?;
        self.refresh_with_lifetime(handoff, token, lifetime)
    }
}

fn map_rotation_update(error: CredentialUpdateError) -> TokenRotationError {
    match error {
        CredentialUpdateError::StateUnavailable => TokenRotationError::StateUnavailable,
        CredentialUpdateError::GenerationExhausted => TokenRotationError::GenerationExhausted,
        CredentialUpdateError::LifetimeRequired => TokenRotationError::LifetimeRequired,
        CredentialUpdateError::LifetimeForbidden => TokenRotationError::LifetimeForbidden,
    }
}

impl fmt::Debug for CredentialStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CredentialStore([redacted])")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CredentialUpdateFailure {
    GenerationExhausted,
    LifetimeRequired,
    LifetimeForbidden,
}

fn map_update_failure(error: CredentialUpdateFailure) -> CredentialUpdateError {
    match error {
        CredentialUpdateFailure::GenerationExhausted => CredentialUpdateError::GenerationExhausted,
        CredentialUpdateFailure::LifetimeRequired => CredentialUpdateError::LifetimeRequired,
        CredentialUpdateFailure::LifetimeForbidden => CredentialUpdateError::LifetimeForbidden,
    }
}

fn map_refresh_update(error: CredentialUpdateFailure) -> TokenRefreshError {
    match error {
        CredentialUpdateFailure::GenerationExhausted => TokenRefreshError::GenerationExhausted,
        CredentialUpdateFailure::LifetimeRequired => TokenRefreshError::LifetimeRequired,
        CredentialUpdateFailure::LifetimeForbidden => TokenRefreshError::LifetimeForbidden,
    }
}

fn replace_current(
    current: &mut Arc<VersionedToken>,
    token: BearerToken,
    lifetime: Option<CredentialLifetime>,
) -> Result<(Arc<VersionedToken>, CredentialGeneration), CredentialUpdateFailure> {
    match (current.lifetime, lifetime) {
        (Some(_), None) => return Err(CredentialUpdateFailure::LifetimeRequired),
        (None, Some(_)) => return Err(CredentialUpdateFailure::LifetimeForbidden),
        (Some(_), Some(_)) | (None, None) => {}
    }
    let generation =
        current
            .generation
            .checked_next()
            .map_err(|_error: CredentialGenerationError| {
                CredentialUpdateFailure::GenerationExhausted
            })?;
    let replacement = Arc::new(VersionedToken {
        generation,
        token,
        lifetime,
    });
    Ok((core::mem::replace(current, replacement), generation))
}

#[cfg(test)]
mod tests;
