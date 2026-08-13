use alloc::vec::Vec;
use core::net::IpAddr;

use cloud_sdk::rate_limit::DelaySeconds;

use super::RobotTrafficInterval;
use crate::robot::{RobotIpAddress, RobotSubnetAddress};

/// Maximum distinct targets admitted in one traffic query.
pub const MAX_ROBOT_TRAFFIC_TARGETS: usize = 4_092;
/// Maximum targets admitted when each interval value is requested separately.
pub const MAX_ROBOT_TRAFFIC_SINGLE_VALUE_TARGETS: usize = 250;

/// Source-locked traffic-query quota.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RobotTrafficQuota {
    max_requests: u16,
    interval: DelaySeconds,
}

impl RobotTrafficQuota {
    /// Returns the documented request allowance.
    #[must_use]
    pub const fn max_requests(self) -> u16 {
        self.max_requests
    }

    /// Returns the documented quota interval.
    #[must_use]
    pub const fn interval(self) -> DelaySeconds {
        self.interval
    }
}

/// Two hundred traffic requests per one-hour window.
pub const ROBOT_TRAFFIC_QUOTA: RobotTrafficQuota = RobotTrafficQuota {
    max_requests: 200,
    interval: DelaySeconds::new(3_600),
};

/// One protected IP or subnet target for a Robot traffic query.
pub enum RobotTrafficTarget {
    /// A single canonical IP address encoded as `ip[]`.
    Ip(RobotIpAddress),
    /// A canonical subnet base address encoded as `subnet[]`.
    Subnet(RobotSubnetAddress),
}

impl RobotTrafficTarget {
    /// Creates an IP target.
    #[must_use]
    pub const fn ip(address: RobotIpAddress) -> Self {
        Self::Ip(address)
    }

    /// Creates a subnet target.
    #[must_use]
    pub const fn subnet(address: RobotSubnetAddress) -> Self {
        Self::Subnet(address)
    }

    /// Runs a closure with the canonical address and target kind.
    pub fn with_address<R>(&self, inspect: impl FnOnce(IpAddr, bool) -> R) -> R {
        match self {
            Self::Ip(address) => address.with_addr(|address| inspect(address, false)),
            Self::Subnet(address) => address.with_addr(|address| inspect(address, true)),
        }
    }

    fn sort_key(&self) -> (IpAddr, bool) {
        self.with_address(|address, subnet| (address, subnet))
    }

    fn address(&self) -> IpAddr {
        self.with_address(|address, _| address)
    }

    pub(super) fn with_text<R>(&self, inspect: impl FnOnce(&str) -> R) -> R {
        match self {
            Self::Ip(address) => address.with_text(inspect),
            Self::Subnet(address) => address.with_text(inspect),
        }
    }
}

impl core::fmt::Debug for RobotTrafficTarget {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::Ip(_) => "RobotTrafficTarget::Ip([redacted])",
            Self::Subnet(_) => "RobotTrafficTarget::Subnet([redacted])",
        })
    }
}

/// Failure while validating or preparing a Robot traffic query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotTrafficRequestError {
    /// No traffic target was supplied.
    MissingTarget,
    /// The target count exceeds [`MAX_ROBOT_TRAFFIC_TARGETS`].
    TooManyTargets,
    /// A canonical address occurs more than once, including across target kinds.
    DuplicateTarget,
    /// Caller-owned path or body storage was too small.
    Storage,
    /// The constructed request target was rejected.
    InvalidTarget(cloud_sdk::transport::RequestTargetError),
    /// Source-locked request headers were rejected.
    InvalidHeaders(cloud_sdk::transport::HeaderError),
    /// The official Robot endpoint policy was invalid.
    InvalidEndpoint(crate::endpoint::OfficialEndpointError),
    /// Operation safety metadata was internally inconsistent.
    InvalidMetadata(cloud_sdk::operation::OperationMetadataError),
    /// The success-response policy was internally inconsistent.
    InvalidResponsePolicy(cloud_sdk::operation::ResponsePolicyValidationError),
    /// The raw response-wire policy was internally inconsistent.
    InvalidRawPolicy(cloud_sdk::transport::RawResponsePolicyError),
    /// Cross-policy prepared-request validation failed.
    InvalidPreparedPolicy(cloud_sdk::operation::PreparedRequestPolicyError),
}

impl_static_error!(RobotTrafficRequestError,
    Self::MissingTarget => "Robot traffic request has no target",
    Self::TooManyTargets => "Robot traffic request has too many targets",
    Self::DuplicateTarget => "Robot traffic request repeats a target address",
    Self::Storage => "Robot traffic request storage is too small",
    Self::InvalidTarget(_) => "Robot traffic request target is invalid",
    Self::InvalidHeaders(_) => "Robot traffic request headers are invalid",
    Self::InvalidEndpoint(_) => "official Robot endpoint is invalid",
    Self::InvalidMetadata(_) => "Robot traffic operation metadata is invalid",
    Self::InvalidResponsePolicy(_) => "Robot traffic response policy is invalid",
    Self::InvalidRawPolicy(_) => "Robot traffic raw response policy is invalid",
    Self::InvalidPreparedPolicy(_) => "Robot traffic prepared policy is invalid",
);

/// Bounded Robot traffic query for one or more IP and subnet targets.
pub struct RobotTrafficRequest {
    pub(super) interval: RobotTrafficInterval,
    pub(super) targets: Vec<RobotTrafficTarget>,
    pub(super) single_values: bool,
}

impl RobotTrafficRequest {
    /// Creates a query and canonicalizes a bounded, duplicate-free target set.
    pub fn new(
        interval: RobotTrafficInterval,
        mut targets: Vec<RobotTrafficTarget>,
        single_values: bool,
    ) -> Result<Self, RobotTrafficRequestError> {
        if targets.is_empty() {
            return Err(RobotTrafficRequestError::MissingTarget);
        }
        if targets.len() > MAX_ROBOT_TRAFFIC_TARGETS
            || (single_values && targets.len() > MAX_ROBOT_TRAFFIC_SINGLE_VALUE_TARGETS)
        {
            return Err(RobotTrafficRequestError::TooManyTargets);
        }
        targets.sort_unstable_by_key(RobotTrafficTarget::sort_key);
        if targets.windows(2).any(|pair| match pair {
            [left, right] => left.address() == right.address(),
            _ => false,
        }) {
            return Err(RobotTrafficRequestError::DuplicateTarget);
        }
        Ok(Self {
            interval,
            targets,
            single_values,
        })
    }

    pub(super) fn target_index(&self, address: IpAddr) -> Option<usize> {
        self.targets
            .binary_search_by_key(&address, RobotTrafficTarget::address)
            .ok()
    }

    /// Returns the exact protected query interval.
    #[must_use]
    pub const fn interval(&self) -> &RobotTrafficInterval {
        &self.interval
    }

    /// Returns the protected target set.
    #[must_use]
    pub fn targets(&self) -> &[RobotTrafficTarget] {
        &self.targets
    }

    /// Reports whether interval values were requested separately.
    #[must_use]
    pub const fn single_values(&self) -> bool {
        self.single_values
    }
}

impl core::fmt::Debug for RobotTrafficRequest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotTrafficRequest")
            .field("interval", &self.interval)
            .field("targets", &self.targets.len())
            .field("single_values", &self.single_values)
            .finish()
    }
}
