//! End-to-end evidence for the service-typed Console Storage client.

use core::future::Future;
use core::task::{Context, Poll, Waker};

use cloud_sdk::client::{ClientWorkspace, ClientWorkspacePool};
use cloud_sdk::operation::{
    AttemptBudget, PermitClock, PermitContext, PermitTimestamp, PermitValidity, PlanChange,
    PlanFingerprintScope, PreparationStorage, PreparationStorageGuard, PrepareOperation,
    ReplayPolicy,
};
use cloud_sdk::retry::DigestAlgorithm;
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme, StatusCode};
use cloud_sdk_hetzner::association::operations::{ListStorageBoxes, ResetStorageBoxPassword};
use cloud_sdk_hetzner::association::{
    AssociatedDestructivePermit, AssociatedOperation, AssociatedPlanConfirmation, PaginationPolicy,
    PermitClass, Sha256PlanHasher, build_associated_canonical_plan, build_associated_plan_digest,
};
use cloud_sdk_hetzner::client::{HetznerClient, STORAGE_CLIENT_METHODS};
use cloud_sdk_hetzner::pagination::{Page, PerPage};
use cloud_sdk_hetzner::serde::{
    HetznerSuccess, StorageBoxStatus, decode_associated_checked_response,
};
use cloud_sdk_hetzner::storage::storage_boxes::{
    StorageBoxActionEndpoint, StorageBoxEndpoint, StorageBoxId, StorageBoxListRequest,
    StorageBoxPassword, StorageBoxResetPasswordRequest,
};
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, LocalMockTransport, MockExchange, MockTransport,
    RateLimitFixture, ResponseFixture,
};

const STORAGE_BOX: &str = r#"{"id":42,"name":"backup","storage_box_type":{"id":7,"name":"bx11","description":"BX11","snapshot_limit":10,"automatic_snapshot_limit":10,"subaccounts_limit":200,"size":1073741824,"prices":[{"location":"fsn1","price_hourly":{"net":"1.0000","gross":"1.1900"},"price_monthly":{"net":"5.0000","gross":"5.9500"},"setup_fee":{"net":"0.0000","gross":"0.0000"}}],"deprecation":null},"location":{"id":1,"name":"fsn1","description":"Falkenstein DC Park 1","country":"DE","city":"Falkenstein","latitude":50.47612,"longitude":12.370071,"network_zone":"eu-central"},"access_settings":{"reachable_externally":false,"samba_enabled":true,"ssh_enabled":true,"webdav_enabled":false,"zfs_enabled":true},"snapshot_plan":{"max_snapshots":10,"minute":30,"hour":3,"day_of_week":7,"day_of_month":null},"protection":{"delete":true},"labels":{"environment":"test"},"status":"active","username":"u12345","server":"u12345.your-storagebox.de","system":"FSN1-BX355","stats":{"size":3,"size_data":2,"size_snapshots":1},"created":"2026-01-01T00:00:00Z"}"#;
const ACTION: &[u8] = br#"{"action":{"id":42,"command":"reset_password","status":"running","progress":0,"started":"2026-01-01T00:00:00Z","finished":null,"resources":[{"id":42,"type":"storage_box"}],"error":null}}"#;
const PASSWORD: &str = "correct-horse-battery-staple";

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

#[test]
fn storage_client_registry_is_complete_sorted_and_policy_exact() {
    assert_eq!(STORAGE_CLIENT_METHODS.len(), 31);
    assert!(STORAGE_CLIENT_METHODS.windows(2).all(|pair| matches!(
        pair,
        [previous, next]
            if previous.operation().operation_id().as_str()
                < next.operation().operation_id().as_str()
    )));
    assert!(STORAGE_CLIENT_METHODS.iter().all(|method| {
        method.operation().service_id() == cloud_sdk_hetzner::identity::STORAGE_SERVICE_ID
    }));
    for (permit, expected) in [
        (PermitClass::None, 12),
        (PermitClass::Mutation, 9),
        (PermitClass::Destructive, 8),
        (PermitClass::Cost, 2),
    ] {
        assert_eq!(
            STORAGE_CLIENT_METHODS
                .iter()
                .filter(|method| method.permit() == permit)
                .count(),
            expected,
        );
    }
    assert_eq!(
        STORAGE_CLIENT_METHODS
            .iter()
            .filter(|method| method.pagination() == PaginationPolicy::Numbered)
            .count(),
        4,
    );
}

