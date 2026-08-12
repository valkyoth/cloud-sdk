use core::future::Future;
use core::task::{Context, Poll, Waker};

use cloud_sdk::operation::{
    AttemptBudget, ExecutionPermitError, PermitClock, PermitContext, PermitState, PermitTimestamp,
    PermitValidity, PlanChange, PlanFingerprintBuildError, PlanFingerprintScope,
    PreparationStorage, PreparedRequest, ReplayPolicy, SharedPermitState,
};
use cloud_sdk::transport::EndpointIdentity;
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, LocalMockTransport, MockExchange, MockTransport, ResponseFixture,
};

use super::*;
use crate::association::Sha256PlanHasher;
use crate::endpoint::official_robot_endpoint_identity;
use crate::robot::RobotIpAddress;

use super::tests::{DETAIL, MAC_DELETED, MAC_SET};

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

#[test]
fn sensitive_update_requires_digest_and_executes_blocking() {
    let request = RobotIpUpdateRequest::new(
        ip(),
        RobotIpTrafficUpdate::warnings(true)
            .with_hourly(50)
            .with_daily(500)
            .with_monthly(8),
    );
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 256];
    let mut digest_target = [0_u8; 128];
    let mut digest_body = [0_u8; 256];
    let prepared = request
        .prepare_bound(PreparationStorage::new(
            &mut digest_target,
            &mut digest_body,
        ))
        .unwrap_or_else(|_| unreachable!("update preparation failed"));
    let expected = expected_request(prepared.as_untyped());
    let endpoint = endpoint();
    let mut exact = [0xa5_u8; 4_096];
    assert!(matches!(
        build_robot_ip_canonical_plan(plan(prepared, endpoint), &mut exact),
        Err(PlanFingerprintBuildError::SensitiveBodyRequiresDigest)
    ));
    assert_eq!(exact, [0_u8; 4_096]);

    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("update preparation failed"));
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    let fingerprint = build_robot_ip_plan_digest(
        plan(prepared, endpoint),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("update digest failed"));
    assert_eq!(scratch, [0_u8; 4_096]);
    let mut permit =
        RobotIpMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100))
            .unwrap_or_else(|_| unreachable!("mutation permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("mutation attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(DETAIL))];
    let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [0_u8; 1_024];
    let mut response_headers = [0_u8; 128];
    let checked = attempt
        .execute_blocking(
            &FixedClock,
            &transport,
            &mut response_body,
            &mut response_headers,
        )
        .unwrap_or_else(|_| unreachable!("blocking update execution failed"));
    assert!(checked.decode_response().is_ok());
    assert!(transport.is_complete());
}

#[test]
fn bodyless_set_executes_through_shared_send_async_permit() {
    let request = RobotIpMacSetRequest::new(ip());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("MAC set preparation failed"));
    let expected = expected_request(prepared.as_untyped());
    let endpoint = endpoint();
    let mut exact = [0_u8; 4_096];
    let fingerprint = build_robot_ip_canonical_plan(plan(prepared, endpoint), &mut exact)
        .unwrap_or_else(|_| unreachable!("MAC set fingerprint failed"));
    let mut state = SharedPermitState::new();
    let permit = RobotIpSharedMutationPermit::new(
        &mut state,
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("shared mutation permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("shared mutation attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(MAC_SET))];
    let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [0_u8; 256];
    let mut response_headers = [0_u8; 128];
    let checked = ready(attempt.execute_async(
        &FixedClock,
        &transport,
        &mut response_body,
        &mut response_headers,
    ))
    .unwrap_or_else(|_| unreachable!("async MAC set failed"));
    assert!(checked.decode_response().is_ok());
    assert!(transport.is_complete());
}

#[test]
fn delete_executes_through_shared_local_destructive_permit() {
    let request = RobotIpMacDeleteRequest::new(ip());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("MAC delete preparation failed"));
    let expected = expected_request(prepared.as_untyped());
    let endpoint = endpoint();
    let mut exact = [0_u8; 4_096];
    let fingerprint = build_robot_ip_canonical_plan(plan(prepared, endpoint), &mut exact)
        .unwrap_or_else(|_| unreachable!("MAC delete fingerprint failed"));
    let mut state = SharedPermitState::new();
    let permit = RobotIpSharedDestructivePermit::new(
        &mut state,
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("shared destructive permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("destructive attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(MAC_DELETED))];
    let transport = LocalMockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [0_u8; 256];
    let mut response_headers = [0_u8; 128];
    let checked = ready(attempt.execute_local_async(
        &FixedClock,
        &transport,
        &mut response_body,
        &mut response_headers,
    ))
    .unwrap_or_else(|_| unreachable!("local MAC delete failed"));
    assert!(checked.decode_response().is_ok());
    assert!(transport.is_complete());
}

