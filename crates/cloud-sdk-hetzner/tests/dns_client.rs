//! End-to-end evidence for the service-typed DNS client.

use core::future::Future;
use core::task::{Context, Poll, Waker};

use cloud_sdk::client::{ClientWorkspace, ClientWorkspacePool};
use cloud_sdk::operation::{
    AttemptBudget, PermitClock, PermitContext, PermitTimestamp, PermitValidity, PlanChange,
    PlanFingerprintScope, PreparationStorage, PreparationStorageGuard, PrepareOperation,
    ReplayPolicy,
};
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme, StatusCode};
use cloud_sdk_hetzner::actions::ActionStatus;
use cloud_sdk_hetzner::association::operations::{ChangeZoneTtl, CreateZone, ListZones};
use cloud_sdk_hetzner::association::{
    AssociatedMutationPermit, AssociatedOperation, AssociatedPlanConfirmation, PaginationPolicy,
    PermitClass, build_associated_canonical_plan,
};
use cloud_sdk_hetzner::client::{DNS_CLIENT_METHODS, HetznerClient};
use cloud_sdk_hetzner::dns::zones::{
    PrimaryNameserver, PrimaryNameservers, TsigAlgorithm, TsigCredentials, TsigKey,
    ZoneActionEndpoint, ZoneCreateMode, ZoneCreateRequest, ZoneEndpoint, ZoneListRequest, ZoneName,
    ZoneReference, ZoneTtl, ZoneTtlRequest,
};
use cloud_sdk_hetzner::pagination::{Page, PerPage};
use cloud_sdk_hetzner::serde::{DnsResource, HetznerSuccess, decode_associated_checked_response};
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, LocalMockTransport, MockExchange, MockTransport,
    RateLimitFixture, ResponseFixture,
};

const ZONES: &[u8] = br#"{"zones":[{"id":42,"name":"example.com","created":"2026-01-01T00:00:00Z","mode":"primary","status":"ok","ttl":60,"record_count":1,"labels":{},"protection":{"delete":false},"authoritative_nameservers":{"assigned":["helium.ns.hetzner.de."],"delegated":["helium.ns.hetzner.de."],"delegation_last_check":"2026-01-01T00:00:00Z","delegation_status":"valid"},"primary_nameservers":[],"registrar":"hetzner"}],"meta":{"pagination":{"page":1,"per_page":1,"previous_page":null,"next_page":null,"last_page":1,"total_entries":1}}}"#;
const ACTION: &[u8] = br#"{"action":{"id":42,"command":"change_ttl","status":"running","progress":0,"started":"2026-01-01T00:00:00Z","finished":null,"resources":[{"id":42,"type":"zone"}],"error":null}}"#;

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

#[test]
fn dns_client_registry_is_complete_sorted_and_policy_exact() {
    assert_eq!(DNS_CLIENT_METHODS.len(), 24);
    assert!(DNS_CLIENT_METHODS.windows(2).all(|pair| matches!(
        pair,
        [previous, next]
            if previous.operation().operation_id().as_str()
                < next.operation().operation_id().as_str()
    )));
    assert!(DNS_CLIENT_METHODS.iter().all(|method| {
        method.operation().service_id() == cloud_sdk_hetzner::identity::DNS_SERVICE_ID
    }));

    for (permit, expected) in [
        (PermitClass::None, 8),
        (PermitClass::Mutation, 9),
        (PermitClass::Destructive, 7),
        (PermitClass::Cost, 0),
    ] {
        assert_eq!(
            DNS_CLIENT_METHODS
                .iter()
                .filter(|method| method.permit() == permit)
                .count(),
            expected,
        );
    }
    assert_eq!(
        DNS_CLIENT_METHODS
            .iter()
            .filter(|method| method.pagination() == PaginationPolicy::Numbered)
            .count(),
        4,
    );
}

