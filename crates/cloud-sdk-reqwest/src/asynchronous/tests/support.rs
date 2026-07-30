use cloud_sdk::authentication::{
    AuthenticatedRequest, AuthenticationScopePolicy, ScopeRequirement,
};
use cloud_sdk::transport::{BoundTransport, EndpointIdentity, TransportRequest};

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

pub(super) fn authenticated<'request, 'endpoint>(
    client: &'endpoint AsyncClient,
    request: TransportRequest<'request>,
) -> AuthenticatedRequest<'request, 'endpoint> {
    let endpoint = client
        .endpoint_identity()
        .unwrap_or_else(|_| unreachable!());
    AuthenticatedRequest::new(request, test_authentication_policy(endpoint))
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
