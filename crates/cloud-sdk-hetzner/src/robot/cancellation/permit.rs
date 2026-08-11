//! Request-bound destructive execution permits for Robot cancellations.

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

use super::{CheckedCancellation, PreparedCancellation};

struct CancellationBinding<'request, R>(&'request R);

impl<R> Clone for CancellationBinding<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R> Copy for CancellationBinding<'_, R> {}

/// Exact cancellation request plus caller policy ready for plan fingerprinting.
pub struct CancellationPlanConfirmation<'plan, 'storage, 'request, R> {
    inner: PlanConfirmation<'plan, 'storage>,
    binding: CancellationBinding<'request, R>,
}

impl<'plan, 'storage, 'request, R> CancellationPlanConfirmation<'plan, 'storage, 'request, R> {
    /// Binds caller policy to the exact request that must validate the response.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        prepared: PreparedCancellation<'storage, 'request, R>,
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
            binding: CancellationBinding(request),
        }
    }
}

impl<R> fmt::Debug for CancellationPlanConfirmation<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CancellationPlanConfirmation")
            .field("plan", &self.inner)
            .field("request", &"[bound]")
            .finish()
    }
}

/// Exact caller-buffer plan fingerprint retaining cancellation provenance.
pub struct CancellationCanonicalPlanFingerprint<'output, 'plan, 'storage, 'request, R> {
    inner: CanonicalPlanFingerprint<'output, 'plan, 'storage>,
    binding: CancellationBinding<'request, R>,
}

impl<R> CancellationCanonicalPlanFingerprint<'_, '_, '_, '_, R> {
    /// Borrows the authorized plan and its non-forgeable request association.
    #[must_use]
    pub fn subject(&self) -> CancellationPlanSubject<'_, '_, '_, R> {
        CancellationPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Strong-digest plan fingerprint retaining cancellation provenance.
pub struct CancellationPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R> {
    inner: PlanFingerprintDigest<'output, 'plan, 'storage>,
    binding: CancellationBinding<'request, R>,
}

impl<R> CancellationPlanFingerprintDigest<'_, '_, '_, '_, R> {
    /// Returns the admitted collision-resistant digest algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> cloud_sdk::retry::DigestAlgorithm {
        self.inner.algorithm()
    }

    /// Borrows the authorized plan and its non-forgeable request association.
    #[must_use]
    pub fn subject(&self) -> CancellationPlanSubject<'_, '_, '_, R> {
        CancellationPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Builds an exact cancellation plan fingerprint in caller-owned storage.
pub fn build_cancellation_canonical_plan<'output, 'plan, 'storage, 'request, R>(
    plan: CancellationPlanConfirmation<'plan, 'storage, 'request, R>,
    output: &'output mut [u8],
) -> Result<
    CancellationCanonicalPlanFingerprint<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<core::convert::Infallible>,
> {
    let inner = cloud_sdk::operation::build_canonical_plan(plan.inner, output)?;
    Ok(CancellationCanonicalPlanFingerprint {
        inner,
        binding: plan.binding,
    })
}

/// Builds a cancellation plan digest and clears its scratch storage.
pub fn build_cancellation_plan_digest<'output, 'plan, 'storage, 'request, R, H>(
    plan: CancellationPlanConfirmation<'plan, 'storage, 'request, R>,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<
    CancellationPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<H::Error>,
>
where
    H: FingerprintHasher,
{
    let inner = cloud_sdk::operation::build_plan_digest(plan.inner, scratch, output, hasher)?;
    Ok(CancellationPlanFingerprintDigest {
        inner,
        binding: plan.binding,
    })
}

/// Opaque cancellation subject used to create or recheck destructive permits.
pub struct CancellationPlanSubject<'storage, 'fingerprint, 'request, R> {
    inner: PlanSubject<'storage, 'fingerprint>,
    binding: CancellationBinding<'request, R>,
}

impl<R> Clone for CancellationPlanSubject<'_, '_, '_, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R> Copy for CancellationPlanSubject<'_, '_, '_, R> {}

impl<R> fmt::Debug for CancellationPlanSubject<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CancellationPlanSubject")
            .field("subject", &self.inner)
            .field("request", &"[bound]")
            .finish()
    }
}

