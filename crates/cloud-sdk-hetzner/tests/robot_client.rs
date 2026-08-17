//! End-to-end evidence for the official-endpoint Robot client.

use core::future::Future;
use core::task::{Context, Poll, Waker};

use cloud_sdk::authentication::{CredentialAttemptStatus, CredentialReconfirmation};
use cloud_sdk::client::{ClientWorkspace, ClientWorkspacePool};
use cloud_sdk::operation::{
    AttemptBudget, PermitClock, PermitContext, PermitTimestamp, PermitValidity, PlanChange,
    PlanFingerprintScope, PreparationStorageGuard, ReplayPolicy,
};
use cloud_sdk::transport::{
    EndpointIdentity, EndpointScheme, MediaType, RequestHeader, RequestHeaders, RequestTarget,
    StatusCode,
};
use cloud_sdk_hetzner::client::{
    ROBOT_CLIENT_METHODS, RobotClient, RobotClientExecutionError, RobotClientResponse,
    RobotMutationClientExecutionError, RobotMutationPermit, RobotPermitClientExecutionError,
    RobotResponseDecodeError, build_robot_mutation_canonical_plan, prepare_robot_client_mutation,
};
use cloud_sdk_hetzner::robot::{
    RobotServerDecodeError, RobotServerList, RobotServerListRequest, RobotServerName,
    RobotServerNumber, RobotServerRequestError, RobotServerUpdateRequest,
};
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, LocalMockTransport, MockExchange, MockTransport, ResponseFixture,
};

const SUMMARY: &str = r#"{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8:1::","server_number":321,"server_name":"server-1","product":"AX42","dc":"FSN1-DC10","traffic":"unlimited","status":"ready","cancelled":false,"paid_until":"2028-02-29","ip":["192.0.2.10","2001:db8:1::1"],"subnet":null}"#;
const ACCEPT: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

#[test]
fn public_client_inventory_matches_every_active_source_locked_operation() {
    let source: serde_json::Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/robot-api/v0.74.0.json"
    ))
    .unwrap_or_else(|_| unreachable!("Robot inventory fixture failed"));
    let active = source
        .get("operations")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| unreachable!("Robot operation inventory is not an array"))
        .iter()
        .filter(|operation| {
            operation.get("status").and_then(serde_json::Value::as_str) == Some("active")
        })
        .collect::<Vec<_>>();
    assert_eq!(ROBOT_CLIENT_METHODS.len(), 89);
    assert_eq!(active.len(), ROBOT_CLIENT_METHODS.len());
    for (source, client) in active.into_iter().zip(ROBOT_CLIENT_METHODS) {
        assert_eq!(
            source.get("id").and_then(serde_json::Value::as_str),
            Some(client.id())
        );
        assert_eq!(
            source.get("method").and_then(serde_json::Value::as_str),
            Some(client.method().as_str()),
        );
        assert_eq!(
            source.get("path").and_then(serde_json::Value::as_str),
            Some(client.path())
        );
    }
}

#[test]
fn official_constructor_rejects_every_non_robot_endpoint() {
    let exchanges = [];
    let wrong = EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1")
        .unwrap_or_else(|_| unreachable!("wrong-endpoint fixture failed"));
    assert!(RobotClient::official(MockTransport::new(&exchanges).with_endpoint(wrong)).is_err());
}

#[test]
fn typed_server_list_has_blocking_send_and_local_executor_parity() {
    let request = RobotServerListRequest::new();
    let expected = expected_request();
    let payload = list_payload();
    let fixture = success_fixture(payload.as_bytes());
    let endpoint = official_endpoint();

    let exchanges = [MockExchange::new(expected, fixture)];
    let client = RobotClient::official(MockTransport::new(&exchanges).with_endpoint(endpoint))
        .unwrap_or_else(|_| unreachable!("blocking Robot client construction failed"));
    let mut workspace = ReadWorkspace::new();
    let pool = workspace_pool();
    let result = client.execute_blocking(&request, workspace.lease(&pool));
    assert_server_list(result);
    assert!(client.transport().is_complete());

    let exchanges = [MockExchange::new(
        expected,
        success_fixture(payload.as_bytes()),
    )];
    let client = RobotClient::official(MockTransport::new(&exchanges).with_endpoint(endpoint))
        .unwrap_or_else(|_| unreachable!("Send Robot client construction failed"));
    let mut workspace = ReadWorkspace::new();
    let pool = workspace_pool();
    assert_server_list(ready(
        client.execute_async(&request, workspace.lease(&pool)),
    ));
    assert!(client.transport().is_complete());

    let exchanges = [MockExchange::new(
        expected,
        success_fixture(payload.as_bytes()),
    )];
    let client = RobotClient::official(LocalMockTransport::new(&exchanges).with_endpoint(endpoint))
        .unwrap_or_else(|_| unreachable!("local Robot client construction failed"));
    let mut workspace = ReadWorkspace::new();
    let pool = workspace_pool();
    assert_server_list(ready(
        client.execute_local_async(&request, workspace.lease(&pool)),
    ));
    assert!(client.transport().is_complete());
}

