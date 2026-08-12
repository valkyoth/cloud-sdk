use alloc::vec::Vec;
use core::fmt;
use core::net::IpAddr;

use super::RobotMacAddress;
use crate::robot::{RobotIpAddress, RobotServerNumber};

/// Maximum IP resources admitted from one Robot list response.
pub const MAX_ROBOT_IP_LIST_ITEMS: usize = 4_096;

/// Traffic-warning configuration returned for one address.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RobotIpTrafficPolicy {
    pub(super) enabled: bool,
    pub(super) hourly_megabytes: u64,
    pub(super) daily_megabytes: u64,
    pub(super) monthly_gigabytes: u64,
}

impl RobotIpTrafficPolicy {
    /// Reports whether warning notifications are enabled.
    #[must_use]
    pub const fn enabled(self) -> bool {
        self.enabled
    }
    /// Returns the hourly threshold in megabytes.
    #[must_use]
    pub const fn hourly_megabytes(self) -> u64 {
        self.hourly_megabytes
    }
    /// Returns the daily threshold in megabytes.
    #[must_use]
    pub const fn daily_megabytes(self) -> u64 {
        self.daily_megabytes
    }
    /// Returns the monthly threshold in gigabytes.
    #[must_use]
    pub const fn monthly_gigabytes(self) -> u64 {
        self.monthly_gigabytes
    }
}

/// Source-complete entry returned by `GET /ip`.
pub struct RobotIpSummary {
    pub(super) address: RobotIpAddress,
    pub(super) server_address: RobotIpAddress,
    pub(super) server_number: RobotServerNumber,
    pub(super) locked: bool,
    pub(super) separate_mac: Option<RobotMacAddress>,
    pub(super) traffic: RobotIpTrafficPolicy,
}

impl RobotIpSummary {
    /// Runs a closure with the assigned address.
    pub fn with_address<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.address.with_addr(inspect)
    }
    /// Runs a closure with the owning server's main address.
    pub fn with_server_address<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.server_address.with_addr(inspect)
    }
    /// Returns the owning server number.
    #[must_use]
    pub const fn server_number(&self) -> &RobotServerNumber {
        &self.server_number
    }
    /// Reports whether Robot marks the address locked.
    #[must_use]
    pub const fn is_locked(&self) -> bool {
        self.locked
    }
    /// Returns a separate generated MAC when assigned.
    #[must_use]
    pub const fn separate_mac(&self) -> Option<&RobotMacAddress> {
        self.separate_mac.as_ref()
    }
    /// Returns the exact traffic-warning policy.
    #[must_use]
    pub const fn traffic(&self) -> RobotIpTrafficPolicy {
        self.traffic
    }
}

impl fmt::Debug for RobotIpSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotIpSummary([redacted])")
    }
}

/// Detailed IP resource returned by get and update operations.
pub struct RobotIp {
    pub(super) summary: RobotIpSummary,
    pub(super) gateway: RobotIpAddress,
    pub(super) prefix: u8,
    pub(super) broadcast: RobotIpAddress,
}

impl RobotIp {
    /// Returns the common assignment and traffic summary.
    #[must_use]
    pub const fn summary(&self) -> &RobotIpSummary {
        &self.summary
    }
    /// Runs a closure with the canonical gateway.
    pub fn with_gateway<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.gateway.with_addr(inspect)
    }
    /// Returns the source-locked CIDR prefix length.
    #[must_use]
    pub const fn prefix(&self) -> u8 {
        self.prefix
    }
    /// Runs a closure with the source-locked broadcast value.
    pub fn with_broadcast<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.broadcast.with_addr(inspect)
    }
}

impl fmt::Debug for RobotIp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotIp([redacted])")
    }
}

/// Bounded list of Robot single-IP resources.
pub struct RobotIpList(pub(super) Vec<RobotIpSummary>);

impl RobotIpList {
    /// Returns the protected entries.
    #[must_use]
    pub fn as_slice(&self) -> &[RobotIpSummary] {
        &self.0
    }
    /// Returns the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Reports whether the list is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for RobotIpList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotIpList([redacted])")
    }
}

/// Exact response from one separate-MAC operation.
pub struct RobotIpMac {
    pub(super) address: RobotIpAddress,
    pub(super) mac: Option<RobotMacAddress>,
}

impl RobotIpMac {
    /// Runs a closure with the bound IP address.
    pub fn with_address<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.address.with_addr(inspect)
    }
    /// Returns the generated MAC, or `None` only for a delete acknowledgement.
    #[must_use]
    pub const fn mac(&self) -> Option<&RobotMacAddress> {
        self.mac.as_ref()
    }
}

impl fmt::Debug for RobotIpMac {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotIpMac([redacted])")
    }
}
