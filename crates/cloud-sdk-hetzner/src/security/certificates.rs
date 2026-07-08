//! Certificate endpoint domains.

use crate::EndpointGroup;

/// Certificate endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::Certificates,
    EndpointGroup::CertificateActions,
];
