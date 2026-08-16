//! Exact cost authority for catalog-derived Robot orders.

use core::fmt;

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, LocalAsyncAuthenticatedTransport,
};
use cloud_sdk::operation::{
    AttemptBudget, CanonicalPlanFingerprint, CostPermit, ExecutionPermitError, PermitClock,
    PermitContext, PermitDisposition, PermitExecutionError, PermitIdempotencyKey, PermitState,
    PermitTimestamp, PermitValidity, PlanChange, PlanConfirmation, PlanFingerprintBuildError,
    PlanFingerprintDigest, PlanFingerprintScope, PlanSubject, ReconciliationToken, RecoveryToken,
    ReplayPolicy,
};
use cloud_sdk::retry::FingerprintHasher;
use cloud_sdk::transport::{BoundTransport, DeliveryClassified, DeliveryPhase, EndpointIdentity};

use super::cost::RobotOrderAccount;
use super::exchange::{CheckedRobotOrderMutation, PreparedRobotOrderMutation};
use super::reconcile::RobotOrderNotApplied;
use super::request::{
    RobotAddonOrderCreateRequest, RobotMarketOrderCreateRequest, RobotStandardOrderCreateRequest,
};

mod sealed {
    pub trait Sealed {}
}

/// Sealed billable Robot request accepted by cost authority.
pub trait RobotOrderPermitRequest: sealed::Sealed {
    #[doc(hidden)]
    fn plan_cost(&self) -> cloud_sdk::operation::PlanCost;
}

macro_rules! permit_request {
    ($($type:ty),+ $(,)?) => {$ (
        impl sealed::Sealed for $type {}
        impl RobotOrderPermitRequest for $type {
            fn plan_cost(&self) -> cloud_sdk::operation::PlanCost { self.cost() }
        }
    )+ };
}

permit_request!(
    RobotStandardOrderCreateRequest<'_>,
    RobotMarketOrderCreateRequest<'_>,
    RobotAddonOrderCreateRequest<'_, '_>,
);

struct OrderBinding<'request, R>(&'request R);
impl<R> Clone for OrderBinding<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R> Copy for OrderBinding<'_, R> {}

/// Complete exact billable-order intent ready for canonical fingerprinting.
pub struct RobotOrderPlanConfirmation<'plan, 'storage, 'request, R: RobotOrderPermitRequest> {
    inner: PlanConfirmation<'plan, 'storage>,
    binding: OrderBinding<'request, R>,
}

impl<'plan, 'storage, 'request, R: RobotOrderPermitRequest>
    RobotOrderPlanConfirmation<'plan, 'storage, 'request, R>
{
    /// Binds exact request bytes, price, account, expiry, replay, and fresh identity.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        prepared: PreparedRobotOrderMutation<'storage, 'request, R>,
        endpoint: EndpointIdentity<'plan>,
        account: RobotOrderAccount<'plan>,
        context: PermitContext<'plan>,
        validity: PermitValidity,
        replay: ReplayPolicy,
        attempts: AttemptBudget,
        idempotency: Option<PermitIdempotencyKey<'plan>>,
    ) -> Self {
        let (prepared, request) = prepared.into_plan_parts();
        Self {
            inner: PlanConfirmation::new(
                prepared,
                endpoint,
                PlanFingerprintScope::Value(account.bytes()),
                PlanFingerprintScope::Absent,
                context,
                validity,
                replay,
                attempts,
                PlanChange::ChangesState,
                Some(request.plan_cost()),
                idempotency,
            ),
            binding: OrderBinding(request),
        }
    }
}

impl<R: RobotOrderPermitRequest> fmt::Debug for RobotOrderPlanConfirmation<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotOrderPlanConfirmation([redacted])")
    }
}

/// Caller-buffer exact fingerprint retaining order request provenance.
pub struct RobotOrderCanonicalPlanFingerprint<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotOrderPermitRequest,
> {
    inner: CanonicalPlanFingerprint<'output, 'plan, 'storage>,
    binding: OrderBinding<'request, R>,
}

