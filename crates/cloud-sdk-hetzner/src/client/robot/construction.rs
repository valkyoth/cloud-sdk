use core::fmt;

use cloud_sdk::authentication::{
    BoundCredentialTransport, CredentialAttemptError, CredentialAttemptGeneration,
    CredentialAttemptStatus, CredentialBinding, CredentialReconfirmation,
    OwnedCredentialAttemptState,
};
use cloud_sdk::client::ClientKernel;
use cloud_sdk::transport::BoundTransport;

use crate::endpoint::{OfficialEndpointError, verify_official_robot_endpoint};

/// Failure while binding a Basic-authenticated transport to official Robot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotClientConstructionError {
    /// The transport destination is not the exact official Robot endpoint.
    OfficialEndpoint(OfficialEndpointError),
}

impl fmt::Display for RobotClientConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Robot client official endpoint verification failed")
    }
}

impl core::error::Error for RobotClientConstructionError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::OfficialEndpoint(error) => Some(error),
        }
    }
}

/// Fail-closed Robot credential-attempt lifecycle error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotClientLifecycleError {
    /// The shared attempt generation rejected an execution or transition.
    CredentialAttempt(CredentialAttemptError),
    /// Transport credential lineage changed without explicit replacement.
    CredentialBindingChanged,
    /// Replacement retained the same credential lineage identity.
    CredentialBindingNotReplaced,
}

impl fmt::Display for RobotClientLifecycleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::CredentialAttempt(_) => "Robot credential attempt was rejected",
            Self::CredentialBindingChanged => "Robot transport credential binding changed",
            Self::CredentialBindingNotReplaced => {
                "Robot replacement transport retained the credential binding"
            }
        })
    }
}

impl core::error::Error for RobotClientLifecycleError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::CredentialAttempt(error) => Some(error),
            Self::CredentialBindingChanged | Self::CredentialBindingNotReplaced => None,
        }
    }
}

impl From<CredentialAttemptError> for RobotClientLifecycleError {
    fn from(error: CredentialAttemptError) -> Self {
        Self::CredentialAttempt(error)
    }
}

/// Official Robot client with one shared authentication-rejection generation.
///
/// The client accepts no custom endpoint. Clones of a transport may be used by
/// callers, but every execution through this object is checked against the
/// credential binding captured at construction.
pub struct RobotClient<T> {
    pub(super) kernel: ClientKernel<T>,
    pub(super) binding: CredentialBinding,
    pub(super) attempts: OwnedCredentialAttemptState,
}

impl<T> RobotClient<T>
where
    T: BoundCredentialTransport + BoundTransport,
{
    /// Constructs a client only for `https://robot-ws.your-server.de/`.
    pub fn official(transport: T) -> Result<Self, RobotClientConstructionError> {
        verify_official_robot_endpoint(&transport)
            .map_err(RobotClientConstructionError::OfficialEndpoint)?;
        let binding = transport.credential_binding();
        Ok(Self {
            kernel: ClientKernel::new(transport),
            binding,
            attempts: OwnedCredentialAttemptState::new(),
        })
    }

    /// Returns the current lockout generation and status.
    #[must_use]
    pub fn credential_status(&self) -> (CredentialAttemptGeneration, CredentialAttemptStatus) {
        self.attempts.observe()
    }

    /// Explicitly reopens unchanged credentials after authentication rejection.
    pub fn reconfirm_credentials(
        &self,
        acknowledgement: CredentialReconfirmation,
    ) -> Result<CredentialAttemptGeneration, RobotClientLifecycleError> {
        self.attempts
            .reconfirm(self.attempts.observe().0, acknowledgement)
            .map_err(Into::into)
    }

    /// Replaces the transport and advances only to a different credential lineage.
    pub fn replace_transport(
        &mut self,
        transport: T,
    ) -> Result<CredentialAttemptGeneration, RobotClientLifecycleError> {
        verify_official_robot_endpoint(&transport)
            .map_err(|_| RobotClientLifecycleError::CredentialBindingChanged)?;
        let binding = transport.credential_binding();
        if binding.matches(self.binding) {
            return Err(RobotClientLifecycleError::CredentialBindingNotReplaced);
        }
        let generation = self.attempts.replace(self.attempts.observe().0)?;
        self.kernel = ClientKernel::new(transport);
        self.binding = binding;
        Ok(generation)
    }

    /// Returns the immutable endpoint-bound Basic-authenticated transport.
    #[must_use]
    pub const fn transport(&self) -> &T {
        self.kernel.transport()
    }

    /// Consumes the client and returns its transport.
    #[must_use]
    pub fn into_transport(self) -> T {
        self.kernel.into_transport()
    }
}

impl<T> fmt::Debug for RobotClient<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (generation, status) = self.attempts.observe();
        formatter
            .debug_struct("RobotClient")
            .field("endpoint", &"[official Robot]")
            .field("credential", &"[redacted]")
            .field("generation", &generation)
            .field("status", &status)
            .finish_non_exhaustive()
    }
}
