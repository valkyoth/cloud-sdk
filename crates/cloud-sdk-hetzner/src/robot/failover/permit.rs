//! Request-bound execution permits for Robot failover mutations.

mod direct;
mod shared;

use core::fmt;

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, LocalAsyncAuthenticatedTransport,
};
use cloud_sdk::operation::{
    AttemptBudget, CanonicalPlanFingerprint, PermitClock, PermitContext, PermitDisposition,
    PermitExecutionError, PermitIdempotencyKey, PermitValidity, PlanChange, PlanConfirmation,
    PlanCost, PlanFingerprintBuildError, PlanFingerprintDigest, PlanFingerprintScope, PlanSubject,
    ReplayPolicy,
};
use cloud_sdk::retry::FingerprintHasher;
use cloud_sdk::transport::{BoundTransport, DeliveryClassified, DeliveryPhase, EndpointIdentity};

use super::{CheckedRobotFailover, PreparedRobotFailover};

pub use direct::{RobotFailoverDestructivePermit, RobotFailoverMutationPermit};
pub use shared::{RobotFailoverSharedDestructivePermit, RobotFailoverSharedMutationPermit};

mod sealed {
    pub trait Sealed {}
}

/// Sealed marker for Robot failover operations requiring execution authority.
pub trait RobotFailoverPermitRequest: sealed::Sealed {}

macro_rules! permit_request {
    ($($type:ty),+ $(,)?) => {$ (
        impl sealed::Sealed for $type {}
        impl RobotFailoverPermitRequest for $type {}
    )+ };
}

permit_request!(
    super::RobotFailoverRerouteRequest,
    super::RobotFailoverDeleteRouteRequest,
);

pub(super) struct FailoverBinding<'request, R>(pub(super) &'request R);

impl<R> Clone for FailoverBinding<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R> Copy for FailoverBinding<'_, R> {}

/// Exact Robot failover mutation plus caller policy ready for fingerprinting.
pub struct RobotFailoverPlanConfirmation<'plan, 'storage, 'request, R: RobotFailoverPermitRequest> {
    inner: PlanConfirmation<'plan, 'storage>,
    binding: FailoverBinding<'request, R>,
}

impl<'plan, 'storage, 'request, R: RobotFailoverPermitRequest>
    RobotFailoverPlanConfirmation<'plan, 'storage, 'request, R>
{
    /// Binds caller policy to the exact mutation request.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        prepared: PreparedRobotFailover<'storage, 'request, R>,
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
            binding: FailoverBinding(request),
        }
    }
}

impl<R: RobotFailoverPermitRequest> fmt::Debug for RobotFailoverPlanConfirmation<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotFailoverPlanConfirmation")
            .field("plan", &self.inner)
            .field("request", &"[bound]")
            .finish()
    }
}

/// Caller-buffer plan fingerprint retaining exact Robot failover request provenance.
pub struct RobotFailoverCanonicalPlanFingerprint<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotFailoverPermitRequest,
> {
    inner: CanonicalPlanFingerprint<'output, 'plan, 'storage>,
    binding: FailoverBinding<'request, R>,
}

impl<R: RobotFailoverPermitRequest> RobotFailoverCanonicalPlanFingerprint<'_, '_, '_, '_, R> {
    /// Borrows the exact authorized plan and request association.
    #[must_use]
    pub fn subject(&self) -> RobotFailoverPlanSubject<'_, '_, '_, R> {
        RobotFailoverPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Strong-digest plan fingerprint retaining exact Robot failover request provenance.
pub struct RobotFailoverPlanFingerprintDigest<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotFailoverPermitRequest,
> {
    inner: PlanFingerprintDigest<'output, 'plan, 'storage>,
    binding: FailoverBinding<'request, R>,
}

impl<R: RobotFailoverPermitRequest> RobotFailoverPlanFingerprintDigest<'_, '_, '_, '_, R> {
    /// Returns the admitted collision-resistant digest algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> cloud_sdk::retry::DigestAlgorithm {
        self.inner.algorithm()
    }
    /// Borrows the exact authorized plan and request association.
    #[must_use]
    pub fn subject(&self) -> RobotFailoverPlanSubject<'_, '_, '_, R> {
        RobotFailoverPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Builds an exact Robot failover mutation plan in caller storage.
pub fn build_robot_failover_canonical_plan<'output, 'plan, 'storage, 'request, R>(
    plan: RobotFailoverPlanConfirmation<'plan, 'storage, 'request, R>,
    output: &'output mut [u8],
) -> Result<
    RobotFailoverCanonicalPlanFingerprint<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<core::convert::Infallible>,
>
where
    R: RobotFailoverPermitRequest,
{
    let inner = cloud_sdk::operation::build_canonical_plan(plan.inner, output)?;
    Ok(RobotFailoverCanonicalPlanFingerprint {
        inner,
        binding: plan.binding,
    })
}

/// Builds a strong Robot failover mutation digest and clears scratch storage.
pub fn build_robot_failover_plan_digest<'output, 'plan, 'storage, 'request, R, H>(
    plan: RobotFailoverPlanConfirmation<'plan, 'storage, 'request, R>,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<
    RobotFailoverPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<H::Error>,
>
where
    R: RobotFailoverPermitRequest,
    H: FingerprintHasher,
{
    let inner = cloud_sdk::operation::build_plan_digest(plan.inner, scratch, output, hasher)?;
    Ok(RobotFailoverPlanFingerprintDigest {
        inner,
        binding: plan.binding,
    })
}

/// Opaque request-bound Robot failover plan subject.
pub struct RobotFailoverPlanSubject<'storage, 'fingerprint, 'request, R: RobotFailoverPermitRequest>
{
    pub(super) inner: PlanSubject<'storage, 'fingerprint>,
    pub(super) binding: FailoverBinding<'request, R>,
}

impl<R: RobotFailoverPermitRequest> Clone for RobotFailoverPlanSubject<'_, '_, '_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R: RobotFailoverPermitRequest> Copy for RobotFailoverPlanSubject<'_, '_, '_, R> {}

impl<R: RobotFailoverPermitRequest> fmt::Debug for RobotFailoverPlanSubject<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotFailoverPlanSubject([redacted])")
    }
}

/// One in-flight Robot failover attempt retaining exact response provenance.
#[must_use]
pub struct RobotFailoverPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R> {
    pub(super) inner: cloud_sdk::operation::PermitAttempt<'permit, 'storage, 'fingerprint>,
    pub(super) binding: FailoverBinding<'request, R>,
}

impl<'permit, 'storage, 'fingerprint, 'request, R>
    RobotFailoverPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R>
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
    ) -> Result<CheckedRobotFailover<'buffer, 'request, R>, PermitExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
    {
        let binding = self.binding;
        self.inner
            .execute_blocking(clock, transport, body, headers)
            .map(|inner| CheckedRobotFailover::from_executed(binding.0, inner))
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
        Output = Result<CheckedRobotFailover<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
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
                .map(|inner| CheckedRobotFailover::from_executed(binding.0, inner))
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
        Output = Result<CheckedRobotFailover<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
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
                .map(|inner| CheckedRobotFailover::from_executed(binding.0, inner))
        }
    }
}

impl<R> fmt::Debug for RobotFailoverPermitAttempt<'_, '_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotFailoverPermitAttempt([redacted])")
    }
}
