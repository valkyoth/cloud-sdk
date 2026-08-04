use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationImpact, OperationMetadata, PreparedRequest,
    ProviderService, RequestIdPolicy, RequestSemantics, ResponseBodyPolicy, ResponsePolicy,
    RetryEligibility,
};
use cloud_sdk::transport::{
    BoundTransport, EndpointIdentity, EndpointPolicy, HeaderName, MediaType, RawResponsePolicy,
    ResponseMediaPolicy, StatusCode, TransportRequest,
};

use super::super::{
    AsyncClient, BearerCredential, BearerCredentialScope, BearerToken, HttpsEndpoint,
};

pub(super) fn test_credential(token: BearerToken, endpoint: &HttpsEndpoint) -> BearerCredential {
    BearerCredential::new(
        token,
        BearerCredentialScope::new(
            cloud_sdk::provider_id!("example"),
            cloud_sdk::service_id!("compute"),
            endpoint.clone(),
        ),
    )
}

pub(super) fn prepared<'request>(
    client: &'request AsyncClient,
    request: TransportRequest<'request>,
) -> PreparedRequest<'request> {
    let endpoint = client
        .endpoint_identity()
        .unwrap_or_else(|_| unreachable!());
    prepared_with_policy(
        request,
        ProviderService::new(
            cloud_sdk::provider_id!("example"),
            cloud_sdk::service_id!("compute"),
            EndpointPolicy::fixed(endpoint),
        ),
        test_authentication_policy(endpoint),
    )
}

pub(super) fn prepared_with_policy<'request>(
    request: TransportRequest<'request>,
    service: ProviderService<'request>,
    authentication: AuthenticationScopePolicy<'request>,
) -> PreparedRequest<'request> {
    let metadata = OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::Never,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .unwrap_or_else(|_| unreachable!());
    let response = ResponsePolicy::new(
        &[StatusCode::OK],
        ContentTypePolicy::Optional(&[MediaType::JSON]),
        ResponseBodyPolicy::Optional,
        8192,
    )
    .unwrap_or_else(|_| unreachable!());
    PreparedRequest::new(
        request,
        service,
        metadata,
        response,
        authentication,
        test_raw_response_policy(),
    )
    .unwrap_or_else(|_| unreachable!())
}

pub(super) fn test_raw_response_policy() -> RawResponsePolicy<'static> {
    let names = [
        "content-type",
        "ratelimit-limit",
        "ratelimit-remaining",
        "ratelimit-reset",
    ];
    let headers = names.map(|name| HeaderName::new(name).unwrap_or_else(|_| std::process::abort()));
    RawResponsePolicy::new(
        8192,
        8192,
        ResponseMediaPolicy::Optional(&[MediaType::JSON]),
        ResponseMediaPolicy::Optional(&[MediaType::JSON]),
        &headers,
        8,
    )
    .unwrap_or_else(|_| std::process::abort())
}

const fn test_authentication_policy(
    endpoint: EndpointIdentity<'_>,
) -> AuthenticationScopePolicy<'_> {
    AuthenticationScopePolicy::new(
        ScopeRequirement::Required(cloud_sdk::provider_id!("example")),
        ScopeRequirement::Required(cloud_sdk::service_id!("compute")),
        ScopeRequirement::Required(endpoint),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    )
}
