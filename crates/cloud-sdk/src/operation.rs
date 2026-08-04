//! Allocation-free operation preparation and checked response policy.

mod local_async;
mod metadata;
mod operation_id;
mod permit;
mod policy;
mod prepared;
mod storage;

pub use metadata::{
    CostIntent, OperationImpact, OperationMetadata, OperationMetadataError, RequestIdPolicy,
    RequestSemantics, RetryEligibility,
};
pub use operation_id::{OperationId, OperationIdError};
pub use permit::{
    AttemptBudget, AttemptBudgetError, CanonicalPlanFingerprint, CostPermit, CurrencyCode,
    CurrencyCodeError, DestructivePermit, ExecutionPermitError, MutationPermit, PermitAttempt,
    PermitContext, PermitContextError, PermitDisposition, PermitExecutionError,
    PermitIdempotencyKey, PermitIdempotencyKeyError, PermitScope, PermitState, PermitTimestamp,
    PermitValidity, PermitValidityError, PlanChange, PlanConfirmation, PlanCost, PlanCostError,
    PlanFingerprintBuildError, PlanFingerprintDigest, PlanFingerprintRef, PlanFingerprintScope,
    PlanSubject, ReconciliationToken, RecoveryToken, ReplayPolicy, SharedCostPermit,
    SharedDestructivePermit, SharedMutationPermit, SharedPermitState, build_canonical_plan,
    build_plan_digest,
};
pub use policy::{
    CheckedResponse, CheckedResponseGuard, ContentTypePolicy, ResponseBodyPolicy, ResponsePolicy,
    ResponsePolicyError, ResponsePolicyValidationError,
};
pub use prepared::{
    BodyReplayability, PreparationStorage, PrepareOperation, PreparedExecutionError,
    PreparedRequest, PreparedRequestPolicyError, ProviderService,
};
#[cfg(feature = "alloc")]
pub use storage::OwnedPreparationStorage;
pub use storage::{
    DEFAULT_BODY_BYTES, EMBEDDED_BODY_BYTES, LARGE_BODY_BYTES, PreparationCapacityError,
    PreparationCapacityProfile, PreparationStorageGuard,
};

#[cfg(test)]
mod local_async_tests;
#[cfg(test)]
mod prepared_policy_tests;
#[cfg(test)]
mod response_tests;
#[cfg(test)]
mod tests;
