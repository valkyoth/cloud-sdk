use alloc::string::String;
use core::fmt;

use cloud_sdk::authentication::{
    AuthenticationScope, AuthenticationScopePolicy, CredentialAttemptError,
    CredentialAttemptGeneration, CredentialAttemptStatus, CredentialReconfirmation,
    OwnedCredentialAttempt, OwnedCredentialAttemptState, ScopeRequirement,
};
use cloud_sdk_sanitization::{SecretBuffer, SecretString, sanitize_bytes};

use crate::endpoint::{OfficialEndpointError, official_robot_endpoint_identity};
use crate::identity::{HETZNER_PROVIDER_ID, ROBOT_SERVICE_ID};

/// Maximum Robot Webservice username bytes.
pub const MAX_ROBOT_USERNAME_BYTES: usize = 256;
/// Maximum Robot Webservice password bytes.
pub const MAX_ROBOT_PASSWORD_BYTES: usize = 2048;

/// Robot credential construction or rotation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotCredentialError {
    /// The username is empty.
    EmptyUsername,
    /// The username exceeds [`MAX_ROBOT_USERNAME_BYTES`].
    UsernameTooLong,
    /// The username is not visible ASCII or contains a colon.
    InvalidUsername,
    /// The password is empty.
    EmptyPassword,
    /// The password exceeds [`MAX_ROBOT_PASSWORD_BYTES`].
    PasswordTooLong,
    /// The password is outside the source-locked printable ASCII profile.
    InvalidPassword,
    /// Protected credential storage could not be allocated.
    AllocationFailed,
}

impl_static_error!(RobotCredentialError,
    Self::EmptyUsername => "Robot username is empty",
    Self::UsernameTooLong => "Robot username exceeds the length limit",
    Self::InvalidUsername => "Robot username contains an invalid byte",
    Self::EmptyPassword => "Robot password is empty",
    Self::PasswordTooLong => "Robot password exceeds the length limit",
    Self::InvalidPassword => "Robot password contains an invalid byte",
    Self::AllocationFailed => "Robot credential allocation failed",
);

/// Failure while accessing or transitioning Robot credentials.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotCredentialStateError {
    /// The lockout-aware attempt lifecycle rejected the transition.
    Attempt(CredentialAttemptError),
    /// Protected UTF-8 storage rejected closure-scoped access.
    ProtectedStorageUnavailable,
}

impl fmt::Display for RobotCredentialStateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Attempt(_) => "Robot credential attempt was rejected",
            Self::ProtectedStorageUnavailable => "Robot credential storage is unavailable",
        })
    }
}

impl core::error::Error for RobotCredentialStateError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Attempt(error) => Some(error),
            Self::ProtectedStorageUnavailable => None,
        }
    }
}

impl From<CredentialAttemptError> for RobotCredentialStateError {
    fn from(error: CredentialAttemptError) -> Self {
        Self::Attempt(error)
    }
}

/// Robot credential replacement failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotCredentialRotationError {
    /// Replacement credential material was rejected.
    Credential(RobotCredentialError),
    /// The attempt generation could not advance.
    Attempt(CredentialAttemptError),
}

impl fmt::Display for RobotCredentialRotationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Credential(_) => "Robot replacement credential was rejected",
            Self::Attempt(_) => "Robot credential generation could not advance",
        })
    }
}

impl core::error::Error for RobotCredentialRotationError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Credential(error) => Some(error),
            Self::Attempt(error) => Some(error),
        }
    }
}

/// Fixed provider, service, and endpoint scope of Robot credentials.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RobotCredentialScope;

impl RobotCredentialScope {
    /// Returns the exact scope carried by every Robot credential.
    pub fn authentication_scope(
        self,
    ) -> Result<AuthenticationScope<'static>, OfficialEndpointError> {
        Ok(AuthenticationScope::unscoped()
            .with_provider(HETZNER_PROVIDER_ID)
            .with_service(ROBOT_SERVICE_ID)
            .with_endpoint(official_robot_endpoint_identity()?))
    }

    /// Returns the exact scope policy required for Robot credential use.
    pub fn authentication_policy(
        self,
    ) -> Result<AuthenticationScopePolicy<'static>, OfficialEndpointError> {
        Ok(AuthenticationScopePolicy::new(
            ScopeRequirement::Required(HETZNER_PROVIDER_ID),
            ScopeRequirement::Required(ROBOT_SERVICE_ID),
            ScopeRequirement::Required(official_robot_endpoint_identity()?),
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
        ))
    }
}