#[test]
fn authentication_rejection_sends_once_locks_reuse_and_requires_reconfirmation() {
    let request = RobotServerListRequest::new();
    let expected = expected_request();
    let payload = list_payload();
    let exchanges = [
        MockExchange::new(expected, authentication_fixture()),
        MockExchange::new(expected, success_fixture(payload.as_bytes())),
    ];
    let client =
        RobotClient::official(MockTransport::new(&exchanges).with_endpoint(official_endpoint()))
            .unwrap_or_else(|_| unreachable!("Robot client construction failed"));
    let pool = workspace_pool();

    let mut first = ReadWorkspace::new();
    let result = client.execute_blocking(&request, first.lease(&pool));
    assert!(matches!(
        result,
        Ok(RobotClientResponse::Failure(
            cloud_sdk_hetzner::robot::RobotFailure::AuthenticationRejected
        ))
    ));
    assert_eq!(client.transport().remaining(), 1);
    assert_eq!(
        client.credential_status().1,
        CredentialAttemptStatus::Rejected
    );

    let mut rejected = ReadWorkspace::new();
    assert!(matches!(
        client.execute_blocking(&request, rejected.lease(&pool)),
        Err(RobotClientExecutionError::Lifecycle(_))
    ));
    assert_eq!(client.transport().remaining(), 1);

    client
        .reconfirm_credentials(CredentialReconfirmation::acknowledge_same_credentials())
        .unwrap_or_else(|_| unreachable!("explicit credential reconfirmation failed"));
    let mut confirmed = ReadWorkspace::new();
    assert_server_list(client.execute_blocking(&request, confirmed.lease(&pool)));
    assert!(client.transport().is_complete());
}

#[test]
fn malformed_unauthorized_response_still_closes_the_generation() {
    let request = RobotServerListRequest::new();
    let expected = expected_request();
    let body = FixtureBody::new(b"null")
        .unwrap_or_else(|_| unreachable!("malformed authentication body failed"));
    let fixture = ResponseFixture::error(status(401), body)
        .unwrap_or_else(|_| unreachable!("authentication fixture failed"))
        .with_content_type("application/json");
    let exchanges = [MockExchange::new(expected, fixture)];
    let client =
        RobotClient::official(MockTransport::new(&exchanges).with_endpoint(official_endpoint()))
            .unwrap_or_else(|_| unreachable!("Robot client construction failed"));
    let mut workspace = ReadWorkspace::new();
    let pool = workspace_pool();
    let result = client.execute_blocking(&request, workspace.lease(&pool));
    assert!(matches!(
        result,
        Err(RobotClientExecutionError::Execution(
            cloud_sdk::client::ClientExecutionError::Decode(RobotResponseDecodeError::Failure {
                authentication_rejected: true,
                ..
            })
        ))
    ));
    assert_eq!(
        client.credential_status().1,
        CredentialAttemptStatus::Rejected
    );
    assert!(client.transport().is_complete());
}

#[test]
fn unpolled_send_future_neither_admits_credentials_nor_touches_the_wire() {
    let request = RobotServerListRequest::new();
    let exchanges = [MockExchange::new(
        expected_request(),
        authentication_fixture(),
    )];
    let client =
        RobotClient::official(MockTransport::new(&exchanges).with_endpoint(official_endpoint()))
            .unwrap_or_else(|_| unreachable!("Robot client construction failed"));
    let pool = workspace_pool();
    let mut target = [0xa5_u8; 128];
    let mut request_body = [0xa5_u8; 128];
    let mut response_body = [0xa5_u8; 16_384];
    let mut response_headers = [0xa5_u8; 8_192];
    {
        let workspace = ClientWorkspace::new(
            &mut target,
            &mut request_body,
            &mut response_body,
            &mut response_headers,
        );
        let lease = pool
            .try_acquire(workspace)
            .unwrap_or_else(|_| unreachable!("workspace lease failed"));
        let future = client.execute_async(&request, lease);
        drop(future);
    }
    assert_eq!(pool.active_leases(), 0);
    assert_eq!(client.transport().remaining(), 1);
    assert_eq!(client.credential_status().1, CredentialAttemptStatus::Open);
    assert_eq!(target, [0_u8; 128]);
    assert_eq!(request_body, [0_u8; 128]);
    assert_eq!(response_body, [0_u8; 16_384]);
    assert_eq!(response_headers, [0_u8; 8_192]);
}

