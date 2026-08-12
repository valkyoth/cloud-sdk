//! Plan-confirm execution authority for state-changing operations.

mod clock;
mod direct;
mod execution_error;
mod fingerprint;
mod shared;
mod state;

pub use clock::PermitClock;
pub use direct::{CostPermit, DestructivePermit, MutationPermit};
pub use execution_error::PermitExecutionError;
pub use fingerprint::{
    CanonicalPlanFingerprint, PlanAuthorizationEvidence, PlanConfirmation,
    PlanFingerprintBuildError, PlanFingerprintDigest, PlanFingerprintRef, PlanSubject,
    build_canonical_plan, build_plan_digest, build_plan_digest_with_authorization_evidence,
};
pub use shared::{
    SharedCostPermit, SharedDestructivePermit, SharedMutationPermit, SharedPermitState,
};
pub use state::PermitAttempt;

use core::fmt;

use subtle::{Choice, ConstantTimeEq};

/// Maximum bytes admitted for each account, tenant, or permit-scope value.
pub const MAX_PLAN_SCOPE_BYTES: usize = 1024;
/// Minimum caller-provided idempotency identity length.
pub const MIN_PERMIT_IDEMPOTENCY_BYTES: usize = 16;
/// Maximum caller-provided idempotency identity length.
pub const MAX_PERMIT_IDEMPOTENCY_BYTES: usize = 64;

/// Runtime execution authority required by an operation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PermitScope {
    /// Non-destructive state mutation without a known direct charge.
    Mutation,
    /// Destructive or disabling state mutation.
    Destructive,
    /// Mutation that may directly incur provider charges.
    Cost,
}

/// Caller assessment of whether the planned request changes effective state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanChange {
    /// The request would make no effective change and must not be authorized.
    NoOp,
    /// The caller confirmed an effective state change.
    ChangesState,
}

/// Repetition policy selected during plan confirmation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayPolicy {
    /// Exactly one attempt is authorized.
    SingleAttempt,
    /// A proven `NotSent` attempt may be recovered and repeated.
    RecoverNotSent,
    /// Uncertain delivery may repeat only after operation-specific reconciliation.
    ReconcileThenRetry,
}

/// Invalid attempt budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttemptBudgetError {
    /// At least one attempt is required.
    Zero,
}

impl_static_error!(AttemptBudgetError, Self::Zero => "permit attempt budget must be nonzero");

/// Nonzero maximum number of attempts sharing one authority.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct AttemptBudget(u16);

impl AttemptBudget {
    /// Creates a nonzero attempt budget.
    pub const fn new(value: u16) -> Result<Self, AttemptBudgetError> {
        if value == 0 {
            return Err(AttemptBudgetError::Zero);
        }
        Ok(Self(value))
    }

    /// Returns the total attempt bound.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// Caller-observed wall-clock timestamp in Unix seconds.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PermitTimestamp(u64);

impl PermitTimestamp {
    /// Wraps caller-provided Unix seconds without acquiring a clock.
    #[must_use]
    pub const fn from_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    /// Returns Unix seconds.
    #[must_use]
    pub const fn as_seconds(self) -> u64 {
        self.0
    }
}

/// Invalid bounded permit-validity interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermitValidityError {
    /// Expiry must be later than issuance.
    Empty,
    /// The interval exceeds the shared-state representation.
    TooLong,
}

impl_static_error!(PermitValidityError,
    Self::Empty => "permit expiry must follow issuance",
    Self::TooLong => "permit validity interval is too long",
);

/// Caller-owned issuance and expiry observations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PermitValidity {
    issued_at: PermitTimestamp,
    expires_at: PermitTimestamp,
    duration: u32,
}

impl PermitValidity {
    /// Creates a bounded interval suitable for direct and atomic shared state.
    pub fn new(
        issued_at: PermitTimestamp,
        expires_at: PermitTimestamp,
    ) -> Result<Self, PermitValidityError> {
        let duration = expires_at
            .0
            .checked_sub(issued_at.0)
            .ok_or(PermitValidityError::Empty)?;
        if duration == 0 {
            return Err(PermitValidityError::Empty);
        }
        let duration = u32::try_from(duration).map_err(|_| PermitValidityError::TooLong)?;
        Ok(Self {
            issued_at,
            expires_at,
            duration,
        })
    }

    /// Returns the issuance timestamp.
    #[must_use]
    pub const fn issued_at(self) -> PermitTimestamp {
        self.issued_at
    }

    /// Returns the expiry timestamp.
    #[must_use]
    pub const fn expires_at(self) -> PermitTimestamp {
        self.expires_at
    }

