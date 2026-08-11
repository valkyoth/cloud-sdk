use alloc::vec::Vec;
use core::fmt;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use cloud_sdk_sanitization::sanitize_bytes;

use super::identity::RobotServerNumber;
use crate::serde::SensitiveText;

/// Maximum single addresses or subnets admitted on one server.
pub const MAX_ROBOT_SERVER_ADDRESSES: usize = 4_096;
/// Maximum servers admitted from one list response.
pub const MAX_ROBOT_SERVER_LIST_ITEMS: usize = 4_096;

/// Cleanup-owning IP address used for classified server topology.
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct ProtectedIpAddr(ProtectedIpAddrValue);

#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum ProtectedIpAddrValue {
    V4([u8; 4]),
    V6([u8; 16]),
}

impl ProtectedIpAddr {
    pub(crate) const fn new(value: IpAddr) -> Self {
        match value {
            IpAddr::V4(address) => Self(ProtectedIpAddrValue::V4(address.octets())),
            IpAddr::V6(address) => Self(ProtectedIpAddrValue::V6(address.octets())),
        }
    }

    /// Runs a closure with temporary access to the address.
    pub fn with_addr<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        inspect(self.value())
    }

    pub(crate) const fn value(&self) -> IpAddr {
        match self.0 {
            ProtectedIpAddrValue::V4(bytes) => IpAddr::V4(Ipv4Addr::from_octets(bytes)),
            ProtectedIpAddrValue::V6(bytes) => IpAddr::V6(Ipv6Addr::from_octets(bytes)),
        }
    }

    pub(crate) fn identity_key(&self) -> [u8; 17] {
        let mut output = [0_u8; 17];
        let Some((family, payload)) = output.split_first_mut() else {
            unreachable!("fixed address identity key is empty")
        };
        match self.0 {
            ProtectedIpAddrValue::V4(bytes) => {
                *family = 4;
                let Some(target) = payload.get_mut(..bytes.len()) else {
                    unreachable!("IPv4 identity key storage is too small")
                };
                target.copy_from_slice(&bytes);
            }
            ProtectedIpAddrValue::V6(bytes) => {
                *family = 6;
                payload.copy_from_slice(&bytes);
            }
        }
        output
    }
}

impl Drop for ProtectedIpAddr {
    fn drop(&mut self) {
        match &mut self.0 {
            ProtectedIpAddrValue::V4(bytes) => sanitize_bytes(bytes),
            ProtectedIpAddrValue::V6(bytes) => sanitize_bytes(bytes),
        }
    }
}

impl fmt::Debug for ProtectedIpAddr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ProtectedIpAddr([redacted])")
    }
}

/// Positive linked Storage Box number in cleanup-owning storage.
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct RobotStorageBoxNumber([u8; 8]);

impl RobotStorageBoxNumber {
    pub(crate) const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value.to_be_bytes()))
        }
    }

    /// Runs a closure with temporary access to the provider number.
    pub fn with_number<R>(&self, inspect: impl FnOnce(u64) -> R) -> R {
        inspect(u64::from_be_bytes(self.0))
    }
}

impl Drop for RobotStorageBoxNumber {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.0);
    }
}

impl fmt::Debug for RobotStorageBoxNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotStorageBoxNumber([redacted])")
    }
}

/// Source-locked Robot server state in cleanup-owning storage.
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct RobotServerStatus([u8; 1]);

impl RobotServerStatus {
    pub(crate) const fn ready() -> Self {
        Self([1])
    }

    pub(crate) const fn in_process() -> Self {
        Self([2])
    }

    /// Reports whether server data is ready for use.
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        matches!(self.0, [1])
    }

    /// Reports whether a provider-side operation is in progress.
    #[must_use]
    pub const fn is_in_process(&self) -> bool {
        matches!(self.0, [2])
    }
}

impl Drop for RobotServerStatus {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.0);
    }
}

impl fmt::Debug for RobotServerStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotServerStatus([redacted])")
    }
}

/// Calendar-valid provider date in cleanup-owning storage.
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct RobotServerDate([u8; 4]);

impl RobotServerDate {
    pub(crate) const fn new(year: u16, month: u8, day: u8) -> Option<Self> {
        if year == 0 || month == 0 || month > 12 || day == 0 || day > days_in_month(year, month) {
            return None;
        }
        let [high, low] = year.to_be_bytes();
        Some(Self([high, low, month, day]))
    }

    /// Runs a closure with temporary access to year, month, and day.
    pub fn with_date<R>(&self, inspect: impl FnOnce(u16, u8, u8) -> R) -> R {
        let [high, low, month, day] = self.0;
        inspect(u16::from_be_bytes([high, low]), month, day)
    }
}

impl Drop for RobotServerDate {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.0);
    }
}

impl fmt::Debug for RobotServerDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotServerDate([redacted])")
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

