//! Request-bound execution permits for Robot subnet mutations.

mod direct;
mod shared;

use core::fmt;

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, LocalAsyncAuthenticatedTransport,
};
use cloud_sdk::operation::{
    AttemptBudget, CanonicalPlanFingerprint, PermitClock, PermitContext, PermitDisposition,
    PermitExecutionError, PermitIdempotencyKey, PermitValidity, PlanAuthorizationEvidence,
    PlanChange, PlanConfirmation, PlanCost, PlanFingerprintBuildError, PlanFingerprintDigest,
    PlanFingerprintScope, PlanSubject, ReplayPolicy,
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
pub trait RobotSubnetPermitRequest: sealed::Sealed + PlanAuthorizationEvidence {
    /// Whether the request carries non-wire authorization evidence.
    const HAS_SENSITIVE_AUTHORIZATION_EVIDENCE: bool;

    /// Rejects stale evidence before authority is created or consumed.
    fn validate_authorization_evidence(
        &self,
        now: cloud_sdk::operation::PermitTimestamp,
    ) -> Result<(), cloud_sdk::operation::ExecutionPermitError>;
}

macro_rules! ordinary_permit_request {
    ($($type:ty),+ $(,)?) => {$ (
        impl sealed::Sealed for $type {}
        impl PlanAuthorizationEvidence for $type {
            fn encode<E: Copy>(
                &self,
                _writer: &mut cloud_sdk::buffer::SnapshotEncoder<
                    '_,
                    PlanFingerprintBuildError<E>,
                >,
            ) -> Result<(), PlanFingerprintBuildError<E>> {
                Ok(())
            }
        }
        impl RobotSubnetPermitRequest for $type {
            const HAS_SENSITIVE_AUTHORIZATION_EVIDENCE: bool = false;

            fn validate_authorization_evidence(
                &self,
                _now: cloud_sdk::operation::PermitTimestamp,
            ) -> Result<(), cloud_sdk::operation::ExecutionPermitError> {
                Ok(())
            }
        }
    )+ };
}

ordinary_permit_request!(
    super::RobotSubnetUpdateRequest,
    super::RobotSubnetMacSetRequest,
);

impl sealed::Sealed for super::RobotSubnetMacDeleteRequest {}

impl PlanAuthorizationEvidence for super::RobotSubnetMacDeleteRequest {
    fn valid_until(&self) -> Option<cloud_sdk::operation::PermitTimestamp> {
        let evidence = self.observations.fields().2;
        Some(core::cmp::min(evidence, self.mutation_lease.expires_at()))
    }

    fn encode<E: Copy>(
        &self,
        writer: &mut cloud_sdk::buffer::SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    ) -> Result<(), PlanFingerprintBuildError<E>> {
        writer.bytes(b"hetzner/robot/subnet-delete-evidence/v1\0")?;
        self.expected_server
            .with_text(|server| encode_evidence_field(writer, 1, server.as_bytes()))?;
        self.expected_default_mac
            .try_with_text(|mac| encode_evidence_field(writer, 2, mac.as_bytes()))
            .map_err(|_| PlanFingerprintBuildError::InputTooLarge)??;
        let (subnet_at, mac_at, evidence_expiry) = self.observations.fields();
        encode_evidence_field(writer, 3, &subnet_at.as_seconds().to_be_bytes())?;
        encode_evidence_field(writer, 4, &mac_at.as_seconds().to_be_bytes())?;
        encode_evidence_field(writer, 5, &evidence_expiry.as_seconds().to_be_bytes())?;
        self.mutation_lease
            .with_identity(|identity| encode_evidence_field(writer, 6, identity))?;
        encode_evidence_field(
            writer,
            7,
            &self.mutation_lease.expires_at().as_seconds().to_be_bytes(),
        )
    }
}

impl RobotSubnetPermitRequest for super::RobotSubnetMacDeleteRequest {
    const HAS_SENSITIVE_AUTHORIZATION_EVIDENCE: bool = true;

    fn validate_authorization_evidence(
        &self,
        now: cloud_sdk::operation::PermitTimestamp,
    ) -> Result<(), cloud_sdk::operation::ExecutionPermitError> {
        self.observations.validate_at(now)?;
        self.mutation_lease.validate_at(now)
    }
}

fn encode_evidence_field<E: Copy>(
    writer: &mut cloud_sdk::buffer::SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    tag: u8,
    bytes: &[u8],
) -> Result<(), PlanFingerprintBuildError<E>> {
    writer.byte(tag)?;
    let len = u64::try_from(bytes.len()).map_err(|_| PlanFingerprintBuildError::InputTooLarge)?;
    writer.bytes(&len.to_be_bytes())?;
    writer.bytes(bytes)
}

pub(super) struct SubnetBinding<'request, R>(pub(super) &'request R);

impl<R> Clone for SubnetBinding<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R> Copy for SubnetBinding<'_, R> {}

struct SampledPermitClock(cloud_sdk::operation::PermitTimestamp);

impl PermitClock for SampledPermitClock {
    fn now(&self) -> cloud_sdk::operation::PermitTimestamp {
        self.0
    }
}

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
    if R::HAS_SENSITIVE_AUTHORIZATION_EVIDENCE {
        cloud_sdk_sanitization::sanitize_bytes(output);
        return Err(PlanFingerprintBuildError::SensitiveAuthorizationEvidenceRequiresDigest);
    }
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
    let inner = if R::HAS_SENSITIVE_AUTHORIZATION_EVIDENCE {
        cloud_sdk::operation::build_plan_digest_with_authorization_evidence(
            plan.inner,
            plan.binding.0,
            scratch,
            output,
            hasher,
        )?
    } else {
        cloud_sdk::operation::build_plan_digest(plan.inner, scratch, output, hasher)?
    };
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

impl<'permit, 'storage, 'fingerprint, 'request, R: RobotSubnetPermitRequest>
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
        cloud_sdk_sanitization::sanitize_bytes(body);
        cloud_sdk_sanitization::sanitize_bytes(headers);
        let now = clock.now();
        if let Err(error) = binding.0.validate_authorization_evidence(now) {
            return Err(self.inner.reject_authorization(error, body, headers));
        }
        self.inner
            .execute_blocking(&SampledPermitClock(now), transport, body, headers)
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
        cloud_sdk_sanitization::sanitize_bytes(body);
        cloud_sdk_sanitization::sanitize_bytes(headers);
        async move {
            let now = clock.now();
            if let Err(error) = binding.0.validate_authorization_evidence(now) {
                return Err(self.inner.reject_authorization(error, body, headers));
            }
            self.inner
                .execute_async(&SampledPermitClock(now), transport, body, headers)
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
        cloud_sdk_sanitization::sanitize_bytes(body);
        cloud_sdk_sanitization::sanitize_bytes(headers);
        async move {
            let now = clock.now();
            if let Err(error) = binding.0.validate_authorization_evidence(now) {
                return Err(self.inner.reject_authorization(error, body, headers));
            }
            self.inner
                .execute_local_async(&SampledPermitClock(now), transport, body, headers)
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