    pub(crate) fn offset(self, now: PermitTimestamp) -> Result<u32, ExecutionPermitError> {
        let elapsed = now
            .0
            .checked_sub(self.issued_at.0)
            .ok_or(ExecutionPermitError::NotYetValid)?;
        if elapsed >= u64::from(self.duration) {
            return Err(ExecutionPermitError::Expired);
        }
        u32::try_from(elapsed).map_err(|_| ExecutionPermitError::Expired)
    }
}

/// Invalid three-letter currency code.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrencyCodeError {
    /// Currency codes must be exactly three ASCII letters.
    Invalid,
}

impl_static_error!(CurrencyCodeError, Self::Invalid => "currency code must be three uppercase ASCII letters");

/// Exact ISO-style uppercase currency code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyCode([u8; 3]);

impl CurrencyCode {
    /// Validates an exact uppercase three-letter code.
    pub fn new(value: &str) -> Result<Self, CurrencyCodeError> {
        let bytes: [u8; 3] = value
            .as_bytes()
            .try_into()
            .map_err(|_| CurrencyCodeError::Invalid)?;
        if !bytes.iter().all(u8::is_ascii_uppercase) {
            return Err(CurrencyCodeError::Invalid);
        }
        Ok(Self(bytes))
    }

    /// Returns exact code bytes.
    #[must_use]
    pub const fn as_bytes(self) -> [u8; 3] {
        self.0
    }
}

/// Invalid cost confirmation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanCostError {
    /// An observed price must be nonzero for cost authority.
    ZeroObservedPrice,
    /// The observed price exceeds the caller's spending ceiling.
    SpendingCeilingExceeded,
}

impl_static_error!(PlanCostError,
    Self::ZeroObservedPrice => "observed price must be nonzero",
    Self::SpendingCeilingExceeded => "observed price exceeds spending ceiling",
);

/// Exact caller-observed price and spending ceiling in common scaled units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanCost {
    currency: CurrencyCode,
    scale: u8,
    observed_units: u128,
    ceiling_units: u128,
}

impl PlanCost {
    /// Creates one exact bounded cost confirmation.
    pub fn new(
        currency: CurrencyCode,
        scale: u8,
        observed_units: u128,
        ceiling_units: u128,
    ) -> Result<Self, PlanCostError> {
        if observed_units == 0 {
            return Err(PlanCostError::ZeroObservedPrice);
        }
        if observed_units > ceiling_units {
            return Err(PlanCostError::SpendingCeilingExceeded);
        }
        Ok(Self {
            currency,
            scale,
            observed_units,
            ceiling_units,
        })
    }

    pub(crate) const fn fields(self) -> (CurrencyCode, u8, u128, u128) {
        (
            self.currency,
            self.scale,
            self.observed_units,
            self.ceiling_units,
        )
    }
}

/// Invalid account, tenant, or permit-context value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermitContextError {
    /// Permit context must not be empty.
    Empty,
    /// Permit context exceeds its fixed policy bound.
    TooLong,
}

impl_static_error!(PermitContextError,
    Self::Empty => "permit context is empty",
    Self::TooLong => "permit context exceeds the length limit",
);

/// Exact bounded context binding a permit to caller policy.
#[derive(Clone, Copy)]
pub struct PermitContext<'a>(&'a [u8]);

impl<'a> PermitContext<'a> {
    /// Validates a nonempty bounded context.
    pub fn new(value: &'a [u8]) -> Result<Self, PermitContextError> {
        if value.is_empty() {
            return Err(PermitContextError::Empty);
        }
        if value.len() > MAX_PLAN_SCOPE_BYTES {
            return Err(PermitContextError::TooLong);
        }
        Ok(Self(value))
    }

    pub(crate) const fn bytes(self) -> &'a [u8] {
        self.0
    }
}

impl fmt::Debug for PermitContext<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PermitContext([redacted])")
    }
}

/// Optional exact account or tenant identity.
#[derive(Clone, Copy, Debug)]
pub enum PlanFingerprintScope<'a> {
    /// No identity applies in this dimension.
    Absent,
    /// Exact bounded identity bytes.
    Value(&'a [u8]),
}

impl<'a> PlanFingerprintScope<'a> {
    pub(crate) fn bytes(self) -> Result<Option<&'a [u8]>, PermitContextError> {
        match self {
            Self::Absent => Ok(None),
            Self::Value([]) => Err(PermitContextError::Empty),
            Self::Value(value) if value.len() > MAX_PLAN_SCOPE_BYTES => {
                Err(PermitContextError::TooLong)
            }
            Self::Value(value) => Ok(Some(value)),
        }
    }
}

