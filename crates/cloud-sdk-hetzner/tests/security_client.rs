//! End-to-end evidence for the service-typed Security client.

use core::future::Future;
use core::task::{Context, Poll, Waker};

use cloud_sdk::client::{ClientWorkspace, ClientWorkspacePool};
use cloud_sdk::operation::{
    AttemptBudget, PermitClock, PermitContext, PermitTimestamp, PermitValidity, PlanChange,
    PlanFingerprintScope, PreparationStorage, PreparationStorageGuard, PrepareOperation,
    ReplayPolicy,
};
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme, StatusCode};
use cloud_sdk_hetzner::association::operations::{CreateCertificate, ListCertificates};
use cloud_sdk_hetzner::association::{
    AssociatedMutationPermit, AssociatedOperation, AssociatedPlanConfirmation, PaginationPolicy,
    PermitClass, build_associated_canonical_plan,
};
use cloud_sdk_hetzner::client::{HetznerClient, SECURITY_CLIENT_METHODS};
use cloud_sdk_hetzner::pagination::{Page, PerPage};
use cloud_sdk_hetzner::security::certificates::{
    CertificateCreateMode, CertificateCreateRequest, CertificateEndpoint, CertificateListRequest,
    CertificateName, certificate_pem, private_key_pem,
};
use cloud_sdk_hetzner::serde::{
    HetznerSuccess, SecurityResource, SecurityResourceKind, decode_associated_checked_response,
};
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, LocalMockTransport, MockExchange, MockTransport,
    RateLimitFixture, ResponseFixture,
};

const CERTIFICATE: &str =
    "-----BEGIN CERTIFICATE-----\nY2xvdWQtc2RrLXRlc3Q=\n-----END CERTIFICATE-----";
const PRIVATE_KEY: &str =
    "-----BEGIN PRIVATE KEY-----\nY2xvdWQtc2RrLXNlY3JldA==\n-----END PRIVATE KEY-----";
const CERTIFICATES: &[u8] = br#"{"certificates":[{"id":42,"name":"website","labels":{},"type":"uploaded","certificate":"-----BEGIN CERTIFICATE-----\nY2xvdWQtc2RrLXRlc3Q=\n-----END CERTIFICATE-----","created":"2026-01-01T00:00:00Z","not_valid_before":"2026-01-01T00:00:00Z","not_valid_after":"2027-01-01T00:00:00Z","domain_names":["example.com"],"fingerprint":"03:c7:55","status":null,"used_by":[]}],"meta":{"pagination":{"page":1,"per_page":1,"previous_page":null,"next_page":null,"last_page":1,"total_entries":1}}}"#;
const CREATED: &[u8] = br#"{"certificate":{"id":42,"name":"website","labels":{},"type":"uploaded","certificate":"-----BEGIN CERTIFICATE-----\nY2xvdWQtc2RrLXRlc3Q=\n-----END CERTIFICATE-----","created":"2026-01-01T00:00:00Z","not_valid_before":"2026-01-01T00:00:00Z","not_valid_after":"2027-01-01T00:00:00Z","domain_names":["example.com"],"fingerprint":"03:c7:55","status":null,"used_by":[]},"action":{"id":42,"command":"create_certificate","status":"running","progress":0,"started":"2026-01-01T00:00:00Z","finished":null,"resources":[{"id":42,"type":"certificate"}],"error":null}}"#;

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

#[test]
fn security_client_registry_is_complete_sorted_and_policy_exact() {
    assert_eq!(SECURITY_CLIENT_METHODS.len(), 14);
    assert!(SECURITY_CLIENT_METHODS.windows(2).all(|pair| matches!(
        pair,
        [previous, next]
            if previous.operation().operation_id().as_str()
                < next.operation().operation_id().as_str()
    )));
    assert!(SECURITY_CLIENT_METHODS.iter().all(|method| {
        method.operation().service_id() == cloud_sdk_hetzner::identity::SECURITY_SERVICE_ID
    }));
    for (permit, expected) in [
        (PermitClass::None, 7),
        (PermitClass::Mutation, 5),
        (PermitClass::Destructive, 2),
        (PermitClass::Cost, 0),
    ] {
        assert_eq!(
            SECURITY_CLIENT_METHODS
                .iter()
                .filter(|method| method.permit() == permit)
                .count(),
            expected,
        );
    }
    assert_eq!(
        SECURITY_CLIENT_METHODS
            .iter()
            .filter(|method| method.pagination() == PaginationPolicy::Numbered)
            .count(),
        4,
    );
}