#[test]
fn named_dns_reads_preserve_pagination_quota_and_executor_parity() {
    let operation = list_zones_operation();
    let mut expected_target = [0_u8; 128];
    let mut expected_body = [0_u8; 16];
    let prepared = operation
        .prepare(PreparationStorage::new(
            &mut expected_target,
            &mut expected_body,
        ))
        .unwrap_or_else(|_| unreachable!("list-zones preparation failed"));
    let request = prepared.transport_request();
    let expected = ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers());
    let endpoint = official_endpoint();

    let exchanges = [MockExchange::new(expected, zones_fixture())];
    let client = HetznerClient::dns(MockTransport::new(&exchanges).with_endpoint(endpoint));
    let Ok(client) = client else {
        unreachable!("official blocking DNS client construction failed")
    };
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 16];
    let mut response_body = [0_u8; 1_024];
    let mut response_headers = [0_u8; 8_192];
    let pool = ClientWorkspacePool::<1>::new()
        .unwrap_or_else(|_| unreachable!("blocking DNS workspace pool construction failed"));
    let lease = pool
        .try_acquire(ClientWorkspace::new(
            &mut target,
            &mut request_body,
            &mut response_body,
            &mut response_headers,
        ))
        .unwrap_or_else(|_| unreachable!("blocking DNS workspace acquisition failed"));
    let result = client.list_zones_blocking(&operation, lease);
    let Ok(result) = result else {
        unreachable!("named blocking DNS read failed")
    };
    assert_zone_metadata(&result);
    assert!(client.transport().is_complete());

    let exchanges = [MockExchange::new(expected, zones_fixture())];
    let client = HetznerClient::dns(MockTransport::new(&exchanges).with_endpoint(endpoint));
    let Ok(client) = client else {
        unreachable!("official Send-async DNS client construction failed")
    };
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 16];
    let mut response_body = [0_u8; 1_024];
    let mut response_headers = [0_u8; 8_192];
    let pool = ClientWorkspacePool::<1>::new()
        .unwrap_or_else(|_| unreachable!("async DNS workspace pool construction failed"));
    let lease = pool
        .try_acquire(ClientWorkspace::new(
            &mut target,
            &mut request_body,
            &mut response_body,
            &mut response_headers,
        ))
        .unwrap_or_else(|_| unreachable!("async DNS workspace acquisition failed"));
    let future = client.list_zones_async(&operation, lease);
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    let Poll::Ready(Ok(result)) = Future::poll(future.as_mut(), &mut context) else {
        unreachable!("named Send-async DNS read did not complete")
    };
    assert_zone_metadata(&result);
    assert!(client.transport().is_complete());

    let exchanges = [MockExchange::new(expected, zones_fixture())];
    let client = HetznerClient::dns(LocalMockTransport::new(&exchanges).with_endpoint(endpoint));
    let Ok(client) = client else {
        unreachable!("official local DNS client construction failed")
    };
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 16];
    let mut response_body = [0_u8; 1_024];
    let mut response_headers = [0_u8; 8_192];
    let pool = ClientWorkspacePool::<1>::new()
        .unwrap_or_else(|_| unreachable!("local DNS workspace pool construction failed"));
    let lease = pool
        .try_acquire(ClientWorkspace::new(
            &mut target,
            &mut request_body,
            &mut response_body,
            &mut response_headers,
        ))
        .unwrap_or_else(|_| unreachable!("local DNS workspace acquisition failed"));
    let future = client.list_zones_local_async(&operation, lease);
    let mut future = core::pin::pin!(future);
    let Poll::Ready(Ok(result)) = Future::poll(future.as_mut(), &mut context) else {
        unreachable!("named local DNS read did not complete")
    };
    assert_zone_metadata(&result);
    assert!(client.transport().is_complete());
}