#[test]
fn named_storage_reads_preserve_large_pages_quota_and_executor_parity() {
    const ITEMS: usize = 32;
    let operation = list_storage_boxes_operation(ITEMS);
    let mut expected_target = [0_u8; 128];
    let mut expected_body = [0_u8; 1];
    let prepared = operation
        .prepare(PreparationStorage::new(
            &mut expected_target,
            &mut expected_body,
        ))
        .unwrap_or_else(|_| unreachable!("Storage list preparation failed"));
    let request = prepared.transport_request();
    let expected = ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers());
    let endpoint = official_endpoint();
    let payload = storage_boxes_payload(ITEMS);
    assert!(payload.len() > 32_768);

    let exchanges = [MockExchange::new(expected, storage_boxes_fixture(&payload))];
    let client = HetznerClient::storage(MockTransport::new(&exchanges).with_endpoint(endpoint))
        .unwrap_or_else(|_| unreachable!("blocking Storage client construction failed"));
    let mut workspace = ReadWorkspace::new();
    let pool = ClientWorkspacePool::<1>::new()
        .unwrap_or_else(|_| unreachable!("blocking Storage workspace pool failed"));
    let result = client.list_storage_boxes_blocking(&operation, workspace.lease(&pool));
    let Ok(result) = result else {
        unreachable!("named blocking Storage read failed")
    };
    assert_storage_metadata(&result, ITEMS);
    assert!(client.transport().is_complete());

    let exchanges = [MockExchange::new(expected, storage_boxes_fixture(&payload))];
    let client = HetznerClient::storage(MockTransport::new(&exchanges).with_endpoint(endpoint))
        .unwrap_or_else(|_| unreachable!("Send-async Storage client construction failed"));
    let mut workspace = ReadWorkspace::new();
    let pool = ClientWorkspacePool::<1>::new()
        .unwrap_or_else(|_| unreachable!("async Storage workspace pool failed"));
    let future = client.list_storage_boxes_async(&operation, workspace.lease(&pool));
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    let Poll::Ready(Ok(result)) = Future::poll(future.as_mut(), &mut context) else {
        unreachable!("named Send-async Storage read did not complete")
    };
    assert_storage_metadata(&result, ITEMS);
    assert!(client.transport().is_complete());

    let exchanges = [MockExchange::new(expected, storage_boxes_fixture(&payload))];
    let client =
        HetznerClient::storage(LocalMockTransport::new(&exchanges).with_endpoint(endpoint))
            .unwrap_or_else(|_| unreachable!("local Storage client construction failed"));
    let mut workspace = ReadWorkspace::new();
    let pool = ClientWorkspacePool::<1>::new()
        .unwrap_or_else(|_| unreachable!("local Storage workspace pool failed"));
    let future = client.list_storage_boxes_local_async(&operation, workspace.lease(&pool));
    let mut future = core::pin::pin!(future);
    let Poll::Ready(Ok(result)) = Future::poll(future.as_mut(), &mut context) else {
        unreachable!("named local Storage read did not complete")
    };
    assert_storage_metadata(&result, ITEMS);
    assert!(client.transport().is_complete());
}

