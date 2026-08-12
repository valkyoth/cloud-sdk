use cloud_sdk::authentication::CredentialBinding;
use cloud_sdk::operation::{ExecutionPermitError, PermitTimestamp};

use super::RobotReset;

/// Maximum lifetime of one authenticated Robot reset preflight, in seconds.
pub const MAX_ROBOT_RESET_EVIDENCE_AGE_SECONDS: u64 = 30;

/// Failure while constructing authenticated reset authorization evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotResetEvidenceError {
    /// The fixed observation lifetime overflowed the caller clock domain.
    ObservationExpiryOverflow,
    /// The transport credential changed while the preflight was in flight.
    CredentialChangedDuringPreflight,
}

impl_static_error!(RobotResetEvidenceError,
    Self::ObservationExpiryOverflow => "Robot reset observation expiry overflowed",
    Self::CredentialChangedDuringPreflight => "Robot reset credential changed during preflight",
);

/// Authenticated, short-lived reset capability state.
///
/// Only exact execution of `GET /reset/{server-number}` can construct this
/// type. Raw or caller-supplied response decoding produces [`RobotReset`],
/// which remains inspectable but cannot authorize a reset.
pub struct AuthorizedRobotReset {
    reset: RobotReset,
    credential: CredentialBinding,
    observed_at: PermitTimestamp,
    expires_at: PermitTimestamp,
}

impl AuthorizedRobotReset {
    pub(super) fn new(
        reset: RobotReset,
        credential: CredentialBinding,
        observed_at: PermitTimestamp,
    ) -> Result<Self, RobotResetEvidenceError> {
        let expires_at = observed_at
            .as_seconds()
            .checked_add(MAX_ROBOT_RESET_EVIDENCE_AGE_SECONDS)
            .map(PermitTimestamp::from_seconds)
            .ok_or(RobotResetEvidenceError::ObservationExpiryOverflow)?;
        Ok(Self {
            reset,
            credential,
            observed_at,
            expires_at,
        })
    }

    /// Returns the checked reset state without granting standalone authority.
    #[must_use]
    pub const fn reset(&self) -> &RobotReset {
        &self.reset
    }

    /// Returns when the authenticated response was observed.
    #[must_use]
    pub const fn observed_at(&self) -> PermitTimestamp {
        self.observed_at
    }

    /// Returns the exclusive authorization expiry.
    #[must_use]
    pub const fn expires_at(&self) -> PermitTimestamp {
        self.expires_at
    }

    pub(super) fn validate_at(
        &self,
        credential: CredentialBinding,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        if !self.credential.matches(credential) {
            return Err(ExecutionPermitError::CredentialMismatch);
        }
        if now < self.observed_at {
            return Err(ExecutionPermitError::NotYetValid);
        }
        if now >= self.expires_at {
            return Err(ExecutionPermitError::Expired);
        }
        Ok(())
    }

    pub(super) const fn credential(&self) -> CredentialBinding {
        self.credential
    }
}

impl core::fmt::Debug for AuthorizedRobotReset {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("AuthorizedRobotReset([redacted])")
    }
}