impl<R: RobotOrderPermitRequest> RobotOrderCanonicalPlanFingerprint<'_, '_, '_, '_, R> {
    /// Borrows the exact confirmed order subject.
    #[must_use]
    pub fn subject(&self) -> RobotOrderPlanSubject<'_, '_, '_, R> {
        RobotOrderPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Strong fingerprint retaining exact order request provenance.
pub struct RobotOrderPlanFingerprintDigest<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotOrderPermitRequest,
> {
    inner: PlanFingerprintDigest<'output, 'plan, 'storage>,
    binding: OrderBinding<'request, R>,
}

impl<R: RobotOrderPermitRequest> RobotOrderPlanFingerprintDigest<'_, '_, '_, '_, R> {
    /// Returns the admitted collision-resistant digest algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> cloud_sdk::retry::DigestAlgorithm {
        self.inner.algorithm()
    }
    /// Borrows the exact confirmed order subject.
    #[must_use]
    pub fn subject(&self) -> RobotOrderPlanSubject<'_, '_, '_, R> {
        RobotOrderPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Builds an exact billable-order fingerprint in caller-owned storage.
pub fn build_robot_order_canonical_plan<'output, 'plan, 'storage, 'request, R>(
    plan: RobotOrderPlanConfirmation<'plan, 'storage, 'request, R>,
    output: &'output mut [u8],
) -> Result<
    RobotOrderCanonicalPlanFingerprint<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<core::convert::Infallible>,
>
where
    R: RobotOrderPermitRequest,
{
    let inner = cloud_sdk::operation::build_canonical_plan(plan.inner, output)?;
    Ok(RobotOrderCanonicalPlanFingerprint {
        inner,
        binding: plan.binding,
    })
}

/// Builds a collision-resistant billable-order fingerprint and clears scratch.
pub fn build_robot_order_plan_digest<'output, 'plan, 'storage, 'request, R, H>(
    plan: RobotOrderPlanConfirmation<'plan, 'storage, 'request, R>,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<
    RobotOrderPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<H::Error>,
>
where
    R: RobotOrderPermitRequest,
    H: FingerprintHasher,
{
    let inner = cloud_sdk::operation::build_plan_digest(plan.inner, scratch, output, hasher)?;
    Ok(RobotOrderPlanFingerprintDigest {
        inner,
        binding: plan.binding,
    })
}

/// Opaque exact order subject accepted by [`RobotOrderCostPermit`].
pub struct RobotOrderPlanSubject<'storage, 'fingerprint, 'request, R: RobotOrderPermitRequest> {
    inner: PlanSubject<'storage, 'fingerprint>,
    binding: OrderBinding<'request, R>,
}
impl<R: RobotOrderPermitRequest> Clone for RobotOrderPlanSubject<'_, '_, '_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R: RobotOrderPermitRequest> Copy for RobotOrderPlanSubject<'_, '_, '_, R> {}
impl<R: RobotOrderPermitRequest> fmt::Debug for RobotOrderPlanSubject<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotOrderPlanSubject([redacted])")
    }
}

/// Direct, non-cloneable authority for one exact billable Robot order intent.
pub struct RobotOrderCostPermit<'storage, 'fingerprint, 'request, R: RobotOrderPermitRequest> {
    inner: CostPermit<'storage, 'fingerprint>,
    binding: OrderBinding<'request, R>,
}

