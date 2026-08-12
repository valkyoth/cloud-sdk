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
use crate::robot::{RobotMacAddress, RobotSubnetAddress};

use super::test_fixtures::{delete_request, delete_request_with};
use super::tests::{DETAIL, MAC_DELETED, MAC_SET};

mod dispatch_evidence_tests;

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

#[test]
fn sensitive_update_requires_digest_and_executes_blocking() {
    let request = RobotSubnetUpdateRequest::new(
        ip(),
        RobotSubnetTrafficUpdate::warnings(true)
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
        build_robot_subnet_canonical_plan(plan(prepared, endpoint), &mut exact),
        Err(PlanFingerprintBuildError::SensitiveBodyRequiresDigest)
    ));
    assert_eq!(exact, [0_u8; 4_096]);

    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("update preparation failed"));
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    let fingerprint = build_robot_subnet_plan_digest(
        plan(prepared, endpoint),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("update digest failed"));
    assert_eq!(scratch, [0_u8; 4_096]);
    let mut permit =
        RobotSubnetMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100))
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
fn sensitive_set_executes_through_shared_send_async_permit() {
    let request = RobotSubnetMacSetRequest::new(mac_ip(), mac());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("MAC set preparation failed"));
    let expected = expected_request(prepared.as_untyped());
    let endpoint = endpoint();
    let mut scratch = [0_u8; 4_096];
    let mut digest = [0_u8; 32];
    let fingerprint = build_robot_subnet_plan_digest(
        plan(prepared, endpoint),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("MAC set fingerprint failed"));
    assert_eq!(scratch, [0_u8; 4_096]);
    let mut state = SharedPermitState::new();
    let permit = RobotSubnetSharedMutationPermit::new(
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
    let request = delete_request();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("MAC delete preparation failed"));
    let expected = expected_request(prepared.as_untyped());
    let endpoint = endpoint();
    let mut scratch = [0_u8; 4_096];
    let mut digest = [0_u8; 32];
    let fingerprint = build_robot_subnet_plan_digest(
        plan(prepared, endpoint),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("MAC delete fingerprint failed"));
    let mut state = SharedPermitState::new();
    let permit = RobotSubnetSharedDestructivePermit::new(
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
    let update = RobotSubnetUpdateRequest::new(ip(), RobotSubnetTrafficUpdate::warnings(true));
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
    let fingerprint = build_robot_subnet_plan_digest(
        plan(prepared, endpoint()),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("update digest failed"));
    assert!(matches!(
        RobotSubnetDestructivePermit::new(
            fingerprint.subject(),
            PermitTimestamp::from_seconds(100)
        ),
        Err(ExecutionPermitError::ScopeMismatch)
    ));

    let delete = delete_request();
    let mut delete_target = [0_u8; 128];
    let mut delete_body = [0_u8; 1];
    let prepared = delete
        .prepare_bound(PreparationStorage::new(
            &mut delete_target,
            &mut delete_body,
        ))
        .unwrap_or_else(|_| unreachable!("delete preparation failed"));
    let mut scratch = [0_u8; 4_096];
    let mut digest = [0_u8; 32];
    let fingerprint = build_robot_subnet_plan_digest(
        plan(prepared, endpoint()),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("delete fingerprint failed"));
    assert!(matches!(
        RobotSubnetMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100)),
        Err(ExecutionPermitError::ScopeMismatch)
    ));
}

