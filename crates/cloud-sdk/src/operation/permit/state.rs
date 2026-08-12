//! Direct permit transitions and execution-attempt ownership.

use super::{
    ExecutionPermitError, PermitClock, PermitDisposition, PermitExecutionError,
    PermitIdempotencyKey, PermitScope, PermitState, PermitTimestamp, PlanSubject,
    ReconciliationToken, RecoveryToken, ReplayPolicy,
};
use crate::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, LocalAsyncAuthenticatedTransport,
};
use crate::operation::{CheckedResponseGuard, PreparedExecutionError};
use crate::transport::{BoundTransport, DeliveryClassified, DeliveryPhase};
use cloud_sdk_sanitization::sanitize_bytes;

pub(super) struct DirectState<'request, 'fingerprint> {
    subject: PlanSubject<'request, 'fingerprint>,
    state: PermitState,
    generation: u16,
    remaining: u16,
    last_offset: u32,
}

impl<'request, 'fingerprint> DirectState<'request, 'fingerprint> {
    pub(super) fn new(
        subject: PlanSubject<'request, 'fingerprint>,
        expected: PermitScope,
        now: PermitTimestamp,
    ) -> Result<Self, ExecutionPermitError> {
        if subject.scope() != expected {
            return Err(ExecutionPermitError::ScopeMismatch);
        }
        let last_offset = subject.validity().offset(now)?;
        Ok(Self {
            subject,
            state: PermitState::Ready,
            generation: 0,
            remaining: subject.attempt_budget().get(),
            last_offset,
        })
    }

    pub(super) const fn state(&self) -> PermitState {
        self.state
    }

