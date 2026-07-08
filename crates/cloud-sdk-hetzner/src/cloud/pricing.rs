//! Location, type, and pricing endpoint domains.

use crate::EndpointGroup;

/// Catalog and billing endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::ServerTypes,
    EndpointGroup::LoadBalancerTypes,
    EndpointGroup::Locations,
    EndpointGroup::Pricing,
];
