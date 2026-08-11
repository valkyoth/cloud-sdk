use alloc::vec::Vec;
use core::fmt;
use core::net::{Ipv4Addr, Ipv6Addr};

use super::identity::RobotServerNumber;
use super::protected::{
    ProtectedFlag, ProtectedIpAddr, RobotServerCapabilities, RobotServerDate, RobotServerStatus,
    RobotServerSubnet, RobotStorageBoxNumber,
};
use crate::serde::SensitiveText;

/// Maximum single addresses or subnets admitted on one server.
pub const MAX_ROBOT_SERVER_ADDRESSES: usize = 4_096;
/// Maximum servers admitted from one list response.
pub const MAX_ROBOT_SERVER_LIST_ITEMS: usize = 4_096;

/// Source-complete summary returned by `GET /server`.
pub struct RobotServerSummary {
    pub(super) number: RobotServerNumber,
    pub(super) main_ipv4: ProtectedIpAddr,
    pub(super) main_ipv6_network: ProtectedIpAddr,
    pub(super) name: SensitiveText,
    pub(super) product: SensitiveText,
    pub(super) datacenter: SensitiveText,
    pub(super) traffic: SensitiveText,
    pub(super) status: RobotServerStatus,
    pub(super) cancelled: ProtectedFlag,
    pub(super) paid_until: RobotServerDate,
    pub(super) addresses: Vec<ProtectedIpAddr>,
    pub(super) subnets: Option<Vec<RobotServerSubnet>>,
}

impl RobotServerSummary {
    /// Returns the protected canonical server number.
    #[must_use]
    pub const fn number(&self) -> &RobotServerNumber {
        &self.number
    }

    /// Runs a closure with temporary access to the main IPv4 address.
    pub fn with_main_ipv4<R>(&self, inspect: impl FnOnce(Ipv4Addr) -> R) -> R {
        self.main_ipv4.with_addr(|address| match address {
            core::net::IpAddr::V4(address) => inspect(address),
            core::net::IpAddr::V6(_) => unreachable!("validated main IPv4 changed family"),
        })
    }

    /// Runs a closure with temporary access to the main IPv6 network address.
    pub fn with_main_ipv6_network<R>(&self, inspect: impl FnOnce(Ipv6Addr) -> R) -> R {
        self.main_ipv6_network.with_addr(|address| match address {
            core::net::IpAddr::V6(address) => inspect(address),
            core::net::IpAddr::V4(_) => unreachable!("validated main IPv6 changed family"),
        })
    }

    /// Returns the protected finite server state.
    #[must_use]
    pub const fn status(&self) -> &RobotServerStatus {
        &self.status
    }
    /// Reports whether cancellation is active.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.get()
    }
    /// Returns the protected paid-through date.
    #[must_use]
    pub const fn paid_until(&self) -> &RobotServerDate {
        &self.paid_until
    }
    /// Returns all protected assigned single addresses.
    #[must_use]
    pub fn addresses(&self) -> &[ProtectedIpAddr] {
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
        formatter.write_str("RobotServerSummary([redacted])")
    }
}

/// Detailed server returned by canonical get and update operations.
pub struct RobotServer {
    pub(super) summary: RobotServerSummary,
    pub(super) capabilities: RobotServerCapabilities,
    pub(super) linked_storage_box: Option<RobotStorageBoxNumber>,
}

impl RobotServer {
    /// Returns the common server summary.
    #[must_use]
    pub const fn summary(&self) -> &RobotServerSummary {
        &self.summary
    }
    /// Returns the protected source-locked capability flags.
    #[must_use]
    pub const fn capabilities(&self) -> &RobotServerCapabilities {
        &self.capabilities
    }
    /// Returns the protected linked Storage Box when present.
    #[must_use]
    pub const fn linked_storage_box(&self) -> Option<&RobotStorageBoxNumber> {
        self.linked_storage_box.as_ref()
    }
}

impl fmt::Debug for RobotServer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotServer([redacted])")
    }
}

/// Bounded server-list result.
pub struct RobotServerList(pub(super) Vec<RobotServerSummary>);

impl RobotServerList {
    /// Returns the bounded protected server slice.
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

impl fmt::Debug for RobotServerList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotServerList([redacted])")
    }
}
