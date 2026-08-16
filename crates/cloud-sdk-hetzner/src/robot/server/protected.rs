use core::cmp::Ordering;
use core::convert::Infallible;
use core::fmt;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use cloud_sdk_sanitization::SecretBoxBytes;

pub(crate) use super::protected_parse::ProtectedValueError;
use super::protected_parse::{self, AddressFamily};

fn protected(
    len: usize,
    make_byte: impl FnMut(usize) -> u8,
) -> Result<SecretBoxBytes, ProtectedValueError> {
    let mut make_byte = make_byte;
    SecretBoxBytes::try_from_fn_bounded(len, len, |index| Ok::<u8, Infallible>(make_byte(index)))
        .map_err(|_| ProtectedValueError::Allocation)
}

fn protected_cmp(left: &SecretBoxBytes, right: &SecretBoxBytes) -> Ordering {
    left.with_secret(|left| right.with_secret(|right| left.cmp(right)))
}

fn byte_at(bytes: &[u8], index: usize) -> u8 {
    bytes.get(index).copied().unwrap_or(0)
}

fn ipv4_value(bytes: &[u8], offset: usize) -> Ipv4Addr {
    let end = offset
        .checked_add(4)
        .unwrap_or_else(|| unreachable!("fixed IPv4 offset overflowed"));
    let value = (offset..end).fold(0_u32, |value, index| {
        (value << 8) | u32::from(byte_at(bytes, index))
    });
    Ipv4Addr::from(value)
}

fn ipv6_value(bytes: &[u8], offset: usize) -> Ipv6Addr {
    let end = offset
        .checked_add(16)
        .unwrap_or_else(|| unreachable!("fixed IPv6 offset overflowed"));
    let value = (offset..end).fold(0_u128, |value, index| {
        (value << 8) | u128::from(byte_at(bytes, index))
    });
    Ipv6Addr::from(value)
}

/// Stable protected IP address used for classified server topology.
pub struct ProtectedIpAddr(SecretBoxBytes);

impl ProtectedIpAddr {
    pub(crate) fn parse(value: &str) -> Result<Self, ProtectedValueError> {
        protected_parse::address(value, AddressFamily::Any).map(Self)
    }

    pub(super) fn parse_ipv4(value: &str) -> Result<Self, ProtectedValueError> {
        protected_parse::address(value, AddressFamily::V4).map(Self)
    }

    pub(super) fn parse_ipv6(value: &str) -> Result<Self, ProtectedValueError> {
        protected_parse::address(value, AddressFamily::V6).map(Self)
    }

    /// Runs a closure with temporary access to the address.
    pub fn with_addr<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.0.with_secret(|bytes| {
            let address = match bytes.first() {
                Some(4) => IpAddr::V4(ipv4_value(bytes, 1)),
                Some(6) => IpAddr::V6(ipv6_value(bytes, 1)),
                _ => unreachable!("protected address family changed"),
            };
            inspect(address)
        })
    }
}

impl PartialEq for ProtectedIpAddr {
    fn eq(&self, other: &Self) -> bool {
        protected_cmp(&self.0, &other.0) == Ordering::Equal
    }
}
impl Eq for ProtectedIpAddr {}
impl PartialOrd for ProtectedIpAddr {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ProtectedIpAddr {
    fn cmp(&self, other: &Self) -> Ordering {
        protected_cmp(&self.0, &other.0)
    }
}
impl fmt::Debug for ProtectedIpAddr {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ProtectedIpAddr([redacted])")
    }
}

/// Positive linked Storage Box number in stable protected storage.
pub struct RobotStorageBoxNumber(SecretBoxBytes);

impl RobotStorageBoxNumber {
    pub(super) fn from_decimal_bytes(digits: &[u8]) -> Result<Option<Self>, ProtectedValueError> {
        if digits == b"0" {
            return Ok(None);
        }
        if !valid_u64_decimal(digits) {
            return Err(ProtectedValueError::Invalid);
        }
        SecretBoxBytes::try_from_slice(digits, 20)
            .map(Self)
            .map(Some)
            .map_err(|_| ProtectedValueError::Allocation)
    }

    /// Runs a closure with temporary access to the provider number.
    pub fn with_number<R>(&self, inspect: impl FnOnce(u64) -> R) -> R {
        self.0.with_secret(|bytes| {
            inspect(bytes.iter().fold(0_u64, |value, byte| {
                value
                    .saturating_mul(10)
                    .saturating_add(u64::from(byte.saturating_sub(b'0')))
            }))
        })
    }
}