#[test]
fn delete_digest_binds_server_mac_freshness_and_lock_evidence() {
    let request = delete_request();
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("delete preparation failed"));
    let mut exact = [0xa5_u8; 4_096];
    assert!(matches!(
        build_robot_subnet_canonical_plan(plan(prepared, endpoint()), &mut exact),
        Err(PlanFingerprintBuildError::SensitiveAuthorizationEvidenceRequiresDigest)
    ));
    assert_eq!(exact, [0_u8; 4_096]);

    let mut target = [0_u8; 128];
    let mut body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("delete preparation failed"));
    let mut scratch = [0_u8; 4_096];
    let mut digest = [0_u8; 32];
    let fingerprint = build_robot_subnet_plan_digest(
        plan(prepared, endpoint()),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("delete digest failed"));
    let mut permit = RobotSubnetDestructivePermit::new(
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("delete permit failed"));

    for candidate in [
        delete_request_with(
            "192.0.2.2",
            "00:21:85:62:3e:9c",
            b"test-lock-generation-0001",
            99,
            100,
            130,
        ),
        delete_request_with(
            "192.0.2.1",
            "00:21:85:62:3e:9d",
            b"test-lock-generation-0001",
            99,
            100,
            130,
        ),
        delete_request_with(
            "192.0.2.1",
            "00:21:85:62:3e:9c",
            b"test-lock-generation-0002",
            99,
            100,
            130,
        ),
        delete_request_with(
            "192.0.2.1",
            "00:21:85:62:3e:9c",
            b"test-lock-generation-0001",
            99,
            101,
            130,
        ),
    ] {
        let mut candidate_target = [0_u8; 128];
        let mut candidate_body = [0_u8; 1];
        let candidate_prepared = candidate
            .prepare_bound(PreparationStorage::new(
                &mut candidate_target,
                &mut candidate_body,
            ))
            .unwrap_or_else(|_| unreachable!("candidate preparation failed"));
        let mut candidate_scratch = [0_u8; 4_096];
        let mut candidate_digest = [0_u8; 32];
        let candidate_fingerprint = build_robot_subnet_plan_digest(
            plan(candidate_prepared, endpoint()),
            &mut candidate_scratch,
            &mut candidate_digest,
            &Sha256PlanHasher,
        )
        .unwrap_or_else(|_| unreachable!("candidate digest failed"));
        assert!(matches!(
            permit.begin_for(
                candidate_fingerprint.subject(),
                PermitTimestamp::from_seconds(101),
            ),
            Err(ExecutionPermitError::FingerprintMismatch)
        ));
    }

    assert!(matches!(
        permit.begin(PermitTimestamp::from_seconds(129)),
        Err(ExecutionPermitError::Expired)
    ));
}

#[test]
fn delete_digest_rejects_permit_validity_beyond_evidence() {
    let request = delete_request();
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("delete preparation failed"));
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    assert!(matches!(
        build_robot_subnet_plan_digest(
            plan_expires(prepared, endpoint(), 130),
            &mut scratch,
            &mut digest,
            &Sha256PlanHasher,
        ),
        Err(PlanFingerprintBuildError::AuthorizationEvidenceValidityMismatch)
    ));
    assert_eq!(scratch, [0_u8; 4_096]);
    assert_eq!(digest, [0_u8; 32]);
}

#[test]
fn unpolled_attempt_clears_buffers_and_consumes_authority() {
    let request = RobotSubnetMacSetRequest::new(mac_ip(), mac());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("MAC set preparation failed"));
    let endpoint = endpoint();
    let mut scratch = [0_u8; 4_096];
    let mut digest = [0_u8; 32];
    let fingerprint = build_robot_subnet_plan_digest(
        plan(prepared, endpoint),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("MAC set fingerprint failed"));
    let mut permit =
        RobotSubnetMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100))
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
    prepared: PreparedRobotSubnet<'storage, 'request, R>,
    endpoint: EndpointIdentity<'static>,
) -> RobotSubnetPlanConfirmation<'static, 'storage, 'request, R>
where
    R: RobotSubnetPermitRequest,
{
    plan_expires(prepared, endpoint, 129)
}

fn plan_expires<'storage, 'request, R>(
    prepared: PreparedRobotSubnet<'storage, 'request, R>,
    endpoint: EndpointIdentity<'static>,
    expires_at: u64,
) -> RobotSubnetPlanConfirmation<'static, 'storage, 'request, R>
where
    R: RobotSubnetPermitRequest,
{
    let context = PermitContext::new(b"v0.81 Robot subnet permit fixture")
        .unwrap_or_else(|_| unreachable!("permit context failed"));
    let validity = PermitValidity::new(
        PermitTimestamp::from_seconds(100),
        PermitTimestamp::from_seconds(expires_at),
    )
    .unwrap_or_else(|_| unreachable!("permit validity failed"));
    let attempts = AttemptBudget::new(1).unwrap_or_else(|_| unreachable!("budget failed"));
    RobotSubnetPlanConfirmation::new(
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

fn ip() -> RobotSubnetAddress {
    RobotSubnetAddress::new("192.0.2.10").unwrap_or_else(|_| unreachable!("IP fixture failed"))
}

fn mac_ip() -> RobotSubnetAddress {
    RobotSubnetAddress::new("2001:db8::")
        .unwrap_or_else(|_| unreachable!("subnet MAC fixture failed"))
}

fn mac() -> RobotMacAddress {
    RobotMacAddress::new("00:21:85:62:3e:9d").unwrap_or_else(|_| unreachable!("MAC fixture failed"))
}
