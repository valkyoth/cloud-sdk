use super::fixture::endpoint;
use super::{
    FingerprintScope, MonotonicDuration, MonotonicInstant, RetryController, RetryDecision,
    RetryEvent, RetryPolicyError, build_canonical_fingerprint, policy,
};
use crate::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use crate::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationImpact, OperationMetadata,
    PreparedRequest, ProviderService, RequestIdPolicy, RequestSemantics, ResponseBodyPolicy,
    ResponsePolicy, RetryEligibility,
};
use crate::transport::{
    AcknowledgedCustomEndpoint, CustomEndpointAcknowledgement, EndpointPolicy, MediaType,
    RawResponsePolicy, RequestTarget, ResponseMediaPolicy, StatusCode, TransportRequest,
};
use crate::{Method, ProviderId, ServiceId};

static OK: [StatusCode; 1] = [StatusCode::OK];
static JSON: [MediaType<'static>; 1] = [MediaType::JSON];

#[derive(Clone, Copy)]
enum PolicyVariant {
    Baseline,
    Metadata,
    Replayability,
    EndpointPolicy,
    ResponsePolicy,
    AuthenticationPolicy,
    RawResponsePolicy,
}

#[test]
fn identical_wire_bytes_cannot_launder_any_prepared_policy() {
    for variant in [
        PolicyVariant::Metadata,
        PolicyVariant::Replayability,
        PolicyVariant::EndpointPolicy,
        PolicyVariant::ResponsePolicy,
        PolicyVariant::AuthenticationPolicy,
        PolicyVariant::RawResponsePolicy,
    ] {
        assert_policy_rejected(variant);
    }
}

fn assert_policy_rejected(variant: PolicyVariant) {
    let Some(endpoint) = endpoint() else { return };
    let Some(initial) = request(PolicyVariant::Baseline) else {
        return;
    };
    let Some(replay) = request(variant) else {
        return;
    };
    let mut initial_storage = [0_u8; 512];
    let mut replay_storage = [0_u8; 512];
    let Ok(initial_fingerprint) = build_canonical_fingerprint(
        initial,
        endpoint,
        FingerprintScope::Absent,
        &mut initial_storage,
    ) else {
        return;
    };
    let Ok(replay_fingerprint) = build_canonical_fingerprint(
        replay,
        endpoint,
        FingerprintScope::Absent,
        &mut replay_storage,
    ) else {
        return;
    };
    assert!(
        initial_fingerprint
            .as_ref()
            .matches(replay_fingerprint.as_ref())
    );
    let Some(policy) = policy(2, 0, 20) else {
        return;
    };
    let Ok(mut controller) = RetryController::new(
        initial_fingerprint.subject(),
        None,
        policy,
        MonotonicInstant::new(0),
    ) else {
        return;
    };

    assert!(matches!(
        controller.decide_retry(
            RetryEvent::Response(StatusCode::new(500).unwrap_or(StatusCode::OK)),
            replay_fingerprint.subject(),
            MonotonicDuration::new(0),
            MonotonicInstant::new(5),
        ),
        Err(RetryPolicyError::ReplayPolicyMismatch)
    ));
    assert_eq!(controller.attempts(), 1);
    assert_eq!(controller.cumulative_delay().get(), 0);

    let accepted = controller.decide_retry(
        RetryEvent::Response(StatusCode::new(500).unwrap_or(StatusCode::OK)),
        initial_fingerprint.subject(),
        MonotonicDuration::new(0),
        MonotonicInstant::new(0),
    );
    assert!(matches!(accepted, Ok(RetryDecision::Retry(_))));
}

fn request(variant: PolicyVariant) -> Option<PreparedRequest<'static>> {
    let endpoint = endpoint()?;
    let provider = ProviderId::new("example").ok()?;
    let service = ServiceId::new("compute").ok()?;
    let endpoint_policy = if matches!(variant, PolicyVariant::EndpointPolicy) {
        EndpointPolicy::acknowledged_custom(AcknowledgedCustomEndpoint::new(
            endpoint,
            CustomEndpointAcknowledgement::trusted_operator_configuration(),
        ))
    } else {
        EndpointPolicy::fixed(endpoint)
    };
    let metadata = if matches!(variant, PolicyVariant::Metadata) {
        OperationMetadata::new(
            OperationImpact::Destructive,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
            CostIntent::MayIncurCost,
            RequestIdPolicy::Discard,
        )
        .ok()?
    } else {
        OperationMetadata::new(
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
            CostIntent::NoKnownCost,
            RequestIdPolicy::Discard,
        )
        .ok()?
    };
    let response_limit = if matches!(variant, PolicyVariant::ResponsePolicy) {
        63
    } else {
        64
    };
    let response = ResponsePolicy::new(
        &OK,
        ContentTypePolicy::Required(&JSON),
        ResponseBodyPolicy::Required,
        response_limit,
    )
    .ok()?;
    let provider_requirement = if matches!(variant, PolicyVariant::AuthenticationPolicy) {
        ScopeRequirement::Required(provider)
    } else {
        ScopeRequirement::Forbidden
    };
    let authentication = AuthenticationScopePolicy::new(
        provider_requirement,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let raw_limit = if matches!(variant, PolicyVariant::RawResponsePolicy) {
        65
    } else {
        64
    };
    let raw = RawResponsePolicy::new(
        raw_limit,
        raw_limit,
        ResponseMediaPolicy::Required(&JSON),
        ResponseMediaPolicy::Required(&JSON),
        &[],
        0,
    )
    .ok()?;
    let request = TransportRequest::new(Method::Post, RequestTarget::new("/same-wire").ok()?)
        .with_body(b"{}");
    let prepared = PreparedRequest::new(
        request,
        ProviderService::new(provider, service, endpoint_policy),
        metadata,
        response,
        authentication,
        raw,
    )
    .ok()?
    .with_operation_id(OperationId::new("same_operation").ok()?);
    Some(if matches!(variant, PolicyVariant::Replayability) {
        prepared
    } else {
        prepared.with_replayable_body()
    })
}
