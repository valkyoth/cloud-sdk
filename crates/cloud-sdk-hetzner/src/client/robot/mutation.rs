//! Request-bound permits for Robot mutations without specialized evidence.

mod direct;
mod execution;
mod shared;

use core::fmt;

use cloud_sdk::operation::{
    AttemptBudget, CanonicalPlanFingerprint, PermitContext, PermitDisposition,
    PermitIdempotencyKey, PermitValidity, PlanChange, PlanConfirmation, PlanCost,
    PlanFingerprintBuildError, PlanFingerprintDigest, PlanFingerprintScope, PlanSubject,
    PreparationStorageGuard, PrepareOperation, PreparedRequest, ReplayPolicy,
};
use cloud_sdk::retry::FingerprintHasher;
use cloud_sdk::transport::{DeliveryPhase, EndpointIdentity};

use super::operation::RobotClientOperation;

pub use direct::{RobotMutationDestructivePermit, RobotMutationPermit};
pub use execution::RobotMutationClientExecutionError;
pub use shared::{RobotMutationSharedDestructivePermit, RobotMutationSharedPermit};

mod private {
    pub trait Sealed {}
}

/// Sealed Robot operation admitted by the generic mutation permit family.
///
/// Operations requiring stronger provider-specific evidence retain their
/// dedicated permit families and cannot implement this trait externally.
#[allow(private_bounds)]
pub trait RobotClientMutationOperation:
    RobotClientOperation + PrepareOperation + private::Sealed
{
}

macro_rules! mutation_operation {
    ($($type:ty),+ $(,)?) => {$ (
        impl private::Sealed for $type {}
        impl RobotClientMutationOperation for $type {}
    )+ };
}

mutation_operation!(
    crate::robot::RobotServerUpdateRequest<'_>,
    crate::robot::RobotRescueActivateRequest<'_>,
    crate::robot::RobotRescueDeactivateRequest,
    crate::robot::RobotLinuxActivateRequest<'_>,
    crate::robot::RobotLinuxDeactivateRequest,
    crate::robot::RobotVncActivateRequest<'_>,
    crate::robot::RobotVncDeactivateRequest,
    crate::robot::RobotWindowsActivateRequest<'_>,
    crate::robot::RobotWindowsDeactivateRequest,
);

pub(super) struct MutationBinding<'request, R>(pub(super) &'request R);

impl<R> Clone for MutationBinding<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R> Copy for MutationBinding<'_, R> {}

/// Guard-prepared Robot mutation retaining exact request provenance.
pub struct PreparedRobotClientMutation<'storage, 'request, R: RobotClientMutationOperation> {
    inner: PreparedRequest<'storage>,
    binding: MutationBinding<'request, R>,
}

impl<'storage, R: RobotClientMutationOperation> PreparedRobotClientMutation<'storage, '_, R> {
    /// Borrows the provider-neutral request for policy inspection.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }
}

impl<R: RobotClientMutationOperation> fmt::Debug for PreparedRobotClientMutation<'_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedRobotClientMutation")
            .field("request", &"[bound]")
            .field("prepared", &self.inner)
            .finish()
    }
}

/// Prepares one generic Robot mutation in cleanup-owning storage.
pub fn prepare_robot_client_mutation<'storage, 'request, R>(
    request: &'request R,
    storage: &'storage mut PreparationStorageGuard<'_>,
) -> Result<PreparedRobotClientMutation<'storage, 'request, R>, R::Error>
where
    R: RobotClientMutationOperation,
{
    let inner = storage.prepare(request)?;
    Ok(PreparedRobotClientMutation {
        inner,
        binding: MutationBinding(request),
    })
}

/// Exact generic Robot mutation plus caller policy ready for fingerprinting.
pub struct RobotMutationPlanConfirmation<'plan, 'storage, 'request, R>
where
    R: RobotClientMutationOperation,
{
    inner: PlanConfirmation<'plan, 'storage>,
    binding: MutationBinding<'request, R>,
}

impl<'plan, 'storage, 'request, R> RobotMutationPlanConfirmation<'plan, 'storage, 'request, R>
where
    R: RobotClientMutationOperation,
{
    /// Binds caller policy to one exact guarded mutation request.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        prepared: PreparedRobotClientMutation<'storage, 'request, R>,
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
        Self {
            inner: PlanConfirmation::new(
                prepared.inner,
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
            binding: prepared.binding,
        }
    }
}