    pub(super) fn begin(
        &mut self,
        now: PermitTimestamp,
    ) -> Result<PermitAttempt<'_, 'request, 'fingerprint>, ExecutionPermitError> {
        self.begin_for(self.subject, now)
    }

    pub(super) fn begin_for(
        &mut self,
        candidate: PlanSubject<'_, '_>,
        now: PermitTimestamp,
    ) -> Result<PermitAttempt<'_, 'request, 'fingerprint>, ExecutionPermitError> {
        if !self.subject.fingerprint().matches(candidate.fingerprint()) {
            return Err(ExecutionPermitError::FingerprintMismatch);
        }
        self.observe(now)?;
        match self.state {
            PermitState::Ready if self.remaining != 0 => {}
            PermitState::Ready | PermitState::Spent => return Err(ExecutionPermitError::Spent),
            PermitState::InFlight => return Err(ExecutionPermitError::AttemptInFlight),
            PermitState::Recoverable => return Err(ExecutionPermitError::RecoveryRequired),
            PermitState::PendingReconciliation => {
                return Err(ExecutionPermitError::ReconciliationRequired);
            }
        }
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or(ExecutionPermitError::Spent)?;
        self.state = PermitState::InFlight;
        Ok(PermitAttempt::direct(self, self.generation))
    }

    pub(super) fn recover_not_sent(
        &mut self,
        token: RecoveryToken,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.observe(now)?;
        if self.state != PermitState::Recoverable || token.0 != self.generation {
            return Err(ExecutionPermitError::StaleGeneration);
        }
        if self.subject.replay_policy() == ReplayPolicy::SingleAttempt {
            return Err(ExecutionPermitError::ReplayForbidden);
        }
        self.rearm()
    }

    pub(super) fn reconcile_not_applied(
        &mut self,
        token: ReconciliationToken,
        candidate: PlanSubject<'_, '_>,
        idempotency: PermitIdempotencyKey<'_>,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.observe(now)?;
        if self.state != PermitState::PendingReconciliation || token.0 != self.generation {
            return Err(ExecutionPermitError::StaleGeneration);
        }
        if self.subject.replay_policy() != ReplayPolicy::ReconcileThenRetry {
            return Err(ExecutionPermitError::ReplayForbidden);
        }
        if !self.subject.fingerprint().matches(candidate.fingerprint()) {
            return Err(ExecutionPermitError::FingerprintMismatch);
        }
        if !self
            .subject
            .idempotency()
            .is_some_and(|expected| expected.matches(idempotency))
        {
            return Err(ExecutionPermitError::IdempotencyMismatch);
        }
        self.rearm()
    }

    fn observe(&mut self, now: PermitTimestamp) -> Result<(), ExecutionPermitError> {
        let offset = match self.subject.validity().offset(now) {
            Ok(offset) => offset,
            Err(error) => {
                self.state = PermitState::Spent;
                self.remaining = 0;
                return Err(error);
            }
        };
        if offset < self.last_offset {
            self.state = PermitState::Spent;
            self.remaining = 0;
            return Err(ExecutionPermitError::ClockRollback);
        }
        self.last_offset = offset;
        Ok(())
    }

    fn rearm(&mut self) -> Result<(), ExecutionPermitError> {
        if self.remaining == 0 {
            self.state = PermitState::Spent;
            return Err(ExecutionPermitError::Spent);
        }
        let Some(generation) = self.generation.checked_add(1) else {
            self.state = PermitState::Spent;
            return Err(ExecutionPermitError::GenerationExhausted);
        };
        self.generation = generation;
        self.state = PermitState::Ready;
        Ok(())
    }

    fn complete(&mut self, generation: u16, phase: AttemptPhase) -> PermitDisposition {
        if self.state != PermitState::InFlight || self.generation != generation {
            self.state = PermitState::Spent;
            return PermitDisposition::Spent;
        }
        match phase {
            AttemptPhase::Applied | AttemptPhase::Rejected => {
                self.state = PermitState::Spent;
                PermitDisposition::Spent
            }
            AttemptPhase::NotSent if self.remaining == 0 => {
                self.state = PermitState::Spent;
                PermitDisposition::Spent
            }
            AttemptPhase::NotSent => {
                self.state = PermitState::Recoverable;
                PermitDisposition::Recoverable(RecoveryToken(generation))
            }
            AttemptPhase::Uncertain => {
                self.state = PermitState::PendingReconciliation;
                PermitDisposition::PendingReconciliation(ReconciliationToken(generation))
            }
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum AttemptPhase {
    Applied,
    Rejected,
    NotSent,
    Uncertain,
}

enum AttemptOwner<'permit, 'request, 'fingerprint> {
    Direct(&'permit mut DirectState<'request, 'fingerprint>),
    Shared(&'permit super::shared::SharedPermitState),
}

/// One in-flight attempt. Dropping it records uncertain delivery.
///
/// The prepared request cannot be extracted from this capability.
///
/// ```compile_fail
/// use cloud_sdk::operation::PermitAttempt;
///
/// fn extract(attempt: &PermitAttempt<'_, '_, '_>) {
///     let _ = attempt.prepared();
/// }
/// ```
#[must_use]
pub struct PermitAttempt<'permit, 'request, 'fingerprint> {
    owner: AttemptOwner<'permit, 'request, 'fingerprint>,
    subject: PlanSubject<'request, 'fingerprint>,
    generation: u16,
    finished: bool,
}

impl<'permit, 'request, 'fingerprint> PermitAttempt<'permit, 'request, 'fingerprint> {
    pub(super) fn direct(
        owner: &'permit mut DirectState<'request, 'fingerprint>,
        generation: u16,
    ) -> Self {
        Self {
            subject: owner.subject,
            owner: AttemptOwner::Direct(owner),
            generation,
            finished: false,
        }
    }

    pub(super) fn shared(
        owner: &'permit super::shared::SharedPermitState,
        subject: PlanSubject<'request, 'fingerprint>,
        generation: u16,
    ) -> Self {
        Self {
            owner: AttemptOwner::Shared(owner),
            subject,
            generation,
            finished: false,
        }
    }

    /// Completes a manually driven attempt with conservative delivery state.
    ///
    /// `NotSent` is sound only when a delivery-aware transport boundary proves
    /// that no request bytes reached the peer. Unknown state must be reported
    /// as `PossiblySent`; ordinary callers should prefer the execute methods.
    pub fn complete(mut self, phase: DeliveryPhase) -> PermitDisposition {
        let phase = match phase {
            DeliveryPhase::NotSent => AttemptPhase::NotSent,
            DeliveryPhase::PossiblySent | DeliveryPhase::ResponseStarted => AttemptPhase::Uncertain,
        };
        self.finish(phase)
    }

    /// Marks a successful checked provider response and spends authority.
    pub fn complete_applied(mut self) -> PermitDisposition {
        self.finish(AttemptPhase::Applied)
    }

    /// Rejects this in-flight attempt before transport dispatch.
    ///
    /// Provider wrappers use this after validating request-bound evidence with
    /// the same clock sample supplied to the generic permit check.
    pub fn reject_authorization<E>(
        mut self,
        error: ExecutionPermitError,
        response_storage: &mut [u8],
        response_header_storage: &mut [u8],
    ) -> PermitExecutionError<E> {
        sanitize_bytes(response_storage);
        sanitize_bytes(response_header_storage);
        let disposition = self.finish(AttemptPhase::Rejected);
        PermitExecutionError {
            execution: PreparedExecutionError::AuthorizationInvalid(error),
            disposition,
        }
    }

    /// Executes once through a delivery-classified blocking transport.
    pub fn execute_blocking<'buffer, T, C>(
        mut self,
        clock: &C,
        transport: &T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponseGuard<'buffer>, PermitExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
    {
        sanitize_bytes(response_storage);
        sanitize_bytes(response_header_storage);
        self.ensure_fresh(clock.now(), response_storage, response_header_storage)?;
        let result = self.subject.prepared().execute_blocking_authorized(
            transport,
            Some(self.subject.endpoint()),
            response_storage,
            response_header_storage,
        );
        self.finish_result(result)
    }

    /// Executes once through a delivery-classified Send-async transport.
    #[allow(clippy::manual_async_fn)]
    pub fn execute_async<'transport, 'buffer, T, C>(
        mut self,
        clock: &'transport C,
        transport: &'transport T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> impl core::future::Future<
        Output = Result<CheckedResponseGuard<'buffer>, PermitExecutionError<T::Error>>,
    > + 'transport
    where
        T: AsyncAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + Sync + ?Sized,
        'request: 'transport,
        'permit: 'transport,
        'buffer: 'transport,
    {
        sanitize_bytes(response_storage);
        sanitize_bytes(response_header_storage);
        async move {
            self.ensure_fresh(clock.now(), response_storage, response_header_storage)?;
            let result = self
                .subject
                .prepared()
                .execute_async_authorized(
                    transport,
                    Some(self.subject.endpoint()),
                    response_storage,
                    response_header_storage,
                )
                .await;
            self.finish_result(result)
        }
    }

    /// Executes once through a delivery-classified local-async transport.
    #[allow(clippy::manual_async_fn)]
    pub fn execute_local_async<'transport, 'buffer, T, C>(
        mut self,
        clock: &'transport C,
        transport: &'transport T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> impl core::future::Future<
        Output = Result<CheckedResponseGuard<'buffer>, PermitExecutionError<T::Error>>,
    > + 'transport
    where
        T: LocalAsyncAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
        'request: 'transport,
        'permit: 'transport,
        'buffer: 'transport,
    {
        sanitize_bytes(response_storage);
        sanitize_bytes(response_header_storage);
        async move {
            self.ensure_fresh(clock.now(), response_storage, response_header_storage)?;
            let result = self
                .subject
                .prepared()
                .execute_local_async_authorized(
                    transport,
                    Some(self.subject.endpoint()),
                    response_storage,
                    response_header_storage,
                )
                .await;
            self.finish_result(result)
        }
    }

    fn ensure_fresh<E>(
        &mut self,
        now: PermitTimestamp,
        response_storage: &mut [u8],
        response_header_storage: &mut [u8],
    ) -> Result<(), PermitExecutionError<E>> {
        let observed = match &mut self.owner {
            AttemptOwner::Direct(owner) => owner.observe(now),
            AttemptOwner::Shared(owner) => {
                owner.observe_attempt(self.subject, self.generation, now)
            }
        };
        if let Err(error) = observed {
            sanitize_bytes(response_storage);
            sanitize_bytes(response_header_storage);
            let disposition = self.finish(AttemptPhase::Rejected);
            return Err(PermitExecutionError {
                execution: PreparedExecutionError::AuthorizationInvalid(error),
                disposition,
            });
        }
        Ok(())
    }

    fn finish_result<'buffer, E: DeliveryClassified>(
        &mut self,
        result: Result<CheckedResponseGuard<'buffer>, PreparedExecutionError<E>>,
    ) -> Result<CheckedResponseGuard<'buffer>, PermitExecutionError<E>> {
        match result {
            Ok(response) => {
                let _ = self.finish(AttemptPhase::Applied);
                Ok(response)
            }
            Err(execution) => {
                let phase = match execution.delivery_phase() {
                    DeliveryPhase::NotSent => AttemptPhase::NotSent,
                    DeliveryPhase::PossiblySent | DeliveryPhase::ResponseStarted => {
                        AttemptPhase::Uncertain
                    }
                };
                let disposition = self.finish(phase);
                Err(PermitExecutionError {
                    execution,
                    disposition,
                })
            }
        }
    }

    fn finish(&mut self, phase: AttemptPhase) -> PermitDisposition {
        self.finished = true;
        match &mut self.owner {
            AttemptOwner::Direct(owner) => owner.complete(self.generation, phase),
            AttemptOwner::Shared(owner) => owner.complete(self.generation, phase),
        }
    }
}

impl Drop for PermitAttempt<'_, '_, '_> {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.finish(AttemptPhase::Uncertain);
        }
    }
}

impl core::fmt::Debug for PermitAttempt<'_, '_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PermitAttempt")
            .field("generation", &self.generation)
            .field("plan", &"[redacted]")
            .finish()
    }
}
