use core::fmt;
use std::vec::Vec;

use aws_lc_rs::rand::{SecureRandom, SystemRandom};
use base64_ng::{STANDARD, checked_encoded_len};
use cloud_sdk::authentication::{CREDENTIAL_BINDING_BYTES, CredentialBinding};
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};
use reqwest::header::HeaderValue;

use super::{BasicCredentialScope, sensitive_header_value};

/// Maximum Basic username bytes accepted by the adapter.
pub const MAX_BASIC_USERNAME_BYTES: usize = 256;
/// Maximum Basic password bytes accepted by the adapter.
pub const MAX_BASIC_PASSWORD_BYTES: usize = 2048;
/// Maximum complete `Authorization: Basic ...` value bytes.
pub const MAX_BASIC_AUTHORIZATION_BYTES: usize = 4096;

const BASIC_PREFIX: &[u8] = b"Basic ";

/// Basic username validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BasicUsernameError {
    /// Usernames must not be empty.
    Empty,
    /// Usernames exceed [`MAX_BASIC_USERNAME_BYTES`].
    TooLong,
    /// Usernames must be visible ASCII and must not contain a colon.
    InvalidByte,
    /// Adapter-owned secret storage could not be allocated.
    AllocationFailed,
}

impl_static_error!(BasicUsernameError,
    Self::Empty => "Basic username is empty",
    Self::TooLong => "Basic username exceeds the length limit",
    Self::InvalidByte => "Basic username contains an invalid byte",
    Self::AllocationFailed => "Basic username allocation failed",
);

/// Basic password validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BasicPasswordError {
    /// Passwords must not be empty.
    Empty,
    /// Passwords exceed [`MAX_BASIC_PASSWORD_BYTES`].
    TooLong,
    /// Passwords must use the source-locked ASCII interoperability profile.
    InvalidByte,
    /// Adapter-owned secret storage could not be allocated.
    AllocationFailed,
}

impl_static_error!(BasicPasswordError,
    Self::Empty => "Basic password is empty",
    Self::TooLong => "Basic password exceeds the length limit",
    Self::InvalidByte => "Basic password contains an invalid byte",
    Self::AllocationFailed => "Basic password allocation failed",
);

/// Basic credential construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BasicCredentialError {
    /// The username was rejected.
    UsernameRejected(BasicUsernameError),
    /// The password was rejected.
    PasswordRejected(BasicPasswordError),
    /// The encoded authorization value exceeds its aggregate cap.
    AuthorizationTooLong,
    /// Adapter-owned authorization storage could not be allocated.
    AllocationFailed,
    /// The admitted RFC 4648 encoder rejected the exact-sized destination.
    EncodingFailed,
    /// The operating-system CSPRNG could not mint a credential binding.
    BindingGenerationFailed,
}

impl fmt::Display for BasicCredentialError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UsernameRejected(_) => "Basic username was rejected",
            Self::PasswordRejected(_) => "Basic password was rejected",
            Self::AuthorizationTooLong => "Basic authorization exceeds the length limit",
            Self::AllocationFailed => "Basic authorization allocation failed",
            Self::EncodingFailed => "Basic authorization encoding failed",
            Self::BindingGenerationFailed => "Basic credential binding generation failed",
        })
    }
}

impl core::error::Error for BasicCredentialError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::UsernameRejected(error) => Some(error),
            Self::PasswordRejected(error) => Some(error),
            Self::AuthorizationTooLong
            | Self::AllocationFailed
            | Self::EncodingFailed
            | Self::BindingGenerationFailed => None,
        }
    }
}

/// Owned Basic username with redacted diagnostics and drop-time cleanup.
pub struct BasicUsername(SecretBytes);

impl BasicUsername {
    /// Validates and copies an immutable username.
    pub fn new(value: &str) -> Result<Self, BasicUsernameError> {
        Self::from_bytes(value.as_bytes())
    }

