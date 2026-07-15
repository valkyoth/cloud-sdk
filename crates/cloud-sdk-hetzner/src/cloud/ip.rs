//! Allocation-free IP address and CIDR validation for Cloud request domains.

use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use core::str::FromStr;

/// IP validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IpValidationError {
    /// The value is not an IP address or CIDR in canonical request form.
    InvalidSyntax,
    /// A CIDR prefix exceeds the address-family boundary.
    InvalidPrefix,
    /// Host bits are set where a canonical network address is required.
    HostBitsSet,
    /// The range is not wholly contained in an RFC 1918 private IPv4 block.
    NotPrivateIpv4,
    /// The network or subnet is smaller than the Hetzner endpoint admits.
    RangeTooSmall,
    /// The address is reserved by Hetzner and cannot be used as a gateway.
    ReservedGateway,
    /// The route destination overlaps Hetzner's reserved public gateway address.
    ReservedRouteDestination,
}

impl_static_error!(IpValidationError,
    Self::InvalidSyntax => "IP address or CIDR syntax is invalid",
    Self::InvalidPrefix => "CIDR prefix is invalid",
    Self::HostBitsSet => "CIDR has host bits set",
    Self::NotPrivateIpv4 => "IP range is not private IPv4",
    Self::RangeTooSmall => "IP range is smaller than the provider limit",
    Self::ReservedGateway => "IP address is reserved as a provider gateway",
    Self::ReservedRouteDestination => "route destination overlaps a reserved provider range",
);

/// IP address family carried by a validated CIDR.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IpFamily {
    /// IPv4.
    Ipv4,
    /// IPv6.
    Ipv6,
}

/// Borrowed IPv4 or IPv6 CIDR suitable for Firewall selectors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IpCidr<'a> {
    value: &'a str,
    family: IpFamily,
}

impl<'a> IpCidr<'a> {
    /// Validates CIDR syntax, prefix bounds, and canonical network host bits.
    pub fn new(value: &'a str) -> Result<Self, IpValidationError> {
        let (address, prefix) = split_cidr(value)?;
        let address = IpAddr::from_str(address).map_err(|_| IpValidationError::InvalidSyntax)?;
        let family = match address {
            IpAddr::V4(address) if prefix <= 32 => {
                ensure_ipv4_host_bits_clear(address, prefix)?;
                IpFamily::Ipv4
            }
            IpAddr::V6(address) if prefix <= 128 => {
                ensure_ipv6_host_bits_clear(address, prefix)?;
                IpFamily::Ipv6
            }
            IpAddr::V4(_) | IpAddr::V6(_) => return Err(IpValidationError::InvalidPrefix),
        };
        Ok(Self { value, family })
    }

    /// Returns the validated CIDR.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }

    /// Returns the address family.
    #[must_use]
    pub const fn family(self) -> IpFamily {
        self.family
    }
}

/// Canonical private IPv4 range used by a Network.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkIpRange<'a>(&'a str);

impl<'a> NetworkIpRange<'a> {
    /// Validates an RFC 1918 network range with a minimum size of `/24`.
    pub fn new(value: &'a str) -> Result<Self, IpValidationError> {
        let (address, prefix) = private_ipv4_network(value)?;
        if prefix > 24 {
            return Err(IpValidationError::RangeTooSmall);
        }
        ensure_private_block(address, prefix)?;
        Ok(Self(value))
    }

    /// Returns the validated range.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// Canonical private IPv4 subnet range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SubnetIpRange<'a>(&'a str);

impl<'a> SubnetIpRange<'a> {
    /// Validates an RFC 1918 subnet with a minimum size of `/30`.
    pub fn new(value: &'a str) -> Result<Self, IpValidationError> {
        let (address, prefix) = private_ipv4_network(value)?;
        if prefix > 30 {
            return Err(IpValidationError::RangeTooSmall);
        }
        ensure_private_block(address, prefix)?;
        Ok(Self(value))
    }

    /// Returns the validated range.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// Canonical private IPv4 route destination or the default route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouteDestination<'a>(&'a str);

impl<'a> RouteDestination<'a> {
    /// Validates a route destination.
    pub fn new(value: &'a str) -> Result<Self, IpValidationError> {
        let (address, prefix) = ipv4_network(value)?;
        if !(address.is_unspecified() && prefix == 0) {
            ensure_private_block(address, prefix)?;
            if network_contains(address, prefix, Ipv4Addr::new(172, 31, 1, 1)) {
                return Err(IpValidationError::ReservedRouteDestination);
            }
        }
        Ok(Self(value))
    }

    /// Returns the validated destination.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// Private IPv4 route gateway.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouteGateway<'a>(&'a str);

impl<'a> RouteGateway<'a> {
    /// Validates a private gateway and rejects Hetzner's reserved gateway.
    pub fn new(value: &'a str) -> Result<Self, IpValidationError> {
        let address = Ipv4Addr::from_str(value).map_err(|_| IpValidationError::InvalidSyntax)?;
        if !address.is_private() {
            return Err(IpValidationError::NotPrivateIpv4);
        }
        if address == Ipv4Addr::new(172, 31, 1, 1) {
            return Err(IpValidationError::ReservedGateway);
        }
        Ok(Self(value))
    }

