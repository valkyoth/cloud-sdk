//! Versioned plan-confirm identity with exact or strong-digest comparison.

use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;
use subtle::ConstantTimeEq;

use super::{
    AttemptBudget, PermitContext, PermitIdempotencyKey, PermitScope, PermitValidity, PlanChange,
    PlanCost, PlanFingerprintScope, ReplayPolicy,
};
use crate::buffer::{encode_snapshot_bounded, measure_snapshot_bounded};
use crate::operation::{CostIntent, OperationImpact, PreparedRequest};
use crate::retry::{DigestAlgorithm, FingerprintHasher};
use crate::transport::EndpointIdentity;

mod encoding;
mod error;
use encoding::encode;
pub use error::PlanFingerprintBuildError;
use error::map_infallible;

const DOMAIN: &[u8] = b"cloud-sdk/plan-confirm/v1\0";
/// Maximum complete exact plan-confirm input.
pub const MAX_CANONICAL_PLAN_BYTES: usize = 16_777_216;

/// Complete immutable plan-confirm inputs.
#[derive(Clone, Copy)]
pub struct PlanConfirmation<'a, 'request> {
    prepared: PreparedRequest<'request>,
    endpoint: EndpointIdentity<'a>,
    account: PlanFingerprintScope<'a>,
    tenant: PlanFingerprintScope<'a>,
    context: PermitContext<'a>,
    validity: PermitValidity,
    replay: ReplayPolicy,
    attempts: AttemptBudget,
    change: PlanChange,
    cost: Option<PlanCost>,
    idempotency: Option<PermitIdempotencyKey<'a>>,
}

impl<'a, 'request> PlanConfirmation<'a, 'request> {
    /// Binds caller policy and current plan observations to one prepared request.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        prepared: PreparedRequest<'request>,
        endpoint: EndpointIdentity<'a>,
        account: PlanFingerprintScope<'a>,
        tenant: PlanFingerprintScope<'a>,
        context: PermitContext<'a>,
        validity: PermitValidity,
        replay: ReplayPolicy,
        attempts: AttemptBudget,
        change: PlanChange,
        cost: Option<PlanCost>,
        idempotency: Option<PermitIdempotencyKey<'a>>,
    ) -> Self {
        Self {
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
        }
    }
}

impl fmt::Debug for PlanConfirmation<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PlanConfirmation")
            .field("prepared", &self.prepared)
            .field("endpoint", &self.endpoint)
            .field("account", &"[redacted]")
            .field("tenant", &"[redacted]")
            .field("context", &"[redacted]")
            .field("validity", &self.validity)
            .field("replay", &self.replay)
            .field("attempts", &self.attempts)
            .field("change", &self.change)
            .field("cost", &self.cost)
            .field("idempotency", &"[redacted]")
            .finish()
    }
}

/// Caller-buffer exact plan-confirm input, cleared in full on drop.
pub struct CanonicalPlanFingerprint<'output, 'plan, 'request> {
    storage: &'output mut [u8],
    len: usize,
    plan: PlanConfirmation<'plan, 'request>,
    scope: PermitScope,
}

impl<'output, 'plan, 'request> CanonicalPlanFingerprint<'output, 'plan, 'request> {
    /// Returns a redacted exact comparison reference.
    #[must_use]
    pub fn as_ref(&self) -> PlanFingerprintRef<'_> {
        PlanFingerprintRef(PlanFingerprintKind::Exact(self.bytes()))
    }

    /// Returns the exact prepared request and confirmed authority metadata.
    #[must_use]
    pub fn subject(&self) -> PlanSubject<'request, '_> {
        subject(&self.plan, self.scope, self.as_ref())
    }

    /// Returns the complete canonical byte length without exposing bytes.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Reports whether the canonical input is empty. It is never empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }

    fn bytes(&self) -> &[u8] {
        self.storage.get(..self.len).unwrap_or_default()
    }
}