impl<'storage, 'fingerprint, 'request, R: RobotOrderPermitRequest>
    RobotOrderCostPermit<'storage, 'fingerprint, 'request, R>
{
    /// Creates authority from an exact catalog-derived cost subject.
    pub fn new(
        subject: RobotOrderPlanSubject<'storage, 'fingerprint, 'request, R>,
        now: PermitTimestamp,
    ) -> Result<Self, ExecutionPermitError> {
        Ok(Self {
            inner: CostPermit::new(subject.inner, now)?,
            binding: subject.binding,
        })
    }

    /// Returns the fail-closed lifecycle state.
    #[must_use]
    pub const fn state(&self) -> PermitState {
        self.inner.state()
    }

    /// Starts one attempt for the exact confirmed order.
    pub fn begin(
        &mut self,
        now: PermitTimestamp,
    ) -> Result<
        RobotOrderPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
        ExecutionPermitError,
    > {
        Ok(RobotOrderPermitAttempt {
            inner: self.inner.begin(now)?,
            binding: self.binding,
        })
    }

    /// Starts only when a freshly prepared candidate has the same complete fingerprint.
    pub fn begin_for(
        &mut self,
        candidate: RobotOrderPlanSubject<'_, '_, '_, R>,
        now: PermitTimestamp,
    ) -> Result<
        RobotOrderPermitAttempt<'_, 'storage, 'fingerprint, 'request, R>,
        ExecutionPermitError,
    > {
        Ok(RobotOrderPermitAttempt {
            inner: self.inner.begin_for(candidate.inner, now)?,
            binding: self.binding,
        })
    }

    /// Rearms only after a generation-matched proven-not-sent result.
    pub fn recover_not_sent(
        &mut self,
        token: RecoveryToken,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.inner.recover_not_sent(token, now)
    }

    /// Rearms uncertain delivery only with request-bound absent-transaction proof.
    pub fn reconcile_not_applied(
        &mut self,
        token: ReconciliationToken,
        candidate: RobotOrderPlanSubject<'_, '_, '_, R>,
        proof: RobotOrderNotApplied<'_, R>,
        idempotency: PermitIdempotencyKey<'_>,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        if !core::ptr::eq(self.binding.0, candidate.binding.0)
            || !core::ptr::eq(self.binding.0, proof.request)
        {
            return Err(ExecutionPermitError::FingerprintMismatch);
        }
        self.inner
            .reconcile_not_applied(token, candidate.inner, idempotency, now)
    }
}

impl<R: RobotOrderPermitRequest> fmt::Debug for RobotOrderCostPermit<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotOrderCostPermit")
            .field("state", &self.inner.state())
            .field("request", &"[bound]")
            .finish()
    }
}

/// One in-flight billable order attempt retaining exact response provenance.
#[must_use]
pub struct RobotOrderPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R> {
    inner: cloud_sdk::operation::PermitAttempt<'permit, 'storage, 'fingerprint>,
    binding: OrderBinding<'request, R>,
}

impl<'permit, 'storage, 'fingerprint, 'request, R>
    RobotOrderPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R>
{
    /// Completes a manually driven attempt with conservative delivery state.
    pub fn complete(self, phase: DeliveryPhase) -> PermitDisposition {
        self.inner.complete(phase)
    }

    /// Executes once through a blocking authenticated transport.
    pub fn execute_blocking<'buffer, T, C>(
        self,
        clock: &C,
        transport: &T,
        body: &'buffer mut [u8],
        headers: &'buffer mut [u8],
    ) -> Result<CheckedRobotOrderMutation<'buffer, 'request, R>, PermitExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
    {
        let binding = self.binding;
        self.inner
            .execute_blocking(clock, transport, body, headers)
            .map(|inner| CheckedRobotOrderMutation::from_executed(binding.0, inner))
    }

    /// Executes once through a Send-async authenticated transport.
    #[allow(clippy::manual_async_fn)]
    pub fn execute_async<'transport, 'buffer, T, C>(
        self,
        clock: &'transport C,
        transport: &'transport T,
        body: &'buffer mut [u8],
        headers: &'buffer mut [u8],
    ) -> impl core::future::Future<
        Output = Result<
            CheckedRobotOrderMutation<'buffer, 'request, R>,
            PermitExecutionError<T::Error>,
        >,
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
                .map(|inner| CheckedRobotOrderMutation::from_executed(binding.0, inner))
        }
    }

    /// Executes once through a local-async authenticated transport.
    #[allow(clippy::manual_async_fn)]
    pub fn execute_local_async<'transport, 'buffer, T, C>(
        self,
        clock: &'transport C,
        transport: &'transport T,
        body: &'buffer mut [u8],
        headers: &'buffer mut [u8],
    ) -> impl core::future::Future<
        Output = Result<
            CheckedRobotOrderMutation<'buffer, 'request, R>,
            PermitExecutionError<T::Error>,
        >,
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
                .map(|inner| CheckedRobotOrderMutation::from_executed(binding.0, inner))
        }
    }
}

impl<R> fmt::Debug for RobotOrderPermitAttempt<'_, '_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotOrderPermitAttempt([redacted])")
    }
}