#[test]
fn permit_scope_mismatch_fails_closed() {
    let update = RobotIpUpdateRequest::new(ip(), RobotIpTrafficUpdate::warnings(true));
    let mut update_target = [0_u8; 128];
    let mut update_body = [0_u8; 128];
    let prepared = update
        .prepare_bound(PreparationStorage::new(
            &mut update_target,
            &mut update_body,
        ))
        .unwrap_or_else(|_| unreachable!("update preparation failed"));
    let mut scratch = [0_u8; 4_096];
    let mut digest = [0_u8; 32];
    let fingerprint = build_robot_ip_plan_digest(
        plan(prepared, endpoint()),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("update digest failed"));
    assert!(matches!(
        RobotIpDestructivePermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100)),
        Err(ExecutionPermitError::ScopeMismatch)
    ));

    let delete = RobotIpMacDeleteRequest::new(ip());
    let mut delete_target = [0_u8; 128];
    let mut delete_body = [0_u8; 1];
    let prepared = delete
        .prepare_bound(PreparationStorage::new(
            &mut delete_target,
            &mut delete_body,
        ))
        .unwrap_or_else(|_| unreachable!("delete preparation failed"));
    let mut exact = [0_u8; 4_096];
    let fingerprint = build_robot_ip_canonical_plan(plan(prepared, endpoint()), &mut exact)
        .unwrap_or_else(|_| unreachable!("delete fingerprint failed"));
    assert!(matches!(
        RobotIpMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100)),
        Err(ExecutionPermitError::ScopeMismatch)
    ));
}

#[test]
fn unpolled_attempt_clears_buffers_and_consumes_authority() {
    let request = RobotIpMacSetRequest::new(ip());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("MAC set preparation failed"));
    let endpoint = endpoint();
    let mut exact = [0_u8; 4_096];
    let fingerprint = build_robot_ip_canonical_plan(plan(prepared, endpoint), &mut exact)
        .unwrap_or_else(|_| unreachable!("MAC set fingerprint failed"));
    let mut permit =
        RobotIpMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100))
            .unwrap_or_else(|_| unreachable!("mutation permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("mutation attempt failed"));
    let exchanges = [];
    let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [0xa5_u8; 256];
    let mut response_headers = [0x5a_u8; 128];

    let future = attempt.execute_async(
        &FixedClock,
        &transport,
        &mut response_body,
        &mut response_headers,
    );
    drop(future);

    assert_eq!(response_body, [0_u8; 256]);
    assert_eq!(response_headers, [0_u8; 128]);
    assert_eq!(permit.state(), PermitState::PendingReconciliation);
    assert!(transport.is_complete());
}

fn plan<'storage, 'request, R>(
    prepared: PreparedRobotIp<'storage, 'request, R>,
    endpoint: EndpointIdentity<'static>,
) -> RobotIpPlanConfirmation<'static, 'storage, 'request, R>
where
    R: RobotIpPermitRequest,
{
    let context = PermitContext::new(b"v0.80 Robot IP permit fixture")
        .unwrap_or_else(|_| unreachable!("permit context failed"));
    let validity = PermitValidity::new(
        PermitTimestamp::from_seconds(100),
        PermitTimestamp::from_seconds(200),
    )
    .unwrap_or_else(|_| unreachable!("permit validity failed"));
    let attempts = AttemptBudget::new(1).unwrap_or_else(|_| unreachable!("budget failed"));
    RobotIpPlanConfirmation::new(
        prepared,
        endpoint,
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

fn expected_request(prepared: PreparedRequest<'_>) -> ExpectedRequest<'_> {
    let request = prepared.transport_request();
    ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers())
}

fn json_fixture(body: &'static [u8]) -> ResponseFixture<'static> {
    let body = FixtureBody::new(body).unwrap_or_else(|_| unreachable!("fixture body failed"));
    ResponseFixture::success(body).with_content_type("application/json")
}

fn ready<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match Future::poll(future.as_mut(), &mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => unreachable!("deterministic future remained pending"),
    }
}

fn endpoint() -> EndpointIdentity<'static> {
    official_robot_endpoint_identity()
        .unwrap_or_else(|_| unreachable!("official Robot endpoint failed"))
}

fn ip() -> RobotIpAddress {
    RobotIpAddress::new("192.0.2.10").unwrap_or_else(|_| unreachable!("IP fixture failed"))
}
