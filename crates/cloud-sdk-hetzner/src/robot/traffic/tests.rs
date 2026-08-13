use alloc::{format, vec, vec::Vec};

use cloud_sdk::Method;
use cloud_sdk::operation::{
    BodyReplayability, OperationImpact, PreparationStorage, PrepareOperation,
    RequestBodySensitivity, RequestSemantics, RetryEligibility,
};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::robot::{RobotIpAddress, RobotSubnetAddress};

const AGGREGATE: &[u8] = br#"{"traffic":{"type":"month","from":"2026-07-01","to":"2026-07-31","data":{"192.0.2.10":{"in":1.25,"out":2,"sum":3.25}}}}"#;
const SINGLE: &[u8] = br#"{"traffic":{"data":{"2001:db8::/64":{"02":{"sum":4.5,"out":3.5,"in":1},"01":{"in":0,"out":2,"sum":2}}},"to":"2026-07-31","from":"2026-07-01","type":"month"}}"#;

#[test]
fn interval_grammar_is_bounded_source_compatible_and_ordered() {
    for (kind, from, to) in [
        (
            RobotTrafficGranularity::Day,
            "2026-07-01T00",
            "2026-07-01T23",
        ),
        (RobotTrafficGranularity::Month, "2010-09-01", "2010-09-31"),
        (RobotTrafficGranularity::Year, "2025-01", "2026-12"),
    ] {
        assert!(RobotTrafficInterval::new(kind, from, to).is_ok());
    }
    for value in [
        "0000-01-01",
        "2026-00-01",
        "2026-13-01",
        "2026-01-00",
        "2026-01-32",
    ] {
        assert_eq!(
            RobotTrafficInterval::new(RobotTrafficGranularity::Month, value, value).err(),
            Some(RobotTrafficIntervalError::InvalidBound)
        );
    }
    assert_eq!(
        RobotTrafficInterval::new(RobotTrafficGranularity::Year, "2026-02", "2026-01").err(),
        Some(RobotTrafficIntervalError::Reversed)
    );
}

#[test]
fn request_rejects_empty_duplicate_and_grouped_overflow_targets() {
    let interval = month();
    assert_eq!(
        RobotTrafficRequest::new(interval, Vec::new(), false).err(),
        Some(RobotTrafficRequestError::MissingTarget)
    );
    let targets = vec![ip_target("192.0.2.10"), subnet_target("192.0.2.10")];
    assert_eq!(
        RobotTrafficRequest::new(month(), targets, false).err(),
        Some(RobotTrafficRequestError::DuplicateTarget)
    );
    let mut targets = Vec::new();
    for index in 0..=MAX_ROBOT_TRAFFIC_SINGLE_VALUE_TARGETS {
        let third = index / 256;
        let fourth = index % 256;
        targets.push(ip_target(&format!("198.18.{third}.{fourth}")));
    }
    assert_eq!(
        RobotTrafficRequest::new(month(), targets, true).err(),
        Some(RobotTrafficRequestError::TooManyTargets)
    );
}

#[test]
fn preparation_preserves_repeated_fields_and_read_only_policy() {
    assert_eq!(ROBOT_TRAFFIC_QUOTA.max_requests(), 200);
    assert_eq!(ROBOT_TRAFFIC_QUOTA.interval().get(), 3_600);
    let request = RobotTrafficRequest::new(
        month(),
        vec![ip_target("192.0.2.10"), subnet_target("2001:db8::")],
        true,
    )
    .unwrap_or_else(|_| unreachable!("traffic request fixture failed"));
    let mut target = [0_u8; 32];
    let mut body = [0_u8; 512];
    let prepared = request
        .prepare(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|error| unreachable!("traffic preparation failed: {error:?}"));
    assert_eq!(prepared.transport_request().method(), Method::Post);
    assert_eq!(prepared.transport_request().target().as_str(), "/traffic");
    assert_eq!(
        prepared.transport_request().body(),
        b"ip%5B%5D=192.0.2.10&subnet%5B%5D=2001%3Adb8%3A%3A&from=2026-07-01&to=2026-07-31&type=month&single_values=true"
    );
    assert_eq!(prepared.metadata().impact(), OperationImpact::ReadOnly);
    assert_eq!(prepared.metadata().semantics(), RequestSemantics::Safe);
    assert_eq!(
        prepared.metadata().retry_eligibility(),
        RetryEligibility::ExplicitPolicy
    );
    assert_eq!(
        prepared.body_sensitivity(),
        RequestBodySensitivity::Sensitive
    );
    assert_eq!(prepared.body_replayability(), BodyReplayability::Replayable);
}