impl fmt::Debug for CanonicalPlanFingerprint<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CanonicalPlanFingerprint")
            .field("len", &self.len)
            .field("scope", &self.scope)
            .field("bytes", &"[redacted]")
            .finish()
    }
}

impl Drop for CanonicalPlanFingerprint<'_, '_, '_> {
    fn drop(&mut self) {
        sanitize_bytes(self.storage);
        self.len = 0;
    }
}

/// Caller-buffer strong digest of a plan confirmation, cleared on drop.
pub struct PlanFingerprintDigest<'output, 'plan, 'request> {
    algorithm: DigestAlgorithm,
    storage: &'output mut [u8],
    len: usize,
    plan: PlanConfirmation<'plan, 'request>,
    scope: PermitScope,
}

impl PlanFingerprintDigest<'_, '_, '_> {
    /// Returns the admitted collision-resistant digest algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> DigestAlgorithm {
        self.algorithm
    }

    /// Returns a redacted strong-digest comparison reference.
    #[must_use]
    pub fn as_ref(&self) -> PlanFingerprintRef<'_> {
        PlanFingerprintRef(PlanFingerprintKind::Digest {
            algorithm: self.algorithm,
            bytes: self.storage.get(..self.len).unwrap_or_default(),
        })
    }

    /// Returns the exact request and confirmed authority metadata.
    #[must_use]
    pub fn subject(&self) -> PlanSubject<'_, '_> {
        subject(&self.plan, self.scope, self.as_ref())
    }
}

impl fmt::Debug for PlanFingerprintDigest<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PlanFingerprintDigest")
            .field("algorithm", &self.algorithm)
            .field("scope", &self.scope)
            .field("bytes", &"[redacted]")
            .finish()
    }
}

impl Drop for PlanFingerprintDigest<'_, '_, '_> {
    fn drop(&mut self) {
        sanitize_bytes(self.storage);
        self.len = 0;
    }
}

/// Borrowed exact or strong-digest plan identity.
#[derive(Clone, Copy)]
pub struct PlanFingerprintRef<'a>(PlanFingerprintKind<'a>);

#[derive(Clone, Copy)]
enum PlanFingerprintKind<'a> {
    Exact(&'a [u8]),
    Digest {
        algorithm: DigestAlgorithm,
        bytes: &'a [u8],
    },
}

impl PlanFingerprintRef<'_> {
    pub(crate) fn matches(self, other: Self) -> bool {
        match (self.0, other.0) {
            (PlanFingerprintKind::Exact(left), PlanFingerprintKind::Exact(right)) => {
                constant_time_eq(left, right)
            }
            (
                PlanFingerprintKind::Digest {
                    algorithm: left,
                    bytes: left_bytes,
                },
                PlanFingerprintKind::Digest {
                    algorithm: right,
                    bytes: right_bytes,
                },
            ) => left == right && constant_time_eq(left_bytes, right_bytes),
            _ => false,
        }
    }
}

impl fmt::Debug for PlanFingerprintRef<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PlanFingerprintRef([redacted])")
    }
}

/// One prepared request inseparably bound to a validated plan confirmation.
#[derive(Clone, Copy)]
pub struct PlanSubject<'request, 'fingerprint> {
    prepared: &'fingerprint PreparedRequest<'request>,
    fingerprint: PlanFingerprintRef<'fingerprint>,
    endpoint: EndpointIdentity<'fingerprint>,
    scope: PermitScope,
    validity: PermitValidity,
    replay: ReplayPolicy,
    attempts: AttemptBudget,
    idempotency: Option<PermitIdempotencyKey<'fingerprint>>,
}

impl<'request, 'fingerprint> PlanSubject<'request, 'fingerprint> {
    /// Returns the required permit scope.
    #[must_use]
    pub const fn scope(self) -> PermitScope {
        self.scope
    }

