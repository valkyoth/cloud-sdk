//! Validated Network request value types.

use super::super::super::ip::{RouteDestination, RouteGateway, SubnetIpRange};
use super::super::super::shared::{CloudLabels, CloudName, CloudRequestError, CloudResourceId};

/// Network identifier.
pub type NetworkId = CloudResourceId;
/// Network name.
pub type NetworkName<'a> = CloudName<'a>;
/// Network zone name.
pub type NetworkZone<'a> = CloudName<'a>;
/// Network labels.
pub type NetworkLabels<'a> = CloudLabels<'a>;
/// Network request error.
pub type NetworkRequestError = CloudRequestError;

/// Nonzero Robot vSwitch identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkVswitchId(u64);

impl NetworkVswitchId {
    /// Creates a nonzero vSwitch identifier.
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    /// Returns the identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Non-deprecated Network subnet type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkSubnetType {
    /// Connect Cloud servers and load balancers.
    Cloud,
    /// Connect Cloud resources to a Robot vSwitch.
    Vswitch,
}

/// Validated Network subnet request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkSubnet<'a> {
    subnet_type: NetworkSubnetType,
    network_zone: NetworkZone<'a>,
    ip_range: Option<SubnetIpRange<'a>>,
    vswitch_id: Option<NetworkVswitchId>,
}

impl<'a> NetworkSubnet<'a> {
    /// Creates a Cloud subnet. The API may allocate the range when omitted.
    #[must_use]
    pub const fn cloud(network_zone: NetworkZone<'a>, ip_range: Option<SubnetIpRange<'a>>) -> Self {
        Self {
            subnet_type: NetworkSubnetType::Cloud,
            network_zone,
            ip_range,
            vswitch_id: None,
        }
    }

    /// Creates a vSwitch subnet with the required vSwitch identifier.
    #[must_use]
    pub const fn vswitch(
        network_zone: NetworkZone<'a>,
        ip_range: Option<SubnetIpRange<'a>>,
        vswitch_id: NetworkVswitchId,
    ) -> Self {
        Self {
            subnet_type: NetworkSubnetType::Vswitch,
            network_zone,
            ip_range,
            vswitch_id: Some(vswitch_id),
        }
    }

    /// Returns the non-deprecated subnet type.
    #[must_use]
    pub const fn subnet_type(self) -> NetworkSubnetType {
        self.subnet_type
    }

    /// Returns the network zone.
    #[must_use]
    pub const fn network_zone(self) -> NetworkZone<'a> {
        self.network_zone
    }

    /// Returns the optional range.
    #[must_use]
    pub const fn ip_range(self) -> Option<SubnetIpRange<'a>> {
        self.ip_range
    }

    /// Returns the vSwitch ID only for vSwitch subnets.
    #[must_use]
    pub const fn vswitch_id(self) -> Option<NetworkVswitchId> {
        self.vswitch_id
    }
}

/// Validated Network route request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkRoute<'a> {
    destination: RouteDestination<'a>,
    gateway: RouteGateway<'a>,
}

impl<'a> NetworkRoute<'a> {
    /// Creates a route with an explicit destination and gateway.
    #[must_use]
    pub const fn new(destination: RouteDestination<'a>, gateway: RouteGateway<'a>) -> Self {
        Self {
            destination,
            gateway,
        }
    }

    /// Returns the route destination.
    #[must_use]
    pub const fn destination(self) -> RouteDestination<'a> {
        self.destination
    }

    /// Returns the gateway.
    #[must_use]
    pub const fn gateway(self) -> RouteGateway<'a> {
        self.gateway
    }
}

/// Network sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkSortField {
    /// Sort by ID.
    Id,
    /// Sort by name.
    Name,
    /// Sort by creation timestamp.
    Created,
}
