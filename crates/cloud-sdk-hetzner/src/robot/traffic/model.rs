use alloc::vec::Vec;
use core::net::IpAddr;

use cloud_sdk_sanitization::SecretString;

use super::RobotTrafficGranularity;
use crate::robot::{RobotIpAddress, RobotSubnetAddress};

/// Exact non-negative Robot traffic amount in gigabytes.
pub struct RobotTrafficAmount(SecretString);

impl RobotTrafficAmount {
    pub(super) fn new(value: &str) -> Result<Self, ()> {
        if value.is_empty() || value.len() > 128 || value.as_bytes().starts_with(b"-") {
            return Err(());
        }
        SecretString::try_from_secret_str_bounded(value, 128)
            .map(Self)
            .map_err(|_| ())
    }

    /// Runs a closure with the exact provider number token.
    pub fn try_with_lexical<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0.try_with_secret(inspect)
    }
}

impl core::fmt::Debug for RobotTrafficAmount {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotTrafficAmount([redacted])")
    }
}

/// Exact inbound, outbound, and total traffic values for one period.
pub struct RobotTrafficData {
    pub(super) incoming: RobotTrafficAmount,
    pub(super) outgoing: RobotTrafficAmount,
    pub(super) total: RobotTrafficAmount,
}

impl RobotTrafficData {
    /// Returns exact inbound traffic in gigabytes.
    #[must_use]
    pub const fn incoming(&self) -> &RobotTrafficAmount {
        &self.incoming
    }

    /// Returns exact outbound traffic in gigabytes.
    #[must_use]
    pub const fn outgoing(&self) -> &RobotTrafficAmount {
        &self.outgoing
    }

    /// Returns exact total traffic in gigabytes.
    #[must_use]
    pub const fn total(&self) -> &RobotTrafficAmount {
        &self.total
    }
}

impl core::fmt::Debug for RobotTrafficData {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotTrafficData([redacted])")
    }
}

/// One hourly, daily, or monthly traffic value.
pub struct RobotTrafficPoint {
    pub(super) ordinal: u8,
    pub(super) data: RobotTrafficData,
}

impl RobotTrafficPoint {
    /// Returns the source ordinal (`0..=23`, `1..=31`, or `1..=12`).
    #[must_use]
    pub const fn ordinal(&self) -> u8 {
        self.ordinal
    }

    /// Returns this interval's exact traffic values.
    #[must_use]
    pub const fn data(&self) -> &RobotTrafficData {
        &self.data
    }
}

impl core::fmt::Debug for RobotTrafficPoint {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotTrafficPoint")
            .field("ordinal", &self.ordinal)
            .field("data", &"[redacted]")
            .finish()
    }
}

/// Request-bound target identity returned by Robot.
pub enum RobotTrafficResultTarget {
    /// One canonical IP address.
    Ip(RobotIpAddress),
    /// One canonical subnet network and family-valid prefix.
    Subnet {
        /// Protected canonical network address.
        address: RobotSubnetAddress,
        /// Prefix length returned by Robot.
        prefix: u8,
    },
}

impl RobotTrafficResultTarget {
    /// Runs a closure with the canonical address.
    pub fn with_address<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        match self {
            Self::Ip(address) => address.with_addr(inspect),
            Self::Subnet { address, .. } => address.with_addr(inspect),
        }
    }

    /// Returns the subnet prefix, or `None` for an IP target.
    #[must_use]
    pub const fn prefix(&self) -> Option<u8> {
        match self {
            Self::Ip(_) => None,
            Self::Subnet { prefix, .. } => Some(*prefix),
        }
    }
}

impl core::fmt::Debug for RobotTrafficResultTarget {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::Ip(_) => "RobotTrafficResultTarget::Ip([redacted])",
            Self::Subnet { .. } => "RobotTrafficResultTarget::Subnet([redacted])",
        })
    }
}

/// Traffic data for one returned target.
pub enum RobotTrafficResult {
    /// One aggregate across the requested interval.
    Aggregate {
        /// Request-bound target.
        target: RobotTrafficResultTarget,
        /// Aggregate traffic values.
        data: RobotTrafficData,
    },
    /// Sparse, sorted individual interval values.
    SingleValues {
        /// Request-bound target.
        target: RobotTrafficResultTarget,
        /// Distinct values sorted by ordinal.
        points: Vec<RobotTrafficPoint>,
    },
}

impl RobotTrafficResult {
    /// Returns this result's request-bound target.
    #[must_use]
    pub const fn target(&self) -> &RobotTrafficResultTarget {
        match self {
            Self::Aggregate { target, .. } | Self::SingleValues { target, .. } => target,
        }
    }

    /// Returns aggregate data when separate values were not requested.
    #[must_use]
    pub const fn aggregate(&self) -> Option<&RobotTrafficData> {
        match self {
            Self::Aggregate { data, .. } => Some(data),
            Self::SingleValues { .. } => None,
        }
    }

    /// Returns separate values when requested.
    #[must_use]
    pub fn points(&self) -> Option<&[RobotTrafficPoint]> {
        match self {
            Self::Aggregate { .. } => None,
            Self::SingleValues { points, .. } => Some(points),
        }
    }
}

impl core::fmt::Debug for RobotTrafficResult {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotTrafficResult([redacted])")
    }
}

/// Bounded request-bound Robot traffic report.
pub struct RobotTrafficReport {
    pub(super) granularity: RobotTrafficGranularity,
    pub(super) results: Vec<RobotTrafficResult>,
}

impl RobotTrafficReport {
    /// Returns the source-locked aggregation granularity.
    #[must_use]
    pub const fn granularity(&self) -> RobotTrafficGranularity {
        self.granularity
    }

    /// Returns the targets for which Robot supplied data.
    #[must_use]
    pub fn results(&self) -> &[RobotTrafficResult] {
        &self.results
    }

    /// Returns the number of targets with traffic data.
    #[must_use]
    pub fn len(&self) -> usize {
        self.results.len()
    }

    /// Reports whether Robot omitted every requested target as having no data.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }
}

impl core::fmt::Debug for RobotTrafficReport {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotTrafficReport")
            .field("granularity", &self.granularity)
            .field("results", &self.results.len())
            .finish()
    }
}
