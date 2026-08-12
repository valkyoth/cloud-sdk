//! Request-bound destructive execution permits for Robot resets.

use core::fmt;

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, LocalAsyncAuthenticatedTransport,
};
use cloud_sdk::operation::{
    AttemptBudget, CanonicalPlanFingerprint, ExecutionPermitError, PermitClock, PermitContext,
    PermitDisposition, PermitExecutionError, PermitIdempotencyKey, PermitState, PermitTimestamp,
    PermitValidity, PlanChange, PlanConfirmation, PlanCost, PlanFingerprintBuildError,
    PlanFingerprintDigest, PlanFingerprintScope, PlanSubject, ReconciliationToken, RecoveryToken,
    ReplayPolicy, SharedPermitState,
};
use cloud_sdk::retry::FingerprintHasher;
use cloud_sdk::transport::{BoundTransport, DeliveryClassified, DeliveryPhase, EndpointIdentity};

use super::{CheckedRobotReset, PreparedRobotReset, RobotResetExecuteRequest};

struct ResetBinding<'request, R>(&'request R);

impl<R> Clone for ResetBinding<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R> Copy for ResetBinding<'_, R> {}

/// Exact Robot reset request plus caller policy ready for plan fingerprinting.
pub struct RobotResetPlanConfirmation<'plan, 'storage, 'request, R> {
    inner: PlanConfirmation<'plan, 'storage>,
    binding: ResetBinding<'request, R>,
}

impl<'plan, 'storage, 'request, 'state>
    RobotResetPlanConfirmation<'plan, 'storage, 'request, RobotResetExecuteRequest<'state>>
{
    /// Binds caller policy to the exact request that must validate the response.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        prepared: PreparedRobotReset<'storage, 'request, RobotResetExecuteRequest<'state>>,
        endpoint: EndpointIdentity<'plan>,
        account: PlanFingerprintScope<'plan>,
        tenant: PlanFingerprintScope<'plan>,
        context: PermitContext<'plan>,
        validity: PermitValidity,
        replay: ReplayPolicy,
        attempts: AttemptBudget,
        change: PlanChange,
        cost: Option<PlanCost>,
        idempotency: Option<PermitIdempotencyKey<'plan>>,
    ) -> Self {
        let (prepared, request) = prepared.into_plan_parts();
        Self {
            inner: PlanConfirmation::new(
                prepared,
                endpoint,
                account,
                tenant,
                context,
                validity,
                replay,
                attempts,
                change,
                cost,
                idempotency,
            ),
            binding: ResetBinding(request),
        }
    }
}

impl<R> fmt::Debug for RobotResetPlanConfirmation<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotResetPlanConfirmation")
            .field("plan", &self.inner)
            .field("request", &"[bound]")
            .finish()
    }
}

/// Exact caller-buffer plan fingerprint retaining Robot reset provenance.
pub struct RobotResetCanonicalPlanFingerprint<'output, 'plan, 'storage, 'request, R> {
    inner: CanonicalPlanFingerprint<'output, 'plan, 'storage>,
    binding: ResetBinding<'request, R>,
}

impl<R> RobotResetCanonicalPlanFingerprint<'_, '_, '_, '_, R> {
    /// Borrows the authorized plan and its non-forgeable request association.
    #[must_use]
    pub fn subject(&self) -> RobotResetPlanSubject<'_, '_, '_, R> {
        RobotResetPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Strong-digest plan fingerprint retaining Robot reset provenance.
pub struct RobotResetPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R> {
    inner: PlanFingerprintDigest<'output, 'plan, 'storage>,
    binding: ResetBinding<'request, R>,
}

impl<R> RobotResetPlanFingerprintDigest<'_, '_, '_, '_, R> {
    /// Returns the admitted collision-resistant digest algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> cloud_sdk::retry::DigestAlgorithm {
        self.inner.algorithm()
    }

