//! Complete blocking DNS client workflow against a deterministic transport.

use cloud_sdk::client::{ClientWorkspace, ClientWorkspacePool};
use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme};
use cloud_sdk_hetzner::association::AssociatedOperation;
use cloud_sdk_hetzner::association::operations::ListZones;
use cloud_sdk_hetzner::client::HetznerClient;
use cloud_sdk_hetzner::dns::zones::{ZoneEndpoint, ZoneListRequest};
use cloud_sdk_hetzner::pagination::{Page, PerPage};
use cloud_sdk_hetzner::serde::HetznerSuccess;
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, MockExchange, MockTransport, ResponseFixture,
};

const RESPONSE: &[u8] = br#"{"zones":[],"meta":{"pagination":{"page":1,"per_page":25,"previous_page":null,"next_page":null,"last_page":1,"total_entries":0}}}"#;

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let page = Page::new(1)?;
    let per_page = PerPage::new(25)?;
    let operation = AssociatedOperation::<ListZones, _, _>::query(
        ZoneEndpoint::List,
        ZoneListRequest::new().with_page(page, per_page),
    )?;

    let mut expected_target = [0_u8; 128];
    let mut expected_body = [0_u8; 1];
    let prepared = operation.prepare(PreparationStorage::new(
        &mut expected_target,
        &mut expected_body,
    ))?;
    let request = prepared.transport_request();
    let expected = ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers());
    let response = FixtureBody::new(RESPONSE)?;
    let exchanges = [MockExchange::new(
        expected,
        ResponseFixture::success(response).with_content_type("application/json"),
    )];
    let endpoint = EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1")?;
    let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let client = HetznerClient::dns(transport)?;

    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let mut response_body = [0_u8; 512];
    let mut response_headers = [0_u8; 8_192];
    let pool = ClientWorkspacePool::<1>::new()?;
    let lease = pool.try_acquire(ClientWorkspace::new(
        &mut target,
        &mut request_body,
        &mut response_body,
        &mut response_headers,
    ))?;

    let checked = client.list_zones_blocking(&operation, lease)?;
    assert!(matches!(
        checked.success(),
        HetznerSuccess::DnsResources { .. }
    ));
    assert!(client.transport().is_complete());
    Ok(())
}