/// Owned lockout-aware proof for one Robot credential generation.
///
/// The attempt can cross task boundaries and does not borrow
/// [`RobotCredentials`]. Owner identity remains opaque and non-hashable.
#[derive(Debug, Eq, PartialEq)]
pub struct RobotCredentialAttempt(OwnedCredentialAttempt);

impl RobotCredentialAttempt {
    /// Returns the credential generation used by this attempt.
    #[must_use]
    pub const fn generation(&self) -> CredentialAttemptGeneration {
        self.0.generation()
    }
}

/// Protected Robot Basic credentials and shared lockout-aware state.
///
/// This type does not encode an authorization header or send a request. Secret
/// text is available only inside [`Self::try_with_attempt`], after revalidating
/// that the attempt belongs to this owner and its generation remains open.
pub struct RobotCredentials {
    username: SecretString,
    password: SecretString,
    attempts: OwnedCredentialAttemptState,
}

impl RobotCredentials {
    /// Validates mutable UTF-8 sources and clears both complete buffers.
    pub fn from_mut_bytes(
        username: &mut [u8],
        password: &mut [u8],
    ) -> Result<Self, RobotCredentialError> {
        let result = Self::from_bytes(username, password);
        sanitize_bytes(username);
        sanitize_bytes(password);
        result
    }

    /// Consumes guarded sources, which clear their complete buffers on return.
    pub fn from_secret_buffers(
        username: SecretBuffer<'_>,
        password: SecretBuffer<'_>,
    ) -> Result<Self, RobotCredentialError> {
        Self::from_bytes(username.as_slice(), password.as_slice())
    }

    /// Returns the immutable Robot-only scope.
    #[must_use]
    pub const fn scope(&self) -> RobotCredentialScope {
        RobotCredentialScope
    }

    /// Returns the current generation and lockout status without secret bytes.
    #[must_use]
    pub fn status(&self) -> (CredentialAttemptGeneration, CredentialAttemptStatus) {
        self.attempts.observe()
    }

    /// Begins one owned execution attempt on the current open generation.
    pub fn begin_attempt(&self) -> Result<RobotCredentialAttempt, RobotCredentialStateError> {
        self.attempts
            .begin()
            .map(RobotCredentialAttempt)
            .map_err(Into::into)
    }

    /// Marks the exact attempted generation rejected by Robot authentication.
    pub fn reject_attempt(
        &self,
        attempt: &RobotCredentialAttempt,
    ) -> Result<(), RobotCredentialStateError> {
        self.attempts.reject(&attempt.0).map_err(Into::into)
    }

    /// Reopens unchanged credentials only after explicit caller confirmation.
    pub fn reconfirm(
        &self,
        acknowledgement: CredentialReconfirmation,
    ) -> Result<CredentialAttemptGeneration, RobotCredentialStateError> {
        self.attempts
            .reconfirm(self.status().0, acknowledgement)
            .map_err(Into::into)
    }

    /// Replaces both protected values and opens one new generation.
    ///
    /// Both mutable sources are cleared on success or rejection. Existing
    /// credentials remain active if validation or allocation fails.
    pub fn rotate_from_mut_bytes(
        &mut self,
        username: &mut [u8],
        password: &mut [u8],
    ) -> Result<CredentialAttemptGeneration, RobotCredentialRotationError> {
        let replacement = Self::protected_pair(username, password);
        sanitize_bytes(username);
        sanitize_bytes(password);
        let (replacement_username, replacement_password) =
            replacement.map_err(RobotCredentialRotationError::Credential)?;
        let expected = self.status().0;
        let generation = self
            .attempts
            .replace(expected)
            .map_err(RobotCredentialRotationError::Attempt)?;
        self.username = replacement_username;
        self.password = replacement_password;
        Ok(generation)
    }

