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
use cloud_sdk_hetzner::serde::decode_response;
use libfuzzer_sys::fuzz_target;

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];

fn prepared() -> Option<PreparedRequest<'static>> {
    let target = RequestTarget::new("/servers/42").ok()?;
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
        OK,
        ContentTypePolicy::Required(JSON),
        ResponseBodyPolicy::Required,
        8_388_608,
    )
    .ok()?;
    let operation = OperationId::new("get_server").ok()?;
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
    Some(
        PreparedRequest::new(
            TransportRequest::new(Method::Get, target),
            service,
            metadata,
            policy,
            authentication_policy,
            raw_policy,
            cloud_sdk::operation::RequestBodySensitivity::Public,
        )
        .ok()?
        .with_operation_id(operation),
    )
}

fn decode_hex(value: &[u8]) -> Option<Vec<u8>> {
    let chunks = value.chunks_exact(2);
    if !chunks.remainder().is_empty() {
        return None;
    }
    chunks
        .map(|chunk| {
            let high = hex_digit(*chunk.first()?)?;
            let low = hex_digit(*chunk.get(1)?)?;
            Some((high << 4) | low)
        })
        .collect()
}

fn hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }
    let Some(prepared) = prepared() else {
        return;
    };
    let status = match data[0] % 5 {
        0 => StatusCode::OK,
        1 => StatusCode::CREATED,
        2 => StatusCode::new(400).unwrap_or(StatusCode::TOO_MANY_REQUESTS),
        3 => StatusCode::TOO_MANY_REQUESTS,
        _ => StatusCode::new(500).unwrap_or(StatusCode::TOO_MANY_REQUESTS),
    };
    let decoded;
    let body = if let Some(hex) = data[2..].strip_prefix(b"hex:") {
        let Some(value) = decode_hex(hex) else {
            return;
        };
        decoded = value;
        decoded.as_slice()
    } else {
        &data[2..]
    };
    let content_type = if data[1] % 3 != 2 {
        Some(if data[1] % 3 == 0 {
            "application/json; charset=utf-8"
        } else {
            "text/plain"
        })
    } else {
        None
    };
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
    if let Some(content_type) = content_type {
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
    let _ = decode_response(prepared, response);
});
