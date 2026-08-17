//! Credential-bound observations for billable Robot ordering decisions.

use cloud_sdk::authentication::CredentialBinding;

/// A decoded Robot value bound to the credential lifecycle that fetched it.
pub struct CredentialObserved<T> {
    value: T,
    credential: CredentialBinding,
}

impl<T> CredentialObserved<T> {
    pub(crate) const fn from_parts(value: T, credential: CredentialBinding) -> Self {
        Self { value, credential }
    }

    /// Borrows the decoded value without exposing its credential identity.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Consumes the observation and returns its decoded value.
    #[must_use]
    pub fn into_value(self) -> T {
        self.value
    }

    pub(in crate::robot::ordering) const fn credential(&self) -> CredentialBinding {
        self.credential
    }
}

impl<T> core::fmt::Debug for CredentialObserved<T> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("CredentialObserved([redacted])")
    }
}
