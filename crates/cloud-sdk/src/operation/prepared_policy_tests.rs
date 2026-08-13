use super::{
    ApprovedReadOnlyPostQuery, ContentTypePolicy, CostIntent, OperationImpact, OperationMetadata,
    PreparedRequest, PreparedRequestPolicyError, ProviderService, RequestBodySensitivity,
    RequestIdPolicy, RequestSemantics, ResponseBodyPolicy, ResponsePolicy, RetryEligibility,
};
use crate::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use crate::transport::{
    ContentType, EndpointIdentity, EndpointPolicy, EndpointScheme, HeaderName, MediaType,
    RawResponsePolicy, RequestHeader, RequestHeaders, RequestTarget, ResponseMediaPolicy,
    StatusCode, TransportRequest,
};
use crate::{
    Method, ProviderId, ProviderMarker, ServiceId, ServiceMarker, operation_id, provider_id,
    service_id,
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
fn approved_read_only_post_query_is_exact_and_permitless() -> Result<(), &'static str> {
    let target = RequestTarget::new("/traffic").map_err(|_| "target")?;
    let endpoint =
        EndpointIdentity::new(EndpointScheme::Https, "robot-ws.your-server.de", 443, "/")
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
        ScopeRequirement::Required(provider_id!("hetzner")),
        ScopeRequirement::Required(service_id!("robot")),
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
    let request_headers = [
        RequestHeader::accept(MediaType::JSON),
        RequestHeader::content_type(ContentType::FORM_URLENCODED),
    ];
    let headers = RequestHeaders::new(&request_headers).map_err(|_| "headers")?;
    let service = ProviderService::new(
        provider_id!("hetzner"),
        service_id!("robot"),
        EndpointPolicy::fixed(endpoint),
    );
    let transport = TransportRequest::new(Method::Post, target)
        .with_headers(headers)
        .with_body(b"ip%5B%5D=192.0.2.10");
    let prepared = PreparedRequest::new_read_only_post_query(
        ApprovedReadOnlyPostQuery::HetznerRobotTraffic,
        transport,
        service,
        metadata,
        response,
        authentication,
        raw,
        RequestBodySensitivity::Sensitive,
    )
    .map_err(|_| "read-only POST")?;
    assert!(!prepared.requires_execution_permit());
    assert_eq!(
        prepared.operation_id(),
        Some(operation_id!("robot_get_traffic"))
    );
    assert_eq!(
        prepared
            .with_operation_id(operation_id!("server_update"))
            .operation_id(),
        Some(operation_id!("robot_get_traffic"))
    );
    let altered_entries = [
        RequestHeader::accept(MediaType::JSON),
        RequestHeader::content_type(ContentType::FORM_URLENCODED),
        RequestHeader::new("x-page", "2").map_err(|_| "pagination header")?,
    ];
    let altered_headers = RequestHeaders::new(&altered_entries).map_err(|_| "altered headers")?;
    assert!(
        prepared
            .with_request_headers(altered_headers)
            .requires_execution_permit()
    );

    assert_eq!(
        PreparedRequest::new_read_only_post_query(
            ApprovedReadOnlyPostQuery::HetznerRobotTraffic,
            transport,
            service,
            metadata,
            response,
            authentication,
            raw,
            RequestBodySensitivity::Public,
        )
        .err(),
        Some(PreparedRequestPolicyError::ReadOnlyPostQueryMismatch)
    );

    for invalid_transport in [
        TransportRequest::new(Method::Get, target)
            .with_headers(headers)
            .with_body(b"ip%5B%5D=192.0.2.10"),
        TransportRequest::new(
            Method::Post,
            RequestTarget::new("/server/1").map_err(|_| "target")?,
        )
        .with_headers(headers)
        .with_body(b"ip%5B%5D=192.0.2.10"),
        TransportRequest::new(Method::Post, target).with_body(b"ip%5B%5D=192.0.2.10"),
    ] {
        assert_eq!(
            PreparedRequest::new_read_only_post_query(
                ApprovedReadOnlyPostQuery::HetznerRobotTraffic,
                invalid_transport,
                service,
                metadata,
                response,
                authentication,
                raw,
                RequestBodySensitivity::Sensitive,
            )
            .err(),
            Some(PreparedRequestPolicyError::ReadOnlyPostQueryMismatch)
        );
    }

    assert_eq!(
        PreparedRequest::new_read_only_post_query(
            ApprovedReadOnlyPostQuery::HetznerRobotTraffic,
            transport,
            ProviderService::from_marker::<TestService>(EndpointPolicy::fixed(endpoint)),
            metadata,
            response,
            authentication,
            raw,
            RequestBodySensitivity::Sensitive,
        )
        .err(),
        Some(PreparedRequestPolicyError::ReadOnlyPostQueryMismatch)
    );

    let alternate_endpoint =
        EndpointIdentity::new(EndpointScheme::Https, "proxy.example.invalid", 443, "/")
            .map_err(|_| "alternate endpoint")?;
    let wrong_endpoint_service = ProviderService::new(
        provider_id!("hetzner"),
        service_id!("robot"),
        EndpointPolicy::fixed(alternate_endpoint),
    );
    let wrong_authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(provider_id!("hetzner")),
        ScopeRequirement::Required(service_id!("robot")),
        ScopeRequirement::Required(alternate_endpoint),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let costly_metadata = OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        CostIntent::MayIncurCost,
        RequestIdPolicy::Discard,
    )
    .map_err(|_| "costly metadata")?;
    for (candidate_service, candidate_metadata, candidate_authentication) in [
        (wrong_endpoint_service, metadata, authentication),
        (service, metadata, wrong_authentication),
        (service, costly_metadata, authentication),
    ] {
        assert_eq!(
            PreparedRequest::new_read_only_post_query(
                ApprovedReadOnlyPostQuery::HetznerRobotTraffic,
                transport,
                candidate_service,
                candidate_metadata,
                response,
                candidate_authentication,
                raw,
                RequestBodySensitivity::Sensitive,
            )
            .err(),
            Some(PreparedRequestPolicyError::ReadOnlyPostQueryMismatch)
        );
    }
    Ok(())
}
