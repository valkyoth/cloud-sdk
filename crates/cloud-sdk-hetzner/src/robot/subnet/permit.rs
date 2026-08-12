//! Request-bound execution permits for Robot subnet mutations.

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

use super::{CheckedRobotSubnet, PreparedRobotSubnet};

pub use direct::{RobotSubnetDestructivePermit, RobotSubnetMutationPermit};
pub use shared::{RobotSubnetSharedDestructivePermit, RobotSubnetSharedMutationPermit};

mod sealed {
    pub trait Sealed {}
}

/// Sealed marker for the three Robot subnet operations requiring execution authority.
pub trait RobotSubnetPermitRequest: sealed::Sealed {}

macro_rules! permit_request {
    ($($type:ty),+ $(,)?) => {$ (
        impl sealed::Sealed for $type {}
        impl RobotSubnetPermitRequest for $type {}
    )+ };
}

permit_request!(
    super::RobotSubnetUpdateRequest,
    super::RobotSubnetMacSetRequest,
    super::RobotSubnetMacDeleteRequest,
);

pub(super) struct SubnetBinding<'request, R>(pub(super) &'request R);

impl<R> Clone for SubnetBinding<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R> Copy for SubnetBinding<'_, R> {}

/// Exact Robot subnet mutation plus caller policy ready for fingerprinting.
pub struct RobotSubnetPlanConfirmation<'plan, 'storage, 'request, R: RobotSubnetPermitRequest> {
    inner: PlanConfirmation<'plan, 'storage>,
    binding: SubnetBinding<'request, R>,
}

impl<'plan, 'storage, 'request, R: RobotSubnetPermitRequest>
    RobotSubnetPlanConfirmation<'plan, 'storage, 'request, R>
{
    /// Binds caller policy to the exact mutation request.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        prepared: PreparedRobotSubnet<'storage, 'request, R>,
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
            binding: SubnetBinding(request),
        }
    }
}

impl<R: RobotSubnetPermitRequest> fmt::Debug for RobotSubnetPlanConfirmation<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotSubnetPlanConfirmation")
            .field("plan", &self.inner)
            .field("request", &"[bound]")
            .finish()
    }
}

/// Caller-buffer plan fingerprint retaining exact Robot subnet request provenance.
pub struct RobotSubnetCanonicalPlanFingerprint<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotSubnetPermitRequest,
> {
    inner: CanonicalPlanFingerprint<'output, 'plan, 'storage>,
    binding: SubnetBinding<'request, R>,
}

impl<R: RobotSubnetPermitRequest> RobotSubnetCanonicalPlanFingerprint<'_, '_, '_, '_, R> {
    /// Borrows the exact authorized plan and request association.
    #[must_use]
    pub fn subject(&self) -> RobotSubnetPlanSubject<'_, '_, '_, R> {
        RobotSubnetPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Strong-digest plan fingerprint retaining exact Robot subnet request provenance.
pub struct RobotSubnetPlanFingerprintDigest<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotSubnetPermitRequest,
> {
    inner: PlanFingerprintDigest<'output, 'plan, 'storage>,
    binding: SubnetBinding<'request, R>,
}

impl<R: RobotSubnetPermitRequest> RobotSubnetPlanFingerprintDigest<'_, '_, '_, '_, R> {
    /// Returns the admitted collision-resistant digest algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> cloud_sdk::retry::DigestAlgorithm {
        self.inner.algorithm()
    }
    /// Borrows the exact authorized plan and request association.
    #[must_use]
    pub fn subject(&self) -> RobotSubnetPlanSubject<'_, '_, '_, R> {
        RobotSubnetPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Builds an exact Robot subnet mutation plan in caller storage.
pub fn build_robot_subnet_canonical_plan<'output, 'plan, 'storage, 'request, R>(
    plan: RobotSubnetPlanConfirmation<'plan, 'storage, 'request, R>,
    output: &'output mut [u8],
) -> Result<
    RobotSubnetCanonicalPlanFingerprint<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<core::convert::Infallible>,
>
where
    R: RobotSubnetPermitRequest,
{
    let inner = cloud_sdk::operation::build_canonical_plan(plan.inner, output)?;
    Ok(RobotSubnetCanonicalPlanFingerprint {
        inner,
        binding: plan.binding,
    })
}

/// Builds a strong Robot subnet mutation digest and clears scratch storage.
pub fn build_robot_subnet_plan_digest<'output, 'plan, 'storage, 'request, R, H>(
    plan: RobotSubnetPlanConfirmation<'plan, 'storage, 'request, R>,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<
    RobotSubnetPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<H::Error>,
>
where
    R: RobotSubnetPermitRequest,
    H: FingerprintHasher,
{
    let inner = cloud_sdk::operation::build_plan_digest(plan.inner, scratch, output, hasher)?;
    Ok(RobotSubnetPlanFingerprintDigest {
        inner,
        binding: plan.binding,
    })
}

/// Opaque request-bound Robot subnet plan subject.
pub struct RobotSubnetPlanSubject<'storage, 'fingerprint, 'request, R: RobotSubnetPermitRequest> {
    pub(super) inner: PlanSubject<'storage, 'fingerprint>,
    pub(super) binding: SubnetBinding<'request, R>,
}

impl<R: RobotSubnetPermitRequest> Clone for RobotSubnetPlanSubject<'_, '_, '_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R: RobotSubnetPermitRequest> Copy for RobotSubnetPlanSubject<'_, '_, '_, R> {}

impl<R: RobotSubnetPermitRequest> fmt::Debug for RobotSubnetPlanSubject<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotSubnetPlanSubject([redacted])")
    }
}

/// One in-flight Robot subnet attempt retaining exact response provenance.
#[must_use]
pub struct RobotSubnetPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R> {
    pub(super) inner: cloud_sdk::operation::PermitAttempt<'permit, 'storage, 'fingerprint>,
    pub(super) binding: SubnetBinding<'request, R>,
}

impl<'permit, 'storage, 'fingerprint, 'request, R>
    RobotSubnetPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R>
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
    ) -> Result<CheckedRobotSubnet<'buffer, 'request, R>, PermitExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
    {
        let binding = self.binding;
        self.inner
            .execute_blocking(clock, transport, body, headers)
            .map(|inner| CheckedRobotSubnet::from_executed(binding.0, inner))
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
        Output = Result<CheckedRobotSubnet<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
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
                .map(|inner| CheckedRobotSubnet::from_executed(binding.0, inner))
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
        Output = Result<CheckedRobotSubnet<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
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
                .map(|inner| CheckedRobotSubnet::from_executed(binding.0, inner))
        }
    }
}

impl<R> fmt::Debug for RobotSubnetPermitAttempt<'_, '_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotSubnetPermitAttempt([redacted])")
    }
}
