use cloud_sdk::Method;
use cloud_sdk::transport::{
    HeaderName, MediaType, RawResponsePolicy, ResponseHeaders, ResponseMediaPolicy, StatusCode,
};
use reqwest::header::{HeaderMap, HeaderName as HttpHeaderName, HeaderValue};

use super::raw::{ResponseBodyBudget, inspect_response_head};

/// Exercises raw response-head validation and streamed-body accounting.
///
/// This entry point exists only under the opt-in `fuzzing` feature.
#[doc(hidden)]
pub fn fuzz_raw_response_parser(data: &[u8]) {
    let method = if byte(data, 0) & 1 == 0 {
        Method::Get
    } else {
        Method::Head
    };
    let status = (u16::from_le_bytes([byte(data, 1), byte(data, 2)]) % 500).saturating_add(100);
    let Some(status) = StatusCode::new(status) else {
        return;
    };
    let success_limit = usize::from(byte(data, 3));
    let error_limit = usize::from(byte(data, 4));
    let writer_capacity = usize::from(byte(data, 5));
    let media = media_policy(byte(data, 6));
    let Ok(content_type) = HeaderName::new("content-type") else {
        return;
    };
    let Ok(date) = HeaderName::new("date") else {
        return;
    };
    let admitted = [content_type, date];
    let Ok(policy) = RawResponsePolicy::new(
        success_limit,
        error_limit,
        media,
        media,
        &admitted,
        byte(data, 7) % 9,
    ) else {
        return;
    };

    let source = arbitrary_headers(data.get(8..).unwrap_or_default());
    let mut storage = [0_u8; cloud_sdk::transport::MAX_RESPONSE_HEADER_BYTES];
    let mut captured = ResponseHeaders::new(&mut storage);
    let selected = inspect_response_head(
        method,
        status,
        &source,
        policy,
        &mut captured,
        writer_capacity,
    );
    let body_limit = selected.unwrap_or_else(|_| policy.body_limit(status).min(writer_capacity));
    let mut budget = ResponseBodyBudget::new(body_limit);
    for value in data.iter().copied().skip(8).take(4_098) {
        let bytes = match value {
            0 => 0,
            u8::MAX => usize::MAX,
            value => usize::from(value),
        };
        let before = budget.len();
        match budget.observe(bytes) {
            Ok(range) => {
                assert_eq!(range.start, before);
                assert_eq!(range.end, budget.len());
                assert!(budget.len() <= body_limit);
            }
            Err(_) => assert_eq!(budget.len(), before),
        }
    }
}

fn media_policy(selector: u8) -> ResponseMediaPolicy<'static> {
    match selector % 3 {
        0 => ResponseMediaPolicy::Required(&[MediaType::JSON]),
        1 => ResponseMediaPolicy::Optional(&[MediaType::JSON]),
        _ => ResponseMediaPolicy::Forbidden,
    }
}

fn arbitrary_headers(data: &[u8]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for line in data.split(|byte| *byte == b'\n').take(101) {
        let line = line.strip_suffix(b"\r").unwrap_or(line);
        let Some(separator) = line.iter().position(|byte| *byte == b':') else {
            continue;
        };
        let Some(name) = line.get(..separator) else {
            continue;
        };
        let Some(value) = line.get(separator.saturating_add(1)..) else {
            continue;
        };
        let value = value.strip_prefix(b" ").unwrap_or(value);
        let (Ok(name), Ok(value)) = (
            HttpHeaderName::from_bytes(name),
            HeaderValue::from_bytes(value),
        ) else {
            continue;
        };
        headers.append(name, value);
    }
    headers
}

fn byte(data: &[u8], index: usize) -> u8 {
    data.get(index).copied().unwrap_or(0)
}