impl fmt::Debug for RobotStorageBoxNumber {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotStorageBoxNumber([redacted])")
    }
}

/// Source-locked Robot server state in stable protected storage.
pub struct RobotServerStatus(SecretBoxBytes);

impl RobotServerStatus {
    pub(super) fn ready() -> Result<Self, ProtectedValueError> {
        protected(1, |_| 1).map(Self)
    }

    pub(super) fn in_process() -> Result<Self, ProtectedValueError> {
        protected(1, |_| 2).map(Self)
    }

    /// Reports whether server data is ready for use.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.0.with_secret(|bytes| bytes == [1])
    }

    /// Reports whether a provider-side operation is in progress.
    #[must_use]
    pub fn is_in_process(&self) -> bool {
        self.0.with_secret(|bytes| bytes == [2])
    }
}

impl fmt::Debug for RobotServerStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotServerStatus([redacted])")
    }
}

/// Calendar-valid provider date in stable protected storage.
pub struct RobotServerDate(SecretBoxBytes);

impl RobotServerDate {
    pub(super) fn parse(value: &str) -> Result<Self, ProtectedValueError> {
        protected_parse::date(value).map(Self)
    }

    /// Runs a closure with temporary access to year, month, and day.
    pub fn with_date<R>(&self, inspect: impl FnOnce(u16, u8, u8) -> R) -> R {
        self.0.with_secret(|bytes| {
            let year = (u16::from(byte_at(bytes, 0)) << 8) | u16::from(byte_at(bytes, 1));
            inspect(year, byte_at(bytes, 2), byte_at(bytes, 3))
        })
    }
}

impl fmt::Debug for RobotServerDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotServerDate([redacted])")
    }
}

/// Canonical assigned subnet in stable protected storage.
pub struct RobotServerSubnet(SecretBoxBytes);

impl RobotServerSubnet {
    pub(super) fn parse(network: &str, prefix: &str) -> Result<Self, ProtectedValueError> {
        protected_parse::subnet(network, prefix).map(Self)
    }

    /// Runs a closure with temporary access to the canonical network and prefix.
    pub fn with_subnet<R>(&self, inspect: impl FnOnce(IpAddr, u8) -> R) -> R {
        self.0.with_secret(|bytes| {
            let network = match bytes.first() {
                Some(4) => IpAddr::V4(ipv4_value(bytes, 1)),
                Some(6) => IpAddr::V6(ipv6_value(bytes, 1)),
                _ => unreachable!("protected subnet family changed"),
            };
            inspect(network, bytes.last().copied().unwrap_or(0))
        })
    }
}

impl PartialEq for RobotServerSubnet {
    fn eq(&self, other: &Self) -> bool {
        protected_cmp(&self.0, &other.0) == Ordering::Equal
    }
}
impl Eq for RobotServerSubnet {}
impl PartialOrd for RobotServerSubnet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RobotServerSubnet {
    fn cmp(&self, other: &Self) -> Ordering {
        protected_cmp(&self.0, &other.0)
    }
}
impl fmt::Debug for RobotServerSubnet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotServerSubnet([redacted])")
    }
}

pub(super) struct ProtectedFlag(SecretBoxBytes);

impl ProtectedFlag {
    pub(super) fn from_protected(
        mut copy: impl FnMut(&mut u8),
    ) -> Result<Self, ProtectedValueError> {
        let mut bytes =
            SecretBoxBytes::try_zeroed(1, 1).map_err(|_| ProtectedValueError::Allocation)?;
        bytes.with_secret_mut(|destination| {
            copy(
                destination
                    .first_mut()
                    .unwrap_or_else(|| unreachable!("fixed protected flag storage was empty")),
            );
        });
        Ok(Self(bytes))
    }

    pub(super) fn get(&self) -> bool {
        self.0.with_secret(|bytes| bytes == [1])
    }
}

/// Source-locked server feature availability in stable protected storage.
pub struct RobotServerCapabilities(SecretBoxBytes);

