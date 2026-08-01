use crate::rate_limit::{
    DelayConflictPolicy, DelayDecisionError, DelaySeconds, DelaySource, ExcessDelayPolicy,
    PastTimestampPolicy, QuotaBucket, QuotaBucketId, QuotaBuckets, QuotaDelayPolicy, QuotaReset,
    RetryAfter, WallClockTimestamp, decide_delay,
};

fn policy(conflict: DelayConflictPolicy, excess: ExcessDelayPolicy) -> QuotaDelayPolicy {
    QuotaDelayPolicy::new(
        DelaySeconds::new(100),
        PastTimestampPolicy::Immediate,
        excess,
        conflict,
    )
}

fn buckets(values: &[(u64, QuotaReset)]) -> QuotaBuckets {
    let ids: [&[u8]; 3] = [b"first", b"second", b"third"];
    let mut buckets = QuotaBuckets::new();
    for (index, (remaining, reset)) in values.iter().copied().enumerate() {
        let Some(id) = ids.get(index) else {
            return buckets;
        };
        let Ok(id) = QuotaBucketId::new(id) else {
            return buckets;
        };
        let Ok(bucket) = QuotaBucket::new(id, 10, remaining, reset) else {
            return buckets;
        };
        if buckets.try_push(bucket).is_err() {
            return buckets;
        }
    }
    buckets
}

#[test]
fn selects_longest_exhausted_bucket_and_ignores_available_buckets() {
    let quotas = buckets(&[
        (0, QuotaReset::After(DelaySeconds::new(20))),
        (0, QuotaReset::At(WallClockTimestamp::new(1_050))),
        (1, QuotaReset::Unknown),
    ]);
    let decision = decide_delay(
        &quotas,
        None,
        WallClockTimestamp::new(1_000),
        None,
        policy(DelayConflictPolicy::Longest, ExcessDelayPolicy::Reject),
    );
    assert_eq!(
        decision.map(|value| value.map(|value| value.delay().get())),
        Ok(Some(50))
    );
}

#[test]
fn applies_conflict_and_maximum_policies_deterministically() {
    let quotas = buckets(&[(0, QuotaReset::After(DelaySeconds::new(80)))]);
    let retry = Some(RetryAfter::Delay(DelaySeconds::new(120)));
    let clamped = decide_delay(
        &quotas,
        retry,
        WallClockTimestamp::new(1_000),
        None,
        policy(
            DelayConflictPolicy::RetryAfterPrecedence,
            ExcessDelayPolicy::Clamp,
        ),
    );
    let Ok(Some(clamped)) = clamped else { return };
    assert_eq!(clamped.delay().get(), 100);
    assert_eq!(clamped.source(), DelaySource::RetryAfter);
    assert!(clamped.was_clamped());
    assert_eq!(
        decide_delay(
            &quotas,
            retry,
            WallClockTimestamp::new(1_000),
            None,
            policy(
                DelayConflictPolicy::RejectMismatch,
                ExcessDelayPolicy::Reject
            ),
        ),
        Err(DelayDecisionError::ConflictingMetadata)
    );
}

#[test]
fn agreeing_sources_are_reported_together() {
    let quotas = buckets(&[(0, QuotaReset::After(DelaySeconds::new(25)))]);
    let decision = decide_delay(
        &quotas,
        Some(RetryAfter::Delay(DelaySeconds::new(25))),
        WallClockTimestamp::new(1_000),
        None,
        policy(
            DelayConflictPolicy::RejectMismatch,
            ExcessDelayPolicy::Reject,
        ),
    );
    let Ok(Some(decision)) = decision else {
        return;
    };
    assert_eq!(decision.source(), DelaySource::Both);
    assert_eq!(decision.delay().get(), 25);
}

#[test]
fn rejects_clock_rollback_unknown_exhausted_reset_and_excess() {
    let unknown = buckets(&[(0, QuotaReset::Unknown)]);
    assert_eq!(
        decide_delay(
            &unknown,
            None,
            WallClockTimestamp::new(999),
            Some(WallClockTimestamp::new(1_000)),
            policy(DelayConflictPolicy::Longest, ExcessDelayPolicy::Reject),
        ),
        Err(DelayDecisionError::ClockRollback)
    );
    assert_eq!(
        decide_delay(
            &unknown,
            None,
            WallClockTimestamp::new(1_000),
            None,
            policy(DelayConflictPolicy::Longest, ExcessDelayPolicy::Reject),
        ),
        Err(DelayDecisionError::ExhaustedBucketResetUnknown)
    );
    let excess = buckets(&[(0, QuotaReset::After(DelaySeconds::new(101)))]);
    assert_eq!(
        decide_delay(
            &excess,
            None,
            WallClockTimestamp::new(1_000),
            None,
            policy(DelayConflictPolicy::Longest, ExcessDelayPolicy::Reject),
        ),
        Err(DelayDecisionError::MaximumExceeded)
    );
}

#[test]
fn past_timestamp_policy_is_explicit() {
    let quotas = buckets(&[(0, QuotaReset::At(WallClockTimestamp::new(999)))]);
    let strict = QuotaDelayPolicy::new(
        DelaySeconds::new(100),
        PastTimestampPolicy::Reject,
        ExcessDelayPolicy::Reject,
        DelayConflictPolicy::Longest,
    );
    assert_eq!(
        decide_delay(&quotas, None, WallClockTimestamp::new(1_000), None, strict),
        Err(DelayDecisionError::PastTimestamp)
    );
}
