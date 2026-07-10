//! Network and IP endpoint domains.

use crate::EndpointGroup;

pub mod actions;
pub mod floating_ips;
pub mod primary_ips;
pub mod resources;

pub use resources::{
    NetworkCreateRequest, NetworkEndpoint, NetworkId, NetworkLabels, NetworkListRequest,
    NetworkName, NetworkRequestError, NetworkRoute, NetworkSortField, NetworkSubnet,
    NetworkSubnetType, NetworkUpdateRequest, NetworkVswitchId, NetworkZone,
};

/// Network and IP endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::PrimaryIps,
    EndpointGroup::PrimaryIpActions,
    EndpointGroup::FloatingIps,
    EndpointGroup::FloatingIpActions,
    EndpointGroup::Networks,
    EndpointGroup::NetworkActions,
];
