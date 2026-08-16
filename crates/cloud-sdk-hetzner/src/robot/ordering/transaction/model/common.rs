use cloud_sdk_sanitization::SecretString;

use crate::robot::ordering::{
    RobotOrderText, RobotOrderTransactionDecodeError, RobotOrderTransactionId,
};

/// Maximum transactions admitted from one fixed 30-day snapshot.
pub const MAX_ROBOT_ORDER_TRANSACTION_ITEMS: usize = 4_096;
/// Maximum authorized or host keys admitted in one transaction.
pub const MAX_ROBOT_ORDER_TRANSACTION_KEYS: usize = 64;
/// Maximum created addon resources admitted in one transaction.
pub const MAX_ROBOT_ORDER_TRANSACTION_RESOURCES: usize = 4_096;
const MAX_TRANSACTION_TIMESTAMP_BYTES: usize = 64;

/// Finite Robot order transaction state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotOrderTransactionStatus {
    /// The order completed and its resulting identity is available.
    Ready,
    /// Robot is still processing the order.
    InProcess,
    /// Robot cancelled the order.
    Cancelled,
}

/// Calendar-valid RFC 3339 transaction timestamp retained in protected storage.
pub struct RobotOrderTransactionTimestamp(SecretString);

impl RobotOrderTransactionTimestamp {
    pub(in crate::robot::ordering) fn from_provider(
        value: SecretString,
    ) -> Result<Self, RobotOrderTransactionDecodeError> {
        let valid = value
            .try_with_secret(|text| {
                text.len() <= MAX_TRANSACTION_TIMESTAMP_BYTES
                    && text.as_bytes().get(10) == Some(&b'T')
                    && crate::serde::models::cloud_constraints::valid_rfc3339(text)
            })
            .map_err(|_| RobotOrderTransactionDecodeError::InvalidTimestamp)?;
        if valid {
            Ok(Self(value))
        } else {
            Err(RobotOrderTransactionDecodeError::InvalidTimestamp)
        }
    }

    /// Runs a closure with temporary access to the exact provider timestamp.
    pub fn try_with_text<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0.try_with_secret(inspect)
    }
}

impl core::fmt::Debug for RobotOrderTransactionTimestamp {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotOrderTransactionTimestamp([redacted])")
    }
}

/// Bounded SSH key metadata retained by a server transaction.
pub struct RobotOrderTransactionKey {
    pub(in crate::robot::ordering) name: Option<RobotOrderText>,
    pub(in crate::robot::ordering) fingerprint: RobotOrderText,
    pub(in crate::robot::ordering) algorithm: RobotOrderText,
    pub(in crate::robot::ordering) size: u64,
}

impl RobotOrderTransactionKey {
    /// Returns the authorized-key name; host keys have no name.
    #[must_use]
    pub const fn name(&self) -> Option<&RobotOrderText> {
        self.name.as_ref()
    }

    /// Returns the protected provider fingerprint.
    #[must_use]
    pub const fn fingerprint(&self) -> &RobotOrderText {
        &self.fingerprint
    }

    /// Returns the protected provider algorithm name.
    #[must_use]
    pub const fn algorithm(&self) -> &RobotOrderText {
        &self.algorithm
    }

    /// Returns the provider key size in bits.
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }
}

impl core::fmt::Debug for RobotOrderTransactionKey {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotOrderTransactionKey([redacted])")
    }
}

/// One bounded resource created by a completed addon transaction.
pub struct RobotOrderTransactionResource {
    pub(in crate::robot::ordering) kind: RobotOrderText,
    pub(in crate::robot::ordering) id: RobotOrderText,
}

impl RobotOrderTransactionResource {
    /// Returns the protected provider resource type.
    #[must_use]
    pub const fn kind(&self) -> &RobotOrderText {
        &self.kind
    }

    /// Returns the protected provider resource identifier.
    #[must_use]
    pub const fn id(&self) -> &RobotOrderText {
        &self.id
    }
}

impl core::fmt::Debug for RobotOrderTransactionResource {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotOrderTransactionResource([redacted])")
    }
}

pub(in crate::robot::ordering) struct ServerTransactionCommon {
    pub id: RobotOrderTransactionId,
    pub date: RobotOrderTransactionTimestamp,
    pub status: RobotOrderTransactionStatus,
    pub server_number: Option<crate::robot::RobotServerNumber>,
    pub server_ip: Option<crate::robot::ProtectedIpAddr>,
    pub authorized_keys: alloc::vec::Vec<RobotOrderTransactionKey>,
    pub host_keys: alloc::vec::Vec<RobotOrderTransactionKey>,
    pub comment: Option<RobotOrderText>,
}
