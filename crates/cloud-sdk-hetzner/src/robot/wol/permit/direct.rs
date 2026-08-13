use cloud_sdk::operation::{
    ExecutionPermitError, PermitIdempotencyKey, PermitState, PermitTimestamp, ReconciliationToken,
    RecoveryToken,
};

use super::{RobotWolPermitAttempt, RobotWolPermitRequest, RobotWolPlanSubject};

/// Direct request-bound authority for one WOL mutation.
pub struct RobotWolMutationPermit<'storage, 'fingerprint, 'request, R: RobotWolPermitRequest> {
    inner: cloud_sdk::operation::MutationPermit<'storage, 'fingerprint>,
    binding: super::WolBinding<'request, R>,
}

impl<'storage, 'fingerprint, 'request, R: RobotWolPermitRequest>
    RobotWolMutationPermit<'storage, 'fingerprint, 'request, R>
{
    /// Creates authority from an exact request-bound subject.
    pub fn new(
        subject: RobotWolPlanSubject<'storage, 'fingerprint, 'request, R>,
        now: PermitTimestamp,
    ) -> Result<Self, ExecutionPermitError> {
        Ok(Self {
            inner: cloud_sdk::operation::MutationPermit::new(subject.inner, now)?,
            binding: subject.binding,
        })
    }

    /// Returns the fail-closed lifecycle state.
    #[must_use]
    pub const fn state(&self) -> PermitState {
        self.inner.state()
    }

    /// Starts one attempt for the exact bound request.
    pub fn begin(
        &mut self,
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
        &mut self,
        candidate: RobotWolPlanSubject<'_, '_, '_, R>,
        now: PermitTimestamp,
    ) -> Result<RobotWolPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>, ExecutionPermitError>
    {
        Ok(RobotWolPermitAttempt {
            inner: self.inner.begin_for(candidate.inner, now)?,
            binding: self.binding,
        })
    }

    /// Rearms after a generation-matched proven-not-sent result.
    pub fn recover_not_sent(
        &mut self,
        token: RecoveryToken,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.inner.recover_not_sent(token, now)
    }

    /// Rearms after operation-specific reconciliation.
    pub fn reconcile_not_applied(
        &mut self,
        token: ReconciliationToken,
        candidate: RobotWolPlanSubject<'_, '_, '_, R>,
        idempotency: PermitIdempotencyKey<'_>,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.inner
            .reconcile_not_applied(token, candidate.inner, idempotency, now)
    }
}

impl<R: RobotWolPermitRequest> core::fmt::Debug for RobotWolMutationPermit<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotWolMutationPermit")
            .field("state", &self.inner.state())
            .field("request", &"[bound]")
            .finish()
    }
}
