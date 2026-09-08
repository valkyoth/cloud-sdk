use alloc::{format, vec};
use core::error::Error;

use cloud_sdk::incremental_json::{
    IncrementalJsonEvent, IncrementalJsonProgress, IncrementalJsonVisitor, VisitControl,
};
use cloud_sdk::rate_limit::{DelaySeconds, RetryAfter, WallClockTimestamp};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::{CratesIoWireError, JsonResponsePolicy, MAX_JSON_RESPONSE_BYTES, ProviderErrorKind};

fn status(value: u16) -> StatusCode {
    StatusCode::new(value).unwrap_or_else(|| unreachable!("valid status fixture"))
}

pub(super) fn fixture<'a>(
    body: &'a mut [u8],
    headers: &'a mut [u8],
    input: &[u8],
    code: u16,
    media: Option<&[u8]>,
    retry: Option<&[u8]>,
) -> ResponseBuffer<'a> {
    let capacity = body.len();
    let mut response = ResponseBuffer::new(body, capacity, headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("response attempt fixture"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("admitted fixture storage"))
        .get_mut(..input.len())
        .unwrap_or_else(|| unreachable!("fixture body too small"))
        .copy_from_slice(input);
    if let Some(media) = media {
        attempt
            .headers_mut()
            .unwrap_or_else(|_| unreachable!("fixture headers"))
            .try_push("content-type", media, HeaderSensitivity::Public)
            .unwrap_or_else(|_| unreachable!("media fixture"));
    }
    if let Some(retry) = retry {
        attempt
            .headers_mut()
            .unwrap_or_else(|_| unreachable!("fixture headers"))
            .try_push("retry-after", retry, HeaderSensitivity::Public)
            .unwrap_or_else(|_| unreachable!("retry fixture"));
    }
    attempt
        .commit(status(code), input.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("fixture commit"));
    drop(attempt);
    response
}

fn check(
    input: &[u8],
    code: u16,
    media: Option<&[u8]>,
    retry: Option<&[u8]>,
) -> Result<(), CratesIoWireError> {
    let mut bytes = vec![0xa5; input.len().saturating_add(16)];
    let mut headers = [0xa5; 512];
    let response = fixture(&mut bytes, &mut headers, input, code, media, retry);
    let result = JsonResponsePolicy::new(StatusCode::OK, 4096)
        .unwrap_or_else(|_| unreachable!("response policy fixture"))
        .admit(response, WallClockTimestamp::new(0))
        .map(drop);
    assert!(bytes.iter().all(|byte| *byte == 0));
    assert!(headers.iter().all(|byte| *byte == 0));
    result
}

