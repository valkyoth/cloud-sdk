use super::dispatch_tests::{mutation_plan, time};
use super::fixture::{ClassifiedTransport, endpoint};
use crate::operation::{MutationPermit, PermitState, build_canonical_plan};

#[test]
fn unpolled_send_async_attempt_clears_complete_response_storage() {
    let Some((mut fingerprint_storage, plan)) = mutation_plan(200) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(fingerprint) = build_canonical_plan(plan, &mut fingerprint_storage) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(mut permit) = MutationPermit::new(fingerprint.subject(), time(100)) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(attempt) = permit.begin(time(101)) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(endpoint) = endpoint() else {
        unreachable!("permit security fixture construction failed");
    };
    let transport = ClassifiedTransport::new(endpoint, None);
    let mut body = [0xa5_u8; 64];
    let mut headers = [0x5a_u8; 128];

    let future = attempt.execute_async(&FixedClock, &transport, &mut body, &mut headers);
    drop(future);

    assert_eq!(transport.calls(), 0);
    assert_eq!(body, [0_u8; 64]);
    assert_eq!(headers, [0_u8; 128]);
    assert_eq!(permit.state(), PermitState::PendingReconciliation);
}

#[test]
fn unpolled_local_async_attempt_clears_complete_response_storage() {
    let Some((mut fingerprint_storage, plan)) = mutation_plan(200) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(fingerprint) = build_canonical_plan(plan, &mut fingerprint_storage) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(mut permit) = MutationPermit::new(fingerprint.subject(), time(100)) else {
        unreachable!("permit security fixture construction failed");
    };
    let Ok(attempt) = permit.begin(time(101)) else {
        unreachable!("permit security fixture construction failed");
    };
    let Some(endpoint) = endpoint() else {
        unreachable!("permit security fixture construction failed");
    };
    let transport = ClassifiedTransport::new(endpoint, None);
    let mut body = [0xa5_u8; 64];
    let mut headers = [0x5a_u8; 128];

    let future = attempt.execute_local_async(&FixedClock, &transport, &mut body, &mut headers);
    drop(future);

    assert_eq!(transport.calls(), 0);
    assert_eq!(body, [0_u8; 64]);
    assert_eq!(headers, [0_u8; 128]);
    assert_eq!(permit.state(), PermitState::PendingReconciliation);
}

struct FixedClock;

impl crate::operation::PermitClock for FixedClock {
    fn now(&self) -> crate::operation::PermitTimestamp {
        time(102)
    }
}
