use alloc::vec;

use cloud_sdk::Method;
use cloud_sdk::operation::{
    OperationImpact, PreparationStorage, PrepareOperation, RequestBodySensitivity,
    RequestSemantics, ResponseBodyPolicy, RetryEligibility,
};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::robot::server::RobotServerNumber;

const SERVER_AVAILABLE: &[u8] = br#"{"cancellation":{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8::","server_number":321,"server_name":"server-1","earliest_cancellation_date":"2028-02-29","cancelled":false,"reservation_possible":true,"reserved":false,"cancellation_date":null,"cancellation_reason":["price","migration"]}}"#;
const SERVER_CANCELLED: &[u8] = br#"{"cancellation":{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8::","server_number":321,"server_name":"server-1","earliest_cancellation_date":"2028-02-29","cancelled":true,"reservation_possible":true,"reserved":true,"cancellation_date":"2028-03-01","cancellation_reason":"migration"}}"#;
const IP_CANCELLED: &[u8] = br#"{"cancellation":{"ip":"192.0.2.10","server_number":"321","earliest_cancellation_date":"2028-02-29","cancelled":true,"cancellation-date":"2028-03-01"}}"#;
const SUBNET_AVAILABLE: &[u8] = br#"{"cancellation":{"ip":"2001:db8::","mask":"64","server_number":321,"earliest_cancellation_date":"2028-02-29","cancelled":false,"cancellation_date":null}}"#;

#[test]
fn prepares_all_nine_source_locked_operations() {
    assert_prepared(
        RobotServerCancellationGetRequest::new(number()),
        Method::Get,
        "/server/321/cancellation",
        b"",
    );
    assert_prepared(
        RobotServerCancellationCreateRequest::new(
            number(),
            RobotCancellationSchedule::Immediate,
            None,
            RobotLocationReservationIntent::Omit,
        ),
        Method::Post,
        "/server/321/cancellation",
        b"cancellation_date=now",
    );
    assert_prepared(
        RobotServerCancellationDeleteRequest::new(number()),
        Method::Delete,
        "/server/321/cancellation",
        b"",
    );
    assert_prepared(
        RobotIpCancellationGetRequest::new(ip()),
        Method::Get,
        "/ip/192.0.2.10/cancellation",
        b"",
    );
    assert_prepared(
        RobotIpCancellationCreateRequest::new(ip(), RobotCancellationSchedule::Immediate),
        Method::Post,
        "/ip/192.0.2.10/cancellation",
        b"cancellation_date=now",
    );
    assert_prepared(
        RobotIpCancellationDeleteRequest::new(ip()),
        Method::Delete,
        "/ip/192.0.2.10/cancellation",
        b"",
    );
    assert_prepared(
        RobotSubnetCancellationGetRequest::new(subnet()),
        Method::Get,
        "/subnet/2001:db8::/cancellation",
        b"",
    );
    assert_prepared(
        RobotSubnetCancellationCreateRequest::new(subnet(), RobotCancellationSchedule::Immediate),
        Method::Post,
        "/subnet/2001:db8::/cancellation",
        b"cancellation_date=now",
    );
    assert_prepared(
        RobotSubnetCancellationDeleteRequest::new(subnet()),
        Method::Delete,
        "/subnet/2001:db8::/cancellation",
        b"",
    );
}

#[test]
fn server_form_is_explicit_sensitive_and_non_retryable() {
    let reason = RobotCancellationReason::new("move to EU")
        .unwrap_or_else(|_| unreachable!("fixture reason failed"));
    let date = RobotCancellationDate::new("2028-03-01")
        .unwrap_or_else(|_| unreachable!("fixture date failed"));
    let request = RobotServerCancellationCreateRequest::new(
        number(),
        RobotCancellationSchedule::On(date),
        Some(reason),
        RobotLocationReservationIntent::DoNotReserve,
    );
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 256];
    let prepared = request
        .prepare(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("fixture preparation failed"));
    assert_eq!(
        prepared.transport_request().body(),
        b"cancellation_date=2028-03-01&cancellation_reason=move+to+EU&reserve_location=false"
    );
    assert_eq!(
        prepared.body_sensitivity(),
        RequestBodySensitivity::Sensitive
    );
    assert_eq!(prepared.metadata().impact(), OperationImpact::Destructive);
    assert_eq!(
        prepared.metadata().semantics(),
        RequestSemantics::NonIdempotent
    );
    assert_eq!(
        prepared.metadata().retry_eligibility(),
        RetryEligibility::Never
    );
}

