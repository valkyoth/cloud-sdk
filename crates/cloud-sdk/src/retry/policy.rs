//! Single-owner retry state and fail-closed decisions.

use core::fmt;

use super::{FingerprintRef, IdempotencyBinding, MonotonicDuration, MonotonicInstant};
use crate::operation::{
    BodyReplayability, OperationImpact, OperationMetadata, PreparedRequest, RequestSemantics,
    RetryEligibility,
};
use crate::transport::{DeliveryPhase, StatusCode};

/// Invalid maximum attempt count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaxAttemptsError {
    /// Every retry policy must admit the initial attempt.
    Zero,
}

impl_static_error!(MaxAttemptsError, Self::Zero => "retry maximum attempts must be nonzero");

/// Nonzero total attempt bound, including the initial attempt.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct MaxAttempts(u16);

impl MaxAttempts {
    /// Creates a nonzero total attempt bound.
    pub const fn new(value: u16) -> Result<Self, MaxAttemptsError> {
        if value == 0 {
            return Err(MaxAttemptsError::Zero);
        }
        Ok(Self(value))
    }

    /// Returns the total attempt bound.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// Complete caller-owned retry budgets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetryPolicy {
    max_attempts: MaxAttempts,
    max_cumulative_delay: MonotonicDuration,
    max_elapsed: MonotonicDuration,
}

impl RetryPolicy {
    /// Creates complete hard attempt, requested-delay, and elapsed budgets.
    #[must_use]
    pub const fn new(
        max_attempts: MaxAttempts,
        max_cumulative_delay: MonotonicDuration,
        max_elapsed: MonotonicDuration,
    ) -> Self {
        Self {
            max_attempts,
            max_cumulative_delay,
            max_elapsed,
        }
    }

    /// Returns the total attempt bound.
    #[must_use]
    pub const fn max_attempts(self) -> MaxAttempts {
        self.max_attempts
    }

    /// Returns the cumulative caller-requested delay bound.
    #[must_use]
    pub const fn max_cumulative_delay(self) -> MonotonicDuration {
        self.max_cumulative_delay
    }

    /// Returns the monotonic elapsed-time bound.
    #[must_use]
    pub const fn max_elapsed(self) -> MonotonicDuration {
        self.max_elapsed
    }
}

/// Failure observation considered by the retry owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryEvent {
    /// Transport failure with conservative delivery state.
    Transport(DeliveryPhase),
    /// Complete HTTP response status.
    Response(StatusCode),
}

/// Fail-closed reason why no new attempt is admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryStopReason {
    /// Provider metadata does not admit retry policy.
    IneligibleOperation,
    /// The request body cannot be reproduced byte-for-byte.
    NonReplayableBody,
    /// A state-changing operation lacks a fresh fingerprint-bound intent.
    MutationRequiresIntent,
    /// The response status is not `429` or `5xx`.
    NonTransientResponse,
    /// The total attempt bound is exhausted.
    AttemptsExhausted,
    /// The cumulative caller-requested delay bound is exhausted.
    CumulativeDelayExhausted,
    /// The monotonic elapsed-time bound is exhausted.
    ElapsedBudgetExhausted,
}

/// Retry admitted or stopped without sleeping or executing transport.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryDecision {
    /// Execute the numbered attempt after the caller-owned delay and jitter.
    Retry {
        /// Attempt number, where the initial attempt is one.
        attempt: u16,
        /// Exact caller-requested delay charged to the cumulative budget.
        delay: MonotonicDuration,
    },
    /// Do not execute another attempt.
    Stop(RetryStopReason),
}

/// Retry-controller construction or state-transition failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryPolicyError {
    /// Retrying a mutation requires a fresh idempotency intent.
    MissingMutationIntent,
    /// The initial and replay request fingerprints differ.
    FingerprintMismatch,
    /// The supplied idempotency binding belongs to another request.
    IdempotencyFingerprintMismatch,
    /// Caller monotonic observations moved backward.
    MonotonicRollback,
    /// Cumulative delay arithmetic overflowed.
    CumulativeDelayOverflow,
}

impl_static_error!(RetryPolicyError,
    Self::MissingMutationIntent => "retrying a mutation requires an idempotency intent",
    Self::FingerprintMismatch => "retry request fingerprint does not match the initial request",
    Self::IdempotencyFingerprintMismatch => "idempotency binding does not match the initial request",
    Self::MonotonicRollback => "retry monotonic observation moved backward",
    Self::CumulativeDelayOverflow => "retry cumulative delay overflowed",
);

/// Non-cloneable owner of one request's retry state and idempotency intent.
///
/// ```compile_fail
/// use cloud_sdk::retry::RetryController;
///
/// fn duplicate(owner: RetryController<'_>) {
///     let _second = owner.clone();
/// }
/// ```
pub struct RetryController<'a> {
    metadata: OperationMetadata,
    body: BodyReplayability,
    fingerprint: FingerprintRef<'a>,
    idempotency: Option<IdempotencyBinding<'a>>,
    policy: RetryPolicy,
    started: MonotonicInstant,
    last_observed: MonotonicInstant,
    attempts: u16,
    cumulative_delay: u64,
}

impl<'a> RetryController<'a> {
    /// Creates the sole retry owner for one initial request attempt.
    ///
    /// The moved intent is bound to the initial fingerprint. Mutating requests
    /// with more than one admitted attempt require a fresh intent.
    pub fn new(
        prepared: PreparedRequest<'_>,
        fingerprint: FingerprintRef<'a>,
        idempotency: Option<IdempotencyBinding<'a>>,
        policy: RetryPolicy,
        started: MonotonicInstant,
    ) -> Result<Self, RetryPolicyError> {
        Self::from_parts(
            prepared.metadata(),
            prepared.body_replayability(),
            fingerprint,
            idempotency,
            policy,
            started,
        )
    }