    /// Replaces credentials from guarded buffers and opens a new generation.
    pub fn rotate_from_secret_buffers(
        &mut self,
        username: SecretBuffer<'_>,
        password: SecretBuffer<'_>,
    ) -> Result<CredentialAttemptGeneration, RobotCredentialRotationError> {
        let replacement = Self::protected_pair(username.as_slice(), password.as_slice())
            .map_err(RobotCredentialRotationError::Credential)?;
        let expected = self.status().0;
        let generation = self
            .attempts
            .replace(expected)
            .map_err(RobotCredentialRotationError::Attempt)?;
        self.username = replacement.0;
        self.password = replacement.1;
        Ok(generation)
    }

    /// Uses credential text only inside a still-open attempt scope.
    ///
    /// The closure cannot return a borrow of either secret:
    ///
    /// ```compile_fail
    /// # use cloud_sdk_hetzner::robot::RobotCredentials;
    /// # fn example(credentials: &RobotCredentials) {
    /// # let Ok(attempt) = credentials.begin_attempt() else { return };
    /// let _escaped = credentials.try_with_attempt(&attempt, |username, _| username);
    /// # }
    /// ```
    pub fn try_with_attempt<T>(
        &self,
        attempt: &RobotCredentialAttempt,
        use_credentials: impl FnOnce(&str, &str) -> T,
    ) -> Result<T, RobotCredentialStateError> {
        self.attempts.validate(&attempt.0)?;
        self.username
            .try_with_secret(|username| {
                self.password
                    .try_with_secret(|password| use_credentials(username, password))
            })
            .map_err(|_| RobotCredentialStateError::ProtectedStorageUnavailable)?
            .map_err(|_| RobotCredentialStateError::ProtectedStorageUnavailable)
    }

    fn from_bytes(username: &[u8], password: &[u8]) -> Result<Self, RobotCredentialError> {
        let (username, password) = Self::protected_pair(username, password)?;
        Ok(Self {
            username,
            password,
            attempts: OwnedCredentialAttemptState::new(),
        })
    }

    fn protected_pair(
        username: &[u8],
        password: &[u8],
    ) -> Result<(SecretString, SecretString), RobotCredentialError> {
        validate_username(username)?;
        validate_password(password)?;
        let username = protected_string(username)?;
        let password = protected_string(password)?;
        Ok((username, password))
    }
}

impl fmt::Debug for RobotCredentials {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (generation, status) = self.status();
        formatter
            .debug_struct("RobotCredentials")
            .field("scope", &RobotCredentialScope)
            .field("generation", &generation)
            .field("status", &status)
            .field("credential", &"[redacted]")
            .finish()
    }
}

fn protected_string(source: &[u8]) -> Result<SecretString, RobotCredentialError> {
    let text = core::str::from_utf8(source).map_err(|_| RobotCredentialError::InvalidPassword)?;
    let mut owned = String::new();
    owned
        .try_reserve_exact(source.len())
        .map_err(|_| RobotCredentialError::AllocationFailed)?;
    owned.push_str(text);
    Ok(SecretString::from_string(owned))
}

fn validate_username(value: &[u8]) -> Result<(), RobotCredentialError> {
    if value.is_empty() {
        return Err(RobotCredentialError::EmptyUsername);
    }
    if value.len() > MAX_ROBOT_USERNAME_BYTES {
        return Err(RobotCredentialError::UsernameTooLong);
    }
    if !value
        .iter()
        .all(|byte| matches!(byte, b'!'..=b'~') && *byte != b':')
    {
        return Err(RobotCredentialError::InvalidUsername);
    }
    Ok(())
}

fn validate_password(value: &[u8]) -> Result<(), RobotCredentialError> {
    if value.is_empty() {
        return Err(RobotCredentialError::EmptyPassword);
    }
    if value.len() > MAX_ROBOT_PASSWORD_BYTES {
        return Err(RobotCredentialError::PasswordTooLong);
    }
    if !value.iter().all(|byte| matches!(byte, b' '..=b'~')) {
        return Err(RobotCredentialError::InvalidPassword);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