#[test]
fn concurrently_admitted_attempt_revalidates_after_another_call_rejects_credentials() {
    let number = RobotServerNumber::new(321)
        .unwrap_or_else(|_| unreachable!("server-number fixture failed"));
    let name = RobotServerName::new("renamed-1")
        .unwrap_or_else(|_| unreachable!("server-name fixture failed"));
    let mutation = RobotServerUpdateRequest::rename(number, name);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let mut storage = PreparationStorageGuard::new(&mut target, &mut request_body);
    let prepared = prepare_robot_client_mutation(&mutation, &mut storage)
        .unwrap_or_else(|_| unreachable!("Robot mutation preparation failed"));
    let mutation_expected = expected_prepared(prepared.as_untyped());
    let plan = mutation_plan(prepared);
    let mut fingerprint_storage = [0_u8; 4_096];
    let fingerprint = build_robot_mutation_canonical_plan(plan, &mut fingerprint_storage)
        .unwrap_or_else(|_| unreachable!("Robot mutation fingerprint failed"));
    let mut permit =
        RobotMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100))
            .unwrap_or_else(|_| unreachable!("Robot mutation permit failed"));
    let exchanges = [
        MockExchange::new(expected_request(), authentication_fixture()),
        MockExchange::new(mutation_expected, authentication_fixture()),
    ];
    let client =
        RobotClient::official(MockTransport::new(&exchanges).with_endpoint(official_endpoint()))
            .unwrap_or_else(|_| unreachable!("Robot client construction failed"));
    let stale = client
        .begin_permit_attempt()
        .unwrap_or_else(|_| unreachable!("concurrent credential attempt failed"));
    let read = RobotServerListRequest::new();
    let mut workspace = ReadWorkspace::new();
    let pool = workspace_pool();
    assert!(matches!(
        client.execute_blocking(&read, workspace.lease(&pool)),
        Ok(RobotClientResponse::Failure(
            cloud_sdk_hetzner::robot::RobotFailure::AuthenticationRejected
        ))
    ));
    assert_eq!(client.transport().remaining(), 1);

    let permit_attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("Robot permit attempt failed"));
    let mut response_body = [0_u8; 1_024];
    let mut response_headers = [0_u8; 8_192];
    let result = stale.execute_mutation_blocking(
        permit_attempt,
        &FixedClock,
        &mut response_body,
        &mut response_headers,
    );
    assert!(matches!(
        result,
        Err(RobotMutationClientExecutionError::Permit(
            RobotPermitClientExecutionError::Lifecycle(_)
        ))
    ));
    assert_eq!(client.transport().remaining(), 1);
}

#[test]
fn mutation_permit_unauthorized_response_spends_authority_and_locks_credentials() {
    let number = RobotServerNumber::new(321)
        .unwrap_or_else(|_| unreachable!("server-number fixture failed"));
    let name = RobotServerName::new("renamed-1")
        .unwrap_or_else(|_| unreachable!("server-name fixture failed"));
    let request = RobotServerUpdateRequest::rename(number, name);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let mut storage = PreparationStorageGuard::new(&mut target, &mut request_body);
    let prepared = prepare_robot_client_mutation(&request, &mut storage)
        .unwrap_or_else(|_| unreachable!("Robot mutation preparation failed"));
    let expected = expected_prepared(prepared.as_untyped());
    let plan = mutation_plan(prepared);
    let mut fingerprint_storage = [0_u8; 4_096];
    let fingerprint = build_robot_mutation_canonical_plan(plan, &mut fingerprint_storage)
        .unwrap_or_else(|_| unreachable!("Robot mutation fingerprint failed"));
    let mut permit =
        RobotMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100))
            .unwrap_or_else(|_| unreachable!("Robot mutation permit failed"));
    let exchanges = [MockExchange::new(expected, authentication_fixture())];
    let client =
        RobotClient::official(MockTransport::new(&exchanges).with_endpoint(official_endpoint()))
            .unwrap_or_else(|_| unreachable!("Robot client construction failed"));
    let client_attempt = client
        .begin_permit_attempt()
        .unwrap_or_else(|_| unreachable!("Robot credential attempt failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("Robot permit attempt failed"));
    let mut response_body = [0_u8; 1_024];
    let mut response_headers = [0_u8; 8_192];
    let result = client_attempt.execute_mutation_blocking(
        attempt,
        &FixedClock,
        &mut response_body,
        &mut response_headers,
    );
    assert!(matches!(
        result,
        Err(RobotMutationClientExecutionError::Permit(
            RobotPermitClientExecutionError::AuthenticationRejected(_)
        ))
    ));
    assert_eq!(
        client.credential_status().1,
        CredentialAttemptStatus::Rejected
    );
    assert!(client.transport().is_complete());
}

fn expected_request() -> ExpectedRequest<'static> {
    let target =
        RequestTarget::new("/server").unwrap_or_else(|_| unreachable!("Robot list target failed"));
    let headers =
        RequestHeaders::new(&ACCEPT).unwrap_or_else(|_| unreachable!("Robot list headers failed"));
    ExpectedRequest::new(cloud_sdk::Method::Get, target).with_headers(headers)
}