#[test]
fn delete_response_shapes_are_target_specific_and_non_retryable() {
    let server = RobotServerCancellationDeleteRequest::new(number());
    let ip = RobotIpCancellationDeleteRequest::new(ip());
    let subnet = RobotSubnetCancellationDeleteRequest::new(subnet());
    let mut server_target = [0_u8; 128];
    let mut ip_target = [0_u8; 128];
    let mut subnet_target = [0_u8; 128];
    let mut server_body = [0_u8; 1];
    let mut ip_body = [0_u8; 1];
    let mut subnet_body = [0_u8; 1];
    let prepared = server
        .prepare(PreparationStorage::new(
            &mut server_target,
            &mut server_body,
        ))
        .unwrap_or_else(|_| unreachable!("server delete preparation failed"));
    assert_eq!(
        prepared.response_policy().body_policy(),
        ResponseBodyPolicy::Forbidden
    );
    assert_eq!(prepared.metadata().impact(), OperationImpact::Destructive);
    assert_eq!(
        prepared.metadata().semantics(),
        RequestSemantics::Idempotent
    );
    assert_eq!(
        prepared.metadata().retry_eligibility(),
        RetryEligibility::Never
    );
    for prepared in [
        ip.prepare(PreparationStorage::new(&mut ip_target, &mut ip_body))
            .unwrap_or_else(|_| unreachable!("IP delete preparation failed")),
        subnet
            .prepare(PreparationStorage::new(
                &mut subnet_target,
                &mut subnet_body,
            ))
            .unwrap_or_else(|_| unreachable!("subnet delete preparation failed")),
    ] {
        assert_eq!(
            prepared.response_policy().body_policy(),
            ResponseBodyPolicy::Required
        );
        assert_eq!(
            prepared.metadata().retry_eligibility(),
            RetryEligibility::Never
        );
    }
}

#[test]
fn protected_value_validation_is_canonical_and_calendar_exact() {
    for invalid in ["192.168.001.1", "2001:0db8::", "2001:DB8::", "not-an-ip"] {
        assert!(RobotIpAddress::new(invalid).is_err());
    }
    for invalid in ["2027-02-29", "2028-2-29", "0000-01-01"] {
        assert!(RobotCancellationDate::new(invalid).is_err());
    }
    assert!(RobotIpAddress::new("2001:db8::").is_ok());
    assert!(RobotCancellationDate::new("2028-02-29").is_ok());
    assert!(RobotCancellationReason::new("line\nbreak").is_err());
    for unsafe_text in [
        "next\u{0085}line",
        "trusted\u{202e}txt",
        "zero\u{200b}width",
    ] {
        assert!(RobotCancellationReason::new(unsafe_text).is_err());
    }
}

#[test]
fn decodes_server_reason_shapes_and_reservation_state() {
    let available = decode_server(SERVER_AVAILABLE, number());
    let Ok(available) = available else {
        unreachable!("available fixture failed")
    };
    assert!(!available.is_cancelled());
    assert!(available.reservation_possible());
    assert!(!available.is_reserved());
    assert_eq!(available.reason().available().map(<[_]>::len), Some(2));
    assert!(available.cancellation_date().is_none());

    let cancelled = decode_server(SERVER_CANCELLED, number());
    let Ok(cancelled) = cancelled else {
        unreachable!("cancelled fixture failed")
    };
    assert!(cancelled.is_cancelled());
    assert!(cancelled.is_reserved());
    assert!(matches!(cancelled.reason().selected(), Some(Some(_))));
}

#[test]
fn decodes_documented_date_spellings_and_server_number_shapes() {
    let result = decode_ip(IP_CANCELLED, ip());
    let Ok(result) = result else {
        unreachable!("IP fixture failed")
    };
    assert!(result.is_cancelled());
    assert_eq!(result.server_number().with_number(|value| value), 321);

    let result = decode_subnet(SUBNET_AVAILABLE, subnet());
    let Ok(result) = result else {
        unreachable!("subnet fixture failed")
    };
    assert_eq!(result.prefix(), 64);
    assert!(!result.is_cancelled());
}

