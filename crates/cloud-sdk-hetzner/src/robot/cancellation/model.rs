use alloc::vec::Vec;

use cloud_sdk_sanitization::SecretBoxBytes;

use super::{RobotCancellationDate, RobotIpAddress, RobotSubnetAddress};
use crate::robot::server::RobotServerNumber;
use crate::serde::SensitiveText;
use crate::serde::strict_json::Value;

/// Maximum cancellation reasons admitted from one server response.
pub const MAX_ROBOT_CANCELLATION_REASONS: usize = 256;
/// Maximum bytes admitted in one cancellation reason.
pub const MAX_ROBOT_CANCELLATION_REASON_BYTES: usize = 4_096;

pub(super) struct ProtectedFlag(SecretBoxBytes);

impl ProtectedFlag {
    pub(super) fn from_value(value: &Value) -> Result<Self, ()> {
        let mut bytes = SecretBoxBytes::try_zeroed(1, 1).map_err(|_| ())?;
        bytes.with_secret_mut(|output| {
            if let Some(byte) = output.first_mut() {
                value
                    .copy_bool_byte_to(byte)
                    .unwrap_or_else(|| unreachable!("validated cancellation flag changed type"));
            }
        });
        Ok(Self(bytes))
    }

    pub(super) fn get(&self) -> bool {
        self.0.with_secret(|bytes| bytes.first() == Some(&1))
    }
}

pub(super) struct ProtectedPrefix(SecretBoxBytes);

impl ProtectedPrefix {
    pub(super) fn new(value: u8) -> Result<Self, ()> {
        let mut bytes = SecretBoxBytes::try_zeroed(1, 1).map_err(|_| ())?;
        bytes.with_secret_mut(|output| {
            if let Some(byte) = output.first_mut() {
                *byte = value;
            }
        });
        Ok(Self(bytes))
    }

    pub(super) fn get(&self) -> u8 {
        self.0
            .with_secret(|bytes| bytes.first().copied().unwrap_or(0))
    }
}

/// Reason shape returned by the server cancellation endpoint.
pub enum RobotServerCancellationReason {
    /// Reasons offered before a cancellation has been scheduled.
    Available(Vec<SensitiveText>),
    /// Selected reason after scheduling, including an explicit provider null.
    Selected(Option<SensitiveText>),
}

impl RobotServerCancellationReason {
    /// Returns available reasons only for a not-yet-cancelled server.
    #[must_use]
    pub fn available(&self) -> Option<&[SensitiveText]> {
        match self {
            Self::Available(values) => Some(values),
            Self::Selected(_) => None,
        }
    }

    /// Returns the selected reason shape only for a scheduled cancellation.
    #[must_use]
    pub fn selected(&self) -> Option<Option<&SensitiveText>> {
        match self {
            Self::Available(_) => None,
            Self::Selected(value) => Some(value.as_ref()),
        }
    }
}

impl core::fmt::Debug for RobotServerCancellationReason {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotServerCancellationReason([redacted])")
    }
}

/// Exact server cancellation state returned by Robot.
pub struct RobotServerCancellation {
    pub(super) server_number: RobotServerNumber,
    pub(super) server_ip: RobotIpAddress,
    pub(super) server_ipv6_network: RobotIpAddress,
    pub(super) server_name: SensitiveText,
    pub(super) earliest_date: RobotCancellationDate,
    pub(super) cancelled: ProtectedFlag,
    pub(super) reservation_possible: ProtectedFlag,
    pub(super) reserved: ProtectedFlag,
    pub(super) cancellation_date: Option<RobotCancellationDate>,
    pub(super) reason: RobotServerCancellationReason,
}

