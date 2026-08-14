use alloc::vec::Vec;
use core::fmt;
use core::net::IpAddr;

use crate::robot::{RobotIpAddress, RobotServerNumber};

use super::{RobotVSwitchId, RobotVSwitchObservedName, RobotVlanId};

/// Maximum vSwitch summaries admitted from one list response.
pub const MAX_ROBOT_VSWITCH_LIST_ITEMS: usize = 4_096;
/// Maximum server memberships admitted in one detail response.
pub const MAX_ROBOT_VSWITCH_MEMBER_SERVERS: usize = 4_096;
/// Maximum routed subnets admitted in one detail response.
pub const MAX_ROBOT_VSWITCH_SUBNETS: usize = 4_096;
/// Maximum linked Cloud Networks admitted in one detail response.
pub const MAX_ROBOT_VSWITCH_CLOUD_NETWORKS: usize = 4_096;

/// One bounded Robot vSwitch inventory summary.
pub struct RobotVSwitchSummary {
    pub(super) id: RobotVSwitchId,
    pub(super) name: RobotVSwitchObservedName,
    pub(super) vlan: RobotVlanId,
    pub(super) cancelled: bool,
}

impl RobotVSwitchSummary {
    /// Returns the vSwitch identity.
    #[must_use]
    pub const fn id(&self) -> RobotVSwitchId {
        self.id
    }
    /// Returns the protected name.
    #[must_use]
    pub const fn name(&self) -> &RobotVSwitchObservedName {
        &self.name
    }
    /// Returns the VLAN identity.
    #[must_use]
    pub const fn vlan(&self) -> RobotVlanId {
        self.vlan
    }
    /// Reports whether cancellation is scheduled or complete.
    #[must_use]
    pub const fn cancelled(&self) -> bool {
        self.cancelled
    }
}

impl fmt::Debug for RobotVSwitchSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotVSwitchSummary([redacted])")
    }
}

/// Bounded vSwitch inventory with unique provider identities.
pub struct RobotVSwitchList(pub(super) Vec<RobotVSwitchSummary>);

impl RobotVSwitchList {
    /// Returns the admitted summaries.
    #[must_use]
    pub fn as_slice(&self) -> &[RobotVSwitchSummary] {
        &self.0
    }
    /// Returns the number of summaries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Reports whether the inventory is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for RobotVSwitchList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotVSwitchList")
            .field("len", &self.0.len())
            .field("items", &"[redacted]")
            .finish()
    }
}

/// Provider state for one vSwitch server membership.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotVSwitchServerStatus {
    /// Membership is active.
    Ready,
    /// A provider transition is still running.
    InProcess,
    /// The provider transition failed.
    Failed,
}

/// One source-complete server membership.
pub struct RobotVSwitchServer {
    pub(super) ipv4: RobotIpAddress,
    pub(super) ipv6_network: RobotIpAddress,
    pub(super) number: RobotServerNumber,
    pub(super) status: RobotVSwitchServerStatus,
}

impl RobotVSwitchServer {
    /// Runs a closure with the main IPv4 address.
    pub fn with_ipv4<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.ipv4.with_addr(inspect)
    }
    /// Runs a closure with the main IPv6 network address.
    pub fn with_ipv6_network<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.ipv6_network.with_addr(inspect)
    }
    /// Returns the protected server number.
    #[must_use]
    pub const fn number(&self) -> &RobotServerNumber {
        &self.number
    }
    /// Returns the provider membership state.
    #[must_use]
    pub const fn status(&self) -> RobotVSwitchServerStatus {
        self.status
    }
}

impl fmt::Debug for RobotVSwitchServer {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotVSwitchServer([redacted])")
    }
}

/// One routed vSwitch subnet.
pub struct RobotVSwitchSubnet {
    pub(super) network: RobotIpAddress,
    pub(super) prefix: u8,
    pub(super) gateway: RobotIpAddress,
}

impl RobotVSwitchSubnet {
    /// Runs a closure with the canonical network address.
    pub fn with_network<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.network.with_addr(inspect)
    }
    /// Returns the CIDR prefix length.
    #[must_use]
    pub const fn prefix(&self) -> u8 {
        self.prefix
    }
    /// Runs a closure with the gateway address.
    pub fn with_gateway<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.gateway.with_addr(inspect)
    }
}

impl fmt::Debug for RobotVSwitchSubnet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotVSwitchSubnet([redacted])")
    }
}

/// One linked Hetzner Cloud Network route.
pub struct RobotVSwitchCloudNetwork {
    pub(super) id: u64,
    pub(super) network: RobotIpAddress,
    pub(super) prefix: u8,
    pub(super) gateway: RobotIpAddress,
}

impl RobotVSwitchCloudNetwork {
    /// Returns the non-zero Cloud Network ID.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }
    /// Runs a closure with the canonical network address.
    pub fn with_network<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.network.with_addr(inspect)
    }
    /// Returns the CIDR prefix length.
    #[must_use]
    pub const fn prefix(&self) -> u8 {
        self.prefix
    }
    /// Runs a closure with the gateway address.
    pub fn with_gateway<R>(&self, inspect: impl FnOnce(IpAddr) -> R) -> R {
        self.gateway.with_addr(inspect)
    }
}

impl fmt::Debug for RobotVSwitchCloudNetwork {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotVSwitchCloudNetwork([redacted])")
    }
}

/// One source-complete Robot vSwitch detail resource.
pub struct RobotVSwitch {
    pub(super) id: RobotVSwitchId,
    pub(super) name: RobotVSwitchObservedName,
    pub(super) vlan: RobotVlanId,
    pub(super) cancelled: bool,
    pub(super) servers: Vec<RobotVSwitchServer>,
    pub(super) subnets: Vec<RobotVSwitchSubnet>,
    pub(super) cloud_networks: Vec<RobotVSwitchCloudNetwork>,
}

impl RobotVSwitch {
    /// Returns the vSwitch identity.
    #[must_use]
    pub const fn id(&self) -> RobotVSwitchId {
        self.id
    }
    /// Returns the protected name.
    #[must_use]
    pub const fn name(&self) -> &RobotVSwitchObservedName {
        &self.name
    }
    /// Returns the VLAN identity.
    #[must_use]
    pub const fn vlan(&self) -> RobotVlanId {
        self.vlan
    }
    /// Reports whether cancellation is scheduled or complete.
    #[must_use]
    pub const fn cancelled(&self) -> bool {
        self.cancelled
    }
    /// Returns server memberships in provider order.
    #[must_use]
    pub fn servers(&self) -> &[RobotVSwitchServer] {
        &self.servers
    }
    /// Returns routed subnets in provider order.
    #[must_use]
    pub fn subnets(&self) -> &[RobotVSwitchSubnet] {
        &self.subnets
    }
    /// Returns linked Cloud Networks in provider order.
    #[must_use]
    pub fn cloud_networks(&self) -> &[RobotVSwitchCloudNetwork] {
        &self.cloud_networks
    }
}

impl fmt::Debug for RobotVSwitch {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotVSwitch")
            .field("id", &self.id)
            .field("name", &"[redacted]")
            .field("vlan", &self.vlan)
            .field("cancelled", &self.cancelled)
            .field("servers", &self.servers.len())
            .field("subnets", &self.subnets.len())
            .field("cloud_networks", &self.cloud_networks.len())
            .finish()
    }
}
