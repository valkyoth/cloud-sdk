//! Request-bound execution permits for Robot firewall mutations.

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

use super::{CheckedRobotFirewall, PreparedRobotFirewall};

pub use direct::{RobotFirewallDestructivePermit, RobotFirewallMutationPermit};
pub use shared::{RobotFirewallSharedDestructivePermit, RobotFirewallSharedMutationPermit};

mod sealed {
    pub trait Sealed {}
}

/// Sealed marker for Robot firewall operations requiring execution authority.
pub trait RobotFirewallPermitRequest: sealed::Sealed {}

macro_rules! permit_request {
    ($($type:ty),+ $(,)?) => {$ (
        impl sealed::Sealed for $type {}
        impl RobotFirewallPermitRequest for $type {}
    )+ };
}

permit_request!(
    super::RobotFirewallReplaceRequest<'_>,
    super::RobotFirewallDeleteRequest,
    super::RobotFirewallTemplateCreateRequest<'_>,
    super::RobotFirewallTemplateUpdateRequest<'_>,
    super::RobotFirewallTemplateDeleteRequest,
);

pub(super) struct FirewallBinding<'request, R>(pub(super) &'request R);

impl<R> Clone for FirewallBinding<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R> Copy for FirewallBinding<'_, R> {}

/// Exact Robot firewall mutation plus caller policy ready for fingerprinting.
pub struct RobotFirewallPlanConfirmation<'plan, 'storage, 'request, R: RobotFirewallPermitRequest> {
    inner: PlanConfirmation<'plan, 'storage>,
    binding: FirewallBinding<'request, R>,
}

impl<'plan, 'storage, 'request, R: RobotFirewallPermitRequest>
    RobotFirewallPlanConfirmation<'plan, 'storage, 'request, R>
{
    /// Binds caller policy to the exact mutation request.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        prepared: PreparedRobotFirewall<'storage, 'request, R>,
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
            binding: FirewallBinding(request),
        }
    }
}

impl<R: RobotFirewallPermitRequest> fmt::Debug for RobotFirewallPlanConfirmation<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotFirewallPlanConfirmation")
            .field("plan", &self.inner)
            .field("request", &"[bound]")
            .finish()
    }
}

/// Caller-buffer plan fingerprint retaining exact Robot firewall request provenance.
pub struct RobotFirewallCanonicalPlanFingerprint<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotFirewallPermitRequest,
> {
    inner: CanonicalPlanFingerprint<'output, 'plan, 'storage>,
    binding: FirewallBinding<'request, R>,
}

impl<R: RobotFirewallPermitRequest> RobotFirewallCanonicalPlanFingerprint<'_, '_, '_, '_, R> {
    /// Borrows the exact authorized plan and request association.
    #[must_use]
    pub fn subject(&self) -> RobotFirewallPlanSubject<'_, '_, '_, R> {
        RobotFirewallPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Strong-digest plan fingerprint retaining exact Robot firewall request provenance.
pub struct RobotFirewallPlanFingerprintDigest<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotFirewallPermitRequest,
> {
    inner: PlanFingerprintDigest<'output, 'plan, 'storage>,
    binding: FirewallBinding<'request, R>,
}

impl<R: RobotFirewallPermitRequest> RobotFirewallPlanFingerprintDigest<'_, '_, '_, '_, R> {
    /// Returns the admitted collision-resistant digest algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> cloud_sdk::retry::DigestAlgorithm {
        self.inner.algorithm()
    }
    /// Borrows the exact authorized plan and request association.
    #[must_use]
    pub fn subject(&self) -> RobotFirewallPlanSubject<'_, '_, '_, R> {
        RobotFirewallPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Builds an exact Robot firewall mutation plan in caller storage.
pub fn build_robot_firewall_canonical_plan<'output, 'plan, 'storage, 'request, R>(
    plan: RobotFirewallPlanConfirmation<'plan, 'storage, 'request, R>,
    output: &'output mut [u8],
) -> Result<
    RobotFirewallCanonicalPlanFingerprint<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<core::convert::Infallible>,
>
where
    R: RobotFirewallPermitRequest,
{
    let inner = cloud_sdk::operation::build_canonical_plan(plan.inner, output)?;
    Ok(RobotFirewallCanonicalPlanFingerprint {
        inner,
        binding: plan.binding,
    })
}

/// Builds a strong Robot firewall mutation digest and clears scratch storage.
pub fn build_robot_firewall_plan_digest<'output, 'plan, 'storage, 'request, R, H>(
    plan: RobotFirewallPlanConfirmation<'plan, 'storage, 'request, R>,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<
    RobotFirewallPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<H::Error>,
>
where
    R: RobotFirewallPermitRequest,
    H: FingerprintHasher,
{
    let inner = cloud_sdk::operation::build_plan_digest(plan.inner, scratch, output, hasher)?;
    Ok(RobotFirewallPlanFingerprintDigest {
        inner,
        binding: plan.binding,
    })
}

/// Opaque request-bound Robot firewall plan subject.
pub struct RobotFirewallPlanSubject<'storage, 'fingerprint, 'request, R: RobotFirewallPermitRequest>
{
    pub(super) inner: PlanSubject<'storage, 'fingerprint>,
    pub(super) binding: FirewallBinding<'request, R>,
}

impl<R: RobotFirewallPermitRequest> Clone for RobotFirewallPlanSubject<'_, '_, '_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R: RobotFirewallPermitRequest> Copy for RobotFirewallPlanSubject<'_, '_, '_, R> {}

impl<R: RobotFirewallPermitRequest> fmt::Debug for RobotFirewallPlanSubject<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotFirewallPlanSubject([redacted])")
    }
}

/// One in-flight Robot firewall attempt retaining exact response provenance.
#[must_use]
pub struct RobotFirewallPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R> {
    pub(super) inner: cloud_sdk::operation::PermitAttempt<'permit, 'storage, 'fingerprint>,
    pub(super) binding: FirewallBinding<'request, R>,
}

impl<'permit, 'storage, 'fingerprint, 'request, R>
    RobotFirewallPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R>
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
    ) -> Result<CheckedRobotFirewall<'buffer, 'request, R>, PermitExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
    {
        let binding = self.binding;
        self.inner
            .execute_blocking(clock, transport, body, headers)
            .map(|inner| CheckedRobotFirewall::from_executed(binding.0, inner))
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
        Output = Result<CheckedRobotFirewall<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
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
                .map(|inner| CheckedRobotFirewall::from_executed(binding.0, inner))
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
        Output = Result<CheckedRobotFirewall<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
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
                .map(|inner| CheckedRobotFirewall::from_executed(binding.0, inner))
        }
    }
}

impl<R> fmt::Debug for RobotFirewallPermitAttempt<'_, '_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotFirewallPermitAttempt([redacted])")
    }
}
