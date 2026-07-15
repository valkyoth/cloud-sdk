use core::fmt;
use std::sync::{Arc, RwLock};

use cloud_sdk_sanitization::SecretBuffer;

use super::{BearerToken, BearerTokenError};

/// Credential-state access failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialStateError {
    /// The short-lived credential-state lock was poisoned by a panic.
    Unavailable,
}

impl_static_error!(CredentialStateError,
    Self::Unavailable => "credential state is unavailable",
);

/// Bearer-token validation or rotation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenRotationError {
    /// The replacement bearer token was rejected before state changed.
    TokenRejected(BearerTokenError),
    /// The credential state could not be read or changed.
    StateUnavailable,
}

impl fmt::Display for TokenRotationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::TokenRejected(_) => "replacement bearer token was rejected",
            Self::StateUnavailable => "credential state is unavailable",
        })
    }
}

impl core::error::Error for TokenRotationError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::TokenRejected(error) => Some(error),
            Self::StateUnavailable => None,
        }
    }
}

pub(crate) struct CredentialStore {
    current: RwLock<Arc<BearerToken>>,
}

impl CredentialStore {
    pub(crate) fn new(token: BearerToken) -> Self {
        Self {
            current: RwLock::new(Arc::new(token)),
        }
    }

    pub(crate) fn snapshot(&self) -> Result<Arc<BearerToken>, CredentialStateError> {
        let current = match self.current.read() {
            Ok(current) => current,
            Err(poisoned) => {
                self.current.clear_poison();
                poisoned.into_inner()
            }
        };
        Ok(Arc::clone(&current))
    }

    pub(crate) fn rotate(&self, token: BearerToken) -> Result<(), CredentialStateError> {
        let replacement = Arc::new(token);
        let retired = {
            let mut current = match self.current.write() {
                Ok(current) => current,
                Err(poisoned) => {
                    self.current.clear_poison();
                    poisoned.into_inner()
                }
            };
            core::mem::replace(&mut *current, replacement)
        };
        drop(retired);
        Ok(())
    }

    pub(crate) fn rotate_from_mut_bytes(
        &self,
        source: &mut [u8],
    ) -> Result<(), TokenRotationError> {
        let token =
            BearerToken::from_mut_bytes(source).map_err(TokenRotationError::TokenRejected)?;
        self.rotate(token)
            .map_err(|_| TokenRotationError::StateUnavailable)
    }

    pub(crate) fn rotate_from_secret_buffer(
        &self,
        source: SecretBuffer<'_>,
    ) -> Result<(), TokenRotationError> {
        let token =
            BearerToken::from_secret_buffer(source).map_err(TokenRotationError::TokenRejected)?;
        self.rotate(token)
            .map_err(|_| TokenRotationError::StateUnavailable)
    }
}

impl fmt::Debug for CredentialStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CredentialStore([redacted])")
    }
}

#[cfg(test)]
mod tests {
    use std::boxed::Box;
    use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use cloud_sdk_sanitization::SecretBuffer;

    use super::{BearerToken, CredentialStore, TokenRotationError};

    #[test]
    fn mutable_and_guarded_sources_clear_on_success_and_failure() {
        let mut valid = *b"replacement";
        let token = BearerToken::from_mut_bytes(&mut valid);
        assert!(token.is_ok());
        assert_eq!(valid, [0; 11]);

        let mut invalid = *b"bad token";
        assert!(BearerToken::from_mut_bytes(&mut invalid).is_err());
        assert_eq!(invalid, [0; 9]);

        let mut guarded = *b"guarded-token";
        let token = BearerToken::from_secret_buffer(SecretBuffer::new(&mut guarded));
        assert!(token.is_ok());
        assert_eq!(guarded, [0; 13]);
    }

    #[test]
    fn rejected_rotation_preserves_the_active_token_and_clears_input() {
        let Ok(active) = BearerToken::new("active-token") else {
            return;
        };
        let store = CredentialStore::new(active);
        let mut rejected = *b"bad token";
        assert!(matches!(
            store.rotate_from_mut_bytes(&mut rejected),
            Err(TokenRotationError::TokenRejected(_))
        ));
        assert_eq!(rejected, [0; 9]);
        let snapshot = store.snapshot();
        assert!(snapshot.is_ok());
        if let Ok(snapshot) = snapshot {
            assert_eq!(snapshot.owned_bytes(), b"Bearer active-token");
        }
    }

    #[test]
    fn retired_token_drops_only_after_the_last_in_flight_snapshot() {
        let drops = Arc::new(AtomicUsize::new(0));
        let active = BearerToken::with_drop_probe("old-token", Arc::clone(&drops));
        let Ok(active) = active else { return };
        let store = CredentialStore::new(active);
        let old_snapshot = store.snapshot();
        let Ok(old_snapshot) = old_snapshot else {
            return;
        };
        let Ok(replacement) = BearerToken::new("new-token") else {
            return;
        };

        assert!(store.rotate(replacement).is_ok());
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        assert_eq!(old_snapshot.owned_bytes(), b"Bearer old-token");
        let new_snapshot = store.snapshot();
        assert!(new_snapshot.is_ok());
        if let Ok(new_snapshot) = new_snapshot {
            assert_eq!(new_snapshot.owned_bytes(), b"Bearer new-token");
        }
        drop(old_snapshot);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn poisoned_state_recovers_for_snapshots_and_rotations() {
        let Ok(active) = BearerToken::new("active-token") else {
            return;
        };
        let store = CredentialStore::new(active);

        poison_state(&store);
        let snapshot = store.snapshot();
        assert!(snapshot.is_ok());
        assert!(!store.current.is_poisoned());
        if let Ok(snapshot) = snapshot {
            assert_eq!(snapshot.owned_bytes(), b"Bearer active-token");
        }

        poison_state(&store);
        let Ok(replacement) = BearerToken::new("replacement-token") else {
            return;
        };
        assert!(store.rotate(replacement).is_ok());
        assert!(!store.current.is_poisoned());
        let snapshot = store.snapshot();
        assert!(snapshot.is_ok());
        if let Ok(snapshot) = snapshot {
            assert_eq!(snapshot.owned_bytes(), b"Bearer replacement-token");
        }
    }

    fn poison_state(store: &CredentialStore) {
        let result = catch_unwind(AssertUnwindSafe(|| {
            let guard = store.current.write();
            let Ok(_guard) = guard else { return };
            resume_unwind(Box::new(()));
        }));
        assert!(result.is_err());
        assert!(store.current.is_poisoned());
    }
}
