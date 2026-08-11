use alloc::format;
use alloc::vec;
use alloc::vec::Vec;

use cloud_sdk::rate_limit::QuotaReset;
use cloud_sdk::transport::{
    DeliveryPhase, HeaderSensitivity, ResponseBuffer, ResponseDecodeWorkspace, ResponseMetadata,
    StatusCode, TransportFailure,
};

use super::{
    MAX_ROBOT_ERROR_BODY_BYTES, MAX_ROBOT_ERROR_MESSAGE_BYTES, MAX_ROBOT_INPUT_FIELDS,
    RobotDecodeError, RobotFailure, RobotFailureCategory, RobotProviderErrorCode,
    RobotRetryDisposition, decode_robot_failure,
};
use crate::serde::strict_json::{JsonError, with_next_failure};

const INVALID: &[u8] = br#"{
  "error": {
    "status": 400,
    "code": "INVALID_INPUT",
    "message": "invalid input sentinel-secret",
    "missing": ["server"],
    "invalid": null
  }
}"#;
const QUOTA: &[u8] = br#"{
  "error": {
    "status": 403,
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "rate limit exceeded sentinel-secret",
    "max_request": 100,
    "interval": 3600
  }
}"#;
const PROVIDER: &[u8] = br#"{
  "error": {
    "status": 404,
    "code": "SERVER_NOT_FOUND",
    "message": "server not found sentinel-secret"
  }
}"#;

#[test]
fn source_locked_failures_decode_into_distinct_categories() {
    let invalid = decode(400, INVALID, Some(b"application/json; charset=utf-8"));
    let Ok(RobotFailure::InvalidInput(invalid)) = invalid else {
        unreachable!("invalid-input fixture did not decode")
    };
    assert_eq!(invalid.missing_len(), 1);
    assert_eq!(invalid.invalid_len(), 0);
    assert_eq!(
        invalid.try_with_missing(0, |field| field == "server"),
        Ok(Some(true))
    );
    assert_eq!(invalid.try_with_missing(1, |_| ()), Ok(None));

    let quota = decode(403, QUOTA, Some(b"application/json"));
    let Ok(RobotFailure::QuotaExceeded(quota)) = quota else {
        unreachable!("quota fixture did not decode")
    };
    assert_eq!(quota.max_requests(), 100);
    let bucket = quota
        .quota_bucket()
        .unwrap_or_else(|_| unreachable!("validated quota did not convert"));
    assert_eq!(bucket.limit(), 100);
    assert_eq!(bucket.remaining(), 0);
    assert_eq!(
        bucket.reset(),
        QuotaReset::After(cloud_sdk::rate_limit::DelaySeconds::new(3600))
    );

    let provider = decode(404, PROVIDER, Some(b"application/json"));
    let Ok(RobotFailure::Provider(provider)) = provider else {
        unreachable!("provider fixture did not decode")
    };
    assert_eq!(provider.code(), RobotProviderErrorCode::ServerNotFound);
}

#[test]
fn authentication_and_maintenance_require_empty_bodies() {
    let authentication = decode(401, b"", None);
    assert_eq!(authentication, Ok(RobotFailure::AuthenticationRejected));
    let maintenance = decode(503, b"", None);
    assert_eq!(maintenance, Ok(RobotFailure::Maintenance));
    assert_eq!(
        decode(401, b"null", Some(b"application/json")),
        Err(RobotDecodeError::UnexpectedBody)
    );
    assert_eq!(
        decode(503, b"{}", Some(b"application/json")),
        Err(RobotDecodeError::UnexpectedBody)
    );
}

#[test]
fn authentication_never_retries_or_falls_back_to_transient() {
    let Ok(authentication) = decode(401, b"", None) else {
        unreachable!("authentication fixture did not decode")
    };
    assert_eq!(
        authentication.category(),
        RobotFailureCategory::AuthenticationRejected
    );
    assert_eq!(
        authentication.retry_disposition(),
        RobotRetryDisposition::Never
    );
    assert!(!authentication.allows_automatic_retry());

    let transport = TransportFailure::not_sent("redacted adapter payload");
    let transient = RobotFailure::transient_transport(&transport);
    assert_eq!(
        transient.category(),
        RobotFailureCategory::TransientTransport
    );
    assert_eq!(
        transient.retry_disposition(),
        RobotRetryDisposition::ExplicitPolicy
    );
    let RobotFailure::TransientTransport(classified) = transient else {
        unreachable!("explicit transport fixture changed category")
    };
    assert_eq!(classified.delivery_phase(), DeliveryPhase::NotSent);
}