#[test]
fn aggregate_decode_preserves_exact_numbers_and_request_binding() {
    let request = request(false, vec![ip_target("192.0.2.10")]);
    let report = decode(&request, AGGREGATE)
        .unwrap_or_else(|_| unreachable!("aggregate traffic fixture failed"));
    assert_eq!(report.granularity(), RobotTrafficGranularity::Month);
    assert_eq!(report.len(), 1);
    let result = report
        .results()
        .first()
        .unwrap_or_else(|| unreachable!("traffic result missing"));
    assert!(result.target().prefix().is_none());
    let data = result
        .aggregate()
        .unwrap_or_else(|| unreachable!("aggregate data missing"));
    assert!(
        data.incoming()
            .try_with_lexical(|value| value == "1.25")
            .unwrap_or(false)
    );
    assert!(!format!("{report:?}").contains("192.0.2.10"));
}

#[test]
fn grouped_decode_sorts_sparse_points_and_accepts_subnet_identity() {
    let request = request(true, vec![subnet_target("2001:db8::")]);
    let report =
        decode(&request, SINGLE).unwrap_or_else(|_| unreachable!("grouped traffic fixture failed"));
    let result = report
        .results()
        .first()
        .unwrap_or_else(|| unreachable!("grouped traffic result missing"));
    assert_eq!(result.target().prefix(), Some(64));
    let points = result
        .points()
        .unwrap_or_else(|| unreachable!("grouped points missing"));
    assert_eq!(points.len(), 2);
    assert_eq!(points.first().map(RobotTrafficPoint::ordinal), Some(1));
    assert_eq!(points.get(1).map(RobotTrafficPoint::ordinal), Some(2));
}

#[test]
fn every_grouped_ordinal_boundary_is_exact() {
    for (kind, from, to, first, last, rejected) in [
        (
            RobotTrafficGranularity::Day,
            "2026-07-01T00",
            "2026-07-01T23",
            "00",
            "23",
            "24",
        ),
        (
            RobotTrafficGranularity::Month,
            "2026-07-01",
            "2026-07-31",
            "01",
            "31",
            "32",
        ),
        (
            RobotTrafficGranularity::Year,
            "2026-01",
            "2026-12",
            "01",
            "12",
            "13",
        ),
    ] {
        let interval = RobotTrafficInterval::new(kind, from, to)
            .unwrap_or_else(|_| unreachable!("ordinal interval fixture failed"));
        let request = RobotTrafficRequest::new(interval, vec![ip_target("192.0.2.10")], true)
            .unwrap_or_else(|_| unreachable!("ordinal request fixture failed"));
        let body = format!(
            r#"{{"traffic":{{"type":"{}","from":"{from}","to":"{to}","data":{{"192.0.2.10":{{"{first}":{{"in":0,"out":1,"sum":1}},"{last}":{{"in":1,"out":0,"sum":1}}}}}}}}}}"#,
            match kind {
                RobotTrafficGranularity::Day => "day",
                RobotTrafficGranularity::Month => "month",
                RobotTrafficGranularity::Year => "year",
            }
        );
        assert!(decode(&request, body.as_bytes()).is_ok());
        let invalid = body.replace(&format!("\"{last}\""), &format!("\"{rejected}\""));
        assert_eq!(
            decode(&request, invalid.as_bytes()).err(),
            Some(RobotTrafficDecodeError::InvalidPoint)
        );
    }
}

