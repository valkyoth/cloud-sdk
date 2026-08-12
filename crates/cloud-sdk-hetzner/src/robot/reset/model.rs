use alloc::vec::Vec;
use core::fmt;
use core::net::{Ipv4Addr, Ipv6Addr};

use crate::robot::{RobotIpAddress, RobotServerNumber};
use crate::serde::SensitiveText;

use super::RobotResetType;

/// Maximum reset-capable servers admitted from one list response.
pub const MAX_ROBOT_RESET_LIST_ITEMS: usize = 4_096;
/// Maximum bytes admitted for one provider operating-status value.
const MAX_ROBOT_RESET_STATUS_BYTES: usize = 128;

/// Protected, bounded operating status returned by Robot.
pub struct RobotResetOperatingStatus(SensitiveText);

impl RobotResetOperatingStatus {
    pub(super) fn new(value: SensitiveText) -> Result<Self, ()> {
        value
            .validate(MAX_ROBOT_RESET_STATUS_BYTES)
            .map_err(|_| ())?;
        Ok(Self(value))
    }

    /// Runs a closure with the provider status text.
    pub fn try_with_status<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0.try_with_secret(inspect)
    }

    /// Reports the source-demonstrated `running` value.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.0
            .try_with_secret(|value| value == "running")
            .unwrap_or(false)
    }

    /// Reports the source-demonstrated `not supported` value.
    #[must_use]
    pub fn is_not_supported(&self) -> bool {
        self.0
            .try_with_secret(|value| value == "not supported")
            .unwrap_or(false)
    }
}

impl fmt::Debug for RobotResetOperatingStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotResetOperatingStatus([redacted])")
    }
}

/// Common reset capabilities returned by list and detail operations.
pub struct RobotResetSummary {
    pub(super) server_ipv4: RobotIpAddress,
    pub(super) server_ipv6_network: RobotIpAddress,
    pub(super) number: RobotServerNumber,
    pub(super) types: Vec<RobotResetType>,
}

impl RobotResetSummary {
    /// Runs a closure with the server's canonical main IPv4 address.
    pub fn with_server_ipv4<R>(&self, inspect: impl FnOnce(Ipv4Addr) -> R) -> R {
        self.server_ipv4.with_addr(|address| match address {
            core::net::IpAddr::V4(address) => inspect(address),
            core::net::IpAddr::V6(_) => unreachable!("validated Robot IPv4 changed family"),
        })
    }

    /// Runs a closure with the server's canonical IPv6 network address.
    pub fn with_server_ipv6_network<R>(&self, inspect: impl FnOnce(Ipv6Addr) -> R) -> R {
        self.server_ipv6_network.with_addr(|address| match address {
            core::net::IpAddr::V6(address) => inspect(address),
            core::net::IpAddr::V4(_) => unreachable!("validated Robot IPv6 changed family"),
        })
    }

    /// Returns the canonical server number.
    #[must_use]
    pub const fn number(&self) -> &RobotServerNumber {
        &self.number
    }

    /// Returns the nonempty, duplicate-free advertised reset types.
    #[must_use]
    pub fn types(&self) -> &[RobotResetType] {
        &self.types
    }

    /// Reports whether the provider advertised one exact reset type.
    #[must_use]
    pub fn supports(&self, reset_type: RobotResetType) -> bool {
        self.types.contains(&reset_type)
    }
}

impl fmt::Debug for RobotResetSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotResetSummary([redacted])")
    }
}

/// Detailed reset state returned for one server.
pub struct RobotReset {
    pub(super) summary: RobotResetSummary,
    pub(super) operating_status: RobotResetOperatingStatus,
}

impl RobotReset {
    /// Returns common server identity and reset capabilities.
    #[must_use]
    pub const fn summary(&self) -> &RobotResetSummary {
        &self.summary
    }

    /// Returns the protected provider operating status.
    #[must_use]
    pub const fn operating_status(&self) -> &RobotResetOperatingStatus {
        &self.operating_status
    }

    /// Reports whether the checked state advertises one reset type.
    #[must_use]
    pub fn supports(&self, reset_type: RobotResetType) -> bool {
        self.summary.supports(reset_type)
    }
}

impl fmt::Debug for RobotReset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotReset([redacted])")
    }
}

/// Bounded list of reset-capable servers.
pub struct RobotResetList(pub(super) Vec<RobotResetSummary>);

impl RobotResetList {
    /// Returns the protected entries.
    #[must_use]
    pub fn as_slice(&self) -> &[RobotResetSummary] {
        &self.0
    }

    /// Returns the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Reports whether no reset-capable server was returned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for RobotResetList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotResetList([redacted])")
    }
}

/// Strict acknowledgement returned after one reset execution.
pub struct RobotResetAction {
    pub(super) server_ipv4: RobotIpAddress,
    pub(super) server_ipv6_network: RobotIpAddress,
    pub(super) number: Option<RobotServerNumber>,
    pub(super) reset_type: RobotResetType,
}

impl RobotResetAction {
    /// Runs a closure with the acknowledged main IPv4 address.
    pub fn with_server_ipv4<R>(&self, inspect: impl FnOnce(Ipv4Addr) -> R) -> R {
        self.server_ipv4.with_addr(|address| match address {
            core::net::IpAddr::V4(address) => inspect(address),
            core::net::IpAddr::V6(_) => unreachable!("validated Robot IPv4 changed family"),
        })
    }

    /// Runs a closure with the acknowledged IPv6 network address.
    pub fn with_server_ipv6_network<R>(&self, inspect: impl FnOnce(Ipv6Addr) -> R) -> R {
        self.server_ipv6_network.with_addr(|address| match address {
            core::net::IpAddr::V6(address) => inspect(address),
            core::net::IpAddr::V4(_) => unreachable!("validated Robot IPv6 changed family"),
        })
    }

    /// Returns the optional number admitted for the table/example discrepancy.
    #[must_use]
    pub const fn server_number(&self) -> Option<&RobotServerNumber> {
        self.number.as_ref()
    }

    /// Returns the acknowledged reset type.
    #[must_use]
    pub const fn reset_type(&self) -> RobotResetType {
        self.reset_type
    }
}

impl fmt::Debug for RobotResetAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotResetAction([redacted])")
    }
}
