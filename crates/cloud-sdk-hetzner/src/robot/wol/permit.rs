//! Request-bound authority for one Wake-on-LAN mutation.

use core::fmt;

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, BoundCredentialTransport,
    LocalAsyncAuthenticatedTransport,
};
use cloud_sdk::operation::{
    AttemptBudget, CanonicalPlanFingerprint, ExecutionPermitError, PermitClock, PermitContext,
    PermitDisposition, PermitExecutionError, PermitIdempotencyKey, PermitTimestamp, PermitValidity,
    PlanAuthorizationEvidence, PlanChange, PlanConfirmation, PlanCost, PlanFingerprintBuildError,
    PlanFingerprintDigest, PlanFingerprintScope, PlanSubject, ReplayPolicy,
};
use cloud_sdk::retry::FingerprintHasher;
use cloud_sdk::transport::{BoundTransport, DeliveryClassified, DeliveryPhase, EndpointIdentity};

use super::{CheckedRobotWol, PreparedRobotWol, RobotWolSendRequest};

mod direct;
mod shared;

pub use direct::RobotWolMutationPermit;
pub use shared::RobotWolSharedMutationPermit;

mod sealed {
    pub trait Sealed {}
}

/// Sealed WOL request carrying authenticated capability evidence.
pub trait RobotWolPermitRequest: sealed::Sealed + PlanAuthorizationEvidence {
    /// Rechecks preflight freshness and credential lineage at dispatch.
    fn validate_authorization_evidence<T: BoundCredentialTransport>(
        &self,
        transport: &T,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError>;
}

impl sealed::Sealed for RobotWolSendRequest<'_> {}

impl PlanAuthorizationEvidence for RobotWolSendRequest<'_> {
    fn valid_until(&self) -> Option<PermitTimestamp> {
        Some(self.wol.expires_at())
    }

    fn encode<E: Copy>(
        &self,
        writer: &mut cloud_sdk::buffer::SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    ) -> Result<(), PlanFingerprintBuildError<E>> {
        writer.bytes(b"hetzner/robot/wol-evidence/v1\0")?;
        let wol = self.wol.wol();
        wol.with_server_ipv4(|address| encode_field(writer, 1, &address.octets()))?;
        wol.with_server_ipv6_network(|address| encode_field(writer, 2, &address.octets()))?;
        wol.server_number()
            .with_decimal_bytes(|number| encode_field(writer, 3, number))?;
        encode_field(writer, 4, b"send")?;
        encode_field(
            writer,
            5,
            &self.wol.observed_at().as_seconds().to_be_bytes(),
        )?;
        encode_field(writer, 6, &self.wol.expires_at().as_seconds().to_be_bytes())?;
        self.wol
            .credential()
            .with_bytes(|binding| encode_field(writer, 7, binding))
    }
}

impl RobotWolPermitRequest for RobotWolSendRequest<'_> {
    fn validate_authorization_evidence<T: BoundCredentialTransport>(
        &self,
        transport: &T,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.wol.validate_at(transport.credential_binding(), now)
    }
}

fn encode_field<E: Copy>(
    writer: &mut cloud_sdk::buffer::SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    tag: u8,
    bytes: &[u8],
) -> Result<(), PlanFingerprintBuildError<E>> {
    writer.byte(tag)?;
    let len = u64::try_from(bytes.len()).map_err(|_| PlanFingerprintBuildError::InputTooLarge)?;
    writer.bytes(&len.to_be_bytes())?;
    writer.bytes(bytes)
}

struct WolBinding<'request, R>(&'request R);

impl<R> Clone for WolBinding<'_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R> Copy for WolBinding<'_, R> {}

/// Exact WOL request plus caller policy ready for fingerprinting.
pub struct RobotWolPlanConfirmation<'plan, 'storage, 'request, R: RobotWolPermitRequest> {
    inner: PlanConfirmation<'plan, 'storage>,
    binding: WolBinding<'request, R>,
}

impl<'plan, 'storage, 'request, R: RobotWolPermitRequest>
    RobotWolPlanConfirmation<'plan, 'storage, 'request, R>
{
    /// Binds caller policy to one exact capability-checked wake request.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        prepared: PreparedRobotWol<'storage, 'request, R>,
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
            binding: WolBinding(request),
        }
    }
}

impl<R: RobotWolPermitRequest> fmt::Debug for RobotWolPlanConfirmation<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotWolPlanConfirmation([redacted])")
    }
}

/// Caller-buffer fingerprint retaining exact WOL request provenance.
pub struct RobotWolCanonicalPlanFingerprint<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotWolPermitRequest,
> {
    inner: CanonicalPlanFingerprint<'output, 'plan, 'storage>,
    binding: WolBinding<'request, R>,
}

