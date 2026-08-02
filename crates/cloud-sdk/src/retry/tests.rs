use super::{
    FingerprintRef, IdempotencyBinding, IdempotencyIntent, MaxAttempts, MonotonicDuration,
    MonotonicInstant, RetryController, RetryDecision, RetryEvent, RetryPolicy, RetryPolicyError,
    RetryStopReason,
};
use crate::operation::{
    BodyReplayability, CostIntent, OperationImpact, OperationMetadata, RequestIdPolicy,
    RequestSemantics, RetryEligibility,
};
use crate::transport::{DeliveryPhase, StatusCode};

#[test]
fn zero_attempt_policy_is_unrepresentable() {
    assert!(MaxAttempts::new(0).is_err());
    assert_eq!(MaxAttempts::new(1).map(MaxAttempts::get), Ok(1));
}

#[test]
fn hard_attempt_and_cumulative_delay_budgets_stop_endless_transients() {
    let fingerprint = FingerprintRef::test_exact(b"request-a");
    let Some(metadata) = metadata(OperationImpact::ReadOnly) else {
        return;
    };
    let Some(policy) = policy(3, 10, 100) else {
        return;
    };
    let owner = RetryController::test_new(
        metadata,
        BodyReplayability::Replayable,
        fingerprint,
        None,
        policy,
        MonotonicInstant::new(50),
    );
    assert!(owner.is_ok());
    let Ok(mut owner) = owner else { return };
    assert_eq!(
        owner.decide_retry(
            RetryEvent::Response(StatusCode::TOO_MANY_REQUESTS),
            fingerprint,
            MonotonicDuration::new(4),
            MonotonicInstant::new(51),
        ),
        Ok(RetryDecision::Retry {
            attempt: 2,
            delay: MonotonicDuration::new(4),
        })
    );
    assert_eq!(
        owner.decide_retry(
            RetryEvent::Response(StatusCode::new(503).unwrap_or(StatusCode::TOO_MANY_REQUESTS)),
            fingerprint,
            MonotonicDuration::new(6),
            MonotonicInstant::new(52),
        ),
        Ok(RetryDecision::Retry {
            attempt: 3,
            delay: MonotonicDuration::new(6),
        })
    );
    assert_eq!(
        owner.decide_retry(
            RetryEvent::Response(StatusCode::TOO_MANY_REQUESTS),
            fingerprint,
            MonotonicDuration::new(0),
            MonotonicInstant::new(53),
        ),
        Ok(RetryDecision::Stop(RetryStopReason::AttemptsExhausted))
    );
    assert_eq!(owner.attempts(), 3);
    assert_eq!(owner.cumulative_delay().get(), 10);
}

#[test]
fn delay_elapsed_and_rollback_checks_fail_closed_without_extending_budgets() {
    let fingerprint = FingerprintRef::test_exact(b"request-a");
    let Some(delay_policy) = policy(4, 5, 100) else {
        return;
    };
    let Ok(mut delay_owner) = owner(fingerprint, delay_policy) else {
        return;
    };
    assert_eq!(
        retry(&mut delay_owner, fingerprint, 6, 1),
        Ok(RetryDecision::Stop(
            RetryStopReason::CumulativeDelayExhausted
        ))
    );
    assert_eq!(delay_owner.attempts(), 1);

    let Some(elapsed_policy) = policy(4, 50, 2) else {
        return;
    };
    let Ok(mut elapsed_owner) = owner(fingerprint, elapsed_policy) else {
        return;
    };
    assert_eq!(
        retry(&mut elapsed_owner, fingerprint, 1, 3),
        Ok(RetryDecision::Stop(RetryStopReason::ElapsedBudgetExhausted))
    );
    let rollback = elapsed_owner.decide_retry(
        RetryEvent::Transport(DeliveryPhase::NotSent),
        fingerprint,
        MonotonicDuration::new(1),
        MonotonicInstant::new(2),
    );
    assert_eq!(rollback, Err(RetryPolicyError::MonotonicRollback));
    assert_eq!(elapsed_owner.attempts(), 1);

    let Some(overflow_policy) = policy(3, u64::MAX, u64::MAX) else {
        return;
    };
    let Ok(mut overflow_owner) = owner(fingerprint, overflow_policy) else {
        return;
    };
    assert!(matches!(
        retry(&mut overflow_owner, fingerprint, u64::MAX, 0),
        Ok(RetryDecision::Retry { attempt: 2, .. })
    ));
    assert_eq!(
        retry(&mut overflow_owner, fingerprint, 1, 0),
        Err(RetryPolicyError::CumulativeDelayOverflow)
    );
    assert_eq!(overflow_owner.attempts(), 2);
}

