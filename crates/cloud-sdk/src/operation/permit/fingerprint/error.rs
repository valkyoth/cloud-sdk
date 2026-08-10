//! Payload-redacting plan-confirm construction errors.

use core::fmt;

use crate::operation::PermitContextError;

/// Plan-confirm construction failure.
#[derive(Clone, Copy)]
pub enum PlanFingerprintBuildError<E> {
    /// Read-only requests do not accept state-change permits.
    ReadOnlyOperation,
    /// The prepared request has no provider operation identifier.
    MissingOperationId,
    /// The endpoint is outside the provider-owned trust policy.
    EndpointNotAdmitted,
    /// Caller marked the plan as an ineffective no-op.
    NoOp,
    /// An account, tenant, or context value is invalid.
    Context(PermitContextError),
    /// Cost metadata is required by operation policy.
    MissingCost,
    /// Cost metadata was supplied for a no-known-cost operation.
    UnexpectedCost,
    /// Single-attempt policy must have an exact budget of one.
    InvalidSingleAttemptBudget,
    /// Reconciliation policy requires an idempotency identity.
    MissingIdempotency,
    /// Identity is forbidden when reconciliation is not admitted.
    UnexpectedIdempotency,
    /// Canonical length arithmetic overflowed or exceeded the hard bound.
    InputTooLarge,
    /// Caller output cannot hold the complete canonical input or digest.
    OutputTooSmall,
    /// Sensitive request bodies may only use collision-resistant digests.
    SensitiveBodyRequiresDigest,
    /// Caller-provided collision-resistant hashing failed.
    Hasher(E),
    /// Hasher output length differs from its admitted algorithm.
    InvalidDigestLength,
}

impl<E> fmt::Debug for PlanFingerprintBuildError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ReadOnlyOperation => "PlanFingerprintBuildError::ReadOnlyOperation",
            Self::MissingOperationId => "PlanFingerprintBuildError::MissingOperationId",
            Self::EndpointNotAdmitted => "PlanFingerprintBuildError::EndpointNotAdmitted",
            Self::NoOp => "PlanFingerprintBuildError::NoOp",
            Self::Context(_) => "PlanFingerprintBuildError::Context",
            Self::MissingCost => "PlanFingerprintBuildError::MissingCost",
            Self::UnexpectedCost => "PlanFingerprintBuildError::UnexpectedCost",
            Self::InvalidSingleAttemptBudget => {
                "PlanFingerprintBuildError::InvalidSingleAttemptBudget"
            }
            Self::MissingIdempotency => "PlanFingerprintBuildError::MissingIdempotency",
            Self::UnexpectedIdempotency => "PlanFingerprintBuildError::UnexpectedIdempotency",
            Self::InputTooLarge => "PlanFingerprintBuildError::InputTooLarge",
            Self::OutputTooSmall => "PlanFingerprintBuildError::OutputTooSmall",
            Self::SensitiveBodyRequiresDigest => {
                "PlanFingerprintBuildError::SensitiveBodyRequiresDigest"
            }
            Self::Hasher(_) => "PlanFingerprintBuildError::Hasher([redacted])",
            Self::InvalidDigestLength => "PlanFingerprintBuildError::InvalidDigestLength",
        })
    }
}

impl<E> fmt::Display for PlanFingerprintBuildError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ReadOnlyOperation => "read-only operation does not accept an execution permit",
            Self::MissingOperationId => "plan confirmation requires an operation identifier",
            Self::EndpointNotAdmitted => "plan confirmation endpoint is not admitted",
            Self::NoOp => "no-op plan cannot receive execution authority",
            Self::Context(_) => "plan confirmation context is invalid",
            Self::MissingCost => "cost-bearing plan requires price confirmation",
            Self::UnexpectedCost => "no-known-cost plan cannot include price authority",
            Self::InvalidSingleAttemptBudget => "single-attempt plan must authorize one attempt",
            Self::MissingIdempotency => "reconciliation replay requires idempotency identity",
            Self::UnexpectedIdempotency => "idempotency identity requires reconciliation replay",
            Self::InputTooLarge => "canonical plan confirmation exceeds its size limit",
            Self::OutputTooSmall => "plan confirmation output is too small",
            Self::SensitiveBodyRequiresDigest => {
                "sensitive request body requires a collision-resistant plan digest"
            }
            Self::Hasher(_) => "plan confirmation hashing failed",
            Self::InvalidDigestLength => "plan confirmation digest length is invalid",
        })
    }
}

impl<E: core::error::Error + 'static> core::error::Error for PlanFingerprintBuildError<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Context(error) => Some(error),
            Self::Hasher(error) => Some(error),
            _ => None,
        }
    }
}

pub(super) fn map_infallible<E>(
    error: PlanFingerprintBuildError<core::convert::Infallible>,
) -> PlanFingerprintBuildError<E> {
    match error {
        PlanFingerprintBuildError::ReadOnlyOperation => {
            PlanFingerprintBuildError::ReadOnlyOperation
        }
        PlanFingerprintBuildError::MissingOperationId => {
            PlanFingerprintBuildError::MissingOperationId
        }
        PlanFingerprintBuildError::EndpointNotAdmitted => {
            PlanFingerprintBuildError::EndpointNotAdmitted
        }
        PlanFingerprintBuildError::NoOp => PlanFingerprintBuildError::NoOp,
        PlanFingerprintBuildError::Context(error) => PlanFingerprintBuildError::Context(error),
        PlanFingerprintBuildError::MissingCost => PlanFingerprintBuildError::MissingCost,
        PlanFingerprintBuildError::UnexpectedCost => PlanFingerprintBuildError::UnexpectedCost,
        PlanFingerprintBuildError::InvalidSingleAttemptBudget => {
            PlanFingerprintBuildError::InvalidSingleAttemptBudget
        }
        PlanFingerprintBuildError::MissingIdempotency => {
            PlanFingerprintBuildError::MissingIdempotency
        }
        PlanFingerprintBuildError::UnexpectedIdempotency => {
            PlanFingerprintBuildError::UnexpectedIdempotency
        }
        PlanFingerprintBuildError::InputTooLarge => PlanFingerprintBuildError::InputTooLarge,
        PlanFingerprintBuildError::OutputTooSmall => PlanFingerprintBuildError::OutputTooSmall,
        PlanFingerprintBuildError::SensitiveBodyRequiresDigest => {
            PlanFingerprintBuildError::SensitiveBodyRequiresDigest
        }
        PlanFingerprintBuildError::InvalidDigestLength => {
            PlanFingerprintBuildError::InvalidDigestLength
        }
        PlanFingerprintBuildError::Hasher(never) => match never {},
    }
}
