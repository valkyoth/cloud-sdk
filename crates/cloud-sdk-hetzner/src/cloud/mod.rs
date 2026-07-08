//! Hetzner Cloud resource modules.

pub mod firewalls;
pub mod images;
pub mod load_balancers;
pub mod networks;
pub mod pricing;
pub mod servers;
pub mod volumes;

use crate::EndpointGroup;

/// Endpoint groups owned by the general Cloud API module.
pub const CLOUD_ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::Actions,
    EndpointGroup::Servers,
    EndpointGroup::ServerActions,
    EndpointGroup::ServerTypes,
    EndpointGroup::Images,
    EndpointGroup::ImageActions,
    EndpointGroup::Isos,
    EndpointGroup::PlacementGroups,
    EndpointGroup::PrimaryIps,
    EndpointGroup::PrimaryIpActions,
    EndpointGroup::Volumes,
    EndpointGroup::VolumeActions,
    EndpointGroup::FloatingIps,
    EndpointGroup::FloatingIpActions,
    EndpointGroup::Firewalls,
    EndpointGroup::FirewallActions,
    EndpointGroup::LoadBalancers,
    EndpointGroup::LoadBalancerActions,
    EndpointGroup::LoadBalancerTypes,
    EndpointGroup::Networks,
    EndpointGroup::NetworkActions,
    EndpointGroup::Locations,
    EndpointGroup::Pricing,
];

#[cfg(test)]
mod tests {
    use super::CLOUD_ENDPOINT_GROUPS;
    use crate::EndpointGroup;

    #[test]
    fn includes_compute_and_billing_groups() {
        assert!(CLOUD_ENDPOINT_GROUPS.contains(&EndpointGroup::Servers));
        assert!(CLOUD_ENDPOINT_GROUPS.contains(&EndpointGroup::Pricing));
    }
}
