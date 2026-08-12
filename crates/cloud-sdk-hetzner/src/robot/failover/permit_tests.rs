use core::future::Future;
use core::task::{Context, Poll, Waker};

use cloud_sdk::operation::{
    AttemptBudget, ExecutionPermitError, PermitClock, PermitContext, PermitTimestamp,
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

use super::tests::{ACTIVE, DELETED, ip};

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

#[test]
fn sensitive_reroute_requires_digest_and_executes_blocking() {
    let request = RobotFailoverRerouteRequest::new(ip("192.0.2.50"), ip("192.0.2.11"))
        .unwrap_or_else(|_| unreachable!("reroute fixture failed"));
    let mut exact_target = [0_u8; 128];
    let mut exact_body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut exact_target, &mut exact_body))
        .unwrap_or_else(|_| unreachable!("reroute preparation failed"));
    let endpoint = endpoint();
    let mut exact = [0xa5_u8; 4_096];
    assert!(matches!(
        build_robot_failover_canonical_plan(plan(prepared, endpoint), &mut exact),
        Err(PlanFingerprintBuildError::SensitiveBodyRequiresDigest)
    ));
    assert_eq!(exact, [0_u8; 4_096]);

    let mut target = [0_u8; 128];
    let mut body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("reroute preparation failed"));
    let expected = expected_request(prepared.as_untyped());
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    let fingerprint = build_robot_failover_plan_digest(
        plan(prepared, endpoint),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("reroute digest failed"));
    assert_eq!(scratch, [0_u8; 4_096]);
    let mut permit =
        RobotFailoverMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100))
            .unwrap_or_else(|_| unreachable!("mutation permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("mutation attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(ACTIVE))];
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
        .unwrap_or_else(|_| unreachable!("reroute execution failed"));
    assert!(checked.decode_response().is_ok());
    assert!(transport.is_complete());
}

#[test]
fn delete_executes_through_shared_local_destructive_permit() {
    let request = RobotFailoverDeleteRouteRequest::new(ip("192.0.2.50"));
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("delete preparation failed"));
    let expected = expected_request(prepared.as_untyped());
    let endpoint = endpoint();
    let mut exact = [0_u8; 4_096];
    let fingerprint = build_robot_failover_canonical_plan(plan(prepared, endpoint), &mut exact)
        .unwrap_or_else(|_| unreachable!("delete fingerprint failed"));
    let mut state = SharedPermitState::new();
    let permit = RobotFailoverSharedDestructivePermit::new(
        &mut state,
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("shared destructive permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("destructive attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(DELETED))];
    let transport = LocalMockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [0_u8; 1_024];
    let mut response_headers = [0_u8; 128];
    let checked = ready(attempt.execute_local_async(
        &FixedClock,
        &transport,
        &mut response_body,
        &mut response_headers,
    ))
    .unwrap_or_else(|_| unreachable!("local delete failed"));
    assert!(checked.decode_response().is_ok());
    assert!(transport.is_complete());
}

#[test]
fn mutation_and_destructive_authority_are_not_interchangeable() {
    let reroute = RobotFailoverRerouteRequest::new(ip("192.0.2.50"), ip("192.0.2.11"))
        .unwrap_or_else(|_| unreachable!("reroute fixture failed"));
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 128];
    let prepared = reroute
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("reroute preparation failed"));
    let mut scratch = [0_u8; 4_096];
    let mut digest = [0_u8; 32];
    let fingerprint = build_robot_failover_plan_digest(
        plan(prepared, endpoint()),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("reroute digest failed"));
    assert!(matches!(
        RobotFailoverDestructivePermit::new(
            fingerprint.subject(),
            PermitTimestamp::from_seconds(100)
        ),
        Err(ExecutionPermitError::ScopeMismatch)
    ));

    let delete = RobotFailoverDeleteRouteRequest::new(ip("192.0.2.50"));
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 1];
    let prepared = delete
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("delete preparation failed"));
    let mut exact = [0_u8; 4_096];
    let fingerprint = build_robot_failover_canonical_plan(plan(prepared, endpoint()), &mut exact)
        .unwrap_or_else(|_| unreachable!("delete fingerprint failed"));
    assert!(matches!(
        RobotFailoverMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100)),
        Err(ExecutionPermitError::ScopeMismatch)
    ));
}

fn plan<'storage, 'request, R>(
    prepared: PreparedRobotFailover<'storage, 'request, R>,
    endpoint: EndpointIdentity<'static>,
) -> RobotFailoverPlanConfirmation<'static, 'storage, 'request, R>
where
    R: RobotFailoverPermitRequest,
{
    RobotFailoverPlanConfirmation::new(
        prepared,
        endpoint,
        PlanFingerprintScope::Value(b"robot-account"),
        PlanFingerprintScope::Absent,
        PermitContext::new(b"v0.83 Robot failover fixture")
            .unwrap_or_else(|_| unreachable!("permit context failed")),
        PermitValidity::new(
            PermitTimestamp::from_seconds(100),
            PermitTimestamp::from_seconds(200),
        )
        .unwrap_or_else(|_| unreachable!("permit validity failed")),
        ReplayPolicy::SingleAttempt,
        AttemptBudget::new(1).unwrap_or_else(|_| unreachable!("attempt budget failed")),
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
    ResponseFixture::success(
        FixtureBody::new(body).unwrap_or_else(|_| unreachable!("fixture body failed")),
    )
    .with_content_type("application/json")
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