/// Invalid caller-provided repetition identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermitIdempotencyKeyError {
    /// Identity has too few entropy-bearing bytes.
    TooShort,
    /// Identity exceeds the fixed policy bound.
    TooLong,
    /// An all-zero identity is rejected.
    AllZero,
}

impl_static_error!(PermitIdempotencyKeyError,
    Self::TooShort => "permit idempotency identity is too short",
    Self::TooLong => "permit idempotency identity is too long",
    Self::AllZero => "permit idempotency identity cannot be all zero",
);

/// Borrowed caller-owned identity used only for exact reconciliation matching.
#[derive(Clone, Copy)]
pub struct PermitIdempotencyKey<'a>(&'a [u8]);

impl<'a> PermitIdempotencyKey<'a> {
    /// Validates identity shape. Entropy quality remains a caller duty.
    pub fn new(value: &'a [u8]) -> Result<Self, PermitIdempotencyKeyError> {
        if value.len() < MIN_PERMIT_IDEMPOTENCY_BYTES {
            return Err(PermitIdempotencyKeyError::TooShort);
        }
        if value.len() > MAX_PERMIT_IDEMPOTENCY_BYTES {
            return Err(PermitIdempotencyKeyError::TooLong);
        }
        let mut nonzero = Choice::from(0);
        for byte in value {
            nonzero |= !byte.ct_eq(&0);
        }
        if !bool::from(nonzero) {
            return Err(PermitIdempotencyKeyError::AllZero);
        }
        Ok(Self(value))
    }

    pub(crate) const fn bytes(self) -> &'a [u8] {
        self.0
    }

    pub(crate) fn matches(self, other: Self) -> bool {
        self.0.len() == other.0.len() && bool::from(self.0.ct_eq(other.0))
    }
}

impl fmt::Debug for PermitIdempotencyKey<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PermitIdempotencyKey([redacted])")
    }
}

/// Observable permit lifecycle state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermitState {
    /// Ready to authorize one attempt.
    Ready,
    /// One caller currently owns the attempt.
    InFlight,
    /// A proven-not-sent attempt awaits generation-bound recovery.
    Recoverable,
    /// Delivery may have happened and operation-specific reconciliation is required.
    PendingReconciliation,
    /// Authority is permanently spent.
    Spent,
}

/// Generation-bound recovery authority returned only for `NotSent`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecoveryToken(pub(crate) u16);

/// Generation-bound reconciliation authority returned after uncertain delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReconciliationToken(pub(crate) u16);

/// State transition produced by one completed or failed attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermitDisposition {
    /// Authority is permanently spent.
    Spent,
    /// A proven-not-sent attempt may be explicitly recovered.
    Recoverable(RecoveryToken),
    /// Operation-specific reconciliation is required.
    PendingReconciliation(ReconciliationToken),
}

/// Permit authorization or lifecycle failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionPermitError {
    /// The plan scope does not match the permit type.
    ScopeMismatch,
    /// The permit is not valid yet.
    NotYetValid,
    /// The permit has expired.
    Expired,
    /// Caller time moved backward.
    ClockRollback,
    /// Another caller owns the only in-flight attempt.
    AttemptInFlight,
    /// The permit needs explicit not-sent recovery.
    RecoveryRequired,
    /// The permit needs operation-specific reconciliation.
    ReconciliationRequired,
    /// The authority or attempt budget is spent.
    Spent,
    /// Recovery or reconciliation belongs to another generation.
    StaleGeneration,
    /// The supplied request fingerprint differs from the confirmed plan.
    FingerprintMismatch,
    /// Reconciliation used a different idempotency identity.
    IdempotencyMismatch,
    /// The selected replay policy forbids this transition.
    ReplayForbidden,
    /// Atomic generation capacity is exhausted.
    GenerationExhausted,
}

impl_static_error!(ExecutionPermitError,
    Self::ScopeMismatch => "execution permit scope does not match the plan",
    Self::NotYetValid => "execution permit is not valid yet",
    Self::Expired => "execution permit has expired",
    Self::ClockRollback => "execution permit clock moved backward",
    Self::AttemptInFlight => "execution permit already has an in-flight attempt",
    Self::RecoveryRequired => "execution permit requires not-sent recovery",
    Self::ReconciliationRequired => "execution permit requires reconciliation",
    Self::Spent => "execution permit is spent",
    Self::StaleGeneration => "execution permit generation is stale",
    Self::FingerprintMismatch => "execution permit fingerprint does not match",
    Self::IdempotencyMismatch => "execution permit idempotency identity does not match",
    Self::ReplayForbidden => "execution permit replay policy forbids repetition",
    Self::GenerationExhausted => "execution permit generation is exhausted",
);

#[cfg(test)]
mod tests;