    /// Validates mutable input and clears the complete source on return.
    pub fn from_mut_bytes(value: &mut [u8]) -> Result<Self, BasicUsernameError> {
        let result = Self::from_bytes(value);
        sanitize_bytes(value);
        result
    }

    /// Consumes guarded input, which clears its complete source on return.
    pub fn from_secret_buffer(value: SecretBuffer<'_>) -> Result<Self, BasicUsernameError> {
        Self::from_bytes(value.as_slice())
    }

    fn from_bytes(value: &[u8]) -> Result<Self, BasicUsernameError> {
        validate_username(value)?;
        SecretBytes::copy_from(value)
            .map(Self)
            .map_err(|()| BasicUsernameError::AllocationFailed)
    }
}

impl fmt::Debug for BasicUsername {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BasicUsername([redacted])")
    }
}

/// Owned Basic password with redacted diagnostics and drop-time cleanup.
pub struct BasicPassword(SecretBytes);

impl BasicPassword {
    /// Validates and copies an immutable password.
    pub fn new(value: &str) -> Result<Self, BasicPasswordError> {
        Self::from_bytes(value.as_bytes())
    }

    /// Validates mutable input and clears the complete source on return.
    pub fn from_mut_bytes(value: &mut [u8]) -> Result<Self, BasicPasswordError> {
        let result = Self::from_bytes(value);
        sanitize_bytes(value);
        result
    }

    /// Consumes guarded input, which clears its complete source on return.
    pub fn from_secret_buffer(value: SecretBuffer<'_>) -> Result<Self, BasicPasswordError> {
        Self::from_bytes(value.as_slice())
    }

    fn from_bytes(value: &[u8]) -> Result<Self, BasicPasswordError> {
        validate_password(value)?;
        SecretBytes::copy_from(value)
            .map(Self)
            .map_err(|()| BasicPasswordError::AllocationFailed)
    }
}

impl fmt::Debug for BasicPassword {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BasicPassword([redacted])")
    }
}

/// Type-separated Basic credential and immutable authentication scope.
pub struct BasicCredential {
    authorization: SecretBytes,
    pub(crate) scope: BasicCredentialScope,
    binding: CredentialBinding,
}

impl BasicCredential {
    /// Encodes a validated username/password pair with RFC 4648 padding.
    pub fn new(
        username: BasicUsername,
        password: BasicPassword,
        scope: BasicCredentialScope,
    ) -> Result<Self, BasicCredentialError> {
        let mut user_pass = SecretBytes::with_capacity(
            username
                .0
                .len()
                .checked_add(1)
                .and_then(|len| len.checked_add(password.0.len()))
                .ok_or(BasicCredentialError::AuthorizationTooLong)?,
        )
        .map_err(|()| BasicCredentialError::AllocationFailed)?;
        user_pass.extend(username.0.as_ref())?;
        user_pass.push(b':')?;
        user_pass.extend(password.0.as_ref())?;
        let encoded_len = checked_encoded_len(user_pass.len(), true)
            .ok_or(BasicCredentialError::AuthorizationTooLong)?;
        let total_len = BASIC_PREFIX
            .len()
            .checked_add(encoded_len)
            .ok_or(BasicCredentialError::AuthorizationTooLong)?;
        if total_len > MAX_BASIC_AUTHORIZATION_BYTES {
            return Err(BasicCredentialError::AuthorizationTooLong);
        }
        let mut authorization = SecretBytes::with_capacity(total_len)
            .map_err(|()| BasicCredentialError::AllocationFailed)?;
        authorization.extend(BASIC_PREFIX)?;
        authorization.resize(total_len)?;
        let destination = authorization
            .as_mut()
            .get_mut(BASIC_PREFIX.len()..)
            .ok_or(BasicCredentialError::EncodingFailed)?;
        let written = STANDARD
            .encode_slice(user_pass.as_ref(), destination)
            .map_err(|_| BasicCredentialError::EncodingFailed)?;
        if written != encoded_len {
            return Err(BasicCredentialError::EncodingFailed);
        }
        let mut binding = [0_u8; CREDENTIAL_BINDING_BYTES];
        SystemRandom::new()
            .fill(&mut binding)
            .map_err(|_| BasicCredentialError::BindingGenerationFailed)?;
        let binding = CredentialBinding::new(binding)
            .map_err(|_| BasicCredentialError::BindingGenerationFailed)?;
        Ok(Self {
            authorization,
            scope,
            binding,
        })
    }