    /// Returns the caller-selected replay policy.
    #[must_use]
    pub const fn replay_policy(self) -> ReplayPolicy {
        self.replay
    }

    /// Returns the hard attempt budget.
    #[must_use]
    pub const fn attempt_budget(self) -> AttemptBudget {
        self.attempts
    }

    pub(crate) const fn endpoint(self) -> EndpointIdentity<'fingerprint> {
        self.endpoint
    }

    pub(crate) const fn prepared(self) -> PreparedRequest<'request> {
        *self.prepared
    }

    pub(crate) const fn fingerprint(self) -> PlanFingerprintRef<'fingerprint> {
        self.fingerprint
    }

    pub(crate) const fn validity(self) -> PermitValidity {
        self.validity
    }

    pub(crate) const fn idempotency(self) -> Option<PermitIdempotencyKey<'fingerprint>> {
        self.idempotency
    }
}

impl fmt::Debug for PlanSubject<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PlanSubject")
            .field("prepared", &self.prepared)
            .field("fingerprint", &"[redacted]")
            .field("endpoint", &self.endpoint)
            .field("scope", &self.scope)
            .field("validity", &self.validity)
            .field("replay", &self.replay)
            .field("attempts", &self.attempts)
            .field("idempotency", &"[redacted]")
            .finish()
    }
}

/// Builds exact canonical plan-confirm bytes into caller-owned storage.
pub fn build_canonical_plan<'output, 'plan, 'request>(
    plan: PlanConfirmation<'plan, 'request>,
    output: &'output mut [u8],
) -> Result<
    CanonicalPlanFingerprint<'output, 'plan, 'request>,
    PlanFingerprintBuildError<core::convert::Infallible>,
> {
    if plan.prepared.body_sensitivity().requires_digest() {
        sanitize_bytes(output);
        return Err(PlanFingerprintBuildError::SensitiveBodyRequiresDigest);
    }
    build_canonical_plan_inner(plan, output)
}

#[allow(
    clippy::large_types_passed_by_value,
    reason = "the returned fingerprint must own the complete confirmed plan"
)]
fn build_canonical_plan_inner<'output, 'plan, 'request>(
    plan: PlanConfirmation<'plan, 'request>,
    output: &'output mut [u8],
) -> Result<
    CanonicalPlanFingerprint<'output, 'plan, 'request>,
    PlanFingerprintBuildError<core::convert::Infallible>,
> {
    sanitize_bytes(output);
    let scope = validate(&plan)?;
    let required = measure_snapshot_bounded(
        &plan,
        MAX_CANONICAL_PLAN_BYTES,
        PlanFingerprintBuildError::InputTooLarge,
        encode,
    )?;
    if output.len() < required {
        return Err(PlanFingerprintBuildError::OutputTooSmall);
    }
    let len = encode_snapshot_bounded(
        &plan,
        output,
        MAX_CANONICAL_PLAN_BYTES,
        PlanFingerprintBuildError::InputTooLarge,
        encode,
    )?;
    Ok(CanonicalPlanFingerprint {
        storage: output,
        len,
        plan,
        scope,
    })
}

/// Builds a caller-selected collision-resistant digest and clears scratch.
pub fn build_plan_digest<'output, 'plan, 'request, H: FingerprintHasher>(
    plan: PlanConfirmation<'plan, 'request>,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<PlanFingerprintDigest<'output, 'plan, 'request>, PlanFingerprintBuildError<H::Error>> {
    sanitize_bytes(output);
    let exact = build_canonical_plan_inner(plan, scratch).map_err(map_infallible)?;
    let algorithm = hasher.algorithm();
    let expected = algorithm.output_len();
    if output.len() < expected {
        return Err(PlanFingerprintBuildError::OutputTooSmall);
    }
    let mut rollback = DigestRollback::new(output);
    let len = hasher
        .digest(exact.bytes(), rollback.target(expected))
        .map_err(PlanFingerprintBuildError::Hasher)?;
    if len != expected {
        return Err(PlanFingerprintBuildError::InvalidDigestLength);
    }
    let output = rollback.disarm();
    Ok(PlanFingerprintDigest {
        algorithm,
        storage: output,
        len,
        plan,
        scope: exact.scope,
    })
}

