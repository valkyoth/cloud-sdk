//! Load balancer endpoint domains.

use crate::EndpointGroup;

/// Load balancer endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::LoadBalancers,
    EndpointGroup::LoadBalancerActions,
    EndpointGroup::LoadBalancerTypes,
];