    /// Borrows the authorized plan and its non-forgeable request association.
    #[must_use]
    pub fn subject(&self) -> RobotResetPlanSubject<'_, '_, '_, R> {
        RobotResetPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Builds an exact Robot reset plan fingerprint in caller-owned storage.
pub fn build_robot_reset_canonical_plan<'output, 'plan, 'storage, 'request, R>(
    plan: RobotResetPlanConfirmation<'plan, 'storage, 'request, R>,
    output: &'output mut [u8],
) -> Result<
    RobotResetCanonicalPlanFingerprint<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<core::convert::Infallible>,
> {
    let inner = cloud_sdk::operation::build_canonical_plan(plan.inner, output)?;
    Ok(RobotResetCanonicalPlanFingerprint {
        inner,
        binding: plan.binding,
    })
}

/// Builds a Robot reset plan digest and clears its scratch storage.
pub fn build_robot_reset_plan_digest<'output, 'plan, 'storage, 'request, R, H>(
    plan: RobotResetPlanConfirmation<'plan, 'storage, 'request, R>,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<
    RobotResetPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<H::Error>,
>
where
    H: FingerprintHasher,
{
    let inner = cloud_sdk::operation::build_plan_digest(plan.inner, scratch, output, hasher)?;
    Ok(RobotResetPlanFingerprintDigest {
        inner,
        binding: plan.binding,
    })
}

/// Opaque Robot reset subject used to create or recheck destructive permits.
pub struct RobotResetPlanSubject<'storage, 'fingerprint, 'request, R> {
    inner: PlanSubject<'storage, 'fingerprint>,
    binding: ResetBinding<'request, R>,
}

impl<R> Clone for RobotResetPlanSubject<'_, '_, '_, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R> Copy for RobotResetPlanSubject<'_, '_, '_, R> {}

impl<R> fmt::Debug for RobotResetPlanSubject<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotResetPlanSubject")
            .field("subject", &self.inner)
            .field("request", &"[bound]")
            .finish()
    }
}

/// One in-flight Robot reset attempt retaining its exact request association.
#[must_use]
pub struct RobotResetPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R> {
    inner: cloud_sdk::operation::PermitAttempt<'permit, 'storage, 'fingerprint>,
    binding: ResetBinding<'request, R>,
}

impl<'permit, 'storage, 'fingerprint, 'request, R>
    RobotResetPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R>
{
    /// Completes a manually driven attempt with conservative delivery state.
    pub fn complete(self, phase: DeliveryPhase) -> PermitDisposition {
        self.inner.complete(phase)
    }

    /// Executes once and returns a response bound to the authorized request.
    pub fn execute_blocking<'buffer, T, C>(
        self,
        clock: &C,
        transport: &T,
        body: &'buffer mut [u8],
        headers: &'buffer mut [u8],
    ) -> Result<CheckedRobotReset<'buffer, 'request, R>, PermitExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
    {
        let binding = self.binding;
        self.inner
            .execute_blocking(clock, transport, body, headers)
            .map(|inner| CheckedRobotReset::from_executed(binding.0, inner))
    }

    /// Executes once through a Send-async transport without losing provenance.
    #[allow(clippy::manual_async_fn)]
    pub fn execute_async<'transport, 'buffer, T, C>(
        self,
        clock: &'transport C,
        transport: &'transport T,
        body: &'buffer mut [u8],
        headers: &'buffer mut [u8],
    ) -> impl core::future::Future<
        Output = Result<CheckedRobotReset<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
    > + 'transport
    where
        T: AsyncAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + Sync + ?Sized,
        'storage: 'transport,
        'permit: 'transport,
        'fingerprint: 'transport,
        'request: 'transport,
        'buffer: 'transport,
    {
        let binding = self.binding;
        let future = self.inner.execute_async(clock, transport, body, headers);
        async move {
            future
                .await
                .map(|inner| CheckedRobotReset::from_executed(binding.0, inner))
        }
    }

    /// Executes once through a local-async transport without losing provenance.
    #[allow(clippy::manual_async_fn)]
    pub fn execute_local_async<'transport, 'buffer, T, C>(
        self,
        clock: &'transport C,
        transport: &'transport T,
        body: &'buffer mut [u8],
        headers: &'buffer mut [u8],
    ) -> impl core::future::Future<
        Output = Result<CheckedRobotReset<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
    > + 'transport
    where
        T: LocalAsyncAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
        'storage: 'transport,
        'permit: 'transport,
        'fingerprint: 'transport,
        'request: 'transport,
        'buffer: 'transport,
    {
        let binding = self.binding;
        let future = self
            .inner
            .execute_local_async(clock, transport, body, headers);
        async move {
            future
                .await
                .map(|inner| CheckedRobotReset::from_executed(binding.0, inner))
        }
    }
}

