#![no_main]

use cloud_sdk::Method;
use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationImpact, OperationMetadata,
    PreparedRequest, ProviderService, RequestIdPolicy, RequestSemantics, ResponseBodyPolicy,
    ResponsePolicy, RetryEligibility,
};
use cloud_sdk::transport::{
    EndpointIdentity, EndpointPolicy, EndpointScheme, HeaderName, HeaderSensitivity, MediaType,
    RawResponsePolicy, RequestTarget, ResponseBuffer, ResponseMediaPolicy, ResponseMetadata,
    StatusCode, TransportRequest,
};
use cloud_sdk_hetzner::CloudService;
use cloud_sdk_hetzner::serde::{ApiErrorEnvelope, ResponseBytes, decode_response};
use libfuzzer_sys::fuzz_target;

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
const CREATED: &[StatusCode] = &[StatusCode::CREATED];

fn prepared(selector: u8) -> Option<(PreparedRequest<'static>, StatusCode)> {
    let (operation, target, method, statuses, status) = match selector % 4 {
        0 => ("get_action", "/actions/1", Method::Get, OK, StatusCode::OK),
        1 => (
            "get_server_metrics",
            "/servers/1/metrics",
            Method::Get,
            OK,
            StatusCode::OK,
        ),
        2 => (
            "create_server",
            "/servers",
            Method::Post,
            CREATED,
            StatusCode::CREATED,
        ),
        _ => (
            "request_server_console",
            "/servers/1/actions/request_console",
            Method::Post,
            CREATED,
            StatusCode::CREATED,
        ),
    };
    let target = RequestTarget::new(target).ok()?;
    let endpoint =
        EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1").ok()?;
    let metadata = OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Protected,
    )
    .ok()?;
    let policy = ResponsePolicy::new(
        statuses,
        ContentTypePolicy::Required(JSON),
        ResponseBodyPolicy::Required,
        8_388_608,
    )
    .ok()?;
    let operation = OperationId::new(operation).ok()?;
    let service = ProviderService::from_marker::<CloudService>(EndpointPolicy::fixed(endpoint));
    let authentication_policy = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(service.provider_id()),
        ScopeRequirement::Required(service.service_id()),
        ScopeRequirement::Required(endpoint),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let content_type = HeaderName::new("content-type").ok()?;
    let request_id = HeaderName::new("x-request-id").ok()?;
    let raw_policy = RawResponsePolicy::new(
        8_388_608,
        8_388_608,
        ResponseMediaPolicy::Required(JSON),
        ResponseMediaPolicy::Required(JSON),
        &[content_type, request_id],
        8,
    )
    .ok()?;
    let request = PreparedRequest::new(
        TransportRequest::new(method, target),
        service,
        metadata,
        policy,
        authentication_policy,
        raw_policy,
    )
    .ok()?
    .with_operation_id(operation);
    Some((request, status))
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 3 {
        return;
    }
    let Some((prepared, success_status)) = prepared(data[0]) else {
        return;
    };
    let status = if data[1] & 1 == 0 {
        success_status
    } else {
        StatusCode::new(400).unwrap_or(StatusCode::TOO_MANY_REQUESTS)
    };
    let body = &data[3..];
    let mut response_storage = body.to_vec();
    let capacity = response_storage.len();
    let mut response_header_storage = [0_u8; 8192];
    let mut response = ResponseBuffer::new(
        &mut response_storage,
        capacity,
        &mut response_header_storage,
    );
    let Ok(mut attempt) = response.writer().begin_attempt() else {
        return;
    };
    if data[2] % 3 != 2 {
        let content_type = if data[2] % 3 == 0 {
            "application/json; charset=utf-8"
        } else {
            "text/plain"
        };
        let Ok(headers) = attempt.headers_mut() else {
            return;
        };
        if headers
            .try_push(
                "content-type",
                content_type.as_bytes(),
                HeaderSensitivity::Public,
            )
            .is_err()
        {
            return;
        }
    }
    let Ok(output) = attempt.body_mut() else {
        return;
    };
    output.copy_from_slice(body);
    if attempt
        .commit(status, body.len(), ResponseMetadata::EMPTY)
        .is_err()
    {
        return;
    }
    drop(attempt);
    if let Ok(admitted) = ResponseBytes::new(body) {
        let _ = serde_json::from_slice::<ApiErrorEnvelope<'_>>(admitted.as_slice());
    }
    let _ = decode_response(prepared, response);
});
