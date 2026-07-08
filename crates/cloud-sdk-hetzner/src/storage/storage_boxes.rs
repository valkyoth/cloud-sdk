//! Storage Box endpoint domains.

use crate::EndpointGroup;

/// Storage Box endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::StorageBoxes,
    EndpointGroup::StorageBoxActions,
    EndpointGroup::StorageBoxSubaccounts,
];