#[test]
fn body_replayability_fingerprint_and_status_are_mandatory() {
    let fingerprint = FingerprintRef::test_exact(b"request-a");
    let different = FingerprintRef::test_exact(b"request-b");
    let Some(metadata) = metadata(OperationImpact::ReadOnly) else {
        return;
    };
    let Some(retry_policy) = policy(2, 1, 1) else {
        return;
    };
    let non_replayable = RetryController::test_new(
        metadata,
        BodyReplayability::NotReplayable,
        fingerprint,
        None,
        retry_policy,
        MonotonicInstant::new(0),
    );
    assert!(non_replayable.is_ok());
    let Ok(mut non_replayable) = non_replayable else {
        return;
    };
    assert_eq!(
        retry(&mut non_replayable, fingerprint, 0, 0),
        Ok(RetryDecision::Stop(RetryStopReason::NonReplayableBody))
    );
    assert_eq!(
        retry(&mut non_replayable, different, 0, 0),
        Err(RetryPolicyError::FingerprintMismatch)
    );

    let Ok(mut status_owner) = owner(fingerprint, retry_policy) else {
        return;
    };
    assert_eq!(
        status_owner.decide_retry(
            RetryEvent::Response(StatusCode::new(400).unwrap_or(StatusCode::OK)),
            fingerprint,
            MonotonicDuration::new(0),
            MonotonicInstant::new(0),
        ),
        Ok(RetryDecision::Stop(RetryStopReason::NonTransientResponse))
    );
}

#[test]
fn mutations_require_a_moved_fresh_intent_and_consume_delivery_phase() {
    let fingerprint = FingerprintRef::test_exact(b"mutation-a");
    let Some(metadata) = metadata(OperationImpact::Mutation) else {
        return;
    };
    let Some(retry_policy) = policy(2, 1, 1) else {
        return;
    };
    let missing = RetryController::test_new(
        metadata,
        BodyReplayability::Replayable,
        fingerprint,
        None,
        retry_policy,
        MonotonicInstant::new(0),
    );
    assert!(matches!(
        missing,
        Err(RetryPolicyError::MissingMutationIntent)
    ));

    let mut mismatched_entropy = [0x5A_u8; 32];
    let mismatched_intent = IdempotencyIntent::new(&mut mismatched_entropy);
    assert!(mismatched_intent.is_ok());
    assert!(mismatched_entropy.iter().all(|byte| *byte == 0));
    let Ok(mismatched_intent) = mismatched_intent else {
        return;
    };
    let mismatched = IdempotencyBinding::bind(
        mismatched_intent,
        FingerprintRef::test_exact(b"different-mutation"),
    );
    assert!(matches!(
        RetryController::test_new(
            metadata,
            BodyReplayability::Replayable,
            fingerprint,
            Some(mismatched),
            retry_policy,
            MonotonicInstant::new(0),
        ),
        Err(RetryPolicyError::IdempotencyFingerprintMismatch)
    ));

    let mut entropy = [0xA5_u8; 32];
    let intent = IdempotencyIntent::new(&mut entropy);
    assert!(intent.is_ok());
    assert!(entropy.iter().all(|byte| *byte == 0));
    let Ok(intent) = intent else { return };
    let binding = IdempotencyBinding::bind(intent, fingerprint);
    let owner = RetryController::test_new(
        metadata,
        BodyReplayability::Replayable,
        fingerprint,
        Some(binding),
        retry_policy,
        MonotonicInstant::new(0),
    );
    assert!(owner.is_ok());
    let Ok(mut owner) = owner else { return };
    assert_eq!(
        owner.decide_retry(
            RetryEvent::Transport(DeliveryPhase::PossiblySent),
            fingerprint,
            MonotonicDuration::new(0),
            MonotonicInstant::new(0),
        ),
        Ok(RetryDecision::Retry {
            attempt: 2,
            delay: MonotonicDuration::new(0),
        })
    );
}

