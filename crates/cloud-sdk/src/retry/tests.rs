use super::{
    FingerprintScope, IdempotencyBinding, IdempotencyIntent, MaxAttempts, MonotonicDuration,
    MonotonicInstant, RetryController, RetryDecision, RetryEvent, RetryExecutionError,
    RetryPermitError, RetryPolicy, RetryPolicyError, RetryStopReason, build_canonical_fingerprint,
};
use crate::operation::{BodyReplayability, OperationImpact, RequestSemantics, RetryEligibility};
use crate::transport::{DeliveryPhase, StatusCode};

mod basic_tests;
mod fail_closed_assurance;
mod fixture;
mod permit_tests;
mod policy_identity_tests;
use fixture::{RecordingTransport, prepared, required_endpoint};

#[test]
fn hard_attempt_and_cumulative_delay_budgets_stop_endless_transients() {
    let Some(prepared) = read_only("/servers") else {
        unreachable!("retry security fixture construction failed");
    };
    let endpoint = required_endpoint();
    let mut storage = [0_u8; 512];
    let Ok(fingerprint) =
        build_canonical_fingerprint(prepared, endpoint, FingerprintScope::Absent, &mut storage)
    else {
        unreachable!("retry security fixture construction failed");
    };
    let Some(policy) = policy(3, 10, 100) else {
        unreachable!("retry security fixture construction failed");
    };
    let Ok(mut owner) = RetryController::new(
        fingerprint.subject(),
        None,
        policy,
        MonotonicInstant::new(50),
    ) else {
        unreachable!("retry security fixture construction failed");
    };

    let first = owner.decide_retry(
        RetryEvent::Response(StatusCode::TOO_MANY_REQUESTS),
        fingerprint.subject(),
        MonotonicDuration::new(4),
        MonotonicInstant::new(51),
    );
    assert!(matches!(&first, Ok(RetryDecision::Retry(_))));
    let Ok(RetryDecision::Retry(first)) = first else {
        unreachable!("retry security fixture construction failed");
    };
    assert_eq!(first.attempt(), 2);
    assert_eq!(first.delay().get(), 4);
    let transport = RecordingTransport::new(endpoint);
    let mut response = [0_u8; 64];
    let mut headers = [0_u8; 8192];
    assert!(
        first
            .execute_blocking(
                MonotonicInstant::new(55),
                &transport,
                &mut response,
                &mut headers,
            )
            .is_ok()
    );
    assert_eq!(transport.calls(), 1);

    let second = owner.decide_retry(
        RetryEvent::Response(StatusCode::new(503).unwrap_or(StatusCode::TOO_MANY_REQUESTS)),
        fingerprint.subject(),
        MonotonicDuration::new(6),
        MonotonicInstant::new(56),
    );
    assert!(matches!(&second, Ok(RetryDecision::Retry(_))));
    let Ok(RetryDecision::Retry(second)) = second else {
        unreachable!("retry security fixture construction failed");
    };
    assert_eq!(second.attempt(), 3);
    assert_eq!(second.delay().get(), 6);
    assert!(
        second
            .execute_blocking(
                MonotonicInstant::new(62),
                &transport,
                &mut response,
                &mut headers,
            )
            .is_ok()
    );

    assert!(matches!(
        owner.decide_retry(
            RetryEvent::Response(StatusCode::TOO_MANY_REQUESTS),
            fingerprint.subject(),
            MonotonicDuration::new(0),
            MonotonicInstant::new(63),
        ),
        Ok(RetryDecision::Stop(RetryStopReason::AttemptsExhausted))
    ));
    assert_eq!(owner.attempts(), 3);
    assert_eq!(owner.cumulative_delay().get(), 10);
}