#[test]
fn password_reset_is_redacted_digest_bound_permitted_and_cleanup_owned() {
    let endpoint = official_endpoint();
    let operation = reset_password_operation();
    let no_exchanges = [];
    let preparation_client =
        HetznerClient::storage(MockTransport::new(&no_exchanges).with_endpoint(endpoint))
            .unwrap_or_else(|_| unreachable!("Storage preparation client failed"));
    let mut target = [0xa5_u8; 128];
    let mut request_body = [0x5a_u8; 256];
    {
        let mut storage = PreparationStorageGuard::new(&mut target, &mut request_body);
        let prepared = preparation_client
            .prepare_reset_storage_box_password(&operation, &mut storage)
            .unwrap_or_else(|_| unreachable!("password reset preparation failed"));
        let request = prepared.as_untyped().transport_request();
        assert_eq!(
            prepared.as_untyped().body_sensitivity(),
            cloud_sdk::operation::RequestBodySensitivity::Sensitive
        );
        assert!(
            request
                .body()
                .windows(PASSWORD.len())
                .any(|part| part == PASSWORD.as_bytes())
        );
        assert!(!format!("{operation:?} {prepared:?}").contains(PASSWORD));
        let expected = ExpectedRequest::new(request.method(), request.target())
            .with_body(request.body())
            .with_headers(request.headers());
        assert!(matches!(
            build_associated_canonical_plan(
                destructive_plan(prepared, endpoint),
                &mut [0xa5_u8; 4_096],
            ),
            Err(cloud_sdk::operation::PlanFingerprintBuildError::SensitiveBodyRequiresDigest)
        ));
        let mut fingerprint_scratch = [0xa5_u8; 4_096];
        let mut digest_storage = [0xa5_u8; 32];
        let fingerprint = build_associated_plan_digest(
            destructive_plan(prepared, endpoint),
            &mut fingerprint_scratch,
            &mut digest_storage,
            &Sha256PlanHasher,
        )
        .unwrap_or_else(|_| unreachable!("password reset fingerprint failed"));
        assert_eq!(fingerprint.algorithm(), DigestAlgorithm::Sha256);
        assert_eq!(fingerprint_scratch, [0_u8; 4_096]);
        let mut permit = AssociatedDestructivePermit::new(
            fingerprint.subject(),
            PermitTimestamp::from_seconds(100),
        )
        .unwrap_or_else(|_| unreachable!("password reset permit failed"));
        let attempt = permit
            .begin(PermitTimestamp::from_seconds(101))
            .unwrap_or_else(|_| unreachable!("password reset attempt failed"));
        let body = FixtureBody::new(ACTION)
            .unwrap_or_else(|_| unreachable!("Storage action fixture failed"));
        let fixture = ResponseFixture::success_at(StatusCode::CREATED, body)
            .unwrap_or_else(|_| unreachable!("Storage action status failed"))
            .with_content_type("application/json");
        let exchanges = [MockExchange::new(expected, fixture)];
        let client = HetznerClient::storage(MockTransport::new(&exchanges).with_endpoint(endpoint))
            .unwrap_or_else(|_| unreachable!("Storage execution client failed"));
        let mut response_body = [0_u8; 1_024];
        let mut response_headers = [0_u8; 8_192];
        let response = client
            .reset_storage_box_password_blocking(
                attempt,
                &FixedClock,
                &mut response_body,
                &mut response_headers,
            )
            .unwrap_or_else(|_| unreachable!("password reset execution failed"));
        let decoded = decode_associated_checked_response(response)
            .unwrap_or_else(|_| unreachable!("password reset decoding failed"));
        assert!(matches!(decoded.success(), HetznerSuccess::Action(_)));
        assert!(client.transport().is_complete());
    }
    assert_eq!(target, [0_u8; 128]);
    assert_eq!(request_body, [0_u8; 256]);
}

struct ReadWorkspace {
    target: [u8; 128],
    request_body: [u8; 1],
    response_body: [u8; 131_072],
    response_headers: [u8; 8_192],
}

impl ReadWorkspace {
    const fn new() -> Self {
        Self {
            target: [0; 128],
            request_body: [0; 1],
            response_body: [0; 131_072],
            response_headers: [0; 8_192],
        }
    }

    fn lease<'pool, 'buffer>(
        &'buffer mut self,
        pool: &'pool ClientWorkspacePool<1>,
    ) -> cloud_sdk::client::ClientWorkspaceLease<'pool, 'buffer, 1> {
        pool.try_acquire(ClientWorkspace::new(
            &mut self.target,
            &mut self.request_body,
            &mut self.response_body,
            &mut self.response_headers,
        ))
        .unwrap_or_else(|_| unreachable!("Storage workspace acquisition failed"))
    }
}