impl<R: RobotWolPermitRequest> RobotWolCanonicalPlanFingerprint<'_, '_, '_, '_, R> {
    /// Borrows the exact plan and request association.
    #[must_use]
    pub fn subject(&self) -> RobotWolPlanSubject<'_, '_, '_, R> {
        RobotWolPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Strong-digest fingerprint retaining WOL authorization evidence.
pub struct RobotWolPlanFingerprintDigest<
    'output,
    'plan,
    'storage,
    'request,
    R: RobotWolPermitRequest,
> {
    inner: PlanFingerprintDigest<'output, 'plan, 'storage>,
    binding: WolBinding<'request, R>,
}

impl<R: RobotWolPermitRequest> RobotWolPlanFingerprintDigest<'_, '_, '_, '_, R> {
    /// Returns the admitted collision-resistant digest algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> cloud_sdk::retry::DigestAlgorithm {
        self.inner.algorithm()
    }

    /// Borrows the exact plan and request association.
    #[must_use]
    pub fn subject(&self) -> RobotWolPlanSubject<'_, '_, '_, R> {
        RobotWolPlanSubject {
            inner: self.inner.subject(),
            binding: self.binding,
        }
    }
}

/// Builds an exact WOL plan in caller-owned storage.
pub fn build_robot_wol_canonical_plan<'output, 'plan, 'storage, 'request, R>(
    plan: RobotWolPlanConfirmation<'plan, 'storage, 'request, R>,
    output: &'output mut [u8],
) -> Result<
    RobotWolCanonicalPlanFingerprint<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<core::convert::Infallible>,
>
where
    R: RobotWolPermitRequest,
{
    let inner = cloud_sdk::operation::build_canonical_plan(plan.inner, output)?;
    Ok(RobotWolCanonicalPlanFingerprint {
        inner,
        binding: plan.binding,
    })
}

/// Builds a strong WOL digest including capability authorization evidence.
pub fn build_robot_wol_plan_digest<'output, 'plan, 'storage, 'request, R, H>(
    plan: RobotWolPlanConfirmation<'plan, 'storage, 'request, R>,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<
    RobotWolPlanFingerprintDigest<'output, 'plan, 'storage, 'request, R>,
    PlanFingerprintBuildError<H::Error>,
>
where
    R: RobotWolPermitRequest,
    H: FingerprintHasher,
{
    let inner = cloud_sdk::operation::build_plan_digest_with_authorization_evidence(
        plan.inner,
        plan.binding.0,
        scratch,
        output,
        hasher,
    )?;
    Ok(RobotWolPlanFingerprintDigest {
        inner,
        binding: plan.binding,
    })
}

/// Opaque request-bound WOL plan subject.
pub struct RobotWolPlanSubject<'storage, 'fingerprint, 'request, R: RobotWolPermitRequest> {
    inner: PlanSubject<'storage, 'fingerprint>,
    binding: WolBinding<'request, R>,
}

impl<R: RobotWolPermitRequest> Clone for RobotWolPlanSubject<'_, '_, '_, R> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<R: RobotWolPermitRequest> Copy for RobotWolPlanSubject<'_, '_, '_, R> {}
impl<R: RobotWolPermitRequest> fmt::Debug for RobotWolPlanSubject<'_, '_, '_, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotWolPlanSubject([redacted])")
    }
}

/// One in-flight WOL attempt retaining exact response provenance.
#[must_use]
pub struct RobotWolPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R> {
    inner: cloud_sdk::operation::PermitAttempt<'permit, 'storage, 'fingerprint>,
    binding: WolBinding<'request, R>,
}

impl<'permit, 'storage, 'fingerprint, 'request, R: RobotWolPermitRequest>
    RobotWolPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R>
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
    ) -> Result<CheckedRobotWol<'buffer, 'request, R>, PermitExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
        T::Error: DeliveryClassified,
        C: PermitClock + ?Sized,
    {
        let binding = self.binding;
        cloud_sdk_sanitization::sanitize_bytes(body);
        cloud_sdk_sanitization::sanitize_bytes(headers);
        let now = clock.now();
        if let Err(error) = binding.0.validate_authorization_evidence(transport, now) {
            return Err(self.inner.reject_authorization(error, body, headers));
        }
        self.inner
            .execute_blocking(&FixedClock(now), transport, body, headers)
            .map(|inner| CheckedRobotWol::from_executed(binding.0, inner))
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
        Output = Result<CheckedRobotWol<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
    > + 'transport
    where
        T: AsyncAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
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
            if let Err(error) = binding.0.validate_authorization_evidence(transport, now) {
                return Err(self.inner.reject_authorization(error, body, headers));
            }
            self.inner
                .execute_async(&FixedClock(now), transport, body, headers)
                .await
                .map(|inner| CheckedRobotWol::from_executed(binding.0, inner))
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
        Output = Result<CheckedRobotWol<'buffer, 'request, R>, PermitExecutionError<T::Error>>,
    > + 'transport
    where
        T: LocalAsyncAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
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
            if let Err(error) = binding.0.validate_authorization_evidence(transport, now) {
                return Err(self.inner.reject_authorization(error, body, headers));
            }
            self.inner
                .execute_local_async(&FixedClock(now), transport, body, headers)
                .await
                .map(|inner| CheckedRobotWol::from_executed(binding.0, inner))
        }
    }
}

struct FixedClock(PermitTimestamp);
impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        self.0
    }
}
