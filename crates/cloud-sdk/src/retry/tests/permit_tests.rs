use super::fixture::{LocalRecordingTransport, RecordingTransport, endpoint};
use super::{
    FingerprintScope, MonotonicDuration, MonotonicInstant, RetryController, RetryDecision,
    RetryEvent, RetryExecutionError, RetryPermitError, build_canonical_fingerprint, policy,
    read_only,
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

    let transport = RecordingTransport::new(endpoint);
    let mut response = [0xA5_u8; 64];
    let mut headers = [0xA5_u8; 8192];
    assert!(matches!(
        permit.execute_blocking(
            MonotonicInstant::new(4),
            &transport,
            &mut response,
            &mut headers,
        ),
        Err(RetryExecutionError::Permit(
            RetryPermitError::MonotonicRollback
        ))
    ));
    assert_eq!(response, [0_u8; 64]);
    assert_eq!(headers, [0_u8; 8192]);
}

#[test]
fn permit_execution_advances_the_controller_clock() {
    let Some(prepared) = read_only("/clock") else {
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let mut storage = [0_u8; 512];
    let Ok(fingerprint) =
        build_canonical_fingerprint(prepared, endpoint, FingerprintScope::Absent, &mut storage)
    else {
        return;
    };
    let Some(policy) = policy(3, 10, 20) else {
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
        MonotonicDuration::new(1),
        MonotonicInstant::new(6),
    ) else {
        return;
    };
    let transport = RecordingTransport::new(endpoint);
    let mut response = [0_u8; 64];
    let mut headers = [0_u8; 8192];
    assert!(
        permit
            .execute_blocking(
                MonotonicInstant::new(7),
                &transport,
                &mut response,
                &mut headers,
            )
            .is_ok()
    );
    assert_eq!(transport.calls(), 1);
    assert!(matches!(
        controller.decide_retry(
            RetryEvent::Transport(DeliveryPhase::NotSent),
            fingerprint.subject(),
            MonotonicDuration::new(0),
            MonotonicInstant::new(6),
        ),
        Err(super::RetryPolicyError::MonotonicRollback)
    ));
}

#[test]
fn async_permit_executes_the_bound_request_once() {
    let Some(prepared) = read_only("/async") else {
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let mut storage = [0_u8; 512];
    let Ok(fingerprint) =
        build_canonical_fingerprint(prepared, endpoint, FingerprintScope::Absent, &mut storage)
    else {
        return;
    };
    let Some(policy) = policy(2, 1, 20) else {
        return;
    };
    let Ok(mut controller) = RetryController::new(
        fingerprint.subject(),
        None,
        policy,
        MonotonicInstant::new(0),
    ) else {
        return;
    };
    let Ok(RetryDecision::Retry(permit)) = controller.decide_retry(
        RetryEvent::Transport(DeliveryPhase::NotSent),
        fingerprint.subject(),
        MonotonicDuration::new(1),
        MonotonicInstant::new(1),
    ) else {
        return;
    };
    let transport = RecordingTransport::new(endpoint);
    let mut response = [0_u8; 64];
    let mut headers = [0_u8; 8192];
    let future = permit.execute_async(
        MonotonicInstant::new(2),
        &transport,
        &mut response,
        &mut headers,
    );
    let mut future = core::pin::pin!(future);
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    assert!(matches!(
        Future::poll(future.as_mut(), &mut context),
        Poll::Ready(Ok(_))
    ));
    assert_eq!(transport.calls(), 1);
}

#[test]
fn local_async_permit_executes_the_bound_request_once() {
    let Some(prepared) = read_only("/local-async") else {
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let mut storage = [0_u8; 512];
    let Ok(fingerprint) =
        build_canonical_fingerprint(prepared, endpoint, FingerprintScope::Absent, &mut storage)
    else {
        return;
    };
    let Some(policy) = policy(2, 1, 20) else {
        return;
    };
    let Ok(mut controller) = RetryController::new(
        fingerprint.subject(),
        None,
        policy,
        MonotonicInstant::new(0),
    ) else {
        return;
    };
    let Ok(RetryDecision::Retry(permit)) = controller.decide_retry(
        RetryEvent::Transport(DeliveryPhase::NotSent),
        fingerprint.subject(),
        MonotonicDuration::new(1),
        MonotonicInstant::new(1),
    ) else {
        return;
    };
    let transport = LocalRecordingTransport::new(endpoint);
    let mut response = [0_u8; 64];
    let mut headers = [0_u8; 8192];
    let future = permit.execute_local_async(
        MonotonicInstant::new(2),
        &transport,
        &mut response,
        &mut headers,
    );
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    assert!(matches!(
        Future::poll(future.as_mut(), &mut context),
        Poll::Ready(Ok(_))
    ));
    assert_eq!(transport.calls(), 1);
}
use core::future::Future;
use core::task::{Context, Poll, Waker};