impl<R> fmt::Debug for RobotResetPermitAttempt<'_, '_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotResetPermitAttempt")
            .field("attempt", &"[redacted]")
            .field("request", &"[bound]")
            .finish()
    }
}

/// Direct destructive permit that preserves Robot reset response provenance.
pub struct RobotResetDestructivePermit<'storage, 'fingerprint, 'request, R> {
    inner: cloud_sdk::operation::DestructivePermit<'storage, 'fingerprint>,
    binding: ResetBinding<'request, R>,
}

impl<'storage, 'fingerprint, 'request, R>
    RobotResetDestructivePermit<'storage, 'fingerprint, 'request, R>
{
    /// Creates authority only from a request-bound Robot reset subject.
    pub fn new(
        subject: RobotResetPlanSubject<'storage, 'fingerprint, 'request, R>,
        now: PermitTimestamp,
    ) -> Result<Self, ExecutionPermitError> {
        Ok(Self {
            inner: cloud_sdk::operation::DestructivePermit::new(subject.inner, now)?,
            binding: subject.binding,
        })
    }

    /// Returns the current fail-closed lifecycle state.
    #[must_use]
    pub const fn state(&self) -> PermitState {
        self.inner.state()
    }

    /// Starts one attempt for the exact bound Robot reset request.
    pub fn begin(
        &mut self,
        now: PermitTimestamp,
    ) -> Result<
        RobotResetPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
        ExecutionPermitError,
    > {
        Ok(RobotResetPermitAttempt {
            inner: self.inner.begin(now)?,
            binding: self.binding,
        })
    }

    /// Starts only when another bound subject has the same plan fingerprint.
    pub fn begin_for(
        &mut self,
        candidate: RobotResetPlanSubject<'_, '_, '_, R>,
        now: PermitTimestamp,
    ) -> Result<
        RobotResetPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
        ExecutionPermitError,
    > {
        Ok(RobotResetPermitAttempt {
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
        candidate: RobotResetPlanSubject<'_, '_, '_, R>,
        idempotency: PermitIdempotencyKey<'_>,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.inner
            .reconcile_not_applied(token, candidate.inner, idempotency, now)
    }
}

/// Shared destructive permit that preserves Robot reset response provenance.
pub struct RobotResetSharedDestructivePermit<'state, 'storage, 'fingerprint, 'request, R> {
    inner: cloud_sdk::operation::SharedDestructivePermit<'state, 'storage, 'fingerprint>,
    binding: ResetBinding<'request, R>,
}

impl<'state, 'storage, 'fingerprint, 'request, R>
    RobotResetSharedDestructivePermit<'state, 'storage, 'fingerprint, 'request, R>
{
    /// Exclusively binds shared state to one Robot reset request and plan.
    pub fn new(
        state: &'state mut SharedPermitState,
        subject: RobotResetPlanSubject<'storage, 'fingerprint, 'request, R>,
        now: PermitTimestamp,
    ) -> Result<Self, ExecutionPermitError> {
        Ok(Self {
            inner: cloud_sdk::operation::SharedDestructivePermit::new(state, subject.inner, now)?,
            binding: subject.binding,
        })
    }

    /// Returns the shared lifecycle state.
    #[must_use]
    pub fn state(&self) -> PermitState {
        self.inner.state()
    }

    /// Atomically starts one request-bound attempt.
    pub fn begin(
        &self,
        now: PermitTimestamp,
    ) -> Result<
        RobotResetPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
        ExecutionPermitError,
    > {
        Ok(RobotResetPermitAttempt {
            inner: self.inner.begin(now)?,
            binding: self.binding,
        })
    }

    /// Starts only for another bound subject with the same fingerprint.
    pub fn begin_for(
        &self,
        candidate: RobotResetPlanSubject<'_, '_, '_, R>,
        now: PermitTimestamp,
    ) -> Result<
        RobotResetPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
        ExecutionPermitError,
    > {
        Ok(RobotResetPermitAttempt {
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
        candidate: RobotResetPlanSubject<'_, '_, '_, R>,
        idempotency: PermitIdempotencyKey<'_>,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.inner
            .reconcile_not_applied(token, candidate.inner, idempotency, now)
    }
}

impl<R> Clone for RobotResetSharedDestructivePermit<'_, '_, '_, '_, R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            binding: self.binding,
        }
    }
}
