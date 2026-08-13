use alloc::vec::Vec;

use cloud_sdk_sanitization::sanitize_bytes;

use crate::robot::{RobotSshKeyFingerprint, RobotSshKeyName};
use crate::serde::SensitiveText;

/// Maximum SSH-key resources admitted from one Robot list response.
pub const MAX_ROBOT_SSH_KEY_LIST_ITEMS: usize = 4_096;

/// Source-reported Robot SSH-key algorithm family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotSshKeyAlgorithm {
    /// RSA public key.
    Rsa,
    /// ECDSA public key.
    Ecdsa,
    /// Ed25519 public key.
    Ed25519,
}

/// Calendar-valid provider-local SSH-key creation timestamp.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RobotSshKeyCreatedAt {
    pub(super) year: u16,
    pub(super) month: u8,
    pub(super) day: u8,
    pub(super) hour: u8,
    pub(super) minute: u8,
    pub(super) second: u8,
}

impl RobotSshKeyCreatedAt {
    /// Runs a closure with exact source components.
    pub fn with_components<R>(&self, inspect: impl FnOnce(u16, u8, u8, u8, u8, u8) -> R) -> R {
        inspect(
            self.year,
            self.month,
            self.day,
            self.hour,
            self.minute,
            self.second,
        )
    }
}

impl core::fmt::Debug for RobotSshKeyCreatedAt {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotSshKeyCreatedAt([redacted])")
    }
}

/// One source-validated Robot SSH key.
pub struct RobotSshKey {
    pub(super) name: RobotSshKeyName,
    pub(super) fingerprint: RobotSshKeyFingerprint,
    pub(super) algorithm: RobotSshKeyAlgorithm,
    pub(super) size_bits: u32,
    pub(super) data: SensitiveText,
    pub(super) sha256_fingerprint: [u8; 32],
    pub(super) created_at: RobotSshKeyCreatedAt,
}

impl RobotSshKey {
    /// Returns the protected key name.
    #[must_use]
    pub const fn name(&self) -> &RobotSshKeyName {
        &self.name
    }

    /// Returns the canonical protected MD5 compatibility fingerprint.
    #[must_use]
    pub const fn fingerprint(&self) -> &RobotSshKeyFingerprint {
        &self.fingerprint
    }

    /// Returns the source-reported and wire-validated algorithm family.
    #[must_use]
    pub const fn algorithm(&self) -> RobotSshKeyAlgorithm {
        self.algorithm
    }

    /// Returns the source-reported and wire-validated key size.
    #[must_use]
    pub const fn size_bits(&self) -> u32 {
        self.size_bits
    }

    /// Runs a closure with the protected normalized OpenSSH public key.
    pub fn try_with_data<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.data.try_with_secret(inspect)
    }

    /// Returns the SDK-computed SHA-256 key-wire fingerprint.
    #[must_use]
    pub const fn sha256_fingerprint(&self) -> &[u8; 32] {
        &self.sha256_fingerprint
    }

    /// Returns the validated creation timestamp.
    #[must_use]
    pub const fn created_at(&self) -> RobotSshKeyCreatedAt {
        self.created_at
    }
}

impl core::fmt::Debug for RobotSshKey {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotSshKey")
            .field("identity", &"[redacted]")
            .field("algorithm", &self.algorithm)
            .field("size_bits", &self.size_bits)
            .field("data", &"[redacted]")
            .field("created_at", &"[redacted]")
            .finish()
    }
}

impl Drop for RobotSshKey {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.sha256_fingerprint);
    }
}

/// Bounded list of SSH keys with distinct fingerprints.
pub struct RobotSshKeyList(pub(super) Vec<RobotSshKey>);

impl RobotSshKeyList {
    /// Returns the validated protected keys.
    #[must_use]
    pub fn as_slice(&self) -> &[RobotSshKey] {
        &self.0
    }

    /// Returns the number of keys.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Reports whether no keys were returned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl core::fmt::Debug for RobotSshKeyList {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotSshKeyList([redacted])")
    }
}
