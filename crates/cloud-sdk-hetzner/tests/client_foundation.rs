//! End-to-end evidence for the service-typed v0.69 client foundation.

use cloud_sdk::client::{ClientWorkspace, ClientWorkspacePool};
use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme};
use cloud_sdk_hetzner::association::AssociatedOperation;
use cloud_sdk_hetzner::association::operations::ListLocations;
use cloud_sdk_hetzner::client::HetznerClient;
use cloud_sdk_hetzner::cloud::catalog::CatalogListEndpoint;
use cloud_sdk_hetzner::serde::HetznerSuccess;
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, MockExchange, MockTransport, ResponseFixture,
};

const LOCATIONS: &[u8] = br#"{"locations":[{"id":42,"name":"fsn1","description":"Falkenstein DC Park 1","country":"DE","city":"Falkenstein","latitude":50.47612,"longitude":12.370071,"network_zone":"eu-central"}],"meta":{"pagination":{"page":1,"per_page":25,"previous_page":null,"next_page":null,"last_page":1,"total_entries":1}}}"#;

#[test]
fn official_cloud_client_executes_and_decodes_one_read_only_operation() {
    let operation =
        AssociatedOperation::<ListLocations, _>::endpoint(CatalogListEndpoint::Locations);
    assert!(operation.is_ok());
    let Ok(operation) = operation else {
        unreachable!("location operation association failed")
    };

    let mut expected_target = [0_u8; 128];
    let mut expected_body = [0_u8; 16];
    let prepared = operation.prepare(PreparationStorage::new(
        &mut expected_target,
        &mut expected_body,
    ));
    assert!(prepared.is_ok());
    let Ok(prepared) = prepared else {
        unreachable!("location operation preparation failed")
    };
    let request = prepared.transport_request();
    let expected = ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers());

    let body = FixtureBody::new(LOCATIONS);
    assert!(body.is_ok());
    let Ok(body) = body else {
        unreachable!("location response fixture failed")
    };
    let exchanges = [MockExchange::new(
        expected,
        ResponseFixture::success(body).with_content_type("application/json"),
    )];
    let endpoint = EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1");
    assert!(endpoint.is_ok());
    let Ok(endpoint) = endpoint else {
        unreachable!("official cloud endpoint fixture failed")
    };
    let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let client = HetznerClient::cloud(transport);
    assert!(client.is_ok());
    let Ok(client) = client else {
        unreachable!("official cloud client construction failed")
    };

    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 16];
    let mut response_body = [0_u8; 512];
    let mut response_headers = [0_u8; 8192];
    let workspace = ClientWorkspace::new(
        &mut target,
        &mut request_body,
        &mut response_body,
        &mut response_headers,
    );
    let pool = ClientWorkspacePool::<1>::new();
    assert!(pool.is_ok());
    let Ok(pool) = pool else {
        unreachable!("client workspace pool construction failed")
    };
    let lease = pool.try_acquire(workspace);
    assert!(lease.is_ok());
    let Ok(lease) = lease else {
        unreachable!("client workspace lease failed")
    };

    let result = client.execute_blocking(&operation, lease);
    assert!(result.is_ok());
    let Ok(result) = result else {
        unreachable!("client read-only execution failed")
    };
    assert!(matches!(result.success(), HetznerSuccess::Locations(_)));
    assert_eq!(pool.active_leases(), 0);
    assert!(client.transport().is_complete());
}