#[test]
fn exact_success_status_and_content_type_are_mandatory() {
    for code in [
        200, 201, 202, 204, 205, 206, 299, 301, 302, 304, 307, 308, 400, 401, 403, 404, 429, 500,
        503,
    ] {
        for media in [
            None,
            Some(b"application/json".as_slice()),
            Some(b"text/html".as_slice()),
        ] {
            let result = check(br#"{"ok":true,"future":{"n":1}}"#, code, media, None);
            assert_eq!(
                result.is_ok(),
                code == 200 && media == Some(b"application/json".as_slice())
            );
        }
    }
    for media in [
        "Application/JSON",
        "application/json; charset=utf-8",
        "application/json;charset=\"UTF-8\"",
    ] {
        assert!(check(b"{}", 200, Some(media.as_bytes()), None).is_ok());
    }
    for media in [
        "application/problem+json",
        "application/json; charset=latin1",
        "application/json; x=y",
        "application/json; charset=utf-8; charset=utf-8",
        "application/json; charset=",
        "application/json;",
    ] {
        assert_eq!(
            check(b"{}", 200, Some(media.as_bytes()), None),
            Err(CratesIoWireError::ContentType)
        );
    }
}

#[test]
fn cargo_errors_on_http_success_never_become_success() {
    for body in [
        br#"{"errors":[{"detail":"private detail"}]}"#.as_slice(),
        br#"{"ok":true,"err\u006frs":[{"detail":"private detail","future":42}]}"#,
        br#"{"errors":[{"detail":"one"},{"detail":"two"}],"future":true}"#,
    ] {
        for code in [200, 201, 429, 503, 400] {
            let error = match check(body, code, Some(b"application/json"), Some(b"2")) {
                Err(error) => error,
                Ok(()) => unreachable!("provider error expected"),
            };
            let CratesIoWireError::Provider(provider) = error else {
                unreachable!("provider envelope not classified");
            };
            assert_eq!(provider.status(), status(code));
            assert!(provider.count() > 0);
            assert_eq!(
                provider.retry_after(),
                Some(RetryAfter::Delay(DelaySeconds::new(2)))
            );
            assert_eq!(
                provider.kind(),
                match code {
                    429 => ProviderErrorKind::RateLimited,
                    503 => ProviderErrorKind::Unavailable,
                    200 | 201 => ProviderErrorKind::CargoErrorOnSuccess,
                    _ => ProviderErrorKind::Other,
                }
            );
            assert!(
                !format!("{error:?} {error} {provider:?} {provider}").contains("private detail")
            );
            assert!(error.source().is_none());
            assert!(provider.source().is_none());
        }
    }
}

#[test]
fn malformed_envelopes_duplicates_and_unknown_nested_values_fail_closed() {
    for body in [
        b"".as_slice(),
        b"[]",
        b"null",
        b"true",
        b"1",
        b"{}{}",
        b"{",
        b"{\"s\":\"\xff\"}",
        br#"{"errors":[]}"#,
        br#"{"errors":null}"#,
        br#"{"errors":false}"#,
        br#"{"errors":[1]}"#,
        br#"{"errors":[{}]}"#,
        br#"{"errors":[{"detail":null}]}"#,
        br#"{"errors":[{"detail":{}}]}"#,
        br#"{"errors":[{"future":{"detail":"not an error detail"}}]}"#,
        br#"{"ok":true,"ok":false}"#,
        br#"{"unknown":{"x":1,"\u0078":2}}"#,
        br#"{"unknown":"\uD800"}"#,
        br#"{"unknown":1e9999999}"#,
        br#"{"errors":[{"detail":"private"}],"errors":[]}"#,
    ] {
        assert!(check(body, 200, Some(b"application/json"), None).is_err());
    }
    assert!(
        check(
            br#"{"unknown":{"errors":[]},"ok":true}"#,
            200,
            Some(b"application/json"),
            None
        )
        .is_ok()
    );
    let deep = format!("{{\"future\":{}0{}}}", "[".repeat(65), "]".repeat(65));
    assert_eq!(
        check(deep.as_bytes(), 200, Some(b"application/json"), None),
        Err(CratesIoWireError::Json)
    );
}

#[test]
fn retry_after_missing_valid_invalid_overflow_and_dates() {
    assert!(check(b"{}", 200, Some(b"application/json"), None).is_ok());
    for valid in [
        "0",
        "1",
        "18446744073709551615",
        "Sun, 06 Nov 1994 08:49:37 GMT",
    ] {
        assert!(
            check(
                b"{}",
                200,
                Some(b"application/json"),
                Some(valid.as_bytes())
            )
            .is_ok()
        );
    }
    for invalid in [
        "",
        "-1",
        "+1",
        "1.5",
        "1,2",
        "18446744073709551616",
        "tomorrow",
    ] {
        assert_eq!(
            check(
                b"{}",
                200,
                Some(b"application/json"),
                Some(invalid.as_bytes())
            ),
            Err(CratesIoWireError::RetryAfter)
        );
    }
}

#[test]
fn policy_and_writer_byte_bounds_are_enforced_and_storage_cleared() {
    for (code, limit) in [
        (200, 0),
        (200, MAX_JSON_RESPONSE_BYTES.saturating_add(1)),
        (204, 1),
        (205, 1),
        (302, 1),
        (500, 1),
    ] {
        assert!(JsonResponsePolicy::new(status(code), limit).is_err());
    }
    let policy =
        JsonResponsePolicy::new(status(200), 2).unwrap_or_else(|_| unreachable!("policy fixture"));
    assert_eq!(policy.maximum_bytes(), 2);
    let mut bytes = [0xa5; 32];
    let mut headers = [0xa5; 256];
    let response = fixture(
        &mut bytes,
        &mut headers,
        b"{} ",
        200,
        Some(b"application/json"),
        None,
    );
    assert!(matches!(
        policy.admit(response, WallClockTimestamp::new(0)),
        Err(CratesIoWireError::ResponseTooLarge)
    ));
    assert!(bytes.iter().all(|byte| *byte == 0));
    let response = fixture(
        &mut bytes,
        &mut headers,
        b"{}",
        200,
        Some(b"application/json"),
        None,
    );
    assert!(policy.admit(response, WallClockTimestamp::new(0)).is_ok());
    let uncommitted = ResponseBuffer::new(&mut bytes, 2, &mut headers);
    assert!(matches!(
        policy.admit(uncommitted, WallClockTimestamp::new(0)),
        Err(CratesIoWireError::Uncommitted)
    ));
}

#[test]
fn success_visitors_run_only_after_complete_admission_and_are_cleanup_owned() {
    struct Counter(usize);
    impl IncrementalJsonVisitor for Counter {
        type Error = core::convert::Infallible;
        fn visit(&mut self, _: IncrementalJsonEvent<'_>) -> Result<VisitControl, Self::Error> {
            self.0 = self.0.saturating_add(1);
            Ok(VisitControl::Continue)
        }
    }
    let mut bytes = [0xa5; 64];
    let mut headers = [0xa5; 256];
    let response = fixture(
        &mut bytes,
        &mut headers,
        br#"{"ok":true}"#,
        200,
        Some(b"application/json"),
        None,
    );
    let success = JsonResponsePolicy::new(status(200), 64)
        .unwrap_or_else(|_| unreachable!("policy fixture"))
        .admit(response, WallClockTimestamp::new(0))
        .unwrap_or_else(|_| unreachable!("valid envelope"));
    assert_eq!(success.retry_after(), None);
    assert!(!format!("{success:?}").contains("true"));
    let mut counter = Counter(0);
    assert_eq!(
        success.visit(&mut counter),
        Ok(IncrementalJsonProgress::Complete)
    );
    assert_eq!(counter.0, 4);
    assert!(bytes.iter().all(|byte| *byte == 0));
    assert!(headers.iter().all(|byte| *byte == 0));
}
