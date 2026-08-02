//! Single-owner retry state and fail-closed decisions.

use core::fmt;

use super::{
    FingerprintRef, IdempotencyBinding, MonotonicDuration, MonotonicInstant, RetrySubject,
};
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
#[derive(Debug)]
pub enum RetryDecision<'request, 'subject> {
    /// One-use authorization for the exact replay subject.
    Retry(RetryPermit<'request, 'subject>),
    /// Do not execute another attempt.
    Stop(RetryStopReason),
}

/// Why a one-use retry permit did not authorize execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryPermitError {
    /// The caller observed a time before the authorized delay completed.
    TooEarly,
    /// The caller observed a time before the controller's start instant.
    MonotonicRollback,
    /// The hard elapsed budget expired before execution.
    ElapsedBudgetExhausted,
}

impl_static_error!(RetryPermitError,
    Self::TooEarly => "retry permit used before its authorized delay",
    Self::MonotonicRollback => "retry permit monotonic observation moved backward",
    Self::ElapsedBudgetExhausted => "retry permit elapsed budget is exhausted",
);

/// Non-cloneable authorization for one exact prepared replay.
///
/// The caller must consume this permit immediately after sleeping and before
/// transport execution. This second monotonic check accounts for scheduler
/// delay and other overhead after the initial retry decision.
///
/// ```compile_fail
/// use cloud_sdk::retry::RetryPermit;
///
/// fn duplicate(permit: RetryPermit<'_, '_>) {
///     let _second = permit.clone();
/// }
/// ```
#[must_use]
pub struct RetryPermit<'request, 'subject> {
    prepared: &'subject PreparedRequest<'request>,
    attempt: u16,
    delay: MonotonicDuration,
    not_before: MonotonicInstant,
    started: MonotonicInstant,
    max_elapsed: MonotonicDuration,
}

impl<'request> RetryPermit<'request, '_> {
    /// Returns the authorized attempt number, including the initial attempt.
    #[must_use]
    pub const fn attempt(&self) -> u16 {
        self.attempt
    }

    /// Returns the exact caller-selected delay charged to retry budgets.
    #[must_use]
    pub const fn delay(&self) -> MonotonicDuration {
        self.delay
    }

    /// Consumes the permit and returns its exact prepared request when timely.
    pub fn authorize_execution(
        self,
        now: MonotonicInstant,
    ) -> Result<PreparedRequest<'request>, RetryPermitError> {
        let elapsed = now
            .checked_duration_since(self.started)
            .ok_or(RetryPermitError::MonotonicRollback)?;
        if now < self.not_before {
            return Err(RetryPermitError::TooEarly);
        }
        if elapsed > self.max_elapsed {
            return Err(RetryPermitError::ElapsedBudgetExhausted);
        }
        Ok(*self.prepared)
    }
}

impl fmt::Debug for RetryPermit<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RetryPermit")
            .field("prepared", &"[bound request]")
            .field("attempt", &self.attempt)
            .field("delay", &self.delay)
            .field("not_before", &self.not_before)
            .field("started", &self.started)
            .field("max_elapsed", &self.max_elapsed)
            .finish()
    }
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
        subject: RetrySubject<'_, 'a>,
        idempotency: Option<IdempotencyBinding<'a>>,
        policy: RetryPolicy,
        started: MonotonicInstant,
    ) -> Result<Self, RetryPolicyError> {
        Self::from_parts(
            subject.prepared().metadata(),
            subject.prepared().body_replayability(),
            subject.fingerprint(),
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
    pub fn decide_retry<'request, 'subject>(
        &mut self,
        event: RetryEvent,
        replay: RetrySubject<'request, 'subject>,
        delay: MonotonicDuration,
        now: MonotonicInstant,
    ) -> Result<RetryDecision<'request, 'subject>, RetryPolicyError> {
        if !self.fingerprint.matches(replay.fingerprint()) {
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
        let projected_elapsed = elapsed.get().checked_add(delay.get());
        if projected_elapsed.is_none_or(|value| value > self.policy.max_elapsed().get()) {
            return Ok(RetryDecision::Stop(RetryStopReason::ElapsedBudgetExhausted));
        }
        let Some(not_before) = now.checked_add(delay) else {
            return Ok(RetryDecision::Stop(RetryStopReason::ElapsedBudgetExhausted));
        };
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
        Ok(RetryDecision::Retry(RetryPermit {
            prepared: replay.prepared(),
            attempt: self.attempts,
            delay,
            not_before,
            started: self.started,
            max_elapsed: self.policy.max_elapsed(),
        }))
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
