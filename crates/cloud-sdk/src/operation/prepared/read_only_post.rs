//! Closed approval registry for provider protocols that query through `POST`.

use crate::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use crate::operation::{
    CostIntent, OperationId, OperationImpact, OperationMetadata, RequestIdPolicy, RequestSemantics,
    RetryEligibility,
};
use crate::transport::{EndpointIdentity, EndpointPolicy, EndpointScheme, TransportRequest};
use crate::{Method, operation_id};

use super::{ProviderService, RequestBodySensitivity};

/// Closed registry of reviewed read-only queries carried by `POST`.
///
/// A variant is only an approval selector. Construction still validates every
/// bound identity and wire property owned by that entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ApprovedReadOnlyPostQuery {
    /// Hetzner Robot's source-locked `POST /traffic` query.
    HetznerRobotTraffic,
}

impl ApprovedReadOnlyPostQuery {
    pub(super) fn validate(
        self,
        request: TransportRequest<'_>,
        service: ProviderService<'_>,
        metadata: OperationMetadata,
        authentication: AuthenticationScopePolicy<'_>,
        body_sensitivity: RequestBodySensitivity,
    ) -> Option<OperationId> {
        match self {
            Self::HetznerRobotTraffic => {
                let endpoint = EndpointIdentity::new(
                    EndpointScheme::Https,
                    "robot-ws.your-server.de",
                    443,
                    "/",
                )
                .ok()?;
                let expected_authentication = AuthenticationScopePolicy::new(
                    ScopeRequirement::Required(service.provider_id()),
                    ScopeRequirement::Required(service.service_id()),
                    ScopeRequirement::Required(endpoint),
                    ScopeRequirement::Forbidden,
                    ScopeRequirement::Forbidden,
                    ScopeRequirement::Forbidden,
                );
                let headers = request.headers();
                let accept = headers.get("accept")?;
                let content_type = headers.get("content-type")?;
                (request.method() == Method::Post
                    && request.target().as_str() == "/traffic"
                    && !request.body().is_empty()
                    && headers.as_slice().len() == 2
                    && accept.value().as_str() == "application/json"
                    && content_type.value().as_str() == "application/x-www-form-urlencoded"
                    && service.provider_id().as_str() == "hetzner"
                    && service.service_id().as_str() == "robot"
                    && service.endpoint_policy() == EndpointPolicy::fixed(endpoint)
                    && authentication == expected_authentication
                    && metadata.impact() == OperationImpact::ReadOnly
                    && metadata.semantics() == RequestSemantics::Safe
                    && metadata.retry_eligibility() == RetryEligibility::ExplicitPolicy
                    && metadata.cost_intent() == CostIntent::NoKnownCost
                    && metadata.request_id_policy() == RequestIdPolicy::Discard
                    && body_sensitivity == RequestBodySensitivity::Sensitive)
                    .then_some(operation_id!("robot_get_traffic"))
            }
        }
    }
}
