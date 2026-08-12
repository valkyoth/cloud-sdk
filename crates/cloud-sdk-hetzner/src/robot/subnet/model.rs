use alloc::vec::Vec;
use core::fmt;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::robot::{RobotIpAddress, RobotMacAddress, RobotServerNumber, RobotSubnetAddress};

/// Maximum subnet resources admitted from one Robot list response.
pub const MAX_ROBOT_SUBNET_LIST_ITEMS: usize = 4_096;
/// Maximum address-to-MAC choices admitted from one Robot response.
pub const MAX_ROBOT_SUBNET_MAC_OPTIONS: usize = 256;

/// Traffic-warning configuration returned for one subnet.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RobotSubnetTrafficPolicy {
    pub(super) enabled: bool,
    pub(super) hourly_megabytes: u64,
    pub(super) daily_megabytes: u64,
    pub(super) monthly_gigabytes: u64,
}

impl RobotSubnetTrafficPolicy {
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

/// Source-complete Robot subnet assignment.
pub struct RobotSubnet {
    pub(super) address: RobotSubnetAddress,
    pub(super) prefix: u8,
    pub(super) gateway: RobotIpAddress,
    pub(super) server_address: Option<RobotIpAddress>,
    pub(super) server_number: RobotServerNumber,
    pub(super) failover: bool,
    pub(super) locked: bool,
    pub(super) traffic: RobotSubnetTrafficPolicy,
}

impl RobotSubnet {
    /// Runs a closure with the exact route identity returned by Robot.
    pub fn with_address<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.address.with_addr(inspect)
    }
    /// Returns the family-specific CIDR prefix length.
    #[must_use]
    pub const fn prefix(&self) -> u8 {
        self.prefix
    }
    /// Runs a closure with the subnet gateway.
    pub fn with_gateway<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.gateway.with_addr(inspect)
    }
    /// Runs a closure with the optional assigned server main address.
    pub fn with_server_address<R>(&self, inspect: impl FnOnce(Option<IpAddr>) -> R) -> R {
        match self.server_address.as_ref() {
            Some(address) => address.with_addr(|address| inspect(Some(address))),
            None => inspect(None),
        }
    }
    /// Returns the owning server number.
    #[must_use]
    pub const fn server_number(&self) -> &RobotServerNumber {
        &self.server_number
    }
    /// Reports whether this is a failover subnet.
    #[must_use]
    pub const fn is_failover(&self) -> bool {
        self.failover
    }
    /// Reports whether Robot marks the subnet locked.
    #[must_use]
    pub const fn is_locked(&self) -> bool {
        self.locked
    }
    /// Returns the exact traffic-warning policy.
    #[must_use]
    pub const fn traffic(&self) -> RobotSubnetTrafficPolicy {
        self.traffic
    }
    /// Runs a closure with the mathematical network address.
    pub fn with_network_address<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.address
            .with_addr(|address| inspect(network_address(address, self.prefix)))
    }
    /// Runs a closure with the IPv4 broadcast address, or `None` for IPv6.
    pub fn with_broadcast<R>(&self, inspect: impl FnOnce(Option<Ipv4Addr>) -> R) -> R {
        self.address
            .with_addr(|address| inspect(broadcast_address(address, self.prefix)))
    }
}

impl fmt::Debug for RobotSubnet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotSubnet([redacted])")
    }
}

/// Bounded list of Robot subnet resources.
pub struct RobotSubnetList(pub(super) Vec<RobotSubnet>);

impl RobotSubnetList {
    /// Returns the protected entries.
    #[must_use]
    pub fn as_slice(&self) -> &[RobotSubnet] {
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

impl fmt::Debug for RobotSubnetList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotSubnetList([redacted])")
    }
}

/// One source-locked selectable address-to-MAC mapping.
pub struct RobotSubnetMacOption {
    pub(super) address: RobotIpAddress,
    pub(super) mac: RobotMacAddress,
}

impl RobotSubnetMacOption {
    /// Runs a closure with the canonical option address.
    pub fn with_address<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.address.with_addr(inspect)
    }
    /// Returns the canonical selectable MAC.
    #[must_use]
    pub const fn mac(&self) -> &RobotMacAddress {
        &self.mac
    }
}

impl fmt::Debug for RobotSubnetMacOption {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotSubnetMacOption([redacted])")
    }
}

/// Exact response from one subnet-MAC operation.
pub struct RobotSubnetMac {
    pub(super) address: RobotSubnetAddress,
    pub(super) prefix: u8,
    pub(super) mac: RobotMacAddress,
    pub(super) possible: Vec<RobotSubnetMacOption>,
}

impl RobotSubnetMac {
    /// Runs a closure with the bound subnet identity.
    pub fn with_address<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.address.with_addr(inspect)
    }
    /// Returns the family-specific CIDR prefix length.
    #[must_use]
    pub const fn prefix(&self) -> u8 {
        self.prefix
    }
    /// Returns the currently assigned MAC.
    #[must_use]
    pub const fn mac(&self) -> &RobotMacAddress {
        &self.mac
    }
    /// Returns the bounded selectable mappings.
    #[must_use]
    pub fn possible(&self) -> &[RobotSubnetMacOption] {
        &self.possible
    }
}

impl fmt::Debug for RobotSubnetMac {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotSubnetMac([redacted])")
    }
}

fn network_address(address: IpAddr, prefix: u8) -> IpAddr {
    match address {
        IpAddr::V4(address) => {
            let mask = prefix_mask_v4(prefix)
                .unwrap_or_else(|| unreachable!("validated IPv4 subnet prefix became invalid"));
            IpAddr::V4(Ipv4Addr::from(u32::from(address) & mask))
        }
        IpAddr::V6(address) => {
            let mask = prefix_mask_v6(prefix)
                .unwrap_or_else(|| unreachable!("validated IPv6 subnet prefix became invalid"));
            IpAddr::V6(Ipv6Addr::from(u128::from(address) & mask))
        }
    }
}

fn broadcast_address(address: IpAddr, prefix: u8) -> Option<Ipv4Addr> {
    let IpAddr::V4(address) = address else {
        return None;
    };
    let mask = prefix_mask_v4(prefix)
        .unwrap_or_else(|| unreachable!("validated IPv4 subnet prefix became invalid"));
    Some(Ipv4Addr::from((u32::from(address) & mask) | !mask))
}

pub(super) fn prefix_mask_v4(prefix: u8) -> Option<u32> {
    if prefix == 0 {
        Some(0)
    } else {
        32_u32
            .checked_sub(u32::from(prefix))
            .and_then(|shift| u32::MAX.checked_shl(shift))
    }
}

pub(super) fn prefix_mask_v6(prefix: u8) -> Option<u128> {
    if prefix == 0 {
        Some(0)
    } else {
        128_u32
            .checked_sub(u32::from(prefix))
            .and_then(|shift| u128::MAX.checked_shl(shift))
    }
}
