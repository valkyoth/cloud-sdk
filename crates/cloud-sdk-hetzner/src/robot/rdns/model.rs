use alloc::vec::Vec;
use core::net::IpAddr;

use crate::robot::{RobotIpAddress, RobotRdnsName};

/// Maximum reverse-DNS resources admitted from one Robot list response.
pub const MAX_ROBOT_RDNS_LIST_ITEMS: usize = 4_096;

/// One exact Robot reverse-DNS entry.
pub struct RobotRdns {
    pub(super) address: RobotIpAddress,
    pub(super) ptr: RobotRdnsName,
}

impl RobotRdns {
    /// Runs a closure with the canonical address.
    pub fn with_address<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.address.with_addr(inspect)
    }

    /// Returns the protected canonical PTR target.
    #[must_use]
    pub const fn ptr(&self) -> &RobotRdnsName {
        &self.ptr
    }
}

impl core::fmt::Debug for RobotRdns {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotRdns([redacted])")
    }
}

/// Bounded list of distinct Robot reverse-DNS entries.
pub struct RobotRdnsList(pub(super) Vec<RobotRdns>);

impl RobotRdnsList {
    /// Returns the protected reverse-DNS entries.
    #[must_use]
    pub fn as_slice(&self) -> &[RobotRdns] {
        &self.0
    }

    /// Returns the number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Reports whether no entries were returned.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl core::fmt::Debug for RobotRdnsList {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotRdnsList([redacted])")
    }
}
