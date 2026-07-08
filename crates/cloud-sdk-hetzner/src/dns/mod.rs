//! DNS resource modules.

pub mod rrsets;
pub mod zones;

use crate::EndpointGroup;

/// Endpoint groups owned by the DNS API module.
pub const DNS_ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::Zones,
    EndpointGroup::ZoneActions,
    EndpointGroup::ZoneRrsets,
    EndpointGroup::ZoneRrsetActions,
];

#[cfg(test)]
mod tests {
    use super::DNS_ENDPOINT_GROUPS;
    use crate::EndpointGroup;

    #[test]
    fn includes_zone_rrset_groups() {
        assert!(DNS_ENDPOINT_GROUPS.contains(&EndpointGroup::Zones));
        assert!(DNS_ENDPOINT_GROUPS.contains(&EndpointGroup::ZoneRrsets));
    }
}
