use core::fmt;
use core::net::{Ipv4Addr, Ipv6Addr};

use crate::robot::{RobotIpAddress, RobotServerNumber};

/// Exact server identity returned by Robot Wake-on-LAN operations.
pub struct RobotWol {
    pub(super) server_ipv4: RobotIpAddress,
    pub(super) server_ipv6_network: RobotIpAddress,
    pub(super) number: RobotServerNumber,
}

impl RobotWol {
    pub(super) fn same_identity(&self, other: &Self) -> bool {
        self.server_ipv4 == other.server_ipv4
            && self.server_ipv6_network == other.server_ipv6_network
            && self.number == other.number
    }

    /// Runs a closure with the server's canonical main IPv4 address.
    pub fn with_server_ipv4<R>(&self, inspect: impl FnOnce(Ipv4Addr) -> R) -> R {
        self.server_ipv4.with_addr(|address| match address {
            core::net::IpAddr::V4(address) => inspect(address),
            core::net::IpAddr::V6(_) => unreachable!("validated Robot WOL IPv4 changed family"),
        })
    }

    /// Runs a closure with the server's canonical IPv6 network address.
    pub fn with_server_ipv6_network<R>(&self, inspect: impl FnOnce(Ipv6Addr) -> R) -> R {
        self.server_ipv6_network.with_addr(|address| match address {
            core::net::IpAddr::V6(address) => inspect(address),
            core::net::IpAddr::V4(_) => unreachable!("validated Robot WOL IPv6 changed family"),
        })
    }

    /// Returns the canonical server number.
    #[must_use]
    pub const fn server_number(&self) -> &RobotServerNumber {
        &self.number
    }
}

impl fmt::Debug for RobotWol {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotWol([redacted])")
    }
}
