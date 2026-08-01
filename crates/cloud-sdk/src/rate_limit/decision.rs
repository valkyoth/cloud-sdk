use super::{DelaySeconds, QuotaBuckets, QuotaReset, RetryAfter, WallClockTimestamp};

/// Handling for timestamps at or before the caller's observation time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PastTimestampPolicy {
    /// Treat a past reset as an immediate zero-second delay.
    Immediate,
    /// Reject stale absolute metadata.
    Reject,
}

/// Handling for a computed delay above the caller's maximum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExcessDelayPolicy {
    /// Clamp the decision to the caller's maximum.
    Clamp,
    /// Reject the response metadata.
    Reject,
}

/// Conflict policy when `Retry-After` and exhausted quota buckets disagree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayConflictPolicy {
    /// Follow the standard `Retry-After` instruction.
    RetryAfterPrecedence,
    /// Use the longest advertised delay.
    Longest,
    /// Reject unequal delay instructions.
    RejectMismatch,
}

/// Caller-owned quota delay policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuotaDelayPolicy {
    maximum: DelaySeconds,
    past: PastTimestampPolicy,
    excess: ExcessDelayPolicy,
    conflict: DelayConflictPolicy,
}

impl QuotaDelayPolicy {
    /// Creates an explicit bounded delay policy.
    #[must_use]
    pub const fn new(
        maximum: DelaySeconds,
        past: PastTimestampPolicy,
        excess: ExcessDelayPolicy,
        conflict: DelayConflictPolicy,
    ) -> Self {
        Self {
            maximum,
            past,
            excess,
            conflict,
        }
    }

    /// Returns the caller's maximum accepted delay.
    #[must_use]
    pub const fn maximum(self) -> DelaySeconds {
        self.maximum
    }
}

/// Metadata source selected for a delay decision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelaySource {
    /// Standard `Retry-After` metadata.
    RetryAfter,
    /// One or more exhausted provider quota buckets.
    ProviderQuota,
    /// Both sources agreed exactly.
    Both,
}

/// Pure bounded delay decision. The caller owns sleeping and clock acquisition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DelayDecision {
    delay: DelaySeconds,
    source: DelaySource,
    clamped: bool,
}

impl DelayDecision {
    /// Returns the selected delay.
    #[must_use]
    pub const fn delay(self) -> DelaySeconds {
        self.delay
    }
    /// Returns the selected metadata source.
    #[must_use]
    pub const fn source(self) -> DelaySource {
        self.source
    }
    /// Reports whether the caller maximum shortened the selected delay.
    #[must_use]
    pub const fn was_clamped(self) -> bool {
        self.clamped
    }
}

/// Derives a bounded delay without reading a clock or sleeping.
pub fn decide_delay(
    buckets: &QuotaBuckets,
    retry_after: Option<RetryAfter>,
    now: WallClockTimestamp,
    previous_now: Option<WallClockTimestamp>,
    policy: QuotaDelayPolicy,
) -> Result<Option<DelayDecision>, DelayDecisionError> {
    if previous_now.is_some_and(|previous| now < previous) {
        return Err(DelayDecisionError::ClockRollback);
    }
    let quota_delay = quota_delay(buckets, now, policy.past)?;
    let retry_delay = retry_after
        .map(|value| retry_delay(value, now, policy.past))
        .transpose()?;
    let selected = match (retry_delay, quota_delay) {
        (None, None) => return Ok(None),
        (Some(delay), None) => (delay, DelaySource::RetryAfter),
        (None, Some(delay)) => (delay, DelaySource::ProviderQuota),
        (Some(retry), Some(quota)) if retry == quota => (retry, DelaySource::Both),
        (Some(retry), Some(quota)) => match policy.conflict {
            DelayConflictPolicy::RetryAfterPrecedence => (retry, DelaySource::RetryAfter),
            DelayConflictPolicy::Longest => (core::cmp::max(retry, quota), DelaySource::Both),
            DelayConflictPolicy::RejectMismatch => {
                return Err(DelayDecisionError::ConflictingMetadata);
            }
        },
    };
    apply_maximum(selected.0, selected.1, policy)
}