fn expected_prepared(prepared: cloud_sdk::operation::PreparedRequest<'_>) -> ExpectedRequest<'_> {
    let wire = prepared.transport_request();
    ExpectedRequest::new(wire.method(), wire.target())
        .with_body(wire.body())
        .with_headers(wire.headers())
}

fn mutation_plan<'storage, 'request>(
    prepared: cloud_sdk_hetzner::client::PreparedRobotClientMutation<
        'storage,
        'request,
        RobotServerUpdateRequest<'request>,
    >,
) -> cloud_sdk_hetzner::client::RobotMutationPlanConfirmation<
    'static,
    'storage,
    'request,
    RobotServerUpdateRequest<'request>,
> {
    let context = PermitContext::new(b"v0.94 Robot client mutation fixture")
        .unwrap_or_else(|_| unreachable!("permit context failed"));
    let validity = PermitValidity::new(
        PermitTimestamp::from_seconds(100),
        PermitTimestamp::from_seconds(200),
    )
    .unwrap_or_else(|_| unreachable!("permit validity failed"));
    let attempts = AttemptBudget::new(1).unwrap_or_else(|_| unreachable!("budget failed"));
    cloud_sdk_hetzner::client::RobotMutationPlanConfirmation::new(
        prepared,
        official_endpoint(),
        PlanFingerprintScope::Value(b"robot-account"),
        PlanFingerprintScope::Absent,
        context,
        validity,
        ReplayPolicy::SingleAttempt,
        attempts,
        PlanChange::ChangesState,
        None,
        None,
    )
}

fn authentication_fixture() -> ResponseFixture<'static> {
    let body = FixtureBody::new(b"")
        .unwrap_or_else(|_| unreachable!("authentication body fixture failed"));
    ResponseFixture::error(status(401), body)
        .unwrap_or_else(|_| unreachable!("authentication fixture failed"))
}

fn success_fixture(body: &[u8]) -> ResponseFixture<'_> {
    let body = FixtureBody::new(body).unwrap_or_else(|_| unreachable!("success body failed"));
    ResponseFixture::success(body).with_content_type("application/json")
}

fn list_payload() -> String {
    format!("[{{\"server\":{SUMMARY}}}]")
}

fn official_endpoint() -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, "robot-ws.your-server.de", 443, "/")
        .unwrap_or_else(|_| unreachable!("official Robot endpoint failed"))
}

fn status(value: u16) -> StatusCode {
    StatusCode::new(value).unwrap_or_else(|| unreachable!("status fixture failed"))
}

fn workspace_pool() -> ClientWorkspacePool<1> {
    ClientWorkspacePool::new().unwrap_or_else(|_| unreachable!("workspace pool failed"))
}

struct ReadWorkspace {
    target: [u8; 128],
    request_body: [u8; 128],
    response_body: [u8; 16_384],
    response_headers: [u8; 8_192],
}

impl ReadWorkspace {
    const fn new() -> Self {
        Self {
            target: [0; 128],
            request_body: [0; 128],
            response_body: [0; 16_384],
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
        .unwrap_or_else(|_| unreachable!("workspace lease failed"))
    }
}

fn assert_server_list<E>(
    result: Result<
        RobotClientResponse<RobotServerList>,
        RobotClientExecutionError<RobotServerRequestError, E, RobotServerDecodeError>,
    >,
) {
    let Ok(RobotClientResponse::Success(list)) = result else {
        unreachable!("Robot server list execution failed")
    };
    assert_eq!(list.len(), 1);
}

fn ready<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match Future::poll(future.as_mut(), &mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => unreachable!("deterministic mock future remained pending"),
    }
}
