use super::fixture::endpoint;
use super::{
    FingerprintScope, MonotonicDuration, MonotonicInstant, RetryController, RetryDecision,
    RetryEvent, RetryPermitError, build_canonical_fingerprint, policy, read_only,
};
use crate::transport::DeliveryPhase;

#[test]
fn permit_rejects_an_observation_before_the_controller_start() {
    let Some(prepared) = read_only("/rollback") else {
        return;
    };
    let mut storage = [0_u8; 512];
    let Some(endpoint) = endpoint() else { return };
    let Ok(fingerprint) =
        build_canonical_fingerprint(prepared, endpoint, FingerprintScope::Absent, &mut storage)
    else {
        return;
    };
    let Some(policy) = policy(2, 10, 20) else {
        return;
    };
    let Ok(mut controller) = RetryController::new(
        fingerprint.subject(),
        None,
        policy,
        MonotonicInstant::new(5),
    ) else {
        return;
    };
    let Ok(RetryDecision::Retry(permit)) = controller.decide_retry(
        RetryEvent::Transport(DeliveryPhase::NotSent),
        fingerprint.subject(),
        MonotonicDuration::new(2),
        MonotonicInstant::new(6),
    ) else {
        return;
    };

    assert!(matches!(
        permit.authorize_execution(MonotonicInstant::new(4)),
        Err(RetryPermitError::MonotonicRollback)
    ));
}
