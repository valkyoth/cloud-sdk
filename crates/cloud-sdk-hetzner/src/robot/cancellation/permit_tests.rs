use cloud_sdk::operation::{
    AttemptBudget, PermitClock, PermitContext, PermitTimestamp, PermitValidity, PlanChange,
    PlanFingerprintScope, PreparationStorage, ReplayPolicy,
};
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, MockExchange, MockTransport, ResponseFixture,
};

use super::*;
use crate::endpoint::official_robot_endpoint_identity;
use crate::robot::server::RobotServerNumber;

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

#[test]
fn destructive_permit_execution_returns_the_exact_request_bound_response() {
    let number = RobotServerNumber::new(321)
        .unwrap_or_else(|_| unreachable!("server number fixture failed"));
    let request = RobotServerCancellationDeleteRequest::new(number);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("cancellation preparation failed"));
    let untyped = prepared.as_untyped();
    let transport_request = untyped.transport_request();
    let expected = ExpectedRequest::new(transport_request.method(), transport_request.target())
        .with_body(transport_request.body())
        .with_headers(transport_request.headers());
    let endpoint = official_robot_endpoint_identity()
        .unwrap_or_else(|_| unreachable!("official Robot endpoint failed"));
    let context = PermitContext::new(b"v0.79 Robot cancellation provenance fixture")
        .unwrap_or_else(|_| unreachable!("permit context failed"));
    let validity = PermitValidity::new(
        PermitTimestamp::from_seconds(100),
        PermitTimestamp::from_seconds(200),
    )
    .unwrap_or_else(|_| unreachable!("permit validity failed"));
    let attempts = AttemptBudget::new(1).unwrap_or_else(|_| unreachable!("attempt budget failed"));
    let plan = CancellationPlanConfirmation::new(
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
    );
    let mut fingerprint_storage = [0_u8; 4_096];
    let fingerprint = build_cancellation_canonical_plan(plan, &mut fingerprint_storage)
        .unwrap_or_else(|_| unreachable!("cancellation fingerprint failed"));
    let mut permit = CancellationDestructivePermit::new(
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("cancellation permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("cancellation attempt failed"));
    let empty =
        FixtureBody::new(&[]).unwrap_or_else(|_| unreachable!("empty response fixture failed"));
    let exchanges = [MockExchange::new(expected, ResponseFixture::success(empty))];
    let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [];
    let mut response_headers = [0_u8; 1];
    let checked = attempt
        .execute_blocking(
            &FixedClock,
            &transport,
            &mut response_body,
            &mut response_headers,
        )
        .unwrap_or_else(|_| unreachable!("cancellation execution failed"));

    assert!(checked.decode_response().is_ok());
    assert!(transport.is_complete());
}
