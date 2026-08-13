use cloud_sdk::authentication::CredentialBinding;
use cloud_sdk::operation::{ExecutionPermitError, PermitTimestamp};

use super::RobotWol;

/// Maximum lifetime of authenticated WOL capability evidence, in seconds.
pub const MAX_ROBOT_WOL_EVIDENCE_AGE_SECONDS: u64 = 30;

/// Failure while constructing authenticated WOL capability evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotWolEvidenceError {
    /// The fixed observation lifetime overflowed the caller clock domain.
    ObservationExpiryOverflow,
    /// The transport credential changed while discovery was in flight.
    CredentialChangedDuringPreflight,
}

impl_static_error!(RobotWolEvidenceError,
    Self::ObservationExpiryOverflow => "Robot WOL observation expiry overflowed",
    Self::CredentialChangedDuringPreflight => "Robot WOL credential changed during preflight",
);

/// Authenticated, short-lived proof that Robot advertised WOL for one server.
pub struct AuthorizedRobotWol {
    wol: RobotWol,
    credential: CredentialBinding,
    observed_at: PermitTimestamp,
    expires_at: PermitTimestamp,
}

impl AuthorizedRobotWol {
    pub(super) fn new(
        wol: RobotWol,
        credential: CredentialBinding,
        observed_at: PermitTimestamp,
    ) -> Result<Self, RobotWolEvidenceError> {
        let expires_at = observed_at
            .as_seconds()
            .checked_add(MAX_ROBOT_WOL_EVIDENCE_AGE_SECONDS)
            .map(PermitTimestamp::from_seconds)
            .ok_or(RobotWolEvidenceError::ObservationExpiryOverflow)?;
        Ok(Self {
            wol,
            credential,
            observed_at,
            expires_at,
        })
    }

    /// Returns checked WOL identity without granting standalone authority.
    #[must_use]
    pub const fn wol(&self) -> &RobotWol {
        &self.wol
    }

    /// Returns when authenticated capability state was observed.
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

impl core::fmt::Debug for AuthorizedRobotWol {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("AuthorizedRobotWol([redacted])")
    }
}
