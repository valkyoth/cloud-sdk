use core::fmt::{self, Write};

use cloud_sdk::transport::{
    BoundTransport, CustomEndpointAcknowledgement, EndpointIdentity, EndpointIdentityError,
    EndpointScheme,
};

use super::{EndpointTrust, HetznerClient, HetznerClientConstructionError};
use crate::identity::{CLOUD_SERVICE_ID, DNS_SERVICE_ID, STORAGE_SERVICE_ID};

struct DebugBuffer {
    bytes: [u8; 256],
    len: usize,
}

impl DebugBuffer {
    fn new() -> Self {
        Self {
            bytes: [0; 256],
            len: 0,
        }
    }

    fn contains(&self, needle: &[u8]) -> bool {
        let Some(written) = self.bytes.get(..self.len) else {
            return false;
        };
        if needle.is_empty() {
            return true;
        }
        written.windows(needle.len()).any(|window| window == needle)
    }
}

impl fmt::Write for DebugBuffer {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let Some(end) = self.len.checked_add(value.len()) else {
            return Err(fmt::Error);
        };
        let Some(output) = self.bytes.get_mut(self.len..end) else {
            return Err(fmt::Error);
        };
        output.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}

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
    let mut diagnostic = DebugBuffer::new();
    assert!(write!(&mut diagnostic, "{client:?}").is_ok());
    assert!(!diagnostic.contains(b"api.hetzner.cloud"));
    assert!(diagnostic.contains(b"[bound]"));
}