#[test]
fn decoder_fails_closed_on_mismatch_unknown_shape_and_negative_amounts() {
    let aggregate_request = request(false, vec![ip_target("192.0.2.10")]);
    let wrong_range = text(AGGREGATE).replace("2026-07-31", "2026-08-01");
    assert_eq!(
        decode(&aggregate_request, wrong_range.as_bytes()).err(),
        Some(RobotTrafficDecodeError::IntervalMismatch)
    );
    let wrong_target = text(AGGREGATE).replace("192.0.2.10", "192.0.2.11");
    assert_eq!(
        decode(&aggregate_request, wrong_target.as_bytes()).err(),
        Some(RobotTrafficDecodeError::InvalidTarget)
    );
    let negative = text(AGGREGATE).replace("1.25", "-1.25");
    assert_eq!(
        decode(&aggregate_request, negative.as_bytes()).err(),
        Some(RobotTrafficDecodeError::InvalidAmount)
    );
    let extra = text(AGGREGATE).replace("\"data\"", "\"future\":true,\"data\"");
    assert_eq!(
        decode(&aggregate_request, extra.as_bytes()).err(),
        Some(RobotTrafficDecodeError::InvalidEnvelope)
    );
    let grouped = request(true, vec![ip_target("192.0.2.10")]);
    let empty = br#"{"traffic":{"type":"month","from":"2026-07-01","to":"2026-07-31","data":{"192.0.2.10":{}}}}"#;
    assert_eq!(
        decode(&grouped, empty).err(),
        Some(RobotTrafficDecodeError::InvalidPoint)
    );
}

#[test]
fn incremental_decode_crosses_the_internal_chunk_boundary() {
    let request = request(false, vec![ip_target("192.0.2.10")]);
    let mut body = vec![b' '; 4_095];
    body.extend_from_slice(AGGREGATE);
    assert!(decode(&request, &body).is_ok());
}

#[test]
fn failed_preparation_clears_complete_storage() {
    let request = request(false, vec![ip_target("192.0.2.10")]);
    let mut target = [0xa5_u8; 4];
    let mut body = [0x5a_u8; 4];
    assert!(
        request
            .prepare(PreparationStorage::new(&mut target, &mut body))
            .is_err()
    );
    assert_eq!(target, [0_u8; 4]);
    assert_eq!(body, [0_u8; 4]);
}

#[test]
fn source_locked_failures_are_narrowed_by_status_and_code() {
    let invalid = decode_failure_body(
        400,
        br#"{"error":{"status":400,"code":"INVALID_INPUT","message":"redacted","missing":null,"invalid":["from"]}}"#,
    )
    .unwrap_or_else(|_| unreachable!("traffic invalid-input fixture was rejected"));
    let crate::robot::RobotFailure::InvalidInput(invalid) = invalid else {
        unreachable!("traffic invalid-input category changed")
    };
    assert_eq!(invalid.missing_len(), 0);
    assert_eq!(invalid.invalid_len(), 1);

    for (status, code, expected) in [
        (
            404,
            "NOT_FOUND",
            crate::robot::RobotProviderErrorCode::NotFound,
        ),
        (
            500,
            "INTERNAL_ERROR",
            crate::robot::RobotProviderErrorCode::TrafficInternalError,
        ),
    ] {
        let failure = decode_failure(status, code)
            .unwrap_or_else(|_| unreachable!("traffic failure fixture was rejected"));
        let crate::robot::RobotFailure::Provider(provider) = failure else {
            unreachable!("traffic failure category changed")
        };
        assert_eq!(provider.code(), expected);
    }
    assert_eq!(
        decode_failure(500, "NOT_FOUND").err(),
        Some(crate::robot::RobotDecodeError::UnknownCode)
    );
}