#[test]
fn named_security_reads_preserve_pagination_quota_and_executor_parity() {
    let operation = list_certificates_operation();
    let mut expected_target = [0_u8; 128];
    let mut expected_body = [0_u8; 16];
    let prepared = operation
        .prepare(PreparationStorage::new(
            &mut expected_target,
            &mut expected_body,
        ))
        .unwrap_or_else(|_| unreachable!("list-certificates preparation failed"));
    let request = prepared.transport_request();
    let expected = ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers());
    let endpoint = official_endpoint();

    let exchanges = [MockExchange::new(expected, certificates_fixture())];
    let client = HetznerClient::security(MockTransport::new(&exchanges).with_endpoint(endpoint))
        .unwrap_or_else(|_| unreachable!("blocking Security client construction failed"));
    let mut workspace = ReadWorkspace::new();
    let pool = ClientWorkspacePool::<1>::new()
        .unwrap_or_else(|_| unreachable!("blocking Security workspace pool failed"));
    let lease = workspace.lease(&pool);
    let result = client.list_certificates_blocking(&operation, lease);
    let Ok(result) = result else {
        unreachable!("named blocking Security read failed")
    };
    assert_certificate_metadata(&result);
    assert!(client.transport().is_complete());

    let exchanges = [MockExchange::new(expected, certificates_fixture())];
    let client = HetznerClient::security(MockTransport::new(&exchanges).with_endpoint(endpoint))
        .unwrap_or_else(|_| unreachable!("Send-async Security client construction failed"));
    let mut workspace = ReadWorkspace::new();
    let pool = ClientWorkspacePool::<1>::new()
        .unwrap_or_else(|_| unreachable!("async Security workspace pool failed"));
    let future = client.list_certificates_async(&operation, workspace.lease(&pool));
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    let Poll::Ready(Ok(result)) = Future::poll(future.as_mut(), &mut context) else {
        unreachable!("named Send-async Security read did not complete")
    };
    assert_certificate_metadata(&result);
    assert!(client.transport().is_complete());

    let exchanges = [MockExchange::new(expected, certificates_fixture())];
    let client =
        HetznerClient::security(LocalMockTransport::new(&exchanges).with_endpoint(endpoint))
            .unwrap_or_else(|_| unreachable!("local Security client construction failed"));
    let mut workspace = ReadWorkspace::new();
    let pool = ClientWorkspacePool::<1>::new()
        .unwrap_or_else(|_| unreachable!("local Security workspace pool failed"));
    let future = client.list_certificates_local_async(&operation, workspace.lease(&pool));
    let mut future = core::pin::pin!(future);
    let Poll::Ready(Ok(result)) = Future::poll(future.as_mut(), &mut context) else {
        unreachable!("named local Security read did not complete")
    };
    assert_certificate_metadata(&result);
    assert!(client.transport().is_complete());
}

#[test]
fn uploaded_private_key_is_redacted_permitted_and_cleanup_owned() {
    let endpoint = official_endpoint();
    let operation = create_certificate_operation();
    let mut target = [0xa5_u8; 128];
    let mut request_body = [0x5a_u8; 512];
    {
        let no_exchanges = [];
        let preparation_client =
            HetznerClient::security(MockTransport::new(&no_exchanges).with_endpoint(endpoint))
                .unwrap_or_else(|_| unreachable!("Security preparation client failed"));
        let mut storage = PreparationStorageGuard::new(&mut target, &mut request_body);
        let prepared = preparation_client
            .prepare_create_certificate(&operation, &mut storage)
            .unwrap_or_else(|_| unreachable!("certificate preparation failed"));
        let request = prepared.as_untyped().transport_request();
        let json = core::str::from_utf8(request.body())
            .unwrap_or_else(|_| unreachable!("certificate JSON was not UTF-8"));
        assert!(json.contains("\"private_key\""));
        assert!(json.contains("Y2xvdWQtc2RrLXNlY3JldA=="));
        assert!(!format!("{operation:?} {prepared:?}").contains(PRIVATE_KEY));
        let expected = ExpectedRequest::new(request.method(), request.target())
            .with_body(request.body())
            .with_headers(request.headers());
        let mut fingerprint_storage = [0_u8; 4_096];
        let fingerprint = build_associated_canonical_plan(
            mutation_plan(prepared, endpoint),
            &mut fingerprint_storage,
        )
        .unwrap_or_else(|_| unreachable!("certificate fingerprint failed"));
        let mut permit = AssociatedMutationPermit::new(
            fingerprint.subject(),
            PermitTimestamp::from_seconds(100),
        )
        .unwrap_or_else(|_| unreachable!("certificate permit failed"));
        let attempt = permit
            .begin(PermitTimestamp::from_seconds(101))
            .unwrap_or_else(|_| unreachable!("certificate attempt failed"));
        let body = FixtureBody::new(CREATED)
            .unwrap_or_else(|_| unreachable!("certificate response fixture failed"));
        let fixture = ResponseFixture::success_at(StatusCode::CREATED, body)
            .unwrap_or_else(|_| unreachable!("certificate response status failed"))
            .with_content_type("application/json");
        let exchanges = [MockExchange::new(expected, fixture)];
        let client =
            HetznerClient::security(MockTransport::new(&exchanges).with_endpoint(endpoint))
                .unwrap_or_else(|_| unreachable!("Security mutation client failed"));
        let mut response_body = [0_u8; 1_024];
        let mut response_headers = [0_u8; 8_192];
        let response = client
            .create_certificate_blocking(
                attempt,
                &FixedClock,
                &mut response_body,
                &mut response_headers,
            )
            .unwrap_or_else(|_| unreachable!("certificate mutation execution failed"));
        let decoded = decode_associated_checked_response(response)
            .unwrap_or_else(|_| unreachable!("certificate create decoding failed"));
        assert!(matches!(decoded.success(), HetznerSuccess::Composite(_)));
        assert!(client.transport().is_complete());
    }
    assert_eq!(target, [0_u8; 128]);
    assert_eq!(request_body, [0_u8; 512]);
}

