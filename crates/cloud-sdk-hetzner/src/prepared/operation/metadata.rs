//! Provider operation safety metadata construction.

use cloud_sdk::operation::{
    CostIntent, OperationImpact, OperationMetadata, RequestIdPolicy, RequestSemantics,
    RetryEligibility,
};

use super::OperationClass;
use crate::prepared::HetznerPreparationError;

pub(crate) fn operation_metadata(
    class: OperationClass,
    cost: CostIntent,
) -> Result<OperationMetadata, HetznerPreparationError> {
    let (impact, semantics, retry) = match class {
        OperationClass::ReadOnly => (
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
        ),
        OperationClass::IdempotentMutation => (
            OperationImpact::Mutation,
            RequestSemantics::Idempotent,
            RetryEligibility::ExplicitPolicy,
        ),
        OperationClass::NonIdempotentMutation => (
            OperationImpact::Mutation,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
        OperationClass::IdempotentDestructive => (
            OperationImpact::Destructive,
            RequestSemantics::Idempotent,
            RetryEligibility::Never,
        ),
        OperationClass::NonIdempotentDestructive => (
            OperationImpact::Destructive,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
    };
    OperationMetadata::new(impact, semantics, retry, cost, RequestIdPolicy::Protected)
        .map_err(HetznerPreparationError::InvalidMetadata)
}