fn month() -> RobotTrafficInterval {
    RobotTrafficInterval::new(RobotTrafficGranularity::Month, "2026-07-01", "2026-07-31")
        .unwrap_or_else(|_| unreachable!("traffic interval fixture failed"))
}

fn request(single: bool, targets: Vec<RobotTrafficTarget>) -> RobotTrafficRequest {
    RobotTrafficRequest::new(month(), targets, single)
        .unwrap_or_else(|_| unreachable!("traffic request fixture failed"))
}

fn ip_target(value: &str) -> RobotTrafficTarget {
    RobotTrafficTarget::ip(
        RobotIpAddress::new(value).unwrap_or_else(|_| unreachable!("IP fixture failed")),
    )
}

fn subnet_target(value: &str) -> RobotTrafficTarget {
    RobotTrafficTarget::subnet(
        RobotSubnetAddress::new(value).unwrap_or_else(|_| unreachable!("subnet fixture failed")),
    )
}

fn decode(
    request: &RobotTrafficRequest,
    body: &[u8],
) -> Result<RobotTrafficReport, RobotTrafficDecodeError> {
    let mut target = [0_u8; 32];
    let mut request_body = vec![0_u8; 1_024];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|error| unreachable!("traffic preparation fixture failed: {error:?}"));
    let mut result = None;
    with_json(body, |response| {
        result = Some(
            prepared
                .validate_response(response)
                .unwrap_or_else(|_| unreachable!("traffic response policy failed"))
                .decode_response(),
        );
    });
    result.unwrap_or_else(|| unreachable!("traffic response was not decoded"))
}

fn with_json(body: &[u8], inspect: impl FnOnce(ResponseBuffer<'_>)) {
    let mut storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut storage, body.len(), &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("response attempt failed"));
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!("response headers failed"))
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!("content type fixture failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("response body failed"))
        .copy_from_slice(body);
    attempt
        .commit(StatusCode::OK, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("response commit failed"));
    drop(attempt);
    inspect(response);
}

fn decode_failure(
    status: u16,
    code: &str,
) -> Result<crate::robot::RobotFailure, crate::robot::RobotDecodeError> {
    let body = format!(r#"{{"error":{{"status":{status},"code":"{code}","message":"redacted"}}}}"#);
    decode_failure_body(status, body.as_bytes())
}

fn decode_failure_body(
    status: u16,
    body: &[u8],
) -> Result<crate::robot::RobotFailure, crate::robot::RobotDecodeError> {
    let mut result = None;
    with_response(status, body, |response| {
        let mut workspace = cloud_sdk::transport::ResponseDecodeWorkspace::new_for_provider();
        result = Some(
            response
                .with_response(|response| {
                    request(false, vec![ip_target("192.0.2.10")])
                        .decode_failure(response, &mut workspace)
                })
                .unwrap_or_else(|_| unreachable!("traffic failure response unavailable")),
        );
    });
    result.unwrap_or_else(|| unreachable!("traffic failure was not decoded"))
}

fn with_response(status: u16, body: &[u8], inspect: impl FnOnce(ResponseBuffer<'_>)) {
    let mut storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut storage, body.len(), &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("failure response attempt failed"));
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!("failure response headers failed"))
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!("failure content type failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("failure response body failed"))
        .copy_from_slice(body);
    attempt
        .commit(
            StatusCode::new(status).unwrap_or_else(|| unreachable!("failure status invalid")),
            body.len(),
            ResponseMetadata::EMPTY,
        )
        .unwrap_or_else(|_| unreachable!("failure response commit failed"));
    drop(attempt);
    inspect(response);
}

fn text(value: &[u8]) -> &str {
    core::str::from_utf8(value).unwrap_or_else(|_| unreachable!("fixture lost UTF-8"))
}
