use cloud_sdk::authentication::{AuthenticationScopePolicy, CredentialLifetime, ScopeRequirement};
use cloud_sdk::operation::{
    AttemptBudget, CheckedResponseGuard, ContentTypePolicy, CostIntent, MutationPermit,
    OperationImpact, OperationMetadata, PermitClock, PermitContext, PermitTimestamp,
    PermitValidity, PlanChange, PlanConfirmation, PlanFingerprintScope, PreparedExecutionError,
    PreparedRequest, ProviderService, ReplayPolicy, RequestIdPolicy, RequestSemantics,
    ResponseBodyPolicy, ResponsePolicy, RetryEligibility, build_canonical_plan,
};
use cloud_sdk::transport::{
    BoundTransport, EndpointIdentity, EndpointPolicy, HeaderName, MediaType, RawResponsePolicy,
    ResponseMediaPolicy, StatusCode, TransportRequest,
};

use super::super::{
    AsyncClient, AsyncClientBuilder, BearerCredential, BearerCredentialScope, BearerToken,
    HttpsEndpoint, TransportError, UserAgent,
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

pub(super) fn expiring_loopback(
    endpoint: &str,
    lifetime: CredentialLifetime,
) -> Option<AsyncClient> {
    let endpoint = HttpsEndpoint::local_http(endpoint).ok()?;
    let token = BearerToken::new("test-token").ok()?;
    let credential = test_expiring_credential(token, &endpoint, lifetime);
    AsyncClientBuilder::new(
        endpoint,
        credential,
        UserAgent::new("cloud-sdk-test/0.18").ok()?,
        super::test_timeouts()?,
    )
    .build_for_loopback()
    .ok()
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

pub(super) async fn execute_test<'request, 'buffer>(
    client: &'request AsyncClient,
    request: TransportRequest<'request>,
    output: &'buffer mut [u8],
    headers: &'buffer mut [u8],
) -> Result<CheckedResponseGuard<'buffer>, TransportError> {
    let direct = matches!(request.method().as_str(), "GET" | "HEAD");
    let prepared = prepared(client, request);
    if direct {
        return prepared
            .execute_async(client, output, headers)
            .await
            .map_err(map_execution_error);
    }

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
        .execute_async(&FixedClock, client, output, headers)
        .await
        .map_err(|error| match error.execution() {
            PreparedExecutionError::Transport(failure) => *failure.error(),
            _ => TransportError::ResponseCommitFailed,
        })
}

fn map_execution_error(
    error: PreparedExecutionError<super::super::AuthenticatedTransportFailure>,
) -> TransportError {
    match error {
        PreparedExecutionError::Transport(failure) => failure.into_error(),
        _ => TransportError::ResponseCommitFailed,
    }
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
