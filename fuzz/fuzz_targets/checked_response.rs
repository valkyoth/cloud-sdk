#![no_main]

use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationImpact, OperationMetadata,
    PreparedRequest, ProviderService, RequestSemantics, ResponseBodyPolicy, ResponsePolicy,
    RetryEligibility,
};
use cloud_sdk::transport::{
    EndpointIdentity, EndpointScheme, MediaType, RequestTarget, ResponseContentType, StatusCode,
    TransportRequest, TransportResponse,
};
use cloud_sdk::{ApiFamily, Method, Provider};
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
    Some(
        PreparedRequest::new(
            TransportRequest::new(Method::Get, target),
            ProviderService::new(Provider::Hetzner, ApiFamily::Cloud, endpoint),
            metadata,
            policy,
        )
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
    let mut response = TransportResponse::new(status, body);
    if data[1] % 3 != 2 {
        let value = if data[1] % 3 == 0 {
            "application/json; charset=utf-8"
        } else {
            "text/plain"
        };
        if let Ok(content_type) = ResponseContentType::new(value) {
            response = response.with_content_type(content_type);
        }
    }
    let _ = decode_response(prepared, response);
});