impl<R: RobotClientMutationOperation> fmt::Debug for RobotMutationPlanConfirmation<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotMutationPlanConfirmation")
            .field("plan", &self.inner)
            .field("request", &"[bound]")
            .finish()
    }
}

/// Caller-buffer exact fingerprint retaining Robot request provenance.
pub struct RobotMutationCanonicalPlanFingerprint<'output, 'plan, 'storage, 'request, R>
where
    R: RobotClientMutationOperation,
{
    inner: CanonicalPlanFingerprint<'output, 'plan, 'storage>,
    binding: MutationBinding<'request, R>,
}

impl<R: RobotClientMutationOperation> RobotMutationCanonicalPlanFingerprint<'_, '_, '_, '_, R> {
    /// Borrows the exact request-bound subject.
    #[must_use]
    pub fn subject(&self) -> RobotMutationPlanSubject<'_, '_, '_, R> {
        RobotMutationPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Strong-digest fingerprint retaining Robot request provenance.
pub struct RobotMutationPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R>
where
    R: RobotClientMutationOperation,
{
    inner: PlanFingerprintDigest<'output, 'plan, 'storage>,
    binding: MutationBinding<'request, R>,
}

impl<R: RobotClientMutationOperation> RobotMutationPlanFingerprintDigest<'_, '_, '_, '_, R> {
    /// Returns the admitted digest algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> cloud_sdk::retry::DigestAlgorithm {
        self.inner.algorithm()
    }

    /// Borrows the exact request-bound subject.
    #[must_use]
    pub fn subject(&self) -> RobotMutationPlanSubject<'_, '_, '_, R> {
        RobotMutationPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Builds an exact generic Robot mutation plan in caller storage.
pub fn build_robot_mutation_canonical_plan<'output, 'plan, 'storage, 'request, R>(
    plan: RobotMutationPlanConfirmation<'plan, 'storage, 'request, R>,
    output: &'output mut [u8],
) -> Result<
    RobotMutationCanonicalPlanFingerprint<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<core::convert::Infallible>,
>
where
    R: RobotClientMutationOperation,
{
    let inner = cloud_sdk::operation::build_canonical_plan(plan.inner, output)?;
    Ok(RobotMutationCanonicalPlanFingerprint {
        inner,
        binding: plan.binding,
    })
}

/// Builds a strong generic Robot mutation digest and clears scratch storage.
pub fn build_robot_mutation_plan_digest<'output, 'plan, 'storage, 'request, R, H>(
    plan: RobotMutationPlanConfirmation<'plan, 'storage, 'request, R>,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<
    RobotMutationPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<H::Error>,
>
where
    R: RobotClientMutationOperation,
    H: FingerprintHasher,
{
    let inner = cloud_sdk::operation::build_plan_digest(plan.inner, scratch, output, hasher)?;
    Ok(RobotMutationPlanFingerprintDigest {
        inner,
        binding: plan.binding,
    })
}

/// Opaque exact Robot mutation plan subject.
pub struct RobotMutationPlanSubject<'storage, 'fingerprint, 'request, R>
where
    R: RobotClientMutationOperation,
{
    pub(super) inner: PlanSubject<'storage, 'fingerprint>,
    pub(super) binding: MutationBinding<'request, R>,
}

impl<R: RobotClientMutationOperation> Clone for RobotMutationPlanSubject<'_, '_, '_, R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: RobotClientMutationOperation> Copy for RobotMutationPlanSubject<'_, '_, '_, R> {}

impl<R: RobotClientMutationOperation> fmt::Debug for RobotMutationPlanSubject<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotMutationPlanSubject([redacted])")
    }
}

/// One in-flight generic Robot mutation retaining its exact request.
#[must_use]
pub struct RobotMutationPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R> {
    pub(super) inner: cloud_sdk::operation::PermitAttempt<'permit, 'storage, 'fingerprint>,
    pub(super) binding: MutationBinding<'request, R>,
}

impl<R> RobotMutationPermitAttempt<'_, '_, '_, '_, R> {
    /// Completes a manually driven attempt conservatively.
    pub fn complete(self, phase: DeliveryPhase) -> PermitDisposition {
        self.inner.complete(phase)
    }
}

impl<R> fmt::Debug for RobotMutationPermitAttempt<'_, '_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotMutationPermitAttempt([redacted])")
    }
}