    /// Returns the validated gateway.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

fn split_cidr(value: &str) -> Result<(&str, u8), IpValidationError> {
    let (address, prefix) = value
        .split_once('/')
        .ok_or(IpValidationError::InvalidSyntax)?;
    if address.is_empty() || prefix.is_empty() || prefix.bytes().any(|byte| !byte.is_ascii_digit())
    {
        return Err(IpValidationError::InvalidSyntax);
    }
    let prefix = prefix
        .parse::<u8>()
        .map_err(|_| IpValidationError::InvalidPrefix)?;
    Ok((address, prefix))
}

fn private_ipv4_network(value: &str) -> Result<(Ipv4Addr, u8), IpValidationError> {
    let (address, prefix) = ipv4_network(value)?;
    ensure_private_block(address, prefix)?;
    Ok((address, prefix))
}

fn ipv4_network(value: &str) -> Result<(Ipv4Addr, u8), IpValidationError> {
    let (address, prefix) = split_cidr(value)?;
    if prefix > 32 {
        return Err(IpValidationError::InvalidPrefix);
    }
    let address = Ipv4Addr::from_str(address).map_err(|_| IpValidationError::InvalidSyntax)?;
    ensure_ipv4_host_bits_clear(address, prefix)?;
    Ok((address, prefix))
}

fn ensure_ipv4_host_bits_clear(address: Ipv4Addr, prefix: u8) -> Result<(), IpValidationError> {
    let shift = 32_u32
        .checked_sub(u32::from(prefix))
        .ok_or(IpValidationError::InvalidPrefix)?;
    let mask = u32::MAX.checked_shl(shift).unwrap_or(0);
    if u32::from(address) & mask != u32::from(address) {
        return Err(IpValidationError::HostBitsSet);
    }
    Ok(())
}

fn ensure_ipv6_host_bits_clear(address: Ipv6Addr, prefix: u8) -> Result<(), IpValidationError> {
    let shift = 128_u32
        .checked_sub(u32::from(prefix))
        .ok_or(IpValidationError::InvalidPrefix)?;
    let mask = u128::MAX.checked_shl(shift).unwrap_or(0);
    if u128::from(address) & mask != u128::from(address) {
        return Err(IpValidationError::HostBitsSet);
    }
    Ok(())
}

fn ensure_private_block(address: Ipv4Addr, prefix: u8) -> Result<(), IpValidationError> {
    let octets = address.octets();
    let contained = match octets {
        [10, _, _, _] => prefix >= 8,
        [172, second, _, _] => (16..=31).contains(&second) && prefix >= 12,
        [192, 168, _, _] => prefix >= 16,
        _ => false,
    };
    if !contained {
        return Err(IpValidationError::NotPrivateIpv4);
    }
    Ok(())
}

fn network_contains(network: Ipv4Addr, prefix: u8, address: Ipv4Addr) -> bool {
    let shift = 32_u32.saturating_sub(u32::from(prefix));
    let mask = u32::MAX.checked_shl(shift).unwrap_or(0);
    u32::from(network) & mask == u32::from(address) & mask
}

#[cfg(test)]
mod tests {
    use super::{
        IpCidr, IpFamily, IpValidationError, NetworkIpRange, RouteDestination, RouteGateway,
        SubnetIpRange,
    };

    #[test]
    fn networks_firewalls_cidr_boundaries_are_validated() {
        assert_eq!(
            IpCidr::new("0.0.0.0/0").map(IpCidr::family),
            Ok(IpFamily::Ipv4)
        );
        assert_eq!(IpCidr::new("::/0").map(IpCidr::family), Ok(IpFamily::Ipv6));
        assert_eq!(
            IpCidr::new("192.0.2.42/24"),
            Err(IpValidationError::HostBitsSet)
        );
        assert_eq!(
            IpCidr::new("2001:db8::42/64"),
            Err(IpValidationError::HostBitsSet)
        );
        assert!(IpCidr::new("192.0.2.42/32").is_ok());
        assert!(IpCidr::new("2001:db8::42/128").is_ok());
        assert_eq!(
            IpCidr::new("192.0.2.1/33"),
            Err(IpValidationError::InvalidPrefix)
        );
        assert_eq!(
            IpCidr::new("2001:db8::1/129"),
            Err(IpValidationError::InvalidPrefix)
        );
        assert_eq!(
            IpCidr::new("192.0.2.1"),
            Err(IpValidationError::InvalidSyntax)
        );

        assert!(NetworkIpRange::new("10.0.0.0/24").is_ok());
        assert_eq!(
            NetworkIpRange::new("10.0.0.0/25"),
            Err(IpValidationError::RangeTooSmall)
        );
        assert_eq!(
            NetworkIpRange::new("10.0.0.1/24"),
            Err(IpValidationError::HostBitsSet)
        );
        assert_eq!(
            NetworkIpRange::new("192.0.2.0/24"),
            Err(IpValidationError::NotPrivateIpv4)
        );
        assert_eq!(
            NetworkIpRange::new("172.0.0.0/11"),
            Err(IpValidationError::NotPrivateIpv4)
        );

        assert!(SubnetIpRange::new("192.168.1.0/30").is_ok());
        assert_eq!(
            SubnetIpRange::new("192.168.1.0/31"),
            Err(IpValidationError::RangeTooSmall)
        );
        assert!(RouteDestination::new("0.0.0.0/0").is_ok());
        assert!(RouteDestination::new("10.0.0.4/32").is_ok());
        assert_eq!(
            RouteDestination::new("172.31.1.1/32"),
            Err(IpValidationError::ReservedRouteDestination)
        );
        assert_eq!(
            RouteDestination::new("8.8.8.8/32"),
            Err(IpValidationError::NotPrivateIpv4)
        );
        assert_eq!(
            RouteGateway::new("172.31.1.1"),
            Err(IpValidationError::ReservedGateway)
        );
    }
}