impl RobotServerCancellation {
    /// Returns the protected server number.
    #[must_use]
    pub const fn server_number(&self) -> &RobotServerNumber {
        &self.server_number
    }
    /// Returns the protected main address.
    #[must_use]
    pub const fn server_ip(&self) -> &RobotIpAddress {
        &self.server_ip
    }
    /// Returns the protected IPv6 network base.
    #[must_use]
    pub const fn server_ipv6_network(&self) -> &RobotIpAddress {
        &self.server_ipv6_network
    }
    /// Runs a closure with temporary access to the protected server name.
    pub fn try_with_server_name<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.server_name.try_with_secret(inspect)
    }
    /// Returns the earliest accepted cancellation date.
    #[must_use]
    pub const fn earliest_date(&self) -> &RobotCancellationDate {
        &self.earliest_date
    }
    /// Reports whether a cancellation is scheduled.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.get()
    }
    /// Reports whether location reservation is offered.
    #[must_use]
    pub fn reservation_possible(&self) -> bool {
        self.reservation_possible.get()
    }
    /// Reports whether the location is reserved.
    #[must_use]
    pub fn is_reserved(&self) -> bool {
        self.reserved.get()
    }
    /// Returns the scheduled date when cancellation is active.
    #[must_use]
    pub const fn cancellation_date(&self) -> Option<&RobotCancellationDate> {
        self.cancellation_date.as_ref()
    }
    /// Returns the source-locked reason shape.
    #[must_use]
    pub const fn reason(&self) -> &RobotServerCancellationReason {
        &self.reason
    }
}

impl core::fmt::Debug for RobotServerCancellation {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotServerCancellation([redacted])")
    }
}

/// Exact IP cancellation state returned by Robot.
pub struct RobotIpCancellation {
    pub(super) ip: RobotIpAddress,
    pub(super) server_number: RobotServerNumber,
    pub(super) earliest_date: RobotCancellationDate,
    pub(super) cancelled: ProtectedFlag,
    pub(super) cancellation_date: Option<RobotCancellationDate>,
}

impl RobotIpCancellation {
    /// Returns the protected address identity.
    #[must_use]
    pub const fn ip(&self) -> &RobotIpAddress {
        &self.ip
    }
    /// Returns the protected owning server number.
    #[must_use]
    pub const fn server_number(&self) -> &RobotServerNumber {
        &self.server_number
    }
    /// Returns the earliest accepted cancellation date.
    #[must_use]
    pub const fn earliest_date(&self) -> &RobotCancellationDate {
        &self.earliest_date
    }
    /// Reports whether a cancellation is scheduled.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.get()
    }
    /// Returns the scheduled date when cancellation is active.
    #[must_use]
    pub const fn cancellation_date(&self) -> Option<&RobotCancellationDate> {
        self.cancellation_date.as_ref()
    }
}

impl core::fmt::Debug for RobotIpCancellation {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotIpCancellation([redacted])")
    }
}

/// Exact subnet cancellation state returned by Robot.
pub struct RobotSubnetCancellation {
    pub(super) subnet: RobotSubnetAddress,
    pub(super) prefix: ProtectedPrefix,
    pub(super) server_number: RobotServerNumber,
    pub(super) earliest_date: RobotCancellationDate,
    pub(super) cancelled: ProtectedFlag,
    pub(super) cancellation_date: Option<RobotCancellationDate>,
}

impl RobotSubnetCancellation {
    /// Returns the protected subnet route identity.
    #[must_use]
    pub const fn subnet(&self) -> &RobotSubnetAddress {
        &self.subnet
    }
    /// Returns the validated network prefix.
    #[must_use]
    pub fn prefix(&self) -> u8 {
        self.prefix.get()
    }
    /// Returns the protected owning server number.
    #[must_use]
    pub const fn server_number(&self) -> &RobotServerNumber {
        &self.server_number
    }
    /// Returns the earliest accepted cancellation date.
    #[must_use]
    pub const fn earliest_date(&self) -> &RobotCancellationDate {
        &self.earliest_date
    }
    /// Reports whether a cancellation is scheduled.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.get()
    }
    /// Returns the scheduled date when cancellation is active.
    #[must_use]
    pub const fn cancellation_date(&self) -> Option<&RobotCancellationDate> {
        self.cancellation_date.as_ref()
    }
}

impl core::fmt::Debug for RobotSubnetCancellation {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotSubnetCancellation([redacted])")
    }
}
