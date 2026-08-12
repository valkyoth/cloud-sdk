use alloc::vec::Vec;
use core::fmt;
use core::net::IpAddr;

use crate::robot::{RobotIpAddress, RobotServerNumber};

/// Maximum failover resources admitted from one Robot list response.
pub const MAX_ROBOT_FAILOVER_LIST_ITEMS: usize = 4_096;

/// One source-complete Robot failover route.
pub struct RobotFailover {
    pub(super) route: RobotIpAddress,
    pub(super) prefix: u8,
    pub(super) server_ipv4: RobotIpAddress,
    pub(super) server_ipv6_network: RobotIpAddress,
    pub(super) server_number: RobotServerNumber,
    pub(super) active_server: Option<RobotIpAddress>,
}

impl RobotFailover {
    /// Runs a closure with the canonical failover route address.
    pub fn with_route<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.route.with_addr(inspect)
    }

    /// Returns the contiguous failover network prefix.
    #[must_use]
    pub const fn prefix(&self) -> u8 {
        self.prefix
    }

    /// Runs a closure with the owning server's main IPv4 address.
    pub fn with_server_ipv4<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.server_ipv4.with_addr(inspect)
    }

    /// Runs a closure with the owning server's main IPv6 network address.
    pub fn with_server_ipv6_network<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.server_ipv6_network.with_addr(inspect)
    }

    /// Returns the owning server number.
    #[must_use]
    pub const fn server_number(&self) -> &RobotServerNumber {
        &self.server_number
    }

    /// Runs a closure with the active destination, when routing is present.
    pub fn with_active_server<R>(&self, inspect: impl FnOnce(Option<IpAddr>) -> R) -> R {
        match self.active_server.as_ref() {
            Some(value) => value.with_addr(|value| inspect(Some(value))),
            None => inspect(None),
        }
    }
}

impl fmt::Debug for RobotFailover {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotFailover([redacted])")
    }
}

/// Bounded list of Robot failover routes.
pub struct RobotFailoverList(pub(super) Vec<RobotFailover>);

impl RobotFailoverList {
    /// Returns the protected failover routes.
    #[must_use]
    pub fn as_slice(&self) -> &[RobotFailover] {
        &self.0
    }

    /// Returns the number of failover routes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Reports whether no failover routes were returned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for RobotFailoverList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotFailoverList([redacted])")
    }
}
