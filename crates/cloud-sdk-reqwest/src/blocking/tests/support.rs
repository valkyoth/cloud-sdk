use std::{string::String, vec::Vec};

use cloud_sdk::authentication::{AuthenticationScopePolicy, CredentialLifetime, ScopeRequirement};
use cloud_sdk::operation::{
    AttemptBudget, ContentTypePolicy, CostIntent, MutationPermit, OperationImpact,
    OperationMetadata, PermitClock, PermitContext, PermitTimestamp, PermitValidity, PlanChange,
    PlanConfirmation, PlanFingerprintScope, PreparedExecutionError, PreparedRequest,
    ProviderService, ReplayPolicy, RequestIdPolicy, RequestSemantics, ResponseBodyPolicy,
    ResponsePolicy, RetryEligibility, build_canonical_plan,
};
use cloud_sdk::rate_limit::RateLimit;
use cloud_sdk::transport::{
    BoundTransport, EndpointIdentity, EndpointPolicy, HeaderName, MediaType, RawResponsePolicy,
    ResponseMediaPolicy, StatusCode, TransportRequest,
};

use super::super::{
    AuthenticatedTransportFailure, BearerCredential, BearerCredentialScope, BearerToken,
    BlockingClient, HttpsEndpoint, TransportError,
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

pub(super) fn test_expiring_credential(
    token: BearerToken,
    endpoint: &HttpsEndpoint,
    lifetime: CredentialLifetime,
) -> BearerCredential {
    BearerCredential::new_expiring(
        token,
        BearerCredentialScope::new(
            cloud_sdk::provider_id!("example"),
            cloud_sdk::service_id!("compute"),
            endpoint.clone(),
        ),
        lifetime,
    )
}

pub(super) fn prepared<'request>(
    client: &'request BlockingClient,
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
    let direct = matches!(request.method().as_str(), "GET" | "HEAD");
    let metadata = OperationMetadata::new(
        if direct {
            OperationImpact::ReadOnly
        } else {
            OperationImpact::Mutation
        },
        if direct {
            RequestSemantics::Safe
        } else {
            RequestSemantics::NonIdempotent
        },
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
    .with_operation_id(cloud_sdk::operation_id!("reqwest_transport_test"))
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

pub(super) struct CapturedResponse {
    status: StatusCode,
    body: Vec<u8>,
    content_type: Option<String>,
    rate_limit: Option<RateLimit>,
}

impl CapturedResponse {
    fn capture(response: cloud_sdk::operation::CheckedResponse<'_>) -> Self {
        Self {
            status: response.status(),
            body: response.body().to_vec(),
            content_type: response
                .content_type()
                .map(|content_type| String::from(content_type.as_str())),
            rate_limit: response.rate_limit(),
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
}

pub(super) fn send_test(
    client: &BlockingClient,
    request: TransportRequest<'_>,
    output: &mut [u8],
) -> Result<CapturedResponse, TransportError> {
    let mut headers = [0_u8; 8192];
    let direct = matches!(request.method().as_str(), "GET" | "HEAD");
    let prepared = prepared(client, request);
    let checked = if direct {
        prepared
            .execute_blocking(client, output, &mut headers)
            .map_err(map_execution_error)?
    } else {
        execute_mutation(client, prepared, output, &mut headers)?
    };
    Ok(checked.with_borrowed(CapturedResponse::capture))
}

fn execute_mutation<'buffer>(
    client: &BlockingClient,
    prepared: PreparedRequest<'_>,
    output: &'buffer mut [u8],
    headers: &'buffer mut [u8],
) -> Result<cloud_sdk::operation::CheckedResponseGuard<'buffer>, TransportError> {
    let endpoint = client
        .endpoint_identity()
        .map_err(|_| TransportError::ResponseCommitFailed)?;
    let plan = PlanConfirmation::new(
        prepared,
        endpoint,
        PlanFingerprintScope::Value(b"test-account"),
        PlanFingerprintScope::Absent,
        PermitContext::new(b"reqwest transport test")
            .map_err(|_| TransportError::ResponseCommitFailed)?,
        PermitValidity::new(time(100), time(200))
            .map_err(|_| TransportError::ResponseCommitFailed)?,
        ReplayPolicy::SingleAttempt,
        AttemptBudget::new(1).map_err(|_| TransportError::ResponseCommitFailed)?,
        PlanChange::ChangesState,
        None,
        None,
    );
    let mut fingerprint_storage = [0_u8; 32_768];
    let fingerprint = build_canonical_plan(plan, &mut fingerprint_storage)
        .map_err(|_| TransportError::ResponseCommitFailed)?;
    let mut permit = MutationPermit::new(fingerprint.subject(), time(100))
        .map_err(|_| TransportError::ResponseCommitFailed)?;
    let attempt = permit
        .begin(time(101))
        .map_err(|_| TransportError::ResponseCommitFailed)?;
    attempt
        .execute_blocking(&FixedClock, client, output, headers)
        .map_err(|error| match error.execution() {
            PreparedExecutionError::Transport(failure) => *failure.error(),
            _ => TransportError::ResponseCommitFailed,
        })
}

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        time(102)
    }
}

const fn time(value: u64) -> PermitTimestamp {
    PermitTimestamp::from_seconds(value)
}

fn map_execution_error(
    error: PreparedExecutionError<AuthenticatedTransportFailure>,
) -> TransportError {
    match error {
        PreparedExecutionError::Transport(failure) => failure.into_error(),
        _ => TransportError::ResponseCommitFailed,
    }
}
