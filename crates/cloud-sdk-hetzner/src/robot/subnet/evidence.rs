use cloud_sdk::operation::{ExecutionPermitError, PermitTimestamp};
use cloud_sdk_sanitization::SecretBoxBytes;

use crate::robot::RobotSubnetAddress;

/// Maximum lifetime of either Robot subnet observation, in seconds.
pub const MAX_ROBOT_SUBNET_EVIDENCE_AGE_SECONDS: u64 = 30;
/// Maximum external mutation-lock identity length.
pub const MAX_ROBOT_SUBNET_LOCK_ID_BYTES: usize = 128;

/// Invalid checked-observation or external-lock evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotSubnetEvidenceError {
    /// The two provider reads are too far apart for one destructive decision.
    ObservationWindowTooWide,
    /// The lock identity is empty or exceeds its public bound.
    InvalidLockIdentity,
    /// Protected lock-identity allocation failed.
    Allocation,
    /// The external lease does not cover the checked subnet.
    LockResourceMismatch,
    /// The external lease expires before the checked observations do.
    LockExpiresTooSoon,
}

impl_static_error!(RobotSubnetEvidenceError,
    Self::ObservationWindowTooWide => "Robot subnet observations exceed the freshness window",
    Self::InvalidLockIdentity => "Robot subnet mutation-lock identity is invalid",
    Self::Allocation => "Robot subnet mutation-lock identity allocation failed",
    Self::LockResourceMismatch => "Robot subnet mutation lock covers another resource",
    Self::LockExpiresTooSoon => "Robot subnet mutation lock expires before the evidence",
);

/// Bounded timestamps for the two provider snapshots behind a destructive decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RobotSubnetObservationWindow {
    subnet_observed_at: PermitTimestamp,
    mac_observed_at: PermitTimestamp,
    not_before: PermitTimestamp,
    expires_at: PermitTimestamp,
}

impl RobotSubnetObservationWindow {
    /// Binds two caller-observed provider reads to a fixed 30-second lifetime.
    pub fn new(
        subnet_observed_at: PermitTimestamp,
        mac_observed_at: PermitTimestamp,
    ) -> Result<Self, RobotSubnetEvidenceError> {
        let earliest = subnet_observed_at
            .as_seconds()
            .min(mac_observed_at.as_seconds());
        let latest = subnet_observed_at
            .as_seconds()
            .max(mac_observed_at.as_seconds());
        let expires = earliest
            .checked_add(MAX_ROBOT_SUBNET_EVIDENCE_AGE_SECONDS)
            .ok_or(RobotSubnetEvidenceError::ObservationWindowTooWide)?;
        if latest >= expires {
            return Err(RobotSubnetEvidenceError::ObservationWindowTooWide);
        }
        Ok(Self {
            subnet_observed_at,
            mac_observed_at,
            not_before: PermitTimestamp::from_seconds(latest),
            expires_at: PermitTimestamp::from_seconds(expires),
        })
    }

    pub(super) fn validate_at(self, now: PermitTimestamp) -> Result<(), ExecutionPermitError> {
        if now < self.not_before {
            return Err(ExecutionPermitError::NotYetValid);
        }
        if now >= self.expires_at {
            return Err(ExecutionPermitError::Expired);
        }
        Ok(())
    }

    pub(super) const fn fields(self) -> (PermitTimestamp, PermitTimestamp, PermitTimestamp) {
        (
            self.subnet_observed_at,
            self.mac_observed_at,
            self.expires_at,
        )
    }
}

/// Caller-provided evidence that an external per-subnet mutation lock is held.
///
/// The SDK binds this lease into the destructive digest and checks its expiry.
/// The caller remains responsible for obtaining the identity from a lock
/// service that serializes every mutation of the same subnet.
pub struct RobotSubnetMutationLease {
    address: RobotSubnetAddress,
    identity: SecretBoxBytes,
    expires_at: PermitTimestamp,
}

impl RobotSubnetMutationLease {
    /// Creates a bounded, protected external-lock lease.
    pub fn new(
        address: RobotSubnetAddress,
        identity: &[u8],
        expires_at: PermitTimestamp,
    ) -> Result<Self, RobotSubnetEvidenceError> {
        if identity.is_empty() || identity.len() > MAX_ROBOT_SUBNET_LOCK_ID_BYTES {
            return Err(RobotSubnetEvidenceError::InvalidLockIdentity);
        }
        let identity = SecretBoxBytes::try_from_slice(identity, MAX_ROBOT_SUBNET_LOCK_ID_BYTES)
            .map_err(|_| RobotSubnetEvidenceError::Allocation)?;
        Ok(Self {
            address,
            identity,
            expires_at,
        })
    }

    pub(super) fn covers(
        &self,
        address: &RobotSubnetAddress,
        through: PermitTimestamp,
    ) -> Result<(), RobotSubnetEvidenceError> {
        if &self.address != address {
            return Err(RobotSubnetEvidenceError::LockResourceMismatch);
        }
        if self.expires_at < through {
            return Err(RobotSubnetEvidenceError::LockExpiresTooSoon);
        }
        Ok(())
    }

    pub(super) fn validate_at(&self, now: PermitTimestamp) -> Result<(), ExecutionPermitError> {
        if now >= self.expires_at {
            Err(ExecutionPermitError::Expired)
        } else {
            Ok(())
        }
    }

    pub(super) fn with_identity<R>(&self, inspect: impl FnOnce(&[u8]) -> R) -> R {
        self.identity.with_secret(inspect)
    }

    pub(super) const fn expires_at(&self) -> PermitTimestamp {
        self.expires_at
    }
}

impl core::fmt::Debug for RobotSubnetMutationLease {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotSubnetMutationLease([redacted])")
    }
}
