//! Endpoint ownership domains for the SDK.

/// High-level API surface represented by an SDK module boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ApiSurface {
    /// Hetzner Cloud compute, network, volume, firewall, load balancer, and billing API.
    Cloud,
    /// Hetzner DNS zone, zone action, and RRSet API.
    Dns,
    /// Certificate and SSH key security resources.
    Security,
    /// Storage Box resources.
    Storage,
}

/// Endpoint group from the Hetzner API reference.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EndpointGroup {
    /// Cross-resource action records.
    Actions,
    /// Server resources.
    Servers,
    /// Server action resources.
    ServerActions,
    /// Server type catalog resources.
    ServerTypes,
    /// Image resources.
    Images,
    /// Image action resources.
    ImageActions,
    /// ISO catalog resources.
    Isos,
    /// Placement group resources.
    PlacementGroups,
    /// Primary IP resources.
    PrimaryIps,
    /// Primary IP action resources.
    PrimaryIpActions,
    /// Volume resources.
    Volumes,
    /// Volume action resources.
    VolumeActions,
    /// Storage Box resources.
    StorageBoxes,
    /// Storage Box action resources.
    StorageBoxActions,
    /// Storage Box subaccount resources.
    StorageBoxSubaccounts,
    /// Floating IP resources.
    FloatingIps,
    /// Floating IP action resources.
    FloatingIpActions,
    /// Firewall resources.
    Firewalls,
    /// Firewall action resources.
    FirewallActions,
    /// Load balancer resources.
    LoadBalancers,
    /// Load balancer action resources.
    LoadBalancerActions,
    /// Load balancer type catalog resources.
    LoadBalancerTypes,
    /// Network resources.
    Networks,
    /// Network action resources.
    NetworkActions,
    /// DNS zone resources.
    Zones,
    /// DNS zone action resources.
    ZoneActions,
    /// DNS zone RRSet resources.
    ZoneRrsets,
    /// DNS zone RRSet action resources.
    ZoneRrsetActions,
    /// Certificate resources.
    Certificates,
    /// Certificate action resources.
    CertificateActions,
    /// SSH key resources.
    SshKeys,
    /// Location catalog resources.
    Locations,
    /// Pricing catalog resources.
    Pricing,
}

impl EndpointGroup {
    /// Returns the API surface that owns this endpoint group.
    #[must_use]
    pub const fn surface(self) -> ApiSurface {
        match self {
            Self::Zones | Self::ZoneActions | Self::ZoneRrsets | Self::ZoneRrsetActions => {
                ApiSurface::Dns
            }
            Self::Certificates | Self::CertificateActions | Self::SshKeys => ApiSurface::Security,
            Self::StorageBoxes | Self::StorageBoxActions | Self::StorageBoxSubaccounts => {
                ApiSurface::Storage
            }
            _ => ApiSurface::Cloud,
        }
    }
}