/// Canonical assigned subnet in cleanup-owning storage.
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct RobotServerSubnet {
    network: ProtectedIpAddr,
    prefix: [u8; 1],
}

impl RobotServerSubnet {
    pub(crate) const fn new(network: IpAddr, prefix: u8) -> Self {
        Self {
            network: ProtectedIpAddr::new(network),
            prefix: [prefix],
        }
    }

    /// Runs a closure with temporary access to the canonical network and prefix.
    pub fn with_subnet<R>(&self, inspect: impl FnOnce(IpAddr, u8) -> R) -> R {
        let [prefix] = self.prefix;
        self.network.with_addr(|network| inspect(network, prefix))
    }

    pub(crate) fn identity_key(&self) -> [u8; 18] {
        let address = self.network.identity_key();
        let mut output = [0_u8; 18];
        let (target, prefix_target) = output.split_at_mut(address.len());
        target.copy_from_slice(&address);
        let Some(prefix_target) = prefix_target.first_mut() else {
            unreachable!("subnet identity key prefix is absent")
        };
        let [prefix] = self.prefix;
        *prefix_target = prefix;
        output
    }
}

impl Drop for RobotServerSubnet {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.prefix);
    }
}

impl fmt::Debug for RobotServerSubnet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotServerSubnet([redacted])")
    }
}

/// Source-complete summary returned by `GET /server`.
pub struct RobotServerSummary {
    pub(crate) number: RobotServerNumber,
    pub(crate) main_ipv4: ProtectedIpAddr,
    pub(crate) main_ipv6_network: ProtectedIpAddr,
    pub(crate) name: SensitiveText,
    pub(crate) product: SensitiveText,
    pub(crate) datacenter: SensitiveText,
    pub(crate) traffic: SensitiveText,
    pub(crate) status: RobotServerStatus,
    pub(crate) cancelled: [u8; 1],
    pub(crate) paid_until: RobotServerDate,
    pub(crate) addresses: Vec<ProtectedIpAddr>,
    pub(crate) subnets: Option<Vec<RobotServerSubnet>>,
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
            IpAddr::V4(address) => inspect(address),
            IpAddr::V6(_) => unreachable!("validated main IPv4 changed family"),
        })
    }

    /// Runs a closure with temporary access to the main IPv6 network address.
    pub fn with_main_ipv6_network<R>(&self, inspect: impl FnOnce(Ipv6Addr) -> R) -> R {
        self.main_ipv6_network.with_addr(|address| match address {
            IpAddr::V6(address) => inspect(address),
            IpAddr::V4(_) => unreachable!("validated main IPv6 changed family"),
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
        self.cancelled != [0]
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

impl Drop for RobotServerSummary {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.cancelled);
    }
}

impl fmt::Debug for RobotServerSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotServerSummary([redacted])")
    }
}

/// Source-locked server feature availability in cleanup-owning storage.
#[derive(Eq, PartialEq)]
pub struct RobotServerCapabilities([u8; 8]);

impl RobotServerCapabilities {
    pub(crate) const fn new(values: [bool; 8]) -> Self {
        Self([
            values[0] as u8,
            values[1] as u8,
            values[2] as u8,
            values[3] as u8,
            values[4] as u8,
            values[5] as u8,
            values[6] as u8,
            values[7] as u8,
        ])
    }
    /// Reports reset-system availability.
    #[must_use]
    pub fn reset(&self) -> bool {
        matches!(self.0.first(), Some(1))
    }
    /// Reports rescue-system availability.
    #[must_use]
    pub fn rescue(&self) -> bool {
        matches!(self.0.get(1), Some(1))
    }
    /// Reports VNC installation availability.
    #[must_use]
    pub fn vnc(&self) -> bool {
        matches!(self.0.get(2), Some(1))
    }
    /// Reports Windows installation availability.
    #[must_use]
    pub fn windows(&self) -> bool {
        matches!(self.0.get(3), Some(1))
    }
    /// Reports Plesk installation availability.
    #[must_use]
    pub fn plesk(&self) -> bool {
        matches!(self.0.get(4), Some(1))
    }
    /// Reports cPanel installation availability.
    #[must_use]
    pub fn cpanel(&self) -> bool {
        matches!(self.0.get(5), Some(1))
    }
    /// Reports Wake-on-LAN availability.
    #[must_use]
    pub fn wake_on_lan(&self) -> bool {
        matches!(self.0.get(6), Some(1))
    }
    /// Reports hot-swap availability.
    #[must_use]
    pub fn hot_swap(&self) -> bool {
        matches!(self.0.get(7), Some(1))
    }
}

impl Drop for RobotServerCapabilities {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.0);
    }
}

impl fmt::Debug for RobotServerCapabilities {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotServerCapabilities([redacted])")
    }
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
pub struct RobotServerList(pub(crate) Vec<RobotServerSummary>);

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