#[test]
fn named_dns_preparation_contains_tsig_only_in_cleanup_owned_storage() {
    const KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
    let key = TsigKey::new(KEY).unwrap_or_else(|_| unreachable!("TSIG fixture key failed"));
    let credentials = TsigCredentials::new(key, TsigAlgorithm::HmacSha256);
    let nameserver = PrimaryNameserver::new("1.1.1.1")
        .unwrap_or_else(|_| unreachable!("primary nameserver fixture failed"))
        .with_tsig(credentials);
    let nameservers = [nameserver];
    let nameservers = PrimaryNameservers::new(&nameservers)
        .unwrap_or_else(|_| unreachable!("primary nameserver list fixture failed"));
    let name =
        ZoneName::new("example.com").unwrap_or_else(|_| unreachable!("zone name fixture failed"));
    let request = ZoneCreateRequest::new(name, ZoneCreateMode::Secondary(nameservers));
    let operation = AssociatedOperation::<CreateZone, _, _, _>::json(request.endpoint(), request)
        .unwrap_or_else(|_| unreachable!("create-zone association failed"));
    let no_exchanges = [];
    let client =
        HetznerClient::dns(MockTransport::new(&no_exchanges).with_endpoint(official_endpoint()))
            .unwrap_or_else(|_| unreachable!("DNS preparation client construction failed"));
    let mut target = [0xa5_u8; 128];
    let mut body = [0x5a_u8; 512];
    {
        let mut storage = PreparationStorageGuard::new(&mut target, &mut body);
        let prepared = client
            .prepare_create_zone(&operation, &mut storage)
            .unwrap_or_else(|_| unreachable!("named TSIG preparation failed"));
        let request = prepared.as_untyped().transport_request();
        let json = core::str::from_utf8(request.body())
            .unwrap_or_else(|_| unreachable!("prepared TSIG JSON was not UTF-8"));
        assert!(json.contains(KEY));
        assert!(json.contains("\"tsig_algorithm\":\"hmac-sha256\""));
        assert!(!format!("{operation:?} {prepared:?}").contains(KEY));
    }
    assert_eq!(target, [0_u8; 128]);
    assert_eq!(body, [0_u8; 512]);
}