struct ReadWorkspace {
    target: [u8; 128],
    request_body: [u8; 16],
    response_body: [u8; 1_024],
    response_headers: [u8; 8_192],
}

impl ReadWorkspace {
    const fn new() -> Self {
        Self {
            target: [0; 128],
            request_body: [0; 16],
            response_body: [0; 1_024],
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
        .unwrap_or_else(|_| unreachable!("Security workspace acquisition failed"))
    }
}

fn list_certificates_operation()
-> AssociatedOperation<ListCertificates, CertificateEndpoint, CertificateListRequest<'static>> {
    let page = Page::new(1).unwrap_or_else(|_| unreachable!("Security page fixture failed"));
    let per_page =
        PerPage::new(1).unwrap_or_else(|_| unreachable!("Security per-page fixture failed"));
    AssociatedOperation::<ListCertificates, _, _>::query(
        CertificateEndpoint::List,
        CertificateListRequest::new()
            .with_page(page)
            .with_per_page(per_page),
    )
    .unwrap_or_else(|_| unreachable!("list-certificates association failed"))
}

fn create_certificate_operation() -> AssociatedOperation<
    CreateCertificate,
    CertificateEndpoint,
    cloud_sdk_hetzner::prepared::NoQuery,
    CertificateCreateRequest<'static>,
> {
    let name = CertificateName::new("website")
        .unwrap_or_else(|_| unreachable!("certificate name fixture failed"));
    let certificate = certificate_pem(CERTIFICATE)
        .unwrap_or_else(|_| unreachable!("certificate PEM fixture failed"));
    let private_key =
        private_key_pem(PRIVATE_KEY).unwrap_or_else(|_| unreachable!("private-key fixture failed"));
    let request = CertificateCreateRequest::new(
        name,
        CertificateCreateMode::uploaded(certificate, private_key),
    );
    AssociatedOperation::<CreateCertificate, _, _, _>::json(request.endpoint(), request)
        .unwrap_or_else(|_| unreachable!("create-certificate association failed"))
}

fn certificates_fixture() -> ResponseFixture<'static> {
    let body = FixtureBody::new(CERTIFICATES)
        .unwrap_or_else(|_| unreachable!("Security response fixture body failed"));
    let rate_limit = RateLimitFixture::new(3600, 3599, 42)
        .unwrap_or_else(|_| unreachable!("Security rate-limit fixture failed"));
    ResponseFixture::success(body)
        .with_content_type("application/json")
        .with_rate_limit(rate_limit)
}

fn official_endpoint() -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1")
        .unwrap_or_else(|_| unreachable!("official Security endpoint fixture failed"))
}

fn assert_certificate_metadata(result: &cloud_sdk_hetzner::serde::CheckedHetznerResponse) {
    let HetznerSuccess::SecurityResources {
        resources,
        pagination: Some(pagination),
    } = result.success()
    else {
        unreachable!("Security list decoded another response family")
    };
    assert!(matches!(
        resources.as_slice(),
        [SecurityResource::Certificate(certificate)]
            if certificate.name() == "website"
    ));
    assert_eq!(resources[0].kind(), SecurityResourceKind::Certificate);
    assert_eq!(pagination.total_entries(), Some(1));
    assert_eq!(
        result.rate_limit().map(|value| value.remaining()),
        Some(3599)
    );
}

fn mutation_plan<'request>(
    prepared: cloud_sdk_hetzner::association::Prepared<'request, CreateCertificate>,
    endpoint: EndpointIdentity<'static>,
) -> AssociatedPlanConfirmation<'static, 'request, CreateCertificate> {
    let context = PermitContext::new(b"v0.72 Security mutation fixture")
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
