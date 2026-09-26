use std::format;

use cloud_sdk::transport::{
    HeaderName, MediaType, RawResponsePolicy, ResponseHeaders, ResponseMediaPolicy, StatusCode,
};
use reqwest::header::{HeaderMap, HeaderValue};

use super::{RawHttpError, inspect_response_head};

fn policy<'a>(headers: &[HeaderName<'a>]) -> Option<RawResponsePolicy<'a>> {
    RawResponsePolicy::new(
        8,
        4,
        ResponseMediaPolicy::Required(&[MediaType::JSON]),
        ResponseMediaPolicy::Optional(&[MediaType::JSON]),
        headers,
        2,
    )
    .ok()
}

#[test]
fn selects_status_limit_and_drops_unadmitted_headers() {
    let admitted = HeaderName::new("content-type");
    assert!(admitted.is_ok());
    let Ok(admitted) = admitted else {
        unreachable!("security fixture construction failed")
    };
    let admitted_headers = [admitted];
    let policy = policy(&admitted_headers).unwrap_or_else(|| unreachable!());
    let mut source = HeaderMap::new();
    source.insert("content-type", HeaderValue::from_static("application/json"));
    source.insert("set-cookie", HeaderValue::from_static("secret=1"));
    source.insert("x-unknown", HeaderValue::from_static("secret"));
    let mut storage = [0_u8; 128];
    let mut captured = ResponseHeaders::new(&mut storage);
    let result = inspect_response_head(
        cloud_sdk::Method::Get,
        StatusCode::OK,
        &source,
        policy,
        &mut captured,
        16,
    );
    assert_eq!(result, Ok(8));
    assert!(captured.get("content-type").is_some());
    assert!(captured.get("set-cookie").is_none());
    assert!(captured.get("x-unknown").is_none());
}
#[test]
fn retains_incomplete_admitted_quota_metadata_for_provider_validation() {
    let names = [
        HeaderName::new("ratelimit-limit"),
        HeaderName::new("ratelimit-remaining"),
        HeaderName::new("ratelimit-reset"),
    ];
    let [Ok(limit), Ok(remaining), Ok(reset)] = names else {
        unreachable!("security fixture construction failed");
    };
    let Some(policy) = policy(&[limit, remaining, reset]) else {
        unreachable!("security fixture construction failed");
    };
    let mut source = HeaderMap::new();
    source.insert("ratelimit-remaining", HeaderValue::from_static("7"));
    let mut storage = [0_u8; 128];
    let mut captured = ResponseHeaders::new(&mut storage);
    let result = inspect_response_head(
        cloud_sdk::Method::Get,
        StatusCode::TOO_MANY_REQUESTS,
        &source,
        policy,
        &mut captured,
        16,
    );
    assert_eq!(result, Ok(4));
    assert!(captured.get("ratelimit-limit").is_none());
    assert_eq!(
        captured
            .get("ratelimit-remaining")
            .map(|header| header.value()),
        Some(b"7".as_slice())
    );
    assert!(captured.get("ratelimit-reset").is_none());
}
#[test]
fn rejects_duplicates_and_no_content_framing() {
    let policy = policy(&[]).unwrap_or_else(|| unreachable!());
    let mut duplicate = HeaderMap::new();
    duplicate.append("x-test", HeaderValue::from_static("one"));
    duplicate.append("x-test", HeaderValue::from_static("two"));
    let mut storage = [0_u8; 128];
    let mut captured = ResponseHeaders::new(&mut storage);
    assert_eq!(
        inspect_response_head(
            cloud_sdk::Method::Get,
            StatusCode::new(400).unwrap_or(StatusCode::TOO_MANY_REQUESTS),
            &duplicate,
            policy,
            &mut captured,
            16,
        ),
        Err(RawHttpError::DuplicateResponseHeader)
    );

    let mut no_content = HeaderMap::new();
    no_content.insert("content-length", HeaderValue::from_static("0"));
    assert_eq!(
        inspect_response_head(
            cloud_sdk::Method::Get,
            StatusCode::NO_CONTENT,
            &no_content,
            policy,
            &mut captured,
            16,
        ),
        Err(RawHttpError::InvalidNoBodyFraming)
    );
}

#[test]
fn rejects_media_mismatch_oversized_length_and_hostile_header_count() {
    let policy = policy(&[]).unwrap_or_else(|| unreachable!());
    let mut storage = [0_u8; 128];
    let mut captured = ResponseHeaders::new(&mut storage);

    let mut wrong_media = HeaderMap::new();
    wrong_media.insert("content-type", HeaderValue::from_static("text/plain"));
    assert_eq!(
        inspect_response_head(
            cloud_sdk::Method::Get,
            StatusCode::OK,
            &wrong_media,
            policy,
            &mut captured,
            16,
        ),
        Err(RawHttpError::UnexpectedResponseContentType)
    );

    let mut oversized = HeaderMap::new();
    oversized.insert("content-type", HeaderValue::from_static("application/json"));
    oversized.insert("content-length", HeaderValue::from_static("9"));
    assert_eq!(
        inspect_response_head(
            cloud_sdk::Method::Get,
            StatusCode::OK,
            &oversized,
            policy,
            &mut captured,
            16,
        ),
        Err(RawHttpError::ResponseTooLarge)
    );

    let mut hostile = HeaderMap::new();
    for index in 0..=super::MAX_UPSTREAM_HTTP1_HEADERS {
        let name = format!("x-field-{index}");
        let Ok(name) = reqwest::header::HeaderName::from_bytes(name.as_bytes()) else {
            unreachable!("security fixture construction failed");
        };
        hostile.insert(name, HeaderValue::from_static("value"));
    }
    assert_eq!(
        inspect_response_head(
            cloud_sdk::Method::Get,
            StatusCode::OK,
            &hostile,
            policy,
            &mut captured,
            16,
        ),
        Err(RawHttpError::ResponseHeadTooLarge)
    );
}

#[test]
fn head_and_not_modified_select_zero_body_capacity() {
    let policy = policy(&[]).unwrap_or_else(|| unreachable!());
    let mut source = HeaderMap::new();
    source.insert("content-type", HeaderValue::from_static("application/json"));
    source.insert("content-length", HeaderValue::from_static("8"));
    let mut storage = [0_u8; 128];
    let mut captured = ResponseHeaders::new(&mut storage);
    assert_eq!(
        inspect_response_head(
            cloud_sdk::Method::Head,
            StatusCode::OK,
            &source,
            policy,
            &mut captured,
            16,
        ),
        Ok(0)
    );
    let not_modified = StatusCode::new(304).unwrap_or(StatusCode::NO_CONTENT);
    assert_eq!(
        inspect_response_head(
            cloud_sdk::Method::Get,
            not_modified,
            &source,
            policy,
            &mut captured,
            16,
        ),
        Ok(0)
    );
}
