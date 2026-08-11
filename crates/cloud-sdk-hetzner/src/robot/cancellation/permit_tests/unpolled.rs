use cloud_sdk::operation::{PermitState, PermitTimestamp, PreparationStorage};
use cloud_sdk_testkit::MockTransport;

use super::*;
use crate::association::Sha256PlanHasher;

#[test]
fn unpolled_async_attempt_clears_buffers_and_consumes_authority() {
    let request =
        RobotIpCancellationCreateRequest::new(ip(), RobotCancellationSchedule::On(date()));
    let mut target = [0xa5_u8; 128];
    let mut request_body = [0x5a_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("IP cancellation preparation failed"));
    let endpoint = endpoint();
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    let fingerprint = build_cancellation_plan_digest(
        plan(prepared, endpoint),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("IP cancellation digest failed"));
    let mut permit = CancellationDestructivePermit::new(
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("IP cancellation permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("IP cancellation attempt failed"));
    let exchanges = [];
    let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [0xa5_u8; 512];
    let mut response_headers = [0x5a_u8; 128];

    let future = attempt.execute_async(
        &FixedClock,
        &transport,
        &mut response_body,
        &mut response_headers,
    );
    drop(future);

    assert_eq!(response_body, [0_u8; 512]);
    assert_eq!(response_headers, [0_u8; 128]);
    assert_eq!(permit.state(), PermitState::PendingReconciliation);
    assert!(transport.is_complete());
}
