//! DNS RRSet endpoint domains.

use crate::EndpointGroup;

/// RRSet endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] =
    &[EndpointGroup::ZoneRrsets, EndpointGroup::ZoneRrsetActions];
