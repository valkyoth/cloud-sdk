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

/// Non-empty filtered results whose returned entries match checked IP inventory.
///
/// This proves membership at the time the independent inventory was observed.
/// It does not prove that the filtered response is complete or that an omitted
/// reverse-DNS entry does not exist.
pub struct RobotRdnsFilteredMembership(RobotRdnsList);

impl RobotRdnsFilteredMembership {
    pub(super) fn new(results: RobotRdnsList) -> Option<Self> {
        (!results.is_empty()).then_some(Self(results))
    }

    /// Returns the non-empty membership-verified entries.
    #[must_use]
    pub fn as_slice(&self) -> &[RobotRdns] {
        self.0.as_slice()
    }

    /// Returns the number of membership-verified entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Reports whether the result is empty.
    ///
    /// This is always `false`; empty filtered responses are unverifiable and
    /// cannot construct this type.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }
}

impl core::fmt::Debug for RobotRdnsFilteredMembership {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotRdnsFilteredMembership([redacted])")
    }
}
