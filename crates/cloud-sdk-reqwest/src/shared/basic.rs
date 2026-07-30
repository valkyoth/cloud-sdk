use core::fmt;
use std::vec::Vec;

use base64_ng::{STANDARD, checked_encoded_len};
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
}

impl fmt::Display for BasicCredentialError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UsernameRejected(_) => "Basic username was rejected",
            Self::PasswordRejected(_) => "Basic password was rejected",
            Self::AuthorizationTooLong => "Basic authorization exceeds the length limit",
            Self::AllocationFailed => "Basic authorization allocation failed",
            Self::EncodingFailed => "Basic authorization encoding failed",
        })
    }
}

impl core::error::Error for BasicCredentialError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::UsernameRejected(error) => Some(error),
            Self::PasswordRejected(error) => Some(error),
            Self::AuthorizationTooLong | Self::AllocationFailed | Self::EncodingFailed => None,
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
        Ok(Self {
            authorization,
            scope,
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
mod tests {
    use std::format;

    use cloud_sdk::transport::CustomEndpointAcknowledgement;

    use super::{
        BasicCredential, BasicCredentialError, BasicPassword, BasicPasswordError, BasicUsername,
        BasicUsernameError, MAX_BASIC_PASSWORD_BYTES, MAX_BASIC_USERNAME_BYTES,
    };
    use crate::shared::{BasicCredentialScope, HttpsEndpoint};

    fn test_scope() -> Option<BasicCredentialScope> {
        let endpoint = HttpsEndpoint::new_custom(
            "https://robot-ws.your-server.de",
            CustomEndpointAcknowledgement::trusted_operator_configuration(),
        )
        .ok()?;
        Some(BasicCredentialScope::new(
            cloud_sdk::provider_id!("hetzner"),
            cloud_sdk::service_id!("robot"),
            endpoint,
        ))
    }

    #[test]
    fn rfc_vector_is_exact_bounded_sensitive_and_redacted() {
        let username = BasicUsername::new("Aladdin");
        let password = BasicPassword::new("open sesame");
        let (Ok(username), Ok(password), Some(scope)) = (username, password, test_scope()) else {
            return;
        };
        let credential = BasicCredential::new(username, password, scope);
        assert!(credential.is_ok());
        if let Ok(credential) = credential {
            assert_eq!(
                credential.owned_bytes(),
                b"Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ=="
            );
            assert!(!format!("{credential:?}").contains("Aladdin"));
            let header = credential.header_value();
            assert!(header.as_ref().is_ok_and(|value| value.is_sensitive()));
        }
    }

    #[test]
    fn username_rejects_colon_space_controls_non_ascii_and_bounds() {
        for value in ["", "user:name", "user name", "user\nname", "anv\u{e4}ndare"] {
            let result = BasicUsername::new(value);
            let expected = if value.is_empty() {
                BasicUsernameError::Empty
            } else {
                BasicUsernameError::InvalidByte
            };
            assert_eq!(result.map(|_| ()), Err(expected));
        }
        let accepted = [b'u'; MAX_BASIC_USERNAME_BYTES];
        assert!(BasicUsername::from_bytes(&accepted).is_ok());
        let rejected = [b'u'; MAX_BASIC_USERNAME_BYTES + 1];
        assert_eq!(
            BasicUsername::from_bytes(&rejected).map(|_| ()),
            Err(BasicUsernameError::TooLong)
        );
    }

    #[test]
    fn password_allows_spaces_and_colons_but_rejects_controls_non_ascii_and_bounds() {
        assert!(BasicPassword::new("open sesame:again").is_ok());
        assert_eq!(
            BasicPassword::new("").map(|_| ()),
            Err(BasicPasswordError::Empty)
        );
        for value in ["line\nbreak", "l\u{f6}senord"] {
            assert_eq!(
                BasicPassword::new(value).map(|_| ()),
                Err(BasicPasswordError::InvalidByte)
            );
        }
        let accepted = [b'p'; MAX_BASIC_PASSWORD_BYTES];
        assert!(BasicPassword::from_bytes(&accepted).is_ok());
        let rejected = [b'p'; MAX_BASIC_PASSWORD_BYTES + 1];
        assert_eq!(
            BasicPassword::from_bytes(&rejected).map(|_| ()),
            Err(BasicPasswordError::TooLong)
        );
    }

    #[test]
    fn mutable_sources_clear_on_success_and_rejection() {
        let mut username = *b"robot-user";
        let mut password = *b"secret-pass";
        let Some(scope) = test_scope() else {
            return;
        };
        assert!(BasicCredential::from_mut_bytes(&mut username, &mut password, scope).is_ok());
        assert_eq!(username, [0; 10]);
        assert_eq!(password, [0; 11]);

        let mut invalid_username = *b"bad:user";
        let mut valid_password = *b"password";
        let Some(scope) = test_scope() else {
            return;
        };
        assert_eq!(
            BasicCredential::from_mut_bytes(&mut invalid_username, &mut valid_password, scope,)
                .map(|_| ()),
            Err(BasicCredentialError::UsernameRejected(
                BasicUsernameError::InvalidByte
            ))
        );
        assert_eq!(invalid_username, [0; 8]);
        assert_eq!(valid_password, [0; 8]);
    }

    #[test]
    fn exact_individual_bounds_fit_the_aggregate_authorization_limit() {
        let username = [b'u'; MAX_BASIC_USERNAME_BYTES];
        let password = [b'p'; MAX_BASIC_PASSWORD_BYTES];
        let (Ok(username), Ok(password), Some(scope)) = (
            BasicUsername::from_bytes(&username),
            BasicPassword::from_bytes(&password),
            test_scope(),
        ) else {
            return;
        };
        let credential = BasicCredential::new(username, password, scope);
        assert!(credential.is_ok());
        if let Ok(credential) = credential {
            assert_eq!(credential.owned_bytes().len(), 3_082);
        }
    }
}
