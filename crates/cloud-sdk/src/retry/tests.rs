use super::{
    FingerprintScope, IdempotencyBinding, IdempotencyIntent, MaxAttempts, MonotonicDuration,
    MonotonicInstant, RetryController, RetryDecision, RetryEvent, RetryPermit, RetryPermitError,
    RetryPolicy, RetryPolicyError, RetryStopReason, build_canonical_fingerprint,
};
use crate::operation::{BodyReplayability, OperationImpact, RequestSemantics, RetryEligibility};
use crate::transport::{DeliveryPhase, StatusCode};

mod fixture;
mod permit_tests;
use fixture::{endpoint, prepared};

#[test]
fn zero_attempt_policy_is_unrepresentable() {
    assert!(MaxAttempts::new(0).is_err());
    assert_eq!(MaxAttempts::new(1).map(MaxAttempts::get), Ok(1));
    assert!(core::mem::size_of::<RetryPermit<'static, 'static>>() <= 128);
}

#[test]
fn hard_attempt_and_cumulative_delay_budgets_stop_endless_transients() {
    let Some(prepared) = read_only("/servers") else {
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let mut storage = [0_u8; 512];
    let Ok(fingerprint) =
        build_canonical_fingerprint(prepared, endpoint, FingerprintScope::Absent, &mut storage)
    else {
        return;
    };
    let Some(policy) = policy(3, 10, 100) else {
        return;
    };
    let Ok(mut owner) = RetryController::new(
        fingerprint.subject(),
        None,
        policy,
        MonotonicInstant::new(50),
    ) else {
        return;
    };

    let first = owner.decide_retry(
        RetryEvent::Response(StatusCode::TOO_MANY_REQUESTS),
        fingerprint.subject(),
        MonotonicDuration::new(4),
        MonotonicInstant::new(51),
    );
    assert!(matches!(&first, Ok(RetryDecision::Retry(_))));
    let Ok(RetryDecision::Retry(first)) = first else {
        return;
    };
    assert_eq!(first.attempt(), 2);
    assert_eq!(first.delay().get(), 4);
    let Ok(authorized) = first.authorize_execution(MonotonicInstant::new(55)) else {
        return;
    };
    assert_eq!(
        authorized.transport_request().target().path().as_str(),
        "/servers"
    );

    let second = owner.decide_retry(
        RetryEvent::Response(StatusCode::new(503).unwrap_or(StatusCode::TOO_MANY_REQUESTS)),
        fingerprint.subject(),
        MonotonicDuration::new(6),
        MonotonicInstant::new(52),
    );
    assert!(matches!(&second, Ok(RetryDecision::Retry(_))));
    let Ok(RetryDecision::Retry(second)) = second else {
        return;
    };
    assert_eq!(second.attempt(), 3);
    assert_eq!(second.delay().get(), 6);

    assert!(matches!(
        owner.decide_retry(
            RetryEvent::Response(StatusCode::TOO_MANY_REQUESTS),
            fingerprint.subject(),
            MonotonicDuration::new(0),
            MonotonicInstant::new(53),
        ),
        Ok(RetryDecision::Stop(RetryStopReason::AttemptsExhausted))
    ));
    assert_eq!(owner.attempts(), 3);
    assert_eq!(owner.cumulative_delay().get(), 10);
}

#[test]
fn projected_and_post_sleep_elapsed_budgets_fail_closed() {
    let Some(prepared) = read_only("/servers") else {
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let mut storage = [0_u8; 512];
    let Ok(fingerprint) =
        build_canonical_fingerprint(prepared, endpoint, FingerprintScope::Absent, &mut storage)
    else {
        return;
    };
    let Some(policy) = policy(4, 50, 10) else {
        return;
    };
    let Ok(mut owner) = RetryController::new(
        fingerprint.subject(),
        None,
        policy,
        MonotonicInstant::new(0),
    ) else {
        return;
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
        return;
    };
    let permit = owner.decide_retry(
        RetryEvent::Transport(DeliveryPhase::NotSent),
        fingerprint.subject(),
        MonotonicDuration::new(1),
        MonotonicInstant::new(8),
    );
    assert!(matches!(&permit, Ok(RetryDecision::Retry(_))));
    let Ok(RetryDecision::Retry(permit)) = permit else {
        return;
    };
    assert!(matches!(
        permit.authorize_execution(MonotonicInstant::new(11)),
        Err(RetryPermitError::ElapsedBudgetExhausted)
    ));

    let Ok(mut owner) = RetryController::new(
        fingerprint.subject(),
        None,
        policy,
        MonotonicInstant::new(0),
    ) else {
        return;
    };
    let permit = owner.decide_retry(
        RetryEvent::Transport(DeliveryPhase::NotSent),
        fingerprint.subject(),
        MonotonicDuration::new(1),
        MonotonicInstant::new(8),
    );
    let Ok(RetryDecision::Retry(permit)) = permit else {
        return;
    };
    assert!(matches!(
        permit.authorize_execution(MonotonicInstant::new(8)),
        Err(RetryPermitError::TooEarly)
    ));
}

#[test]
fn retry_subject_prevents_unrelated_request_policy_borrowing() {
    let Some(safe) = read_only("/safe") else {
        return;
    };
    let Some(destructive) = prepared(
        "/destructive",
        OperationImpact::Destructive,
        RequestSemantics::NonIdempotent,
        RetryEligibility::Never,
        BodyReplayability::Replayable,
    ) else {
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let mut safe_storage = [0_u8; 512];
    let mut destructive_storage = [0_u8; 512];
    let Ok(safe_fingerprint) =
        build_canonical_fingerprint(safe, endpoint, FingerprintScope::Absent, &mut safe_storage)
    else {
        return;
    };
    let Ok(destructive_fingerprint) = build_canonical_fingerprint(
        destructive,
        endpoint,
        FingerprintScope::Absent,
        &mut destructive_storage,
    ) else {
        return;
    };
    let Some(policy) = policy(2, 0, 10) else {
        return;
    };
    let mut entropy = [0xA5_u8; 32];
    {
        let Ok(intent) = IdempotencyIntent::new(&mut entropy) else {
            return;
        };
        let binding = IdempotencyBinding::bind(intent, destructive_fingerprint.as_ref());
        let Ok(mut owner) = RetryController::new(
            destructive_fingerprint.subject(),
            Some(binding),
            policy,
            MonotonicInstant::new(0),
        ) else {
            return;
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
        return;
    };
    let Some(different) = read_only("/other") else {
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let mut first_storage = [0_u8; 512];
    let mut second_storage = [0_u8; 512];
    let Ok(first) = build_canonical_fingerprint(
        non_replayable,
        endpoint,
        FingerprintScope::Absent,
        &mut first_storage,
    ) else {
        return;
    };
    let Ok(second) = build_canonical_fingerprint(
        different,
        endpoint,
        FingerprintScope::Absent,
        &mut second_storage,
    ) else {
        return;
    };
    let Some(policy) = policy(2, 1, 5) else {
        return;
    };
    let Ok(mut owner) =
        RetryController::new(first.subject(), None, policy, MonotonicInstant::new(0))
    else {
        return;
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
        return;
    };
    let mut status_storage = [0_u8; 512];
    let Ok(status_fingerprint) = build_canonical_fingerprint(
        replayable,
        endpoint,
        FingerprintScope::Absent,
        &mut status_storage,
    ) else {
        return;
    };
    let Ok(mut status_owner) = RetryController::new(
        status_fingerprint.subject(),
        None,
        policy,
        MonotonicInstant::new(0),
    ) else {
        return;
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
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let mut storage = [0_u8; 512];
    let Ok(fingerprint) =
        build_canonical_fingerprint(mutation, endpoint, FingerprintScope::Absent, &mut storage)
    else {
        return;
    };
    let Some(policy) = policy(2, 0, 2) else {
        return;
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
                return;
            };
            let binding = IdempotencyBinding::bind(intent, fingerprint.as_ref());
            let Ok(mut owner) = RetryController::new(
                fingerprint.subject(),
                Some(binding),
                policy,
                MonotonicInstant::new(0),
            ) else {
                return;
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
        return;
    };
    let Some(endpoint) = endpoint() else { return };
    let mut storage = [0_u8; 512];
    let Ok(fingerprint) =
        build_canonical_fingerprint(prepared, endpoint, FingerprintScope::Absent, &mut storage)
    else {
        return;
    };
    let Some(overflow_policy) = policy(3, u64::MAX, u64::MAX) else {
        return;
    };
    let Ok(mut owner) = RetryController::new(
        fingerprint.subject(),
        None,
        overflow_policy,
        MonotonicInstant::new(0),
    ) else {
        return;
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
        return;
    };
    let Ok(mut owner) = RetryController::new(
        fingerprint.subject(),
        None,
        rollback_policy,
        MonotonicInstant::new(1),
    ) else {
        return;
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

#[test]
fn intent_shape_rejects_and_clears_invalid_values() {
    let mut empty = [];
    let mut short = [1_u8; 15];
    let mut oversized = [1_u8; 65];
    let mut zero = [0_u8; 16];
    assert!(IdempotencyIntent::new(&mut empty).is_err());
    assert!(IdempotencyIntent::new(&mut short).is_err());
    assert!(IdempotencyIntent::new(&mut oversized).is_err());
    assert!(IdempotencyIntent::new(&mut zero).is_err());
    assert!(short.iter().all(|byte| *byte == 0));
    assert!(oversized.iter().all(|byte| *byte == 0));
    assert!(zero.iter().all(|byte| *byte == 0));
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
