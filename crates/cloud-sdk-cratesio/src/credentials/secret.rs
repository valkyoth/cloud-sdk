use super::{CredentialError, CredentialKind, CredentialOrigin};
use cloud_sdk_sanitization::{SecretBuffer, SecretString};
use core::{fmt, marker::PhantomData};

/// Owned, non-cloneable credential with an immutable official origin.
///
/// The SDK stores UTF-8 directly in sanitization's protected allocation.
/// Clear/drop wipe its complete capacity. Process abort, leaked objects,
/// caller/adapter copies and external storage are outside drop guarantees.
///
/// ```compile_fail
/// use cloud_sdk_cratesio::credentials::ApiToken;
/// fn duplicate(token: ApiToken) { let _ = token.clone(); }
/// ```
pub struct Credential<K: CredentialKind> {
    pub(super) secret: SecretString,
    pub(super) origin: CredentialOrigin,
    kind: PhantomData<K>,
}

impl<K: CredentialKind> Credential<K> {
    /// Validates and copies into protected storage, clearing the entire source
    /// on success, error or unwinding. Does not accept header scheme prefixes.
    pub fn from_mut_bytes(
        origin: CredentialOrigin,
        source: &mut [u8],
    ) -> Result<Self, CredentialError> {
        Self::from_secret_buffer(origin, SecretBuffer::new(source))
    }

    /// Consumes a source guard, clearing its complete buffer on return.
    pub fn from_secret_buffer(
        origin: CredentialOrigin,
        source: SecretBuffer<'_>,
    ) -> Result<Self, CredentialError> {
        super::kind::validate::<K>(source.as_slice())?;
        let text =
            core::str::from_utf8(source.as_slice()).map_err(|_| CredentialError::InvalidSyntax)?;
        let mut secret =
            SecretString::try_with_capacity(text.len()).map_err(|_| CredentialError::Allocation)?;
        secret.push_str(text);
        Ok(Self {
            secret,
            origin,
            kind: PhantomData,
        })
    }

    /// Takes already protected UTF-8 without making a second secret allocation.
    /// Invalid values are dropped and cleared before returning the error.
    pub fn from_secret_string(
        origin: CredentialOrigin,
        secret: SecretString,
    ) -> Result<Self, CredentialError> {
        secret
            .try_with_secret(|text| super::kind::validate::<K>(text.as_bytes()))
            .map_err(|_| CredentialError::StorageUnavailable)??;
        Ok(Self {
            secret,
            origin,
            kind: PhantomData,
        })
    }

    /// Returns the credential's immutable destination, not its contents.
    #[must_use]
    pub const fn origin(&self) -> CredentialOrigin {
        self.origin
    }

    /// Replaces the secret and clears the old allocation on success. Failed
    /// validation/allocation retains the old value. Source is always cleared.
    /// Exclusive access prevents rotation during a material callback.
    pub fn rotate_from_mut_bytes(&mut self, source: &mut [u8]) -> Result<(), CredentialError> {
        let replacement = Self::from_mut_bytes(self.origin, source)?;
        *self = replacement;
        Ok(())
    }

    /// Clears the complete protected allocation and makes later use fail closed.
    pub fn clear(&mut self) {
        self.secret.clear_secret();
    }
}

impl<K: CredentialKind> fmt::Debug for Credential<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Credential([redacted])")
    }
}

impl<K: CredentialKind> fmt::Display for Credential<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[redacted crates.io credential]")
    }
}