/// One in-flight cancellation attempt retaining its exact request association.
#[must_use]
pub struct CancellationPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R> {
    inner: cloud_sdk::operation::PermitAttempt<'permit, 'storage, 'fingerprint>,
    binding: CancellationBinding<'request, R>,
}

impl<'permit, 'storage, 'fingerprint, 'request, R>
    CancellationPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R>
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
    ) -> Result<CheckedCancellation<'buffer, 'request, R>, PermitExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
    {
        let binding = self.binding;
        self.inner
            .execute_blocking(clock, transport, body, headers)
            .map(|inner| CheckedCancellation::from_executed(binding.0, inner))
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
        Output = Result<CheckedCancellation<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
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
                .map(|inner| CheckedCancellation::from_executed(binding.0, inner))
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
        Output = Result<CheckedCancellation<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
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
                .map(|inner| CheckedCancellation::from_executed(binding.0, inner))
        }
    }
}

impl<R> fmt::Debug for CancellationPermitAttempt<'_, '_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CancellationPermitAttempt")
            .field("attempt", &"[redacted]")
            .field("request", &"[bound]")
            .finish()
    }
}

/// Direct destructive permit that preserves cancellation response provenance.
pub struct CancellationDestructivePermit<'storage, 'fingerprint, 'request, R> {
    inner: cloud_sdk::operation::DestructivePermit<'storage, 'fingerprint>,
    binding: CancellationBinding<'request, R>,
}

impl<'storage, 'fingerprint, 'request, R>
    CancellationDestructivePermit<'storage, 'fingerprint, 'request, R>
{
    /// Creates authority only from a request-bound cancellation subject.
    pub fn new(
        subject: CancellationPlanSubject<'storage, 'fingerprint, 'request, R>,
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

    /// Starts one attempt for the exact bound cancellation request.
    pub fn begin(
        &mut self,
        now: PermitTimestamp,
    ) -> Result<
        CancellationPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
        ExecutionPermitError,
    > {
        Ok(CancellationPermitAttempt {
            inner: self.inner.begin(now)?,
            binding: self.binding,
        })
    }

    /// Starts only when another bound subject has the same plan fingerprint.
    pub fn begin_for(
        &mut self,
        candidate: CancellationPlanSubject<'_, '_, '_, R>,
        now: PermitTimestamp,
    ) -> Result<
        CancellationPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
        ExecutionPermitError,
    > {
        Ok(CancellationPermitAttempt {
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
        candidate: CancellationPlanSubject<'_, '_, '_, R>,
        idempotency: PermitIdempotencyKey<'_>,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.inner
            .reconcile_not_applied(token, candidate.inner, idempotency, now)
    }
}

/// Shared destructive permit that preserves cancellation response provenance.
pub struct CancellationSharedDestructivePermit<'state, 'storage, 'fingerprint, 'request, R> {
    inner: cloud_sdk::operation::SharedDestructivePermit<'state, 'storage, 'fingerprint>,
    binding: CancellationBinding<'request, R>,
}

impl<'state, 'storage, 'fingerprint, 'request, R>
    CancellationSharedDestructivePermit<'state, 'storage, 'fingerprint, 'request, R>
{
    /// Exclusively binds shared state to one cancellation request and plan.
    pub fn new(
        state: &'state mut SharedPermitState,
        subject: CancellationPlanSubject<'storage, 'fingerprint, 'request, R>,
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
        CancellationPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
        ExecutionPermitError,
    > {
        Ok(CancellationPermitAttempt {
            inner: self.inner.begin(now)?,
            binding: self.binding,
        })
    }

    /// Starts only for another bound subject with the same fingerprint.
    pub fn begin_for(
        &self,
        candidate: CancellationPlanSubject<'_, '_, '_, R>,
        now: PermitTimestamp,
    ) -> Result<
        CancellationPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
        ExecutionPermitError,
    > {
        Ok(CancellationPermitAttempt {
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
        candidate: CancellationPlanSubject<'_, '_, '_, R>,
        idempotency: PermitIdempotencyKey<'_>,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.inner
            .reconcile_not_applied(token, candidate.inner, idempotency, now)
    }
}

impl<R> Clone for CancellationSharedDestructivePermit<'_, '_, '_, '_, R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            binding: self.binding,
        }
    }
}
