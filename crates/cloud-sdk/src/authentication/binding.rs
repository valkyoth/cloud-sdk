use core::fmt;

use subtle::ConstantTimeEq;

/// Exact bytes required for one opaque credential-lineage identity.
pub const CREDENTIAL_BINDING_BYTES: usize = 32;

/// Invalid transport-owned credential-lineage identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialBindingError {
    /// An all-zero value cannot identify a credential lifecycle.
    AllZero,
}

impl_static_error!(CredentialBindingError,
    Self::AllZero => "credential binding cannot be all zero",
);

/// Opaque identity for one transport credential lifecycle.
///
/// Transport implementations must generate this value with a CSPRNG when a
/// credential is created and retain it across clones of that same credential.
/// Rotation or replacement must create a different binding. The value is not
/// an authentication secret, but diagnostics remain redacted so it cannot
/// become an application-visible correlation identifier by accident.
#[derive(Clone, Copy, Eq)]
pub struct CredentialBinding([u8; CREDENTIAL_BINDING_BYTES]);

impl CredentialBinding {
    /// Validates a transport-generated credential-lineage identity.
    pub const fn new(
        value: [u8; CREDENTIAL_BINDING_BYTES],
    ) -> Result<Self, CredentialBindingError> {
        if matches!(
            value,
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]
        ) {
            return Err(CredentialBindingError::AllZero);
        }
        Ok(Self(value))
    }

    /// Runs a closure with the exact binding bytes for protected evidence use.
    pub fn with_bytes<R>(&self, inspect: impl FnOnce(&[u8; CREDENTIAL_BINDING_BYTES]) -> R) -> R {
        inspect(&self.0)
    }

    /// Compares two credential lineages without data-dependent early exit.
    #[must_use]
    pub fn matches(self, other: Self) -> bool {
        bool::from(self.0.ct_eq(&other.0))
    }
}

impl PartialEq for CredentialBinding {
    fn eq(&self, other: &Self) -> bool {
        self.matches(*other)
    }
}

impl fmt::Debug for CredentialBinding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CredentialBinding([redacted])")
    }
}

/// Authenticated transport exposing its current opaque credential lineage.
///
/// This contract is required only by provider operations whose authorization
/// depends on a prior authenticated observation. Implementations must return
/// the binding of the credential that `send_authenticated` will use.
pub trait BoundCredentialTransport {
    /// Returns the current transport credential lifecycle.
    fn credential_binding(&self) -> CredentialBinding;
}

#[cfg(test)]
mod tests {
    use super::{CREDENTIAL_BINDING_BYTES, CredentialBinding, CredentialBindingError};

    #[test]
    fn bindings_reject_zero_compare_exactly_and_redact() {
        assert_eq!(
            CredentialBinding::new([0; CREDENTIAL_BINDING_BYTES]),
            Err(CredentialBindingError::AllZero)
        );
        let first = CredentialBinding::new([1; CREDENTIAL_BINDING_BYTES])
            .unwrap_or_else(|_| unreachable!("binding fixture failed"));
        let same = CredentialBinding::new([1; CREDENTIAL_BINDING_BYTES])
            .unwrap_or_else(|_| unreachable!("binding fixture failed"));
        let other = CredentialBinding::new([2; CREDENTIAL_BINDING_BYTES])
            .unwrap_or_else(|_| unreachable!("binding fixture failed"));
        assert!(first.matches(same));
        assert!(!first.matches(other));
        #[cfg(feature = "std")]
        assert_eq!(std::format!("{first:?}"), "CredentialBinding([redacted])");
    }
}