    fn from_parts(
        metadata: OperationMetadata,
        body: BodyReplayability,
        fingerprint: FingerprintRef<'a>,
        idempotency: Option<IdempotencyBinding<'a>>,
        policy: RetryPolicy,
        started: MonotonicInstant,
    ) -> Result<Self, RetryPolicyError> {
        if policy.max_attempts().get() > 1
            && metadata.impact() != OperationImpact::ReadOnly
            && idempotency.is_none()
        {
            return Err(RetryPolicyError::MissingMutationIntent);
        }
        if idempotency
            .as_ref()
            .is_some_and(|binding| !binding.matches(fingerprint))
        {
            return Err(RetryPolicyError::IdempotencyFingerprintMismatch);
        }
        Ok(Self {
            metadata,
            body,
            fingerprint,
            idempotency,
            policy,
            started,
            last_observed: started,
            attempts: 1,
            cumulative_delay: 0,
        })
    }

    #[cfg(test)]
    pub(crate) fn test_new(
        metadata: OperationMetadata,
        body: BodyReplayability,
        fingerprint: FingerprintRef<'a>,
        idempotency: Option<IdempotencyBinding<'a>>,
        policy: RetryPolicy,
        started: MonotonicInstant,
    ) -> Result<Self, RetryPolicyError> {
        Self::from_parts(metadata, body, fingerprint, idempotency, policy, started)
    }

    /// Returns attempts already consumed, including the initial attempt.
    #[must_use]
    pub const fn attempts(&self) -> u16 {
        self.attempts
    }

    /// Returns caller-requested delay already charged to this owner.
    #[must_use]
    pub const fn cumulative_delay(&self) -> MonotonicDuration {
        MonotonicDuration::new(self.cumulative_delay)
    }

    /// Decides whether one exact replay may execute.
    ///
    /// `delay` includes caller-selected backoff and jitter. This function does
    /// not read clocks, sleep, execute transport, or classify provider errors.
    pub fn decide_retry(
        &mut self,
        event: RetryEvent,
        replay_fingerprint: FingerprintRef<'_>,
        delay: MonotonicDuration,
        now: MonotonicInstant,
    ) -> Result<RetryDecision, RetryPolicyError> {
        if !self.fingerprint.matches(replay_fingerprint) {
            return Err(RetryPolicyError::FingerprintMismatch);
        }
        if now < self.last_observed || now.checked_duration_since(self.started).is_none() {
            return Err(RetryPolicyError::MonotonicRollback);
        }
        self.last_observed = now;

        if self.metadata.retry_eligibility() != RetryEligibility::ExplicitPolicy {
            return Ok(RetryDecision::Stop(RetryStopReason::IneligibleOperation));
        }
        if self.body != BodyReplayability::Replayable {
            return Ok(RetryDecision::Stop(RetryStopReason::NonReplayableBody));
        }
        if self.metadata.impact() != OperationImpact::ReadOnly && self.idempotency.is_none() {
            return Ok(RetryDecision::Stop(RetryStopReason::MutationRequiresIntent));
        }
        if let RetryEvent::Response(status) = event
            && status != StatusCode::TOO_MANY_REQUESTS
            && !(500..=599).contains(&status.get())
        {
            return Ok(RetryDecision::Stop(RetryStopReason::NonTransientResponse));
        }
        if matches!(
            event,
            RetryEvent::Transport(DeliveryPhase::PossiblySent | DeliveryPhase::ResponseStarted)
        ) && self.metadata.impact() != OperationImpact::ReadOnly
            && self.metadata.semantics() != RequestSemantics::Idempotent
        {
            return Ok(RetryDecision::Stop(RetryStopReason::IneligibleOperation));
        }
        if self.attempts >= self.policy.max_attempts().get() {
            return Ok(RetryDecision::Stop(RetryStopReason::AttemptsExhausted));
        }
        let elapsed = now
            .checked_duration_since(self.started)
            .ok_or(RetryPolicyError::MonotonicRollback)?;
        if elapsed > self.policy.max_elapsed() {
            return Ok(RetryDecision::Stop(RetryStopReason::ElapsedBudgetExhausted));
        }
        let cumulative = self
            .cumulative_delay
            .checked_add(delay.get())
            .ok_or(RetryPolicyError::CumulativeDelayOverflow)?;
        if cumulative > self.policy.max_cumulative_delay().get() {
            return Ok(RetryDecision::Stop(
                RetryStopReason::CumulativeDelayExhausted,
            ));
        }
        self.cumulative_delay = cumulative;
        self.attempts = self
            .attempts
            .checked_add(1)
            .ok_or(RetryPolicyError::CumulativeDelayOverflow)?;
        Ok(RetryDecision::Retry {
            attempt: self.attempts,
            delay,
        })
    }
}

impl fmt::Debug for RetryController<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RetryController")
            .field("metadata", &self.metadata)
            .field("body", &self.body)
            .field("fingerprint", &"[redacted]")
            .field(
                "intent_len",
                &self
                    .idempotency
                    .as_ref()
                    .map(IdempotencyBinding::intent_len),
            )
            .field("policy", &self.policy)
            .field("attempts", &self.attempts)
            .field("cumulative_delay", &self.cumulative_delay)
            .finish()
    }
}