#[test]
fn named_dns_mutation_requires_a_bound_permit_and_decodes_its_action() {
    let endpoint = official_endpoint();
    let zone_id = cloud_sdk_hetzner::cloud::shared::CloudResourceId::new(42)
        .unwrap_or_else(|| unreachable!("zone fixture ID failed"));
    let zone = ZoneReference::Id(zone_id);
    let ttl = ZoneTtl::new(300).unwrap_or_else(|_| unreachable!("zone TTL fixture failed"));
    let request = ZoneTtlRequest::new(zone, ttl);
    let operation = AssociatedOperation::<ChangeZoneTtl, _, _, _>::json(
        ZoneActionEndpoint::ChangeTtl(zone),
        request,
    )
    .unwrap_or_else(|_| unreachable!("change-zone-TTL association failed"));
    let no_exchanges = [];
    let preparation_client =
        HetznerClient::dns(MockTransport::new(&no_exchanges).with_endpoint(endpoint))
            .unwrap_or_else(|_| unreachable!("DNS preparation client construction failed"));
    let mut target = [0_u8; 256];
    let mut request_body = [0_u8; 256];
    let mut storage = PreparationStorageGuard::new(&mut target, &mut request_body);
    let prepared = preparation_client
        .prepare_change_zone_ttl(&operation, &mut storage)
        .unwrap_or_else(|_| unreachable!("change-zone-TTL preparation failed"));
    let request = prepared.as_untyped().transport_request();
    let expected = ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers());
    let mut fingerprint_storage = [0_u8; 4_096];
    let fingerprint = build_associated_canonical_plan(
        mutation_plan(prepared, endpoint),
        &mut fingerprint_storage,
    )
    .unwrap_or_else(|_| unreachable!("change-zone-TTL fingerprint failed"));
    let mut permit =
        AssociatedMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100))
            .unwrap_or_else(|_| unreachable!("change-zone-TTL permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("change-zone-TTL attempt failed"));
    let action_body =
        FixtureBody::new(ACTION).unwrap_or_else(|_| unreachable!("DNS action fixture body failed"));
    let action_fixture = ResponseFixture::success_at(StatusCode::CREATED, action_body)
        .unwrap_or_else(|_| unreachable!("DNS action status fixture failed"))
        .with_content_type("application/json");
    let exchanges = [MockExchange::new(expected, action_fixture)];
    let client = HetznerClient::dns(MockTransport::new(&exchanges).with_endpoint(endpoint))
        .unwrap_or_else(|_| unreachable!("DNS mutation client construction failed"));
    let mut response_body = [0_u8; 512];
    let mut response_headers = [0_u8; 8_192];
    let response = client
        .change_zone_ttl_blocking(
            attempt,
            &FixedClock,
            &mut response_body,
            &mut response_headers,
        )
        .unwrap_or_else(|_| unreachable!("named DNS mutation execution failed"));
    let decoded = decode_associated_checked_response(response)
        .unwrap_or_else(|_| unreachable!("named DNS action decoding failed"));
    let HetznerSuccess::Action(action) = decoded.success() else {
        unreachable!("DNS mutation decoded another response family")
    };
    assert_eq!(action.id().get(), 42);
    assert_eq!(action.status(), ActionStatus::Running);
    assert!(client.transport().is_complete());
}

fn list_zones_operation()
-> AssociatedOperation<ListZones, ZoneEndpoint<'static>, ZoneListRequest<'static>> {
    let page = Page::new(1).unwrap_or_else(|_| unreachable!("DNS page fixture failed"));
    let per_page = PerPage::new(1).unwrap_or_else(|_| unreachable!("DNS per-page fixture failed"));
    AssociatedOperation::<ListZones, _, _>::query(
        ZoneEndpoint::List,
        ZoneListRequest::new().with_page(page, per_page),
    )
    .unwrap_or_else(|_| unreachable!("list-zones association failed"))
}

fn zones_fixture() -> ResponseFixture<'static> {
    let body = FixtureBody::new(ZONES)
        .unwrap_or_else(|_| unreachable!("DNS response fixture body failed"));
    let rate_limit = RateLimitFixture::new(3600, 3599, 42)
        .unwrap_or_else(|_| unreachable!("DNS rate-limit fixture failed"));
    ResponseFixture::success(body)
        .with_content_type("application/json")
        .with_rate_limit(rate_limit)
}

fn official_endpoint() -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1")
        .unwrap_or_else(|_| unreachable!("official DNS endpoint fixture failed"))
}

fn assert_zone_metadata(result: &cloud_sdk_hetzner::serde::CheckedHetznerResponse) {
    let HetznerSuccess::DnsResources {
        resources,
        pagination: Some(pagination),
    } = result.success()
    else {
        unreachable!("DNS list decoded another response family")
    };
    assert!(matches!(resources.as_slice(), [DnsResource::Zone(_)]));
    assert_eq!(pagination.total_entries(), Some(1));
    assert_eq!(
        result.rate_limit().map(|value| value.remaining()),
        Some(3599)
    );
}

fn mutation_plan<'request>(
    prepared: cloud_sdk_hetzner::association::Prepared<'request, ChangeZoneTtl>,
    endpoint: EndpointIdentity<'static>,
) -> AssociatedPlanConfirmation<'static, 'request, ChangeZoneTtl> {
    let context = PermitContext::new(b"v0.71 DNS mutation fixture")
        .unwrap_or_else(|_| unreachable!("permit context failed"));
    let validity = PermitValidity::new(
        PermitTimestamp::from_seconds(100),
        PermitTimestamp::from_seconds(200),
    )
    .unwrap_or_else(|_| unreachable!("permit validity failed"));
    let attempts = AttemptBudget::new(1).unwrap_or_else(|_| unreachable!("attempt budget failed"));
    AssociatedPlanConfirmation::new(
        prepared,
        endpoint,
        PlanFingerprintScope::Value(b"account"),
        PlanFingerprintScope::Value(b"project"),
        context,
        validity,
        ReplayPolicy::SingleAttempt,
        attempts,
        PlanChange::ChangesState,
        None,
        None,
    )
}
