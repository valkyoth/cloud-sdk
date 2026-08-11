use core::cmp::Ordering;
use core::convert::Infallible;
use core::fmt;
use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use cloud_sdk_sanitization::SecretBoxBytes;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ProtectedValueError;

fn protected(
    len: usize,
    make_byte: impl FnMut(usize) -> u8,
) -> Result<SecretBoxBytes, ProtectedValueError> {
    let mut make_byte = make_byte;
    SecretBoxBytes::try_from_fn_bounded(len, len, |index| Ok::<u8, Infallible>(make_byte(index)))
        .map_err(|_| ProtectedValueError)
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

fn encoded_address_byte(address: IpAddr, index: usize) -> u8 {
    match address {
        IpAddr::V4(address) => {
            if index == 0 {
                4
            } else {
                let shift = 32_usize.saturating_sub(index.saturating_mul(8));
                u8::try_from((u32::from(address) >> shift) & 0xff)
                    .unwrap_or_else(|_| unreachable!("masked IPv4 byte exceeded u8"))
            }
        }
        IpAddr::V6(address) => {
            if index == 0 {
                6
            } else {
                let shift = 128_usize.saturating_sub(index.saturating_mul(8));
                u8::try_from((u128::from(address) >> shift) & 0xff)
                    .unwrap_or_else(|_| unreachable!("masked IPv6 byte exceeded u8"))
            }
        }
    }
}

/// Stable protected IP address used for classified server topology.
pub struct ProtectedIpAddr(SecretBoxBytes);

impl ProtectedIpAddr {
    pub(super) fn new(value: IpAddr) -> Result<Self, ProtectedValueError> {
        let len = match value {
            IpAddr::V4(_) => 5,
            IpAddr::V6(_) => 17,
        };
        protected(len, |index| encoded_address_byte(value, index)).map(Self)
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
    pub(super) fn new(value: u64) -> Result<Option<Self>, ProtectedValueError> {
        if value == 0 {
            return Ok(None);
        }
        protected(8, |index| {
            let shift = 56_usize.saturating_sub(index.saturating_mul(8));
            u8::try_from((value >> shift) & 0xff)
                .unwrap_or_else(|_| unreachable!("masked Storage Box byte exceeded u8"))
        })
        .map(Self)
        .map(Some)
    }

    /// Runs a closure with temporary access to the provider number.
    pub fn with_number<R>(&self, inspect: impl FnOnce(u64) -> R) -> R {
        self.0.with_secret(|bytes| {
            inspect(
                bytes
                    .iter()
                    .fold(0_u64, |value, byte| (value << 8) | u64::from(*byte)),
            )
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
    pub(super) fn new(year: u16, month: u8, day: u8) -> Result<Option<Self>, ProtectedValueError> {
        if year == 0 || month == 0 || month > 12 || day == 0 || day > days_in_month(year, month) {
            return Ok(None);
        }
        protected(4, |index| match index {
            0 => u8::try_from((year >> 8) & 0xff)
                .unwrap_or_else(|_| unreachable!("masked date byte exceeded u8")),
            1 => u8::try_from(year & 0xff)
                .unwrap_or_else(|_| unreachable!("masked date byte exceeded u8")),
            2 => month,
            _ => day,
        })
        .map(Self)
        .map(Some)
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

/// Canonical assigned subnet in stable protected storage.
pub struct RobotServerSubnet(SecretBoxBytes);

impl RobotServerSubnet {
    pub(super) fn new(network: IpAddr, prefix: u8) -> Result<Self, ProtectedValueError> {
        let address_len: usize = match network {
            IpAddr::V4(_) => 5,
            IpAddr::V6(_) => 17,
        };
        let protected_len = address_len
            .checked_add(1)
            .unwrap_or_else(|| unreachable!("fixed subnet length overflowed"));
        protected(protected_len, |index| {
            if index == address_len {
                prefix
            } else {
                encoded_address_byte(network, index)
            }
        })
        .map(Self)
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
    pub(super) fn new(value: bool) -> Result<Self, ProtectedValueError> {
        protected(1, |_| u8::from(value)).map(Self)
    }

    pub(super) fn get(&self) -> bool {
        self.0.with_secret(|bytes| bytes == [1])
    }
}

/// Source-locked server feature availability in stable protected storage.
pub struct RobotServerCapabilities(SecretBoxBytes);

impl RobotServerCapabilities {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        reset: bool,
        rescue: bool,
        vnc: bool,
        windows: bool,
        plesk: bool,
        cpanel: bool,
        wake_on_lan: bool,
        hot_swap: bool,
    ) -> Result<Self, ProtectedValueError> {
        protected(8, |index| {
            u8::from(match index {
                0 => reset,
                1 => rescue,
                2 => vnc,
                3 => windows,
                4 => plesk,
                5 => cpanel,
                6 => wake_on_lan,
                _ => hot_swap,
            })
        })
        .map(Self)
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

#[cfg(test)]
mod tests {
    use super::{ProtectedIpAddr, protected};
    use core::net::{IpAddr, Ipv4Addr};

    #[test]
    fn classified_allocation_address_survives_owner_moves() {
        let address = ProtectedIpAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)))
            .unwrap_or_else(|_| unreachable!("protected fixture allocation failed"));
        let before = address.0.with_secret(<[u8]>::as_ptr);
        let moved = address;
        assert_eq!(before, moved.0.with_secret(<[u8]>::as_ptr));
    }

    #[test]
    fn impossible_protected_capacity_maps_to_failure() {
        assert!(protected(usize::MAX, |_| 0).is_err());
    }
}
