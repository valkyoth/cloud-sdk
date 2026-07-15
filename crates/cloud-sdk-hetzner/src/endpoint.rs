//! Endpoint ownership domains for the SDK.

use core::fmt;

use cloud_sdk::transport::{BoundTransport, EndpointIdentityError, EndpointScheme};

use crate::request::ApiBaseUrl;

const HTTPS_PORT: u16 = 443;
const V1_BASE_PATH: &str = "/v1";
const CLOUD_API_HOST: &str = "api.hetzner.cloud";
const HETZNER_API_HOST: &str = "api.hetzner.com";

/// Failure while verifying a credential-bound transport destination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OfficialEndpointError {
    /// The transport could not provide a valid normalized endpoint identity.
    InvalidIdentity(EndpointIdentityError),
    /// The transport destination does not exactly match the selected official endpoint.
    DestinationMismatch,
}

impl fmt::Display for OfficialEndpointError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidIdentity(_) => "transport endpoint identity is invalid",
            Self::DestinationMismatch => {
                "transport destination is not an official Hetzner endpoint"
            }
        })
    }
}

impl core::error::Error for OfficialEndpointError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::InvalidIdentity(error) => Some(error),
            Self::DestinationMismatch => None,
        }
    }
}

/// Verifies that a credential-bound transport exactly matches an official API endpoint.
///
/// The check includes the HTTPS scheme, authority, effective port, and `/v1`
/// base path. Call this before sending a credential through a custom endpoint.
pub fn verify_official_endpoint(
    transport: &(impl BoundTransport + ?Sized),
    expected: ApiBaseUrl,
) -> Result<(), OfficialEndpointError> {
    let identity = transport
        .endpoint_identity()
        .map_err(OfficialEndpointError::InvalidIdentity)?;
    let expected_host = match expected {
        ApiBaseUrl::CloudV1 => CLOUD_API_HOST,
        ApiBaseUrl::HetznerV1 => HETZNER_API_HOST,
    };
    if identity.scheme() != EndpointScheme::Https
        || identity.host() != expected_host
        || identity.effective_port() != HTTPS_PORT
        || identity.base_path() != V1_BASE_PATH
    {
        return Err(OfficialEndpointError::DestinationMismatch);
    }
    Ok(())
}

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

    /// Returns the base URL family for this endpoint group.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        match self.surface() {
            ApiSurface::Storage => ApiBaseUrl::HetznerV1,
            ApiSurface::Cloud | ApiSurface::Dns | ApiSurface::Security => ApiBaseUrl::CloudV1,
        }
    }
}

#[cfg(test)]
mod tests {
    use cloud_sdk::transport::{
        BoundTransport, EndpointIdentity, EndpointIdentityError, EndpointScheme,
    };

    use super::{ApiSurface, EndpointGroup, OfficialEndpointError, verify_official_endpoint};
    use crate::request::ApiBaseUrl;

    #[test]
    fn maps_endpoint_groups_to_base_urls() {
        assert_eq!(EndpointGroup::Servers.api_base_url(), ApiBaseUrl::CloudV1);
        assert_eq!(EndpointGroup::Zones.api_base_url(), ApiBaseUrl::CloudV1);
        assert_eq!(
            EndpointGroup::Certificates.api_base_url(),
            ApiBaseUrl::CloudV1
        );
        assert_eq!(
            EndpointGroup::StorageBoxes.api_base_url(),
            ApiBaseUrl::HetznerV1
        );
        assert_eq!(EndpointGroup::Zones.surface(), ApiSurface::Dns);
    }

    #[test]
    fn verifies_both_official_v1_endpoints() {
        let cloud = StubTransport::valid("api.hetzner.cloud");
        let storage = StubTransport::valid("api.hetzner.com");
        assert_eq!(
            verify_official_endpoint(&cloud, ApiBaseUrl::CloudV1),
            Ok(())
        );
        assert_eq!(
            verify_official_endpoint(&storage, ApiBaseUrl::HetznerV1),
            Ok(())
        );
        assert_eq!(
            verify_official_endpoint(&cloud, ApiBaseUrl::HetznerV1),
            Err(OfficialEndpointError::DestinationMismatch)
        );
    }

    #[test]
    fn rejects_every_destination_change_and_invalid_identity() {
        let changed = [
            StubTransport::new(EndpointScheme::Http, "api.hetzner.cloud", 443, "/v1"),
            StubTransport::new(EndpointScheme::Https, "evil.example", 443, "/v1"),
            StubTransport::new(EndpointScheme::Https, "sub.api.hetzner.cloud", 443, "/v1"),
            StubTransport::new(EndpointScheme::Https, "api.hetzner.cloud", 8443, "/v1"),
            StubTransport::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v2"),
        ];
        for transport in &changed {
            assert_eq!(
                verify_official_endpoint(transport, ApiBaseUrl::CloudV1),
                Err(OfficialEndpointError::DestinationMismatch)
            );
        }

        let invalid = StubTransport {
            identity: Err(EndpointIdentityError::InvalidHost),
        };
        assert_eq!(
            verify_official_endpoint(&invalid, ApiBaseUrl::CloudV1),
            Err(OfficialEndpointError::InvalidIdentity(
                EndpointIdentityError::InvalidHost
            ))
        );
    }

    struct StubTransport<'a> {
        identity: Result<EndpointIdentity<'a>, EndpointIdentityError>,
    }

    impl<'a> StubTransport<'a> {
        fn valid(host: &'a str) -> Self {
            Self::new(EndpointScheme::Https, host, 443, "/v1")
        }

        fn new(scheme: EndpointScheme, host: &'a str, port: u16, base_path: &'a str) -> Self {
            Self {
                identity: EndpointIdentity::new(scheme, host, port, base_path),
            }
        }
    }

    impl BoundTransport for StubTransport<'_> {
        fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
            self.identity
        }
    }
}
