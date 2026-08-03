use cloud_sdk::operation::{CostIntent, OperationImpact, RequestSemantics, RetryEligibility};

use super::{OperationClass, operation_metadata};
use crate::EndpointGroup;
use crate::endpoint::official_endpoint_policy;
use crate::prepared::wire_policy::provider_service;

#[test]
fn prepared_services_use_the_canonical_official_policies() {
    for group in [EndpointGroup::Servers, EndpointGroup::StorageBoxes] {
        let base = group.api_base_url();
        let service = provider_service(group);
        let official = official_endpoint_policy(base);
        assert!(service.is_ok() && official.is_ok());
        if let (Ok(service), Ok(official)) = (service, official) {
            assert_eq!(service.endpoint_policy(), official);
        }
    }
}

#[test]
fn operation_classes_own_impact_semantics_and_retry_policy() {
    let cases = [
        (
            OperationClass::ReadOnly,
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
        ),
        (
            OperationClass::IdempotentMutation,
            OperationImpact::Mutation,
            RequestSemantics::Idempotent,
            RetryEligibility::ExplicitPolicy,
        ),
        (
            OperationClass::NonIdempotentMutation,
            OperationImpact::Mutation,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
        (
            OperationClass::IdempotentDestructive,
            OperationImpact::Destructive,
            RequestSemantics::Idempotent,
            RetryEligibility::Never,
        ),
        (
            OperationClass::NonIdempotentDestructive,
            OperationImpact::Destructive,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
    ];

    for (class, impact, semantics, retry) in cases {
        let metadata = operation_metadata(class, CostIntent::NoKnownCost);
        assert!(metadata.is_ok());
        if let Ok(metadata) = metadata {
            assert_eq!(metadata.impact(), impact);
            assert_eq!(metadata.semantics(), semantics);
            assert_eq!(metadata.retry_eligibility(), retry);
        }
    }
}