#[test]
fn rejects_identity_date_state_reservation_and_subnet_conflicts() {
    assert!(matches!(
        decode_server(
            SERVER_AVAILABLE,
            RobotServerNumber::new(999).unwrap_or_else(|_| unreachable!("number fixture failed"))
        ),
        Err(RobotCancellationDecodeError::ResponseIdentityMismatch)
    ));
    let missing_date = SERVER_CANCELLED.windows(10).count();
    assert!(missing_date > 0);
    let state = text(SERVER_CANCELLED).replace("\"2028-03-01\"", "null");
    assert!(matches!(
        decode_server(state.as_bytes(), number()),
        Err(RobotCancellationDecodeError::StateConflict)
    ));
    let reservation = text(SERVER_CANCELLED).replace(
        "\"reservation_possible\":true",
        "\"reservation_possible\":false",
    );
    assert!(matches!(
        decode_server(reservation.as_bytes(), number()),
        Err(RobotCancellationDecodeError::StateConflict)
    ));
    let early = text(IP_CANCELLED).replace("2028-03-01", "2028-01-01");
    assert!(matches!(
        decode_ip(early.as_bytes(), ip()),
        Err(RobotCancellationDecodeError::InvalidDate)
    ));
    let host_bits = text(SUBNET_AVAILABLE).replace("2001:db8::", "2001:db8::1");
    assert!(matches!(
        decode_subnet(
            host_bits.as_bytes(),
            RobotSubnetAddress::new("2001:db8::1")
                .unwrap_or_else(|_| unreachable!("subnet fixture failed"))
        ),
        Err(RobotCancellationDecodeError::InvalidIdentifier)
    ));
}

#[test]
fn failures_clear_caller_storage_and_diagnostics_are_redacted() {
    let reason = RobotCancellationReason::new("classified-reason")
        .unwrap_or_else(|_| unreachable!("fixture reason failed"));
    let request = RobotServerCancellationCreateRequest::new(
        number(),
        RobotCancellationSchedule::Immediate,
        Some(reason),
        RobotLocationReservationIntent::Reserve,
    );
    let mut target = [0x5a_u8; 4];
    let mut body = [0x5a_u8; 128];
    assert!(
        request
            .prepare(PreparationStorage::new(&mut target, &mut body))
            .is_err()
    );
    assert_eq!(target, [0; 4]);
    assert_eq!(body, [0; 128]);
    let diagnostics = alloc::format!(
        "{request:?} {:?} {:?}",
        ip(),
        RobotCancellationDate::new("2028-03-01").unwrap_or_else(|_| unreachable!("date failed"))
    );
    for secret in ["321", "192.0.2.10", "2028", "classified-reason"] {
        assert!(!diagnostics.contains(secret));
    }
}

fn assert_prepared<R: PrepareOperation>(request: R, method: Method, path: &str, body: &[u8]) {
    let mut target = [0_u8; 128];
    let mut body_storage = [0_u8; 256];
    let prepared = request
        .prepare(PreparationStorage::new(&mut target, &mut body_storage))
        .unwrap_or_else(|_| unreachable!("fixture preparation failed"));
    assert_eq!(prepared.transport_request().method(), method);
    assert_eq!(prepared.transport_request().target().as_str(), path);
    assert_eq!(prepared.transport_request().body(), body);
}

fn number() -> RobotServerNumber {
    RobotServerNumber::new(321).unwrap_or_else(|_| unreachable!("number fixture failed"))
}
fn ip() -> RobotIpAddress {
    RobotIpAddress::new("192.0.2.10").unwrap_or_else(|_| unreachable!("IP fixture failed"))
}
fn subnet() -> RobotSubnetAddress {
    RobotSubnetAddress::new("2001:db8::").unwrap_or_else(|_| unreachable!("subnet fixture failed"))
}
fn text(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).unwrap_or_else(|_| unreachable!("fixture lost UTF-8"))
}

fn decode_server(
    body: &[u8],
    expected: RobotServerNumber,
) -> Result<RobotServerCancellation, RobotCancellationDecodeError> {
    let request = RobotServerCancellationGetRequest::new(expected);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("server cancellation preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}
fn decode_ip(
    body: &[u8],
    expected: RobotIpAddress,
) -> Result<RobotIpCancellation, RobotCancellationDecodeError> {
    let request = RobotIpCancellationGetRequest::new(expected);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("IP cancellation preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}
fn decode_subnet(
    body: &[u8],
    expected: RobotSubnetAddress,
) -> Result<RobotSubnetCancellation, RobotCancellationDecodeError> {
    let request = RobotSubnetCancellationGetRequest::new(expected);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("subnet cancellation preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn with_json<R, O>(
    prepared: PreparedCancellation<'_, '_, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedCancellation<'_, '_, R>) -> O,
) -> O {
    let mut response_storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 64];
    let mut response = ResponseBuffer::new(&mut response_storage, body.len(), &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("attempt failed"));
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!("headers failed"))
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!("content type failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("body failed"))
        .copy_from_slice(body);
    attempt
        .commit(StatusCode::OK, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("commit failed"));
    drop(attempt);
    let checked = prepared
        .validate_response(response)
        .unwrap_or_else(|_| unreachable!("response failed"));
    decode(checked)
}
