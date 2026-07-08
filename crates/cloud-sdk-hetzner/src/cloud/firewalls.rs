//! Firewall endpoint domains.

use crate::EndpointGroup;

/// Firewall endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] =
    &[EndpointGroup::Firewalls, EndpointGroup::FirewallActions];