struct DigestRollback<'a> {
    output: &'a mut [u8],
    armed: bool,
}

impl<'a> DigestRollback<'a> {
    fn new(output: &'a mut [u8]) -> Self {
        Self {
            output,
            armed: true,
        }
    }

    fn target(&mut self, len: usize) -> &mut [u8] {
        self.output.get_mut(..len).unwrap_or_default()
    }

    fn disarm(mut self) -> &'a mut [u8] {
        self.armed = false;
        core::mem::take(&mut self.output)
    }
}

impl Drop for DigestRollback<'_> {
    fn drop(&mut self) {
        if self.armed {
            sanitize_bytes(self.output);
        }
    }
}

pub(super) fn validate<E>(
    plan: &PlanConfirmation<'_, '_>,
) -> Result<PermitScope, PlanFingerprintBuildError<E>> {
    if plan.prepared.operation_id().is_none() {
        return Err(PlanFingerprintBuildError::MissingOperationId);
    }
    if !plan
        .prepared
        .service()
        .endpoint_policy()
        .admits(plan.endpoint)
    {
        return Err(PlanFingerprintBuildError::EndpointNotAdmitted);
    }
    if plan.change == PlanChange::NoOp {
        return Err(PlanFingerprintBuildError::NoOp);
    }
    plan.account
        .bytes()
        .map_err(PlanFingerprintBuildError::Context)?;
    plan.tenant
        .bytes()
        .map_err(PlanFingerprintBuildError::Context)?;
    let metadata = plan.prepared.metadata();
    let scope = match (metadata.cost_intent(), metadata.impact()) {
        (CostIntent::MayIncurCost, _) => PermitScope::Cost,
        (_, OperationImpact::Destructive) => PermitScope::Destructive,
        (_, OperationImpact::Mutation) => PermitScope::Mutation,
        (_, OperationImpact::ReadOnly) => return Err(PlanFingerprintBuildError::ReadOnlyOperation),
    };
    match (scope, plan.cost) {
        (PermitScope::Cost, None) => return Err(PlanFingerprintBuildError::MissingCost),
        (PermitScope::Mutation | PermitScope::Destructive, Some(_)) => {
            return Err(PlanFingerprintBuildError::UnexpectedCost);
        }
        _ => {}
    }
    match (plan.replay, plan.attempts.get(), plan.idempotency) {
        (ReplayPolicy::SingleAttempt, 1, None)
        | (ReplayPolicy::RecoverNotSent, _, None)
        | (ReplayPolicy::ReconcileThenRetry, _, Some(_)) => {}
        (ReplayPolicy::SingleAttempt, _, _) => {
            return Err(PlanFingerprintBuildError::InvalidSingleAttemptBudget);
        }
        (ReplayPolicy::ReconcileThenRetry, _, None) => {
            return Err(PlanFingerprintBuildError::MissingIdempotency);
        }
        (_, _, Some(_)) => return Err(PlanFingerprintBuildError::UnexpectedIdempotency),
    }
    Ok(scope)
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len() && bool::from(left.ct_eq(right))
}

fn subject<'request, 'plan: 'fingerprint, 'fingerprint>(
    plan: &'fingerprint PlanConfirmation<'plan, 'request>,
    scope: PermitScope,
    fingerprint: PlanFingerprintRef<'fingerprint>,
) -> PlanSubject<'request, 'fingerprint> {
    PlanSubject {
        prepared: &plan.prepared,
        fingerprint,
        endpoint: plan.endpoint,
        scope,
        validity: plan.validity,
        replay: plan.replay,
        attempts: plan.attempts,
        idempotency: plan.idempotency,
    }
}
