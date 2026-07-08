//! Image and ISO endpoint domains.

use crate::EndpointGroup;

/// Image and ISO endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::Images,
    EndpointGroup::ImageActions,
    EndpointGroup::Isos,
];
