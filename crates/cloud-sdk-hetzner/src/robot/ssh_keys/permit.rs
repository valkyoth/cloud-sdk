//! Request-bound execution permits for Robot SSH-key mutations.

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

use super::{CheckedRobotSshKey, PreparedRobotSshKey};

pub use direct::{RobotSshKeyDestructivePermit, RobotSshKeyMutationPermit};
pub use shared::{RobotSshKeySharedDestructivePermit, RobotSshKeySharedMutationPermit};

mod sealed {
    pub trait Sealed {}
}

/// Sealed marker for Robot SSH-key operations requiring execution authority.
pub trait RobotSshKeyPermitRequest: sealed::Sealed {}

macro_rules! permit_request {
    ($($type:ty),+ $(,)?) => {$ (
        impl sealed::Sealed for $type {}
        impl RobotSshKeyPermitRequest for $type {}
    )+ };
}

permit_request!(
    super::RobotSshKeyCreateRequest<'_>,
    super::RobotSshKeyUpdateRequest,
    super::RobotSshKeyDeleteRequest,
);

pub(super) struct SshKeyBinding<'request, R>(pub(super) &'request R);

impl<R> Clone for SshKeyBinding<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R> Copy for SshKeyBinding<'_, R> {}

/// Exact Robot SSH-key mutation plus caller policy ready for fingerprinting.
pub struct RobotSshKeyPlanConfirmation<'plan, 'storage, 'request, R: RobotSshKeyPermitRequest> {
    inner: PlanConfirmation<'plan, 'storage>,
    binding: SshKeyBinding<'request, R>,
}

impl<'plan, 'storage, 'request, R: RobotSshKeyPermitRequest>
    RobotSshKeyPlanConfirmation<'plan, 'storage, 'request, R>
{
    /// Binds caller policy to the exact mutation request.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        prepared: PreparedRobotSshKey<'storage, 'request, R>,
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
            binding: SshKeyBinding(request),
        }
    }
}

impl<R: RobotSshKeyPermitRequest> fmt::Debug for RobotSshKeyPlanConfirmation<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotSshKeyPlanConfirmation")
            .field("plan", &self.inner)
            .field("request", &"[bound]")
            .finish()
    }
}

/// Caller-buffer plan fingerprint retaining exact Robot SSH-key request provenance.
pub struct RobotSshKeyCanonicalPlanFingerprint<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotSshKeyPermitRequest,
> {
    inner: CanonicalPlanFingerprint<'output, 'plan, 'storage>,
    binding: SshKeyBinding<'request, R>,
}

impl<R: RobotSshKeyPermitRequest> RobotSshKeyCanonicalPlanFingerprint<'_, '_, '_, '_, R> {
    /// Borrows the exact authorized plan and request association.
    #[must_use]
    pub fn subject(&self) -> RobotSshKeyPlanSubject<'_, '_, '_, R> {
        RobotSshKeyPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Strong-digest plan fingerprint retaining exact Robot SSH-key request provenance.
pub struct RobotSshKeyPlanFingerprintDigest<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotSshKeyPermitRequest,
> {
    inner: PlanFingerprintDigest<'output, 'plan, 'storage>,
    binding: SshKeyBinding<'request, R>,
}

impl<R: RobotSshKeyPermitRequest> RobotSshKeyPlanFingerprintDigest<'_, '_, '_, '_, R> {
    /// Returns the admitted collision-resistant digest algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> cloud_sdk::retry::DigestAlgorithm {
        self.inner.algorithm()
    }
    /// Borrows the exact authorized plan and request association.
    #[must_use]
    pub fn subject(&self) -> RobotSshKeyPlanSubject<'_, '_, '_, R> {
        RobotSshKeyPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Builds an exact Robot SSH-key mutation plan in caller storage.
pub fn build_robot_ssh_key_canonical_plan<'output, 'plan, 'storage, 'request, R>(
    plan: RobotSshKeyPlanConfirmation<'plan, 'storage, 'request, R>,
    output: &'output mut [u8],
) -> Result<
    RobotSshKeyCanonicalPlanFingerprint<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<core::convert::Infallible>,
>
where
    R: RobotSshKeyPermitRequest,
{
    let inner = cloud_sdk::operation::build_canonical_plan(plan.inner, output)?;
    Ok(RobotSshKeyCanonicalPlanFingerprint {
        inner,
        binding: plan.binding,
    })
}

/// Builds a strong Robot SSH-key mutation digest and clears scratch storage.
pub fn build_robot_ssh_key_plan_digest<'output, 'plan, 'storage, 'request, R, H>(
    plan: RobotSshKeyPlanConfirmation<'plan, 'storage, 'request, R>,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<
    RobotSshKeyPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<H::Error>,
>
where
    R: RobotSshKeyPermitRequest,
    H: FingerprintHasher,
{
    let inner = cloud_sdk::operation::build_plan_digest(plan.inner, scratch, output, hasher)?;
    Ok(RobotSshKeyPlanFingerprintDigest {
        inner,
        binding: plan.binding,
    })
}

/// Opaque request-bound Robot SSH-key plan subject.
pub struct RobotSshKeyPlanSubject<'storage, 'fingerprint, 'request, R: RobotSshKeyPermitRequest> {
    pub(super) inner: PlanSubject<'storage, 'fingerprint>,
    pub(super) binding: SshKeyBinding<'request, R>,
}

impl<R: RobotSshKeyPermitRequest> Clone for RobotSshKeyPlanSubject<'_, '_, '_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R: RobotSshKeyPermitRequest> Copy for RobotSshKeyPlanSubject<'_, '_, '_, R> {}

impl<R: RobotSshKeyPermitRequest> fmt::Debug for RobotSshKeyPlanSubject<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotSshKeyPlanSubject([redacted])")
    }
}

/// One in-flight Robot SSH-key attempt retaining exact response provenance.
#[must_use]
pub struct RobotSshKeyPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R> {
    pub(super) inner: cloud_sdk::operation::PermitAttempt<'permit, 'storage, 'fingerprint>,
    pub(super) binding: SshKeyBinding<'request, R>,
}

impl<'permit, 'storage, 'fingerprint, 'request, R>
    RobotSshKeyPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R>
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
    ) -> Result<CheckedRobotSshKey<'buffer, 'request, R>, PermitExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
    {
        let binding = self.binding;
        self.inner
            .execute_blocking(clock, transport, body, headers)
            .map(|inner| CheckedRobotSshKey::from_executed(binding.0, inner))
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
        Output = Result<CheckedRobotSshKey<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
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
                .map(|inner| CheckedRobotSshKey::from_executed(binding.0, inner))
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
        Output = Result<CheckedRobotSshKey<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
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
                .map(|inner| CheckedRobotSshKey::from_executed(binding.0, inner))
        }
    }
}

impl<R> fmt::Debug for RobotSshKeyPermitAttempt<'_, '_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotSshKeyPermitAttempt([redacted])")
    }
}