#[test]
fn projected_and_post_sleep_elapsed_budgets_fail_closed() {
    let Some(prepared) = read_only("/servers") else {
        unreachable!("retry security fixture construction failed");
    };
    let endpoint = required_endpoint();
    let mut storage = [0_u8; 512];
    let Ok(fingerprint) =
        build_canonical_fingerprint(prepared, endpoint, FingerprintScope::Absent, &mut storage)
    else {
        unreachable!("retry security fixture construction failed");
    };
    let Some(policy) = policy(4, 50, 10) else {
        unreachable!("retry security fixture construction failed");
    };
    let Ok(mut owner) = RetryController::new(
        fingerprint.subject(),
        None,
        policy,
        MonotonicInstant::new(0),
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    assert!(matches!(
        owner.decide_retry(
            RetryEvent::Transport(DeliveryPhase::NotSent),
            fingerprint.subject(),
            MonotonicDuration::new(2),
            MonotonicInstant::new(9),
        ),
        Ok(RetryDecision::Stop(RetryStopReason::ElapsedBudgetExhausted))
    ));

    let Ok(mut owner) = RetryController::new(
        fingerprint.subject(),
        None,
        policy,
        MonotonicInstant::new(0),
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    let permit = owner.decide_retry(
        RetryEvent::Transport(DeliveryPhase::NotSent),
        fingerprint.subject(),
        MonotonicDuration::new(1),
        MonotonicInstant::new(8),
    );
    assert!(matches!(&permit, Ok(RetryDecision::Retry(_))));
    let Ok(RetryDecision::Retry(permit)) = permit else {
        unreachable!("retry security fixture construction failed");
    };
    let transport = RecordingTransport::new(endpoint);
    let mut response = [0_u8; 64];
    let mut headers = [0_u8; 8192];
    assert!(matches!(
        permit.execute_blocking(
            MonotonicInstant::new(11),
            &transport,
            &mut response,
            &mut headers,
        ),
        Err(RetryExecutionError::Permit(
            RetryPermitError::ElapsedBudgetExhausted
        ))
    ));

    let Ok(mut owner) = RetryController::new(
        fingerprint.subject(),
        None,
        policy,
        MonotonicInstant::new(0),
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    let permit = owner.decide_retry(
        RetryEvent::Transport(DeliveryPhase::NotSent),
        fingerprint.subject(),
        MonotonicDuration::new(1),
        MonotonicInstant::new(8),
    );
    let Ok(RetryDecision::Retry(permit)) = permit else {
        unreachable!("retry security fixture construction failed");
    };
    assert!(matches!(
        permit.execute_blocking(
            MonotonicInstant::new(8),
            &transport,
            &mut response,
            &mut headers,
        ),
        Err(RetryExecutionError::Permit(RetryPermitError::TooEarly))
    ));
}

#[test]
fn retry_subject_prevents_unrelated_request_policy_borrowing() {
    let Some(safe) = read_only("/safe") else {
        unreachable!("retry security fixture construction failed");
    };
    let Some(destructive) = prepared(
        "/destructive",
        OperationImpact::Destructive,
        RequestSemantics::NonIdempotent,
        RetryEligibility::Never,
        BodyReplayability::Replayable,
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    let endpoint = required_endpoint();
    let mut safe_storage = [0_u8; 512];
    let mut destructive_storage = [0_u8; 512];
    let Ok(safe_fingerprint) =
        build_canonical_fingerprint(safe, endpoint, FingerprintScope::Absent, &mut safe_storage)
    else {
        unreachable!("retry security fixture construction failed");
    };
    let Ok(destructive_fingerprint) = build_canonical_fingerprint(
        destructive,
        endpoint,
        FingerprintScope::Absent,
        &mut destructive_storage,
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    let Some(policy) = policy(2, 0, 10) else {
        unreachable!("retry security fixture construction failed");
    };
    let mut entropy = [0xA5_u8; 32];
    {
        let Ok(intent) = IdempotencyIntent::new(&mut entropy) else {
            unreachable!("retry security fixture construction failed");
        };
        let binding = IdempotencyBinding::bind(intent, destructive_fingerprint.as_ref());
        let Ok(mut owner) = RetryController::new(
            destructive_fingerprint.subject(),
            Some(binding),
            policy,
            MonotonicInstant::new(0),
        ) else {
            unreachable!("retry security fixture construction failed");
        };
        assert!(matches!(
            owner.decide_retry(
                RetryEvent::Response(StatusCode::new(500).unwrap_or(StatusCode::OK)),
                destructive_fingerprint.subject(),
                MonotonicDuration::new(0),
                MonotonicInstant::new(0),
            ),
            Ok(RetryDecision::Stop(RetryStopReason::IneligibleOperation))
        ));
        assert!(matches!(
            owner.decide_retry(
                RetryEvent::Response(StatusCode::new(500).unwrap_or(StatusCode::OK)),
                safe_fingerprint.subject(),
                MonotonicDuration::new(0),
                MonotonicInstant::new(0),
            ),
            Err(RetryPolicyError::FingerprintMismatch)
        ));
    }
    assert!(entropy.iter().all(|byte| *byte == 0));
}

#[test]
fn replayability_fingerprint_status_and_delay_are_mandatory() {
    let Some(non_replayable) = prepared(
        "/servers",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        BodyReplayability::NotReplayable,
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    let Some(different) = read_only("/other") else {
        unreachable!("retry security fixture construction failed");
    };
    let endpoint = required_endpoint();
    let mut first_storage = [0_u8; 512];
    let mut second_storage = [0_u8; 512];
    let Ok(first) = build_canonical_fingerprint(
        non_replayable,
        endpoint,
        FingerprintScope::Absent,
        &mut first_storage,
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    let Ok(second) = build_canonical_fingerprint(
        different,
        endpoint,
        FingerprintScope::Absent,
        &mut second_storage,
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    let Some(policy) = policy(2, 1, 5) else {
        unreachable!("retry security fixture construction failed");
    };
    let Ok(mut owner) =
        RetryController::new(first.subject(), None, policy, MonotonicInstant::new(0))
    else {
        unreachable!("retry security fixture construction failed");
    };
    assert!(matches!(
        owner.decide_retry(
            RetryEvent::Transport(DeliveryPhase::NotSent),
            first.subject(),
            MonotonicDuration::new(0),
            MonotonicInstant::new(0),
        ),
        Ok(RetryDecision::Stop(RetryStopReason::NonReplayableBody))
    ));
    assert!(matches!(
        owner.decide_retry(
            RetryEvent::Transport(DeliveryPhase::NotSent),
            second.subject(),
            MonotonicDuration::new(0),
            MonotonicInstant::new(0),
        ),
        Err(RetryPolicyError::FingerprintMismatch)
    ));

    let Some(replayable) = read_only("/status") else {
        unreachable!("retry security fixture construction failed");
    };
    let mut status_storage = [0_u8; 512];
    let Ok(status_fingerprint) = build_canonical_fingerprint(
        replayable,
        endpoint,
        FingerprintScope::Absent,
        &mut status_storage,
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    let Ok(mut status_owner) = RetryController::new(
        status_fingerprint.subject(),
        None,
        policy,
        MonotonicInstant::new(0),
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    assert!(matches!(
        status_owner.decide_retry(
            RetryEvent::Response(StatusCode::new(400).unwrap_or(StatusCode::OK)),
            status_fingerprint.subject(),
            MonotonicDuration::new(0),
            MonotonicInstant::new(0),
        ),
        Ok(RetryDecision::Stop(RetryStopReason::NonTransientResponse))
    ));
}

#[test]
fn mutation_intent_is_borrowed_one_use_and_cleared_on_drop() {
    let Some(mutation) = prepared(
        "/mutation",
        OperationImpact::Mutation,
        RequestSemantics::Idempotent,
        RetryEligibility::ExplicitPolicy,
        BodyReplayability::Replayable,
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    let endpoint = required_endpoint();
    let mut storage = [0_u8; 512];
    let Ok(fingerprint) =
        build_canonical_fingerprint(mutation, endpoint, FingerprintScope::Absent, &mut storage)
    else {
        unreachable!("retry security fixture construction failed");
    };
    let Some(policy) = policy(2, 0, 2) else {
        unreachable!("retry security fixture construction failed");
    };
    assert!(matches!(
        RetryController::new(
            fingerprint.subject(),
            None,
            policy,
            MonotonicInstant::new(0),
        ),
        Err(RetryPolicyError::MissingMutationIntent)
    ));

    for phase in [
        DeliveryPhase::NotSent,
        DeliveryPhase::PossiblySent,
        DeliveryPhase::ResponseStarted,
    ] {
        let mut entropy = [0xC3_u8; 32];
        {
            let Ok(intent) = IdempotencyIntent::new(&mut entropy) else {
                unreachable!("retry security fixture construction failed");
            };
            let binding = IdempotencyBinding::bind(intent, fingerprint.as_ref());
            let Ok(mut owner) = RetryController::new(
                fingerprint.subject(),
                Some(binding),
                policy,
                MonotonicInstant::new(0),
            ) else {
                unreachable!("retry security fixture construction failed");
            };
            let decision = owner.decide_retry(
                RetryEvent::Transport(phase),
                fingerprint.subject(),
                MonotonicDuration::new(0),
                MonotonicInstant::new(0),
            );
            assert!(matches!(decision, Ok(RetryDecision::Retry(_))));
        }
        assert!(entropy.iter().all(|byte| *byte == 0));
    }
}

#[test]
fn rollback_and_arithmetic_overflow_never_extend_budgets() {
    let Some(prepared) = read_only("/budgets") else {
        unreachable!("retry security fixture construction failed");
    };
    let endpoint = required_endpoint();
    let mut storage = [0_u8; 512];
    let Ok(fingerprint) =
        build_canonical_fingerprint(prepared, endpoint, FingerprintScope::Absent, &mut storage)
    else {
        unreachable!("retry security fixture construction failed");
    };
    let Some(overflow_policy) = policy(3, u64::MAX, u64::MAX) else {
        unreachable!("retry security fixture construction failed");
    };
    let Ok(mut owner) = RetryController::new(
        fingerprint.subject(),
        None,
        overflow_policy,
        MonotonicInstant::new(0),
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    assert!(matches!(
        owner.decide_retry(
            RetryEvent::Transport(DeliveryPhase::NotSent),
            fingerprint.subject(),
            MonotonicDuration::new(u64::MAX),
            MonotonicInstant::new(0),
        ),
        Ok(RetryDecision::Retry(_))
    ));
    assert!(matches!(
        owner.decide_retry(
            RetryEvent::Transport(DeliveryPhase::NotSent),
            fingerprint.subject(),
            MonotonicDuration::new(1),
            MonotonicInstant::new(0),
        ),
        Err(RetryPolicyError::CumulativeDelayOverflow)
    ));

    let Some(rollback_policy) = policy(2, 1, 10) else {
        unreachable!("retry security fixture construction failed");
    };
    let Ok(mut owner) = RetryController::new(
        fingerprint.subject(),
        None,
        rollback_policy,
        MonotonicInstant::new(1),
    ) else {
        unreachable!("retry security fixture construction failed");
    };
    assert!(matches!(
        owner.decide_retry(
            RetryEvent::Transport(DeliveryPhase::NotSent),
            fingerprint.subject(),
            MonotonicDuration::new(0),
            MonotonicInstant::new(0),
        ),
        Err(RetryPolicyError::MonotonicRollback)
    ));
}

fn read_only(target: &'static str) -> Option<crate::operation::PreparedRequest<'static>> {
    prepared(
        target,
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        BodyReplayability::Replayable,
    )
}

fn policy(attempts: u16, delay: u64, elapsed: u64) -> Option<RetryPolicy> {
    Some(RetryPolicy::new(
        MaxAttempts::new(attempts).ok()?,
        MonotonicDuration::new(delay),
        MonotonicDuration::new(elapsed),
    ))
}