#[test]
fn unknown_status_and_codes_fail_closed() {
    assert_eq!(
        decode(429, QUOTA, Some(b"application/json")),
        Err(RobotDecodeError::UnsupportedStatus)
    );
    assert_eq!(
        decode(500, b"", None),
        Err(RobotDecodeError::UnsupportedStatus)
    );
    assert_eq!(
        decode(
            403,
            br#"{"error":{"status":403,"code":"AUTH_FAILED","message":"x","max_request":1,"interval":1}}"#,
            Some(b"application/json"),
        ),
        Err(RobotDecodeError::UnknownCode)
    );
    assert_eq!(
        decode(
            404,
            br#"{"error":{"status":404,"code":"FUTURE_ERROR","message":"x"}}"#,
            Some(b"application/json"),
        ),
        Err(RobotDecodeError::UnknownCode)
    );
}

#[test]
fn malformed_duplicate_and_extra_fields_are_rejected() {
    assert_eq!(
        decode(400, br#"{"error": "#, Some(b"application/json")),
        Err(RobotDecodeError::MalformedPayload)
    );
    assert_eq!(
        decode(
            404,
            br#"{"error":{"status":404,"status":404,"code":"SERVER_NOT_FOUND","message":"x"}}"#,
            Some(b"application/json"),
        ),
        Err(RobotDecodeError::MalformedPayload)
    );
    assert_eq!(
        decode(
            404,
            br#"{"error":{"status":404,"code":"SERVER_NOT_FOUND","message":"x","future":true}}"#,
            Some(b"application/json"),
        ),
        Err(RobotDecodeError::InvalidEnvelope)
    );
}

#[test]
fn parser_allocation_failure_remains_distinct_from_hostile_input() {
    assert_eq!(
        super::decode::map_json_error(JsonError::Allocation),
        RobotDecodeError::Allocation
    );
    assert_eq!(
        super::decode::map_json_error(JsonError::InvalidSyntax),
        RobotDecodeError::MalformedPayload
    );
    assert_eq!(
        with_next_failure(|| decode(400, INVALID, Some(b"application/json"))),
        Err(RobotDecodeError::Allocation)
    );
}

#[test]
fn status_content_type_and_size_are_enforced_before_model_use() {
    assert_eq!(
        decode(200, PROVIDER, Some(b"application/json")),
        Err(RobotDecodeError::UnexpectedSuccessStatus)
    );
    assert_eq!(
        decode(404, b"", Some(b"application/json")),
        Err(RobotDecodeError::MissingBody)
    );
    assert_eq!(
        decode(404, PROVIDER, None),
        Err(RobotDecodeError::InvalidContentType)
    );
    assert_eq!(
        decode(404, PROVIDER, Some(b"text/plain")),
        Err(RobotDecodeError::InvalidContentType)
    );
    let oversized = vec![b' '; MAX_ROBOT_ERROR_BODY_BYTES + 1];
    assert_eq!(
        decode(404, &oversized, Some(b"application/json")),
        Err(RobotDecodeError::ResponseTooLarge)
    );
}

#[test]
fn invalid_quota_and_status_mismatch_are_rejected() {
    for body in [
        br#"{"error":{"status":403,"code":"RATE_LIMIT_EXCEEDED","message":"x","max_request":0,"interval":1}}"#.as_slice(),
        br#"{"error":{"status":403,"code":"RATE_LIMIT_EXCEEDED","message":"x","max_request":1,"interval":0}}"#.as_slice(),
    ] {
        assert_eq!(
            decode(403, body, Some(b"application/json")),
            Err(RobotDecodeError::InvalidQuota)
        );
    }
    assert_eq!(
        decode(
            404,
            br#"{"error":{"status":400,"code":"SERVER_NOT_FOUND","message":"x"}}"#,
            Some(b"application/json"),
        ),
        Err(RobotDecodeError::StatusMismatch)
    );
}

#[test]
fn invalid_input_collection_bound_is_exact() {
    let admitted = invalid_input_fixture(MAX_ROBOT_INPUT_FIELDS, 1);
    let Ok(RobotFailure::InvalidInput(invalid)) = decode(400, &admitted, Some(b"application/json"))
    else {
        unreachable!("maximum invalid-input collection did not decode")
    };
    assert_eq!(invalid.missing_len(), MAX_ROBOT_INPUT_FIELDS);

    let rejected = invalid_input_fixture(MAX_ROBOT_INPUT_FIELDS + 1, 1);
    assert_eq!(
        decode(400, &rejected, Some(b"application/json")),
        Err(RobotDecodeError::InvalidEnvelope)
    );
}

#[test]
fn provider_message_bound_is_exact() {
    let admitted = provider_fixture(MAX_ROBOT_ERROR_MESSAGE_BYTES);
    let Ok(RobotFailure::Provider(provider)) = decode(404, &admitted, Some(b"application/json"))
    else {
        unreachable!("maximum provider message did not decode")
    };
    assert_eq!(
        provider.try_with_message(str::len),
        Ok(MAX_ROBOT_ERROR_MESSAGE_BYTES)
    );

    let rejected = provider_fixture(MAX_ROBOT_ERROR_MESSAGE_BYTES + 1);
    assert_eq!(
        decode(404, &rejected, Some(b"application/json")),
        Err(RobotDecodeError::InvalidEnvelope)
    );
}

#[test]
fn diagnostics_redact_provider_text() {
    let Ok(failure) = decode(400, INVALID, Some(b"application/json")) else {
        unreachable!("redaction fixture did not decode")
    };
    let debug = format!("{failure:?}");
    assert!(debug.contains("redacted"));
    assert!(!debug.contains("sentinel-secret"));
    assert_eq!(
        format!("{}", RobotDecodeError::UnknownCode),
        "Robot error code is not source-locked"
    );
}

fn invalid_input_fixture(missing_fields: usize, message_bytes: usize) -> Vec<u8> {
    let missing = vec![r#""field""#; missing_fields].join(",");
    format!(
        r#"{{"error":{{"status":400,"code":"INVALID_INPUT","message":"{}","missing":[{missing}],"invalid":null}}}}"#,
        "m".repeat(message_bytes)
    )
    .into_bytes()
}

fn provider_fixture(message_bytes: usize) -> Vec<u8> {
    format!(
        r#"{{"error":{{"status":404,"code":"SERVER_NOT_FOUND","message":"{}"}}}}"#,
        "m".repeat(message_bytes)
    )
    .into_bytes()
}

fn decode(
    status: u16,
    body: &[u8],
    content_type: Option<&[u8]>,
) -> Result<RobotFailure, RobotDecodeError> {
    let mut storage = vec![0_u8; body.len()];
    let mut header_storage = [0_u8; 512];
    let capacity = storage.len();
    let mut response = ResponseBuffer::new(&mut storage, capacity, &mut header_storage);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("response attempt fixture failed"));
    if let Some(content_type) = content_type {
        attempt
            .headers_mut()
            .unwrap_or_else(|_| unreachable!("response header fixture failed"))
            .try_push("content-type", content_type, HeaderSensitivity::Public)
            .unwrap_or_else(|_| unreachable!("content-type fixture failed"));
    }
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("response body fixture failed"))
        .copy_from_slice(body);
    attempt
        .commit(
            StatusCode::new(status).unwrap_or_else(|| unreachable!("invalid status fixture")),
            body.len(),
            ResponseMetadata::EMPTY,
        )
        .unwrap_or_else(|_| unreachable!("response commit fixture failed"));
    drop(attempt);
    let mut workspace = ResponseDecodeWorkspace::new_for_provider();
    response
        .with_response(|response| decode_robot_failure(response, &mut workspace))
        .unwrap_or_else(|_| unreachable!("committed response fixture was unavailable"))
}
