use std::{string::String, vec::Vec};

use cloud_sdk::authentication::{
    AuthenticatedRequest, AuthenticationScopePolicy, BlockingAuthenticatedTransport,
    ScopeRequirement,
};
use cloud_sdk::rate_limit::RateLimit;
use cloud_sdk::transport::{
    BoundTransport, EndpointIdentity, ResponseBuffer, StatusCode, TransportRequest,
    TransportResponse,
};

use super::super::{
    BearerCredential, BearerCredentialScope, BearerToken, BlockingClient, HttpsEndpoint,
    TransportError,
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
    client: &'endpoint BlockingClient,
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

pub(super) struct CapturedResponse {
    status: StatusCode,
    body: Vec<u8>,
    content_type: Option<String>,
    rate_limit: Option<RateLimit>,
    rate_limit_remaining: Option<Vec<u8>>,
    content_type_header: Option<Vec<u8>>,
}

impl CapturedResponse {
    fn capture(response: TransportResponse<'_, '_>) -> Self {
        Self {
            status: response.status(),
            body: response.body().to_vec(),
            content_type: response
                .content_type()
                .ok()
                .flatten()
                .map(|content_type| String::from(content_type.as_str())),
            rate_limit: response.rate_limit(),
            rate_limit_remaining: response
                .headers()
                .get("ratelimit-remaining")
                .map(|header| header.value().to_vec()),
            content_type_header: response
                .headers()
                .get("content-type")
                .map(|header| header.value().to_vec()),
        }
    }

    pub(super) const fn status(&self) -> StatusCode {
        self.status
    }

    pub(super) fn body(&self) -> &[u8] {
        &self.body
    }

    pub(super) fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    pub(super) const fn rate_limit(&self) -> Option<RateLimit> {
        self.rate_limit
    }

    pub(super) fn rate_limit_remaining_header(&self) -> Option<&[u8]> {
        self.rate_limit_remaining.as_deref()
    }

    pub(super) fn content_type_header(&self) -> Option<&[u8]> {
        self.content_type_header.as_deref()
    }
}

pub(super) fn send_test(
    client: &BlockingClient,
    request: TransportRequest<'_>,
    output: &mut [u8],
) -> Result<CapturedResponse, TransportError> {
    let capacity = output.len();
    let mut headers = [0_u8; 8192];
    let mut response = ResponseBuffer::new(output, capacity, &mut headers);
    BlockingAuthenticatedTransport::send_authenticated(
        client,
        authenticated(client, request),
        response.writer(),
    )?;
    response
        .with_response(CapturedResponse::capture)
        .map_err(|_| TransportError::ResponseCommitFailed)
}