fn quota_delay(
    buckets: &QuotaBuckets,
    now: WallClockTimestamp,
    past: PastTimestampPolicy,
) -> Result<Option<DelaySeconds>, DelayDecisionError> {
    let mut selected: Option<DelaySeconds> = None;
    for bucket in buckets.iter().filter(|bucket| bucket.is_exhausted()) {
        let delay = match bucket.reset() {
            QuotaReset::After(delay) => delay,
            QuotaReset::At(timestamp) => absolute_delay(timestamp, now, past)?,
            QuotaReset::Unknown => return Err(DelayDecisionError::ExhaustedBucketResetUnknown),
        };
        selected = Some(selected.map_or(delay, |current| core::cmp::max(current, delay)));
    }
    Ok(selected)
}

fn retry_delay(
    retry_after: RetryAfter,
    now: WallClockTimestamp,
    past: PastTimestampPolicy,
) -> Result<DelaySeconds, DelayDecisionError> {
    match retry_after {
        RetryAfter::Delay(delay) => Ok(delay),
        RetryAfter::HttpDate(date) => {
            let now =
                i64::try_from(now.get()).map_err(|_| DelayDecisionError::TimestampOverflow)?;
            if date.epoch_seconds() <= now {
                return past_delay(past);
            }
            let difference = date
                .epoch_seconds()
                .checked_sub(now)
                .and_then(|value| u64::try_from(value).ok())
                .ok_or(DelayDecisionError::TimestampOverflow)?;
            Ok(DelaySeconds::new(difference))
        }
    }
}

fn absolute_delay(
    timestamp: WallClockTimestamp,
    now: WallClockTimestamp,
    past: PastTimestampPolicy,
) -> Result<DelaySeconds, DelayDecisionError> {
    if timestamp <= now {
        return past_delay(past);
    }
    let delay = timestamp
        .get()
        .checked_sub(now.get())
        .ok_or(DelayDecisionError::TimestampOverflow)?;
    Ok(DelaySeconds::new(delay))
}

fn past_delay(past: PastTimestampPolicy) -> Result<DelaySeconds, DelayDecisionError> {
    match past {
        PastTimestampPolicy::Immediate => Ok(DelaySeconds::new(0)),
        PastTimestampPolicy::Reject => Err(DelayDecisionError::PastTimestamp),
    }
}

fn apply_maximum(
    delay: DelaySeconds,
    source: DelaySource,
    policy: QuotaDelayPolicy,
) -> Result<Option<DelayDecision>, DelayDecisionError> {
    if delay <= policy.maximum {
        return Ok(Some(DelayDecision {
            delay,
            source,
            clamped: false,
        }));
    }
    match policy.excess {
        ExcessDelayPolicy::Clamp => Ok(Some(DelayDecision {
            delay: policy.maximum,
            source,
            clamped: true,
        })),
        ExcessDelayPolicy::Reject => Err(DelayDecisionError::MaximumExceeded),
    }
}

/// Failure to derive an unambiguous bounded delay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DelayDecisionError {
    /// The supplied wall clock moved behind the previous observation.
    ClockRollback,
    /// An exhausted bucket did not expose actionable reset metadata.
    ExhaustedBucketResetUnknown,
    /// An absolute timestamp was at or before the current observation.
    PastTimestamp,
    /// An absolute date or difference was outside the supported range.
    TimestampOverflow,
    /// Retry and provider quota metadata disagreed under strict policy.
    ConflictingMetadata,
    /// The selected delay exceeded the caller maximum under reject policy.
    MaximumExceeded,
}

impl_static_error!(DelayDecisionError,
    Self::ClockRollback => "wall clock moved behind the previous observation",
    Self::ExhaustedBucketResetUnknown => "exhausted quota bucket has no reset instruction",
    Self::PastTimestamp => "quota reset timestamp is not in the future",
    Self::TimestampOverflow => "quota reset timestamp cannot be represented",
    Self::ConflictingMetadata => "Retry-After and provider quota metadata conflict",
    Self::MaximumExceeded => "quota delay exceeds the caller maximum",
);
