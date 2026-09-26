use std::format;

use cloud_sdk::transport::{
    HeaderName, MediaType, RawResponsePolicy, ResponseHeaders, ResponseMediaPolicy, StatusCode,
};
use reqwest::header::{HeaderMap, HeaderValue};

use super::{RawHttpError, inspect_response_head};

#[test]
fn published_raw_error_variants_and_discriminants_are_preserved() {
    fn ordinal(error: RawHttpError) -> usize {
        match error {
            RawHttpError::ResponseAlreadyCommitted => 0,
            RawHttpError::TargetRejected => 1,
            RawHttpError::MethodRejected => 2,
            RawHttpError::MissingContentType => 3,
            RawHttpError::HeaderRejected => 4,
            RawHttpError::RequestHeaderAllocationFailed => 5,
            RawHttpError::RequestBodyAllocationFailed => 6,
            RawHttpError::RequestBodyTooLarge => 7,
            RawHttpError::RequestBuildFailed => 8,
            RawHttpError::RuntimeInitializationFailed => 9,
            RawHttpError::BlockingRuntimeContext => 10,
            RawHttpError::ConnectFailed => 11,
            RawHttpError::TimedOut => 12,
            RawHttpError::RequestFailed => 13,
            RawHttpError::ResponseOriginChanged => 14,
            RawHttpError::InvalidStatus => 15,
            RawHttpError::SwitchingProtocols => 16,
            RawHttpError::TooManyInformationalResponses => 17,
            RawHttpError::ResponseHeadTooLarge => 18,
            RawHttpError::DuplicateResponseHeader => 19,
            RawHttpError::ResponseTrailersRejected => 20,
            RawHttpError::InvalidNoBodyFraming => 21,
            RawHttpError::MissingResponseContentType => 22,
            RawHttpError::InvalidResponseContentType => 23,
            RawHttpError::UnexpectedResponseContentType => 24,
            RawHttpError::ForbiddenResponseContentType => 25,
            RawHttpError::InvalidResponseHeader => 26,
            RawHttpError::ResponseTooLarge => 27,
            RawHttpError::ResponseChunkLimitExceeded => 28,
            RawHttpError::ResponseReadFailed => 29,
            RawHttpError::ResponseCommitFailed => 30,
        }
    }
    for error in [
        RawHttpError::ResponseAlreadyCommitted,
        RawHttpError::TargetRejected,
        RawHttpError::MethodRejected,
        RawHttpError::MissingContentType,
        RawHttpError::HeaderRejected,
        RawHttpError::RequestHeaderAllocationFailed,
        RawHttpError::RequestBodyAllocationFailed,
        RawHttpError::RequestBodyTooLarge,
        RawHttpError::RequestBuildFailed,
        RawHttpError::RuntimeInitializationFailed,
        RawHttpError::BlockingRuntimeContext,
        RawHttpError::ConnectFailed,
        RawHttpError::TimedOut,
        RawHttpError::RequestFailed,
        RawHttpError::ResponseOriginChanged,
        RawHttpError::InvalidStatus,
        RawHttpError::SwitchingProtocols,
        RawHttpError::TooManyInformationalResponses,
        RawHttpError::ResponseHeadTooLarge,
        RawHttpError::DuplicateResponseHeader,
        RawHttpError::ResponseTrailersRejected,
        RawHttpError::InvalidNoBodyFraming,
        RawHttpError::MissingResponseContentType,
        RawHttpError::InvalidResponseContentType,
        RawHttpError::UnexpectedResponseContentType,
        RawHttpError::ForbiddenResponseContentType,
        RawHttpError::InvalidResponseHeader,
        RawHttpError::ResponseTooLarge,
        RawHttpError::ResponseChunkLimitExceeded,
        RawHttpError::ResponseReadFailed,
        RawHttpError::ResponseCommitFailed,
    ] {
        assert_eq!(error as usize, ordinal(error));
    }
}

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
