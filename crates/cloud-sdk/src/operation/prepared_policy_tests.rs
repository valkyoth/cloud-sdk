use super::{
    ContentTypePolicy, CostIntent, OperationImpact, OperationMetadata, PreparedRequest,
    PreparedRequestPolicyError, ProviderService, RequestBodySensitivity, RequestIdPolicy,
    RequestSemantics, ResponseBodyPolicy, ResponsePolicy, RetryEligibility,
};
use crate::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use crate::transport::{
    EndpointIdentity, EndpointPolicy, EndpointScheme, HeaderName, MediaType, RawResponsePolicy,
    RequestTarget, ResponseMediaPolicy, StatusCode, TransportRequest,
};
use crate::{
    Method, ProviderId, ProviderMarker, ServiceId, ServiceMarker, provider_id, service_id,
};

static JSON: [MediaType<'static>; 1] = [MediaType::JSON];
static OK: [StatusCode; 1] = [StatusCode::OK];

enum TestProvider {}

impl ProviderMarker for TestProvider {
    const ID: ProviderId = provider_id!("prepared-policy-test");
}

enum TestService {}

impl ServiceMarker for TestService {
    type Provider = TestProvider;
    const ID: ServiceId = service_id!("compute");
}

#[test]
fn non_discarded_request_ids_require_raw_header_admission() -> Result<(), &'static str> {
    let target = RequestTarget::new("/resources").map_err(|_| "target")?;
    let endpoint = EndpointIdentity::new(EndpointScheme::Https, "api.example.invalid", 443, "/v1")
        .map_err(|_| "endpoint")?;
    let response = ResponsePolicy::new(
        &OK,
        ContentTypePolicy::Required(&JSON),
        ResponseBodyPolicy::Required,
        16,
    )
    .map_err(|_| "response")?;
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(TestProvider::ID),
        ScopeRequirement::Required(TestService::ID),
        ScopeRequirement::Required(endpoint),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let content_type = HeaderName::new("content-type").map_err(|_| "header")?;
    let raw = RawResponsePolicy::new(
        16,
        16,
        ResponseMediaPolicy::Required(&JSON),
        ResponseMediaPolicy::Required(&JSON),
        &[content_type],
        8,
    )
    .map_err(|_| "raw")?;
    for request_id_policy in [RequestIdPolicy::Protected, RequestIdPolicy::Retain] {
        let metadata = OperationMetadata::new(
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
            CostIntent::NoKnownCost,
            request_id_policy,
        )
        .map_err(|_| "metadata")?;
        assert!(matches!(
            PreparedRequest::new(
                TransportRequest::new(Method::Get, target),
                ProviderService::from_marker::<TestService>(EndpointPolicy::fixed(endpoint)),
                metadata,
                response,
                authentication,
                raw,
                RequestBodySensitivity::Public,
            ),
            Err(PreparedRequestPolicyError::MissingRequestIdHeader)
        ));
    }
    Ok(())
}

#[test]
fn read_only_metadata_rejects_methods_that_can_change_state() -> Result<(), &'static str> {
    let target = RequestTarget::new("/servers/critical").map_err(|_| "target")?;
    let endpoint = EndpointIdentity::new(EndpointScheme::Https, "api.example.invalid", 443, "/v1")
        .map_err(|_| "endpoint")?;
    let response = ResponsePolicy::new(
        &OK,
        ContentTypePolicy::Required(&JSON),
        ResponseBodyPolicy::Required,
        16,
    )
    .map_err(|_| "response")?;
    let metadata = OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .map_err(|_| "metadata")?;
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(TestProvider::ID),
        ScopeRequirement::Required(TestService::ID),
        ScopeRequirement::Required(endpoint),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let raw = RawResponsePolicy::new(
        16,
        16,
        ResponseMediaPolicy::Required(&JSON),
        ResponseMediaPolicy::Required(&JSON),
        &[],
        8,
    )
    .map_err(|_| "raw")?;

    for method in [
        Method::Post,
        Method::Put,
        Method::Patch,
        Method::Delete,
        Method::Options,
    ] {
        assert!(matches!(
            PreparedRequest::new(
                TransportRequest::new(method, target),
                ProviderService::from_marker::<TestService>(EndpointPolicy::fixed(endpoint)),
                metadata,
                response,
                authentication,
                raw,
                RequestBodySensitivity::Public,
            ),
            Err(PreparedRequestPolicyError::ReadOnlyMethodMismatch)
        ));
    }
    Ok(())
}

#[test]
fn explicit_read_only_post_query_is_narrow_and_permitless() -> Result<(), &'static str> {
    let target = RequestTarget::new("/query").map_err(|_| "target")?;
    let endpoint = EndpointIdentity::new(EndpointScheme::Https, "api.example.invalid", 443, "/v1")
        .map_err(|_| "endpoint")?;
    let response = ResponsePolicy::new(
        &OK,
        ContentTypePolicy::Required(&JSON),
        ResponseBodyPolicy::Required,
        16,
    )
    .map_err(|_| "response")?;
    let metadata = OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .map_err(|_| "metadata")?;
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(TestProvider::ID),
        ScopeRequirement::Required(TestService::ID),
        ScopeRequirement::Required(endpoint),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let raw = RawResponsePolicy::new(
        16,
        16,
        ResponseMediaPolicy::Required(&JSON),
        ResponseMediaPolicy::Required(&JSON),
        &[],
        8,
    )
    .map_err(|_| "raw")?;
    let prepared = PreparedRequest::new_read_only_post_query(
        TransportRequest::new(Method::Post, target).with_body(b"query=true"),
        ProviderService::from_marker::<TestService>(EndpointPolicy::fixed(endpoint)),
        metadata,
        response,
        authentication,
        raw,
        RequestBodySensitivity::Sensitive,
    )
    .map_err(|_| "read-only POST")?;
    assert!(!prepared.requires_execution_permit());

    assert!(matches!(
        PreparedRequest::new_read_only_post_query(
            TransportRequest::new(Method::Get, target),
            ProviderService::from_marker::<TestService>(EndpointPolicy::fixed(endpoint)),
            metadata,
            response,
            authentication,
            raw,
            RequestBodySensitivity::Public,
        ),
        Err(PreparedRequestPolicyError::ReadOnlyPostQueryMismatch)
    ));
    Ok(())
}
