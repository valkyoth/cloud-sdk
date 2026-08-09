extern crate std;

use cloud_sdk::transport::{
    BoundTransport, CustomEndpointAcknowledgement, EndpointIdentity, EndpointIdentityError,
    EndpointScheme,
};

use super::{EndpointTrust, HetznerClient, HetznerClientConstructionError};
use crate::identity::{CLOUD_SERVICE_ID, DNS_SERVICE_ID, STORAGE_SERVICE_ID};

#[derive(Clone, Copy)]
struct StubTransport {
    endpoint: Result<EndpointIdentity<'static>, EndpointIdentityError>,
}

impl StubTransport {
    fn https(host: &'static str) -> Self {
        Self {
            endpoint: EndpointIdentity::new(EndpointScheme::Https, host, 443, "/v1"),
        }
    }

    fn http(host: &'static str) -> Self {
        Self {
            endpoint: EndpointIdentity::new(EndpointScheme::Http, host, 80, "/v1"),
        }
    }
}

impl BoundTransport for StubTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.endpoint
    }
}

#[test]
fn official_constructors_bind_the_exact_service_and_endpoint() {
    let cloud = HetznerClient::cloud(StubTransport::https("api.hetzner.cloud"));
    assert!(cloud.is_ok());
    let Ok(cloud) = cloud else {
        unreachable!("official cloud fixture was rejected")
    };
    assert_eq!(cloud.service_id(), CLOUD_SERVICE_ID);
    assert_eq!(cloud.endpoint_trust(), EndpointTrust::Official);

    let dns = HetznerClient::dns(StubTransport::https("api.hetzner.cloud"));
    assert_eq!(dns.map(|client| client.service_id()), Ok(DNS_SERVICE_ID));

    let storage = HetznerClient::storage(StubTransport::https("api.hetzner.com"));
    assert_eq!(
        storage.map(|client| client.service_id()),
        Ok(STORAGE_SERVICE_ID)
    );
}

#[test]
fn official_constructor_rejects_cross_service_and_invalid_destinations() {
    let wrong = HetznerClient::storage(StubTransport::https("api.hetzner.cloud"));
    assert!(matches!(
        wrong,
        Err(HetznerClientConstructionError::OfficialEndpoint(_))
    ));
    let invalid = HetznerClient::cloud(StubTransport {
        endpoint: Err(EndpointIdentityError::UnboundTransport),
    });
    assert!(matches!(
        invalid,
        Err(HetznerClientConstructionError::OfficialEndpoint(_))
    ));
}

#[test]
fn custom_constructor_requires_explicit_acknowledgement_and_https() {
    let acknowledgement = CustomEndpointAcknowledgement::trusted_operator_configuration();
    let custom = HetznerClient::cloud_with_custom_endpoint(
        StubTransport::https("trusted-proxy.example"),
        acknowledgement,
    );
    assert!(custom.is_ok());
    let Ok(custom) = custom else {
        unreachable!("acknowledged HTTPS fixture was rejected")
    };
    assert_eq!(custom.endpoint_trust(), EndpointTrust::Custom);
    assert_eq!(custom.service_id(), CLOUD_SERVICE_ID);

    let insecure = HetznerClient::cloud_with_custom_endpoint(
        StubTransport::http("trusted-proxy.example"),
        acknowledgement,
    );
    assert!(matches!(
        insecure,
        Err(HetznerClientConstructionError::InsecureCustomEndpoint)
    ));
}

#[test]
fn client_is_shared_when_the_bound_transport_is_shared() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<crate::client::CloudClient<StubTransport>>();
}

#[test]
fn debug_output_does_not_expose_endpoint_identity() {
    let client = HetznerClient::cloud(StubTransport::https("api.hetzner.cloud"));
    assert!(client.is_ok());
    let Ok(client) = client else {
        unreachable!("official cloud fixture was rejected")
    };
    let diagnostic = std::format!("{client:?}");
    assert!(!diagnostic.contains("api.hetzner.cloud"));
    assert!(diagnostic.contains("[bound]"));
}
