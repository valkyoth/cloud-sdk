use crate::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use crate::operation::{
    BodyReplayability, ContentTypePolicy, CostIntent, OperationId, OperationImpact,
    OperationMetadata, PreparedRequest, ProviderService, RequestIdPolicy, RequestSemantics,
    ResponseBodyPolicy, ResponsePolicy, RetryEligibility,
};
use crate::transport::{
    EndpointIdentity, EndpointPolicy, EndpointScheme, MediaType, RawResponsePolicy, RequestTarget,
    ResponseMediaPolicy, StatusCode, TransportRequest,
};
use crate::{Method, ProviderId, ServiceId};

static OK: [StatusCode; 1] = [StatusCode::OK];
static JSON: [MediaType<'static>; 1] = [MediaType::JSON];

pub fn endpoint() -> Option<EndpointIdentity<'static>> {
    EndpointIdentity::new(EndpointScheme::Https, "api.example.invalid", 443, "/v1").ok()
}

pub fn prepared(
    target: &'static str,
    impact: OperationImpact,
    semantics: RequestSemantics,
    eligibility: RetryEligibility,
    replayability: BodyReplayability,
) -> Option<PreparedRequest<'static>> {
    let endpoint = endpoint()?;
    let request =
        TransportRequest::new(Method::Post, RequestTarget::new(target).ok()?).with_body(b"{}");
    let metadata = OperationMetadata::new(
        impact,
        semantics,
        eligibility,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .ok()?;
    let response = ResponsePolicy::new(
        &OK,
        ContentTypePolicy::Required(&JSON),
        ResponseBodyPolicy::Required,
        64,
    )
    .ok()?;
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let raw = RawResponsePolicy::new(
        64,
        64,
        ResponseMediaPolicy::Required(&JSON),
        ResponseMediaPolicy::Required(&JSON),
        &[],
        0,
    )
    .ok()?;
    let prepared = PreparedRequest::new(
        request,
        ProviderService::new(
            ProviderId::new("example").ok()?,
            ServiceId::new("compute").ok()?,
            EndpointPolicy::fixed(endpoint),
        ),
        metadata,
        response,
        authentication,
        raw,
    )
    .ok()?
    .with_operation_id(OperationId::new("create_server").ok()?);
    Some(match replayability {
        BodyReplayability::NotReplayable => prepared,
        BodyReplayability::Replayable => prepared.with_replayable_body(),
    })
}