#[test]
fn idempotent_mutations_consume_every_conservative_delivery_phase() {
    for phase in [
        DeliveryPhase::NotSent,
        DeliveryPhase::PossiblySent,
        DeliveryPhase::ResponseStarted,
    ] {
        let fingerprint = FingerprintRef::test_exact(b"mutation-delivery");
        let Some(metadata) = metadata(OperationImpact::Mutation) else {
            return;
        };
        let Some(retry_policy) = policy(2, 1, 1) else {
            return;
        };
        let mut entropy = [0xC3_u8; 32];
        let Ok(intent) = IdempotencyIntent::new(&mut entropy) else {
            return;
        };
        let binding = IdempotencyBinding::bind(intent, fingerprint);
        let owner = RetryController::test_new(
            metadata,
            BodyReplayability::Replayable,
            fingerprint,
            Some(binding),
            retry_policy,
            MonotonicInstant::new(0),
        );
        let Ok(mut owner) = owner else { return };
        assert_eq!(
            owner.decide_retry(
                RetryEvent::Transport(phase),
                fingerprint,
                MonotonicDuration::new(0),
                MonotonicInstant::new(0),
            ),
            Ok(RetryDecision::Retry {
                attempt: 2,
                delay: MonotonicDuration::new(0),
            })
        );
    }
}

#[test]
fn intent_shape_rejects_empty_short_oversized_and_zero_values() {
    let mut empty = [];
    let mut short = [1_u8; 15];
    let mut oversized = [1_u8; 65];
    let mut zero = [0_u8; 16];
    let mut minimum = [1_u8; 16];
    assert!(IdempotencyIntent::new(&mut empty).is_err());
    assert!(IdempotencyIntent::new(&mut short).is_err());
    assert!(IdempotencyIntent::new(&mut oversized).is_err());
    assert!(IdempotencyIntent::new(&mut zero).is_err());
    assert_eq!(
        IdempotencyIntent::new(&mut minimum).map(|value| value.len()),
        Ok(16)
    );
    assert!(short.iter().all(|byte| *byte == 0));
    assert!(oversized.iter().all(|byte| *byte == 0));
    assert!(zero.iter().all(|byte| *byte == 0));
    assert!(minimum.iter().all(|byte| *byte == 0));
    assert!(IdempotencyIntent::new(&mut minimum).is_err());
}

fn owner<'a>(
    fingerprint: FingerprintRef<'a>,
    policy: RetryPolicy,
) -> Result<RetryController<'a>, RetryPolicyError> {
    let Some(metadata) = metadata(OperationImpact::ReadOnly) else {
        return Err(RetryPolicyError::FingerprintMismatch);
    };
    RetryController::test_new(
        metadata,
        BodyReplayability::Replayable,
        fingerprint,
        None,
        policy,
        MonotonicInstant::new(0),
    )
}

fn retry(
    owner: &mut RetryController<'_>,
    fingerprint: FingerprintRef<'_>,
    delay: u64,
    now: u64,
) -> Result<RetryDecision, RetryPolicyError> {
    owner.decide_retry(
        RetryEvent::Transport(DeliveryPhase::NotSent),
        fingerprint,
        MonotonicDuration::new(delay),
        MonotonicInstant::new(now),
    )
}

fn policy(attempts: u16, delay: u64, elapsed: u64) -> Option<RetryPolicy> {
    Some(RetryPolicy::new(
        MaxAttempts::new(attempts).ok()?,
        MonotonicDuration::new(delay),
        MonotonicDuration::new(elapsed),
    ))
}

fn metadata(impact: OperationImpact) -> Option<OperationMetadata> {
    let semantics = if impact == OperationImpact::ReadOnly {
        RequestSemantics::Safe
    } else {
        RequestSemantics::Idempotent
    };
    OperationMetadata::new(
        impact,
        semantics,
        RetryEligibility::ExplicitPolicy,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .ok()
}
