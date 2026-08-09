//! End-to-end evidence for the service-typed Cloud client.

use core::future::Future;
use core::task::{Context, Poll, Waker};

use cloud_sdk::client::{ClientWorkspace, ClientWorkspacePool};
use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme};
use cloud_sdk_hetzner::association::operations::ListLocations;
use cloud_sdk_hetzner::association::{AssociatedOperation, PaginationPolicy, PermitClass};
use cloud_sdk_hetzner::client::{CLOUD_CLIENT_METHODS, HetznerClient};
use cloud_sdk_hetzner::cloud::catalog::CatalogListEndpoint;
use cloud_sdk_hetzner::serde::HetznerSuccess;
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, LocalMockTransport, MockExchange, MockTransport,
    RateLimitFixture, ResponseFixture,
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

    let result = client.list_locations_blocking(&operation, lease);
    assert!(result.is_ok());
    let Ok(result) = result else {
        unreachable!("client read-only execution failed")
    };
    assert!(matches!(result.success(), HetznerSuccess::Locations(_)));
    assert_eq!(pool.active_leases(), 0);
    assert!(client.transport().is_complete());
}

#[test]
fn cloud_client_registry_is_complete_sorted_and_policy_exact() {
    assert_eq!(CLOUD_CLIENT_METHODS.len(), 139);
    assert!(CLOUD_CLIENT_METHODS.windows(2).all(|pair| matches!(
        pair,
        [previous, next]
            if previous.operation().operation_id().as_str()
                < next.operation().operation_id().as_str()
    )));
    assert!(CLOUD_CLIENT_METHODS.iter().all(|method| {
        method.operation().service_id() == cloud_sdk_hetzner::identity::CLOUD_SERVICE_ID
    }));

    let permits = [
        (PermitClass::None, 55),
        (PermitClass::Mutation, 37),
        (PermitClass::Destructive, 37),
        (PermitClass::Cost, 10),
    ];
    for (permit, expected) in permits {
        assert_eq!(
            CLOUD_CLIENT_METHODS
                .iter()
                .filter(|method| method.permit() == permit)
                .count(),
            expected,
        );
    }
    assert_eq!(
        CLOUD_CLIENT_METHODS
            .iter()
            .filter(|method| method.pagination() == PaginationPolicy::Numbered)
            .count(),
        29,
    );
}

#[test]
fn named_cloud_reads_preserve_pagination_quota_and_async_parity() {
    let operation =
        AssociatedOperation::<ListLocations, _>::endpoint(CatalogListEndpoint::Locations);
    let Ok(operation) = operation else {
        unreachable!("location operation association failed")
    };
    let mut expected_target = [0_u8; 128];
    let mut expected_body = [0_u8; 16];
    let prepared = operation.prepare(PreparationStorage::new(
        &mut expected_target,
        &mut expected_body,
    ));
    let Ok(prepared) = prepared else {
        unreachable!("location operation preparation failed")
    };
    let request = prepared.transport_request();
    let expected = ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers());
    let endpoint = official_endpoint();

    let exchanges = [MockExchange::new(expected, location_fixture())];
    let client = HetznerClient::cloud(MockTransport::new(&exchanges).with_endpoint(endpoint));
    let Ok(client) = client else {
        unreachable!("official Cloud client construction failed")
    };
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 16];
    let mut response_body = [0_u8; 512];
    let mut response_headers = [0_u8; 8192];
    let pool = ClientWorkspacePool::<1>::new();
    let Ok(pool) = pool else {
        unreachable!("workspace pool construction failed")
    };
    let lease = pool.try_acquire(ClientWorkspace::new(
        &mut target,
        &mut request_body,
        &mut response_body,
        &mut response_headers,
    ));
    let Ok(lease) = lease else {
        unreachable!("workspace acquisition failed")
    };
    let future = client.list_locations_async(&operation, lease);
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    let Poll::Ready(Ok(result)) = Future::poll(future.as_mut(), &mut context) else {
        unreachable!("named Send-async Cloud read did not complete")
    };
    assert_location_metadata(&result);
    assert!(client.transport().is_complete());

    let exchanges = [MockExchange::new(expected, location_fixture())];
    let client = HetznerClient::cloud(LocalMockTransport::new(&exchanges).with_endpoint(endpoint));
    let Ok(client) = client else {
        unreachable!("official local Cloud client construction failed")
    };
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 16];
    let mut response_body = [0_u8; 512];
    let mut response_headers = [0_u8; 8192];
    let pool = ClientWorkspacePool::<1>::new();
    let Ok(pool) = pool else {
        unreachable!("local workspace pool construction failed")
    };
    let lease = pool.try_acquire(ClientWorkspace::new(
        &mut target,
        &mut request_body,
        &mut response_body,
        &mut response_headers,
    ));
    let Ok(lease) = lease else {
        unreachable!("local workspace acquisition failed")
    };
    let future = client.list_locations_local_async(&operation, lease);
    let mut future = core::pin::pin!(future);
    let Poll::Ready(Ok(result)) = Future::poll(future.as_mut(), &mut context) else {
        unreachable!("named local-async Cloud read did not complete")
    };
    assert_location_metadata(&result);
    assert!(client.transport().is_complete());
}

fn location_fixture() -> ResponseFixture<'static> {
    let body = FixtureBody::new(LOCATIONS);
    let Ok(body) = body else {
        unreachable!("location response fixture failed")
    };
    let rate_limit = RateLimitFixture::new(3600, 3599, 42);
    let Ok(rate_limit) = rate_limit else {
        unreachable!("rate-limit fixture failed")
    };
    ResponseFixture::success(body)
        .with_content_type("application/json")
        .with_rate_limit(rate_limit)
}

fn official_endpoint() -> EndpointIdentity<'static> {
    let endpoint = EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1");
    let Ok(endpoint) = endpoint else {
        unreachable!("official Cloud endpoint fixture failed")
    };
    endpoint
}

fn assert_location_metadata(result: &cloud_sdk_hetzner::serde::CheckedHetznerResponse) {
    let HetznerSuccess::Locations(page) = result.success() else {
        unreachable!("location method decoded another response family")
    };
    assert_eq!(page.pagination.total_entries(), Some(1));
    assert_eq!(
        result.rate_limit().map(|value| value.remaining()),
        Some(3599)
    );
}
