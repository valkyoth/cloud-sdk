use cloud_sdk::operation::{
    ExecutionPermitError, PermitIdempotencyKey, PermitState, PermitTimestamp, ReconciliationToken,
    RecoveryToken, SharedPermitState,
};

use super::{RobotWolPermitAttempt, RobotWolPermitRequest, RobotWolPlanSubject};

/// Shared request-bound authority for one WOL mutation.
pub struct RobotWolSharedMutationPermit<
    'state,
    'storage,
    'fingerprint,
    'request,
    R: RobotWolPermitRequest,
> {
    inner: cloud_sdk::operation::SharedMutationPermit<'state, 'storage, 'fingerprint>,
    binding: super::WolBinding<'request, R>,
}

impl<'state, 'storage, 'fingerprint, 'request, R: RobotWolPermitRequest>
    RobotWolSharedMutationPermit<'state, 'storage, 'fingerprint, 'request, R>
{
    /// Binds shared state to one exact request and plan.
    pub fn new(
        state: &'state mut SharedPermitState,
        subject: RobotWolPlanSubject<'storage, 'fingerprint, 'request, R>,
        now: PermitTimestamp,
    ) -> Result<Self, ExecutionPermitError> {
        Ok(Self {
            inner: cloud_sdk::operation::SharedMutationPermit::new(state, subject.inner, now)?,
            binding: subject.binding,
        })
    }

    /// Returns the shared fail-closed lifecycle state.
    #[must_use]
    pub fn state(&self) -> PermitState {
        self.inner.state()
    }

    /// Atomically starts one request-bound attempt.
    pub fn begin(
        &self,
        now: PermitTimestamp,
    ) -> Result<RobotWolPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>, ExecutionPermitError>
    {
        Ok(RobotWolPermitAttempt {
            inner: self.inner.begin(now)?,
            binding: self.binding,
        })
    }

    /// Starts only when a rechecked subject has the same fingerprint.
    pub fn begin_for(
        &self,
        candidate: RobotWolPlanSubject<'_, '_, '_, R>,
        now: PermitTimestamp,
    ) -> Result<RobotWolPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>, ExecutionPermitError>
    {
        Ok(RobotWolPermitAttempt {
            inner: self.inner.begin_for(candidate.inner, now)?,
            binding: self.binding,
        })
    }

    /// Atomically recovers a generation-matched not-sent attempt.
    pub fn recover_not_sent(
        &self,
        token: RecoveryToken,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.inner.recover_not_sent(token, now)
    }

    /// Rearms after operation-specific reconciliation.
    pub fn reconcile_not_applied(
        &self,
        token: ReconciliationToken,
        candidate: RobotWolPlanSubject<'_, '_, '_, R>,
        idempotency: PermitIdempotencyKey<'_>,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.inner
            .reconcile_not_applied(token, candidate.inner, idempotency, now)
    }
}

impl<R: RobotWolPermitRequest> Clone for RobotWolSharedMutationPermit<'_, '_, '_, '_, R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            binding: self.binding,
        }
    }
}
