//! Request-bound execution permits for Robot IP mutations.

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

use super::{CheckedRobotIp, PreparedRobotIp};

pub use direct::{RobotIpDestructivePermit, RobotIpMutationPermit};
pub use shared::{RobotIpSharedDestructivePermit, RobotIpSharedMutationPermit};

mod sealed {
    pub trait Sealed {}
}

/// Sealed marker for the three Robot IP operations requiring execution authority.
pub trait RobotIpPermitRequest: sealed::Sealed {}

macro_rules! permit_request {
    ($($type:ty),+ $(,)?) => {$ (
        impl sealed::Sealed for $type {}
        impl RobotIpPermitRequest for $type {}
    )+ };
}

permit_request!(
    super::RobotIpUpdateRequest,
    super::RobotIpMacSetRequest,
    super::RobotIpMacDeleteRequest,
);

pub(super) struct IpBinding<'request, R>(pub(super) &'request R);

impl<R> Clone for IpBinding<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R> Copy for IpBinding<'_, R> {}

/// Exact Robot IP mutation plus caller policy ready for fingerprinting.
pub struct RobotIpPlanConfirmation<'plan, 'storage, 'request, R: RobotIpPermitRequest> {
    inner: PlanConfirmation<'plan, 'storage>,
    binding: IpBinding<'request, R>,
}

impl<'plan, 'storage, 'request, R: RobotIpPermitRequest>
    RobotIpPlanConfirmation<'plan, 'storage, 'request, R>
{
    /// Binds caller policy to the exact mutation request.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        prepared: PreparedRobotIp<'storage, 'request, R>,
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
            binding: IpBinding(request),
        }
    }
}

impl<R: RobotIpPermitRequest> fmt::Debug for RobotIpPlanConfirmation<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotIpPlanConfirmation")
            .field("plan", &self.inner)
            .field("request", &"[bound]")
            .finish()
    }
}

/// Caller-buffer plan fingerprint retaining exact Robot IP request provenance.
pub struct RobotIpCanonicalPlanFingerprint<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotIpPermitRequest,
> {
    inner: CanonicalPlanFingerprint<'output, 'plan, 'storage>,
    binding: IpBinding<'request, R>,
}

impl<R: RobotIpPermitRequest> RobotIpCanonicalPlanFingerprint<'_, '_, '_, '_, R> {
    /// Borrows the exact authorized plan and request association.
    #[must_use]
    pub fn subject(&self) -> RobotIpPlanSubject<'_, '_, '_, R> {
        RobotIpPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Strong-digest plan fingerprint retaining exact Robot IP request provenance.
pub struct RobotIpPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R: RobotIpPermitRequest>
{
    inner: PlanFingerprintDigest<'output, 'plan, 'storage>,
    binding: IpBinding<'request, R>,
}

impl<R: RobotIpPermitRequest> RobotIpPlanFingerprintDigest<'_, '_, '_, '_, R> {
    /// Returns the admitted collision-resistant digest algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> cloud_sdk::retry::DigestAlgorithm {
        self.inner.algorithm()
    }
    /// Borrows the exact authorized plan and request association.
    #[must_use]
    pub fn subject(&self) -> RobotIpPlanSubject<'_, '_, '_, R> {
        RobotIpPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Builds an exact Robot IP mutation plan in caller storage.
pub fn build_robot_ip_canonical_plan<'output, 'plan, 'storage, 'request, R>(
    plan: RobotIpPlanConfirmation<'plan, 'storage, 'request, R>,
    output: &'output mut [u8],
) -> Result<
    RobotIpCanonicalPlanFingerprint<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<core::convert::Infallible>,
>
where
    R: RobotIpPermitRequest,
{
    let inner = cloud_sdk::operation::build_canonical_plan(plan.inner, output)?;
    Ok(RobotIpCanonicalPlanFingerprint {
        inner,
        binding: plan.binding,
    })
}

/// Builds a strong Robot IP mutation digest and clears scratch storage.
pub fn build_robot_ip_plan_digest<'output, 'plan, 'storage, 'request, R, H>(
    plan: RobotIpPlanConfirmation<'plan, 'storage, 'request, R>,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<
    RobotIpPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<H::Error>,
>
where
    R: RobotIpPermitRequest,
    H: FingerprintHasher,
{
    let inner = cloud_sdk::operation::build_plan_digest(plan.inner, scratch, output, hasher)?;
    Ok(RobotIpPlanFingerprintDigest {
        inner,
        binding: plan.binding,
    })
}

/// Opaque request-bound Robot IP plan subject.
pub struct RobotIpPlanSubject<'storage, 'fingerprint, 'request, R: RobotIpPermitRequest> {
    pub(super) inner: PlanSubject<'storage, 'fingerprint>,
    pub(super) binding: IpBinding<'request, R>,
}

impl<R: RobotIpPermitRequest> Clone for RobotIpPlanSubject<'_, '_, '_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R: RobotIpPermitRequest> Copy for RobotIpPlanSubject<'_, '_, '_, R> {}

impl<R: RobotIpPermitRequest> fmt::Debug for RobotIpPlanSubject<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotIpPlanSubject([redacted])")
    }
}

/// One in-flight Robot IP attempt retaining exact response provenance.
#[must_use]
pub struct RobotIpPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R> {
    pub(super) inner: cloud_sdk::operation::PermitAttempt<'permit, 'storage, 'fingerprint>,
    pub(super) binding: IpBinding<'request, R>,
}

impl<'permit, 'storage, 'fingerprint, 'request, R>
    RobotIpPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R>
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
    ) -> Result<CheckedRobotIp<'buffer, 'request, R>, PermitExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
    {
        let binding = self.binding;
        self.inner
            .execute_blocking(clock, transport, body, headers)
            .map(|inner| CheckedRobotIp::from_executed(binding.0, inner))
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
        Output = Result<CheckedRobotIp<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
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
                .map(|inner| CheckedRobotIp::from_executed(binding.0, inner))
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
        Output = Result<CheckedRobotIp<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
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
                .map(|inner| CheckedRobotIp::from_executed(binding.0, inner))
        }
    }
}

impl<R> fmt::Debug for RobotIpPermitAttempt<'_, '_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotIpPermitAttempt([redacted])")
    }
}