impl RobotServerCapabilities {
    pub(super) fn from_protected(
        mut copy: impl FnMut(usize, &mut u8),
    ) -> Result<Self, ProtectedValueError> {
        let mut bytes =
            SecretBoxBytes::try_zeroed(8, 8).map_err(|_| ProtectedValueError::Allocation)?;
        bytes.with_secret_mut(|destination| {
            for (index, byte) in destination.iter_mut().enumerate() {
                copy(index, byte);
            }
        });
        Ok(Self(bytes))
    }

    fn capability(&self, index: usize) -> bool {
        self.0.with_secret(|bytes| bytes.get(index) == Some(&1))
    }

    /// Reports reset-system availability.
    #[must_use]
    pub fn reset(&self) -> bool {
        self.capability(0)
    }
    /// Reports rescue-system availability.
    #[must_use]
    pub fn rescue(&self) -> bool {
        self.capability(1)
    }
    /// Reports VNC installation availability.
    #[must_use]
    pub fn vnc(&self) -> bool {
        self.capability(2)
    }
    /// Reports Windows installation availability.
    #[must_use]
    pub fn windows(&self) -> bool {
        self.capability(3)
    }
    /// Reports Plesk installation availability.
    #[must_use]
    pub fn plesk(&self) -> bool {
        self.capability(4)
    }
    /// Reports cPanel installation availability.
    #[must_use]
    pub fn cpanel(&self) -> bool {
        self.capability(5)
    }
    /// Reports Wake-on-LAN availability.
    #[must_use]
    pub fn wake_on_lan(&self) -> bool {
        self.capability(6)
    }
    /// Reports hot-swap availability.
    #[must_use]
    pub fn hot_swap(&self) -> bool {
        self.capability(7)
    }
}

impl fmt::Debug for RobotServerCapabilities {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotServerCapabilities([redacted])")
    }
}

fn valid_u64_decimal(digits: &[u8]) -> bool {
    !digits.is_empty()
        && (digits.len() == 1 || digits.first() != Some(&b'0'))
        && digits.iter().all(u8::is_ascii_digit)
        && (digits.len() < 20 || (digits.len() == 20 && digits <= &b"18446744073709551615"[..]))
}

#[cfg(test)]
mod tests {
    use super::{
        ProtectedIpAddr, ProtectedValueError, RobotServerDate, RobotServerSubnet, protected,
    };
    use core::net::IpAddr;
    use core::str::FromStr;

    #[test]
    fn classified_allocation_address_survives_owner_moves() {
        let address = ProtectedIpAddr::parse("192.0.2.1")
            .unwrap_or_else(|_| unreachable!("protected fixture allocation failed"));
        let before = address.0.with_secret(<[u8]>::as_ptr);
        let moved = address;
        assert_eq!(before, moved.0.with_secret(<[u8]>::as_ptr));
    }

    #[test]
    fn impossible_protected_capacity_maps_to_failure() {
        assert!(matches!(
            protected(usize::MAX, |_| 0),
            Err(ProtectedValueError::Allocation)
        ));
    }

    #[test]
    fn protected_parser_accepts_complete_address_families() {
        for text in [
            "192.0.2.1",
            "2001:db8::1",
            "2001:db8:0:1:2:3:4:5",
            "::ffff:192.0.2.1",
            "::",
        ] {
            let protected = ProtectedIpAddr::parse(text)
                .unwrap_or_else(|_| unreachable!("valid address fixture was rejected"));
            let expected = IpAddr::from_str(text)
                .unwrap_or_else(|_| unreachable!("standard parser rejected fixture"));
            assert!(protected.with_addr(|actual| actual == expected));
        }

        for text in [
            "192.0.2",
            "192.00.2.1",
            "2001:::1",
            "2001:db8:1",
            "0.0.10.7::1",
            "192.0.2.1::",
            "192.0.2.1::1",
        ] {
            assert!(ProtectedIpAddr::parse(text).is_err());
        }
    }

    #[test]
    fn protected_date_and_subnet_parsers_fail_closed() {
        assert!(RobotServerDate::parse("2028-02-29").is_ok());
        assert!(RobotServerDate::parse("2027-02-29").is_err());
        assert!(RobotServerSubnet::parse("192.0.2.0", "24").is_ok());
        assert!(RobotServerSubnet::parse("2001:db8::", "64").is_ok());
        assert!(RobotServerSubnet::parse("192.0.2.1", "24").is_err());
        assert!(RobotServerSubnet::parse("2001:db8::1", "64").is_err());
    }
}
