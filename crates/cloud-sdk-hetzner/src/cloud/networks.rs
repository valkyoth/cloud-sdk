//! Network and IP endpoint domains.

use crate::EndpointGroup;

pub mod primary_ips;

/// Network and IP endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::PrimaryIps,
    EndpointGroup::PrimaryIpActions,
    EndpointGroup::FloatingIps,
    EndpointGroup::FloatingIpActions,
    EndpointGroup::Networks,
    EndpointGroup::NetworkActions,
];