    /// Validates mutable sources, clears both, and constructs one credential.
    pub fn from_mut_bytes(
        username: &mut [u8],
        password: &mut [u8],
        scope: BasicCredentialScope,
    ) -> Result<Self, BasicCredentialError> {
        let username =
            BasicUsername::from_mut_bytes(username).map_err(BasicCredentialError::UsernameRejected);
        let password =
            BasicPassword::from_mut_bytes(password).map_err(BasicCredentialError::PasswordRejected);
        Self::new(username?, password?, scope)
    }

    pub(crate) fn header_value(&self) -> Result<HeaderValue, ()> {
        sensitive_header_value(self.authorization.as_ref())
    }

    pub(crate) const fn scope(&self) -> &BasicCredentialScope {
        &self.scope
    }

    pub(crate) const fn binding(&self) -> CredentialBinding {
        self.binding
    }

    #[cfg(test)]
    pub(crate) fn owned_bytes(&self) -> &[u8] {
        self.authorization.as_ref()
    }
}

impl fmt::Debug for BasicCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BasicCredential([redacted])")
    }
}

struct SecretBytes(Vec<u8>);

impl SecretBytes {
    fn copy_from(value: &[u8]) -> Result<Self, ()> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(value.len()).map_err(|_| ())?;
        bytes.extend_from_slice(value);
        Ok(Self(bytes))
    }

    fn with_capacity(capacity: usize) -> Result<Self, ()> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(capacity).map_err(|_| ())?;
        Ok(Self(bytes))
    }

    fn extend(&mut self, value: &[u8]) -> Result<(), BasicCredentialError> {
        self.0
            .try_reserve(value.len())
            .map_err(|_| BasicCredentialError::AllocationFailed)?;
        self.0.extend_from_slice(value);
        Ok(())
    }

    fn push(&mut self, value: u8) -> Result<(), BasicCredentialError> {
        self.0
            .try_reserve(1)
            .map_err(|_| BasicCredentialError::AllocationFailed)?;
        self.0.push(value);
        Ok(())
    }

    fn resize(&mut self, len: usize) -> Result<(), BasicCredentialError> {
        let additional = len.saturating_sub(self.0.len());
        self.0
            .try_reserve(additional)
            .map_err(|_| BasicCredentialError::AllocationFailed)?;
        self.0.resize(len, 0);
        Ok(())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl AsRef<[u8]> for SecretBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Drop for SecretBytes {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.0);
    }
}

fn validate_username(value: &[u8]) -> Result<(), BasicUsernameError> {
    if value.is_empty() {
        return Err(BasicUsernameError::Empty);
    }
    if value.len() > MAX_BASIC_USERNAME_BYTES {
        return Err(BasicUsernameError::TooLong);
    }
    if !value
        .iter()
        .all(|byte| matches!(byte, b'!'..=b'~') && *byte != b':')
    {
        return Err(BasicUsernameError::InvalidByte);
    }
    Ok(())
}

fn validate_password(value: &[u8]) -> Result<(), BasicPasswordError> {
    if value.is_empty() {
        return Err(BasicPasswordError::Empty);
    }
    if value.len() > MAX_BASIC_PASSWORD_BYTES {
        return Err(BasicPasswordError::TooLong);
    }
    if !value.iter().all(|byte| matches!(byte, b' '..=b'~')) {
        return Err(BasicPasswordError::InvalidByte);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
