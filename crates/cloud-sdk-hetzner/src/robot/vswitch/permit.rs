//! Request-bound execution permits for Robot vSwitch mutations.

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

use super::{CheckedRobotVSwitch, PreparedRobotVSwitch};

pub use direct::{RobotVSwitchDestructivePermit, RobotVSwitchMutationPermit};
pub use shared::{RobotVSwitchSharedDestructivePermit, RobotVSwitchSharedMutationPermit};

mod sealed {
    pub trait Sealed {}
}

/// Sealed marker for Robot vSwitch operations requiring execution authority.
pub trait RobotVSwitchPermitRequest: sealed::Sealed {}

macro_rules! permit_request {
    ($($type:ty),+ $(,)?) => {$ (
        impl sealed::Sealed for $type {}
        impl RobotVSwitchPermitRequest for $type {}
    )+ };
}

permit_request!(
    super::RobotVSwitchCreateRequest,
    super::RobotVSwitchUpdateRequest,
    super::RobotVSwitchCancelRequest,
    super::RobotVSwitchAddServersRequest<'_>,
    super::RobotVSwitchRemoveServersRequest<'_>,
);

pub(super) struct VSwitchBinding<'request, R>(pub(super) &'request R);

impl<R> Clone for VSwitchBinding<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R> Copy for VSwitchBinding<'_, R> {}

/// Exact Robot vSwitch mutation plus caller policy ready for fingerprinting.
pub struct RobotVSwitchPlanConfirmation<'plan, 'storage, 'request, R: RobotVSwitchPermitRequest> {
    inner: PlanConfirmation<'plan, 'storage>,
    binding: VSwitchBinding<'request, R>,
}

impl<'plan, 'storage, 'request, R: RobotVSwitchPermitRequest>
    RobotVSwitchPlanConfirmation<'plan, 'storage, 'request, R>
{
    /// Binds caller policy to the exact mutation request.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        prepared: PreparedRobotVSwitch<'storage, 'request, R>,
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
            binding: VSwitchBinding(request),
        }
    }
}

impl<R: RobotVSwitchPermitRequest> fmt::Debug for RobotVSwitchPlanConfirmation<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotVSwitchPlanConfirmation")
            .field("plan", &self.inner)
            .field("request", &"[bound]")
            .finish()
    }
}

/// Caller-buffer plan fingerprint retaining exact vSwitch request provenance.
pub struct RobotVSwitchCanonicalPlanFingerprint<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotVSwitchPermitRequest,
> {
    inner: CanonicalPlanFingerprint<'output, 'plan, 'storage>,
    binding: VSwitchBinding<'request, R>,
}

impl<R: RobotVSwitchPermitRequest> RobotVSwitchCanonicalPlanFingerprint<'_, '_, '_, '_, R> {
    /// Borrows the exact authorized plan and request association.
    #[must_use]
    pub fn subject(&self) -> RobotVSwitchPlanSubject<'_, '_, '_, R> {
        RobotVSwitchPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Strong-digest plan fingerprint retaining exact vSwitch request provenance.
pub struct RobotVSwitchPlanFingerprintDigest<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotVSwitchPermitRequest,
> {
    inner: PlanFingerprintDigest<'output, 'plan, 'storage>,
    binding: VSwitchBinding<'request, R>,
}

impl<R: RobotVSwitchPermitRequest> RobotVSwitchPlanFingerprintDigest<'_, '_, '_, '_, R> {
    /// Returns the admitted collision-resistant digest algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> cloud_sdk::retry::DigestAlgorithm {
        self.inner.algorithm()
    }
    /// Borrows the exact authorized plan and request association.
    #[must_use]
    pub fn subject(&self) -> RobotVSwitchPlanSubject<'_, '_, '_, R> {
        RobotVSwitchPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Builds an exact Robot vSwitch mutation plan in caller storage.
pub fn build_robot_vswitch_canonical_plan<'output, 'plan, 'storage, 'request, R>(
    plan: RobotVSwitchPlanConfirmation<'plan, 'storage, 'request, R>,
    output: &'output mut [u8],
) -> Result<
    RobotVSwitchCanonicalPlanFingerprint<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<core::convert::Infallible>,
>
where
    R: RobotVSwitchPermitRequest,
{
    let inner = cloud_sdk::operation::build_canonical_plan(plan.inner, output)?;
    Ok(RobotVSwitchCanonicalPlanFingerprint {
        inner,
        binding: plan.binding,
    })
}

/// Builds a strong Robot vSwitch mutation digest and clears scratch storage.
pub fn build_robot_vswitch_plan_digest<'output, 'plan, 'storage, 'request, R, H>(
    plan: RobotVSwitchPlanConfirmation<'plan, 'storage, 'request, R>,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<
    RobotVSwitchPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<H::Error>,
>
where
    R: RobotVSwitchPermitRequest,
    H: FingerprintHasher,
{
    let inner = cloud_sdk::operation::build_plan_digest(plan.inner, scratch, output, hasher)?;
    Ok(RobotVSwitchPlanFingerprintDigest {
        inner,
        binding: plan.binding,
    })
}

/// Opaque request-bound Robot vSwitch plan subject.
pub struct RobotVSwitchPlanSubject<'storage, 'fingerprint, 'request, R: RobotVSwitchPermitRequest> {
    pub(super) inner: PlanSubject<'storage, 'fingerprint>,
    pub(super) binding: VSwitchBinding<'request, R>,
}

impl<R: RobotVSwitchPermitRequest> Clone for RobotVSwitchPlanSubject<'_, '_, '_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R: RobotVSwitchPermitRequest> Copy for RobotVSwitchPlanSubject<'_, '_, '_, R> {}

impl<R: RobotVSwitchPermitRequest> fmt::Debug for RobotVSwitchPlanSubject<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotVSwitchPlanSubject([redacted])")
    }
}

/// One in-flight Robot vSwitch attempt retaining exact response provenance.
#[must_use]
pub struct RobotVSwitchPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R> {
    pub(super) inner: cloud_sdk::operation::PermitAttempt<'permit, 'storage, 'fingerprint>,
    pub(super) binding: VSwitchBinding<'request, R>,
}

impl<'permit, 'storage, 'fingerprint, 'request, R>
    RobotVSwitchPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R>
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
    ) -> Result<CheckedRobotVSwitch<'buffer, 'request, R>, PermitExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
    {
        let binding = self.binding;
        self.inner
            .execute_blocking(clock, transport, body, headers)
            .map(|inner| CheckedRobotVSwitch::from_executed(binding.0, inner))
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
        Output = Result<CheckedRobotVSwitch<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
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
                .map(|inner| CheckedRobotVSwitch::from_executed(binding.0, inner))
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
        Output = Result<CheckedRobotVSwitch<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
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
                .map(|inner| CheckedRobotVSwitch::from_executed(binding.0, inner))
        }
    }
}

impl<R> fmt::Debug for RobotVSwitchPermitAttempt<'_, '_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotVSwitchPermitAttempt([redacted])")
    }
}
