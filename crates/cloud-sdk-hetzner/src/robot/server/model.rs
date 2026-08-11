use alloc::vec::Vec;
use core::fmt;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::request::RobotServerNumber;
use crate::serde::SensitiveText;

/// Maximum single addresses or subnets admitted on one server.
pub const MAX_ROBOT_SERVER_ADDRESSES: usize = 4_096;
/// Maximum servers admitted from one list response.
pub const MAX_ROBOT_SERVER_LIST_ITEMS: usize = 4_096;

/// Positive linked Storage Box number.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RobotStorageBoxNumber(u64);

impl RobotStorageBoxNumber {
    pub(crate) const fn new(value: u64) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    /// Returns the provider number.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Source-locked Robot server state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotServerStatus {
    /// Server data is ready for use.
    Ready,
    /// A provider-side operation is in progress.
    InProcess,
}

/// Calendar-valid provider date.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RobotServerDate {
    year: u16,
    month: u8,
    day: u8,
}

impl RobotServerDate {
    pub(crate) const fn new(year: u16, month: u8, day: u8) -> Option<Self> {
        if year == 0 || month == 0 || month > 12 || day == 0 || day > days_in_month(year, month) {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// Returns the year.
    #[must_use]
    pub const fn year(self) -> u16 {
        self.year
    }
    /// Returns the month.
    #[must_use]
    pub const fn month(self) -> u8 {
        self.month
    }
    /// Returns the day.
    #[must_use]
    pub const fn day(self) -> u8 {
        self.day
    }
}

const fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        2 if year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100)) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Canonical assigned subnet.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RobotServerSubnet {
    network: IpAddr,
    prefix: u8,
}

impl RobotServerSubnet {
    pub(crate) const fn new(network: IpAddr, prefix: u8) -> Self {
        Self { network, prefix }
    }
    /// Returns the canonical network address.
    #[must_use]
    pub const fn network(self) -> IpAddr {
        self.network
    }
    /// Returns the network prefix length.
    #[must_use]
    pub const fn prefix(self) -> u8 {
        self.prefix
    }
}

/// Source-complete summary returned by `GET /server`.
pub struct RobotServerSummary {
    pub(crate) number: RobotServerNumber,
    pub(crate) main_ipv4: Ipv4Addr,
    pub(crate) main_ipv6_network: Ipv6Addr,
    pub(crate) name: SensitiveText,
    pub(crate) product: SensitiveText,
    pub(crate) datacenter: SensitiveText,
    pub(crate) traffic: SensitiveText,
    pub(crate) status: RobotServerStatus,
    pub(crate) cancelled: bool,
    pub(crate) paid_until: RobotServerDate,
    pub(crate) addresses: Vec<IpAddr>,
    pub(crate) subnets: Option<Vec<RobotServerSubnet>>,
}

impl RobotServerSummary {
    /// Returns the canonical server number.
    #[must_use]
    pub const fn number(&self) -> RobotServerNumber {
        self.number
    }
    /// Returns the main IPv4 address.
    #[must_use]
    pub const fn main_ipv4(&self) -> Ipv4Addr {
        self.main_ipv4
    }
    /// Returns the main IPv6 network address reported by Robot.
    #[must_use]
    pub const fn main_ipv6_network(&self) -> Ipv6Addr {
        self.main_ipv6_network
    }
    /// Returns the finite server state.
    #[must_use]
    pub const fn status(&self) -> RobotServerStatus {
        self.status
    }
    /// Reports whether cancellation is active.
    #[must_use]
    pub const fn is_cancelled(&self) -> bool {
        self.cancelled
    }
    /// Returns the paid-through date.
    #[must_use]
    pub const fn paid_until(&self) -> RobotServerDate {
        self.paid_until
    }
    /// Returns all assigned single addresses.
    #[must_use]
    pub fn addresses(&self) -> &[IpAddr] {
        &self.addresses
    }
    /// Returns `None` only when Robot returned JSON `null`.
    #[must_use]
    pub fn subnets(&self) -> Option<&[RobotServerSubnet]> {
        self.subnets.as_deref()
    }
    /// Runs a closure with the protected server name.
    pub fn try_with_name<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.name.try_with_secret(inspect)
    }
    /// Runs a closure with the protected product name.
    pub fn try_with_product<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.product.try_with_secret(inspect)
    }
    /// Runs a closure with the protected data-center name.
    pub fn try_with_datacenter<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.datacenter.try_with_secret(inspect)
    }
    /// Runs a closure with the protected traffic description.
    pub fn try_with_traffic<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.traffic.try_with_secret(inspect)
    }
}

impl fmt::Debug for RobotServerSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotServerSummary")
            .field("number", &self.number)
            .field("address_data", &"[redacted]")
            .field("text", &"[redacted]")
            .field("status", &self.status)
            .field("cancelled", &self.cancelled)
            .finish()
    }
}

/// Source-locked server feature availability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RobotServerCapabilities {
    /// Reset-system availability.
    pub reset: bool,
    /// Rescue-system availability.
    pub rescue: bool,
    /// VNC installation availability.
    pub vnc: bool,
    /// Windows installation availability.
    pub windows: bool,
    /// Plesk installation availability.
    pub plesk: bool,
    /// cPanel installation availability.
    pub cpanel: bool,
    /// Wake-on-LAN availability.
    pub wake_on_lan: bool,
    /// Hot-swap availability.
    pub hot_swap: bool,
}

/// Detailed server returned by canonical get and update operations.
pub struct RobotServer {
    pub(crate) summary: RobotServerSummary,
    pub(crate) capabilities: RobotServerCapabilities,
    pub(crate) linked_storage_box: Option<RobotStorageBoxNumber>,
}

impl RobotServer {
    /// Returns the common server summary.
    #[must_use]
    pub const fn summary(&self) -> &RobotServerSummary {
        &self.summary
    }
    /// Returns all source-locked capability flags.
    #[must_use]
    pub const fn capabilities(&self) -> RobotServerCapabilities {
        self.capabilities
    }
    /// Returns the linked Storage Box when Robot supplied a positive ID.
    #[must_use]
    pub const fn linked_storage_box(&self) -> Option<RobotStorageBoxNumber> {
        self.linked_storage_box
    }
}

impl fmt::Debug for RobotServer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotServer")
            .field("summary", &self.summary)
            .field("capabilities", &self.capabilities)
            .field("linked_storage_box", &self.linked_storage_box)
            .finish()
    }
}

/// Bounded server-list result.
#[derive(Debug)]
pub struct RobotServerList(pub(crate) Vec<RobotServerSummary>);

impl RobotServerList {
    /// Returns the bounded server slice.
    #[must_use]
    pub fn as_slice(&self) -> &[RobotServerSummary] {
        &self.0
    }
    /// Returns the server count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Reports whether no servers were returned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