fn list_storage_boxes_operation(
    per_page: usize,
) -> AssociatedOperation<ListStorageBoxes, StorageBoxEndpoint, StorageBoxListRequest<'static>> {
    let page = Page::new(1).unwrap_or_else(|_| unreachable!("Storage page fixture failed"));
    let per_page = u16::try_from(per_page)
        .ok()
        .and_then(|value| PerPage::new(value).ok())
        .unwrap_or_else(|| unreachable!("Storage per-page fixture failed"));
    AssociatedOperation::<ListStorageBoxes, _, _>::query(
        StorageBoxEndpoint::List,
        StorageBoxListRequest::new()
            .with_page(page)
            .with_per_page(per_page),
    )
    .unwrap_or_else(|_| unreachable!("Storage list association failed"))
}

fn reset_password_operation() -> AssociatedOperation<
    ResetStorageBoxPassword,
    StorageBoxActionEndpoint,
    cloud_sdk_hetzner::prepared::NoQuery,
    StorageBoxResetPasswordRequest<'static>,
> {
    let id = StorageBoxId::new(42).unwrap_or_else(|| unreachable!("Storage Box ID failed"));
    let password = StorageBoxPassword::new(PASSWORD)
        .unwrap_or_else(|_| unreachable!("Storage password fixture failed"));
    AssociatedOperation::<ResetStorageBoxPassword, _, _, _>::json(
        StorageBoxActionEndpoint::ResetPassword(id),
        StorageBoxResetPasswordRequest::new(password),
    )
    .unwrap_or_else(|_| unreachable!("password reset association failed"))
}

fn storage_boxes_payload(items: usize) -> Vec<u8> {
    let resource = serde_json::from_str::<serde_json::Value>(STORAGE_BOX)
        .unwrap_or_else(|_| unreachable!("Storage Box fixture JSON failed"));
    let resources = core::iter::repeat_n(resource, items).collect::<Vec<_>>();
    serde_json::to_vec(&serde_json::json!({
        "storage_boxes": resources,
        "meta": {
            "pagination": {
                "page": 1,
                "per_page": items,
                "previous_page": null,
                "next_page": null,
                "last_page": 1,
                "total_entries": items,
            }
        }
    }))
    .unwrap_or_else(|_| unreachable!("Storage Box page fixture JSON failed"))
}

fn storage_boxes_fixture(payload: &[u8]) -> ResponseFixture<'_> {
    let body = FixtureBody::new(payload)
        .unwrap_or_else(|_| unreachable!("Storage response fixture body failed"));
    let rate_limit = RateLimitFixture::new(3600, 3599, 42)
        .unwrap_or_else(|_| unreachable!("Storage rate-limit fixture failed"));
    ResponseFixture::success(body)
        .with_content_type("application/json")
        .with_rate_limit(rate_limit)
}

fn official_endpoint() -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.com", 443, "/v1")
        .unwrap_or_else(|_| unreachable!("official Storage endpoint fixture failed"))
}

fn assert_storage_metadata(
    result: &cloud_sdk_hetzner::serde::CheckedHetznerResponse,
    items: usize,
) {
    let HetznerSuccess::StorageBoxes(page) = result.success() else {
        unreachable!("Storage list decoded another response family")
    };
    assert_eq!(page.storage_boxes().len(), items);
    assert!(page.storage_boxes().iter().all(|storage_box| {
        storage_box.status() == StorageBoxStatus::Active && storage_box.name() == "backup"
    }));
    assert_eq!(page.pagination().total_entries(), Some(items as u64));
    assert_eq!(
        result.rate_limit().map(|value| value.remaining()),
        Some(3599)
    );
}

fn destructive_plan<'request>(
    prepared: cloud_sdk_hetzner::association::Prepared<'request, ResetStorageBoxPassword>,
    endpoint: EndpointIdentity<'static>,
) -> AssociatedPlanConfirmation<'static, 'request, ResetStorageBoxPassword> {
    let context = PermitContext::new(b"v0.73 Storage password reset fixture")
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
