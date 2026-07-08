//! Server endpoint domains.

use crate::EndpointGroup;

/// Server endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::Servers,
    EndpointGroup::ServerActions,
    EndpointGroup::PlacementGroups,
];
