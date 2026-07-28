//! Allocation-free operation preparation and checked response policy.

mod metadata;
mod operation_id;
mod policy;
mod prepared;

pub use metadata::{
    CostIntent, OperationImpact, OperationMetadata, OperationMetadataError, RequestSemantics,
    RetryEligibility,
};
pub use operation_id::{OperationId, OperationIdError};
pub use policy::{
    CheckedResponse, CheckedResponseGuard, ContentTypePolicy, ResponseBodyPolicy, ResponsePolicy,
    ResponsePolicyError, ResponsePolicyValidationError,
};
pub use prepared::{
    PreparationStorage, PrepareOperation, PreparedExecutionError, PreparedRequest, ProviderService,
};

#[cfg(test)]
mod response_tests;
#[cfg(test)]
mod tests;
