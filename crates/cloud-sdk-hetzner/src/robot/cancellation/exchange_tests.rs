use alloc::vec;

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::robot::server::RobotServerNumber;

const SERVER_CANCELLED: &[u8] = br#"{"cancellation":{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8::","server_number":321,"server_name":"server-1","earliest_cancellation_date":"2028-02-29","cancelled":true,"reservation_possible":true,"reserved":true,"cancellation_date":"2028-03-01","cancellation_reason":"migration"}}"#;
const IP_CANCELLED: &[u8] = br#"{"cancellation":{"ip":"192.0.2.10","server_number":"321","earliest_cancellation_date":"2028-02-29","cancelled":true,"cancellation-date":"2028-03-01"}}"#;
const IP_AVAILABLE: &[u8] = br#"{"cancellation":{"ip":"192.0.2.10","server_number":"321","earliest_cancellation_date":"2028-02-29","cancelled":false,"cancellation-date":null}}"#;
const SUBNET_AVAILABLE: &[u8] = br#"{"cancellation":{"ip":"2001:db8::","mask":"64","server_number":321,"earliest_cancellation_date":"2028-02-29","cancelled":false,"cancellation_date":null}}"#;
const SUBNET_CANCELLED: &[u8] = br#"{"cancellation":{"ip":"2001:db8::","mask":"64","server_number":321,"earliest_cancellation_date":"2028-02-29","cancelled":true,"cancellation_date":"2028-03-01"}}"#;

#[test]
fn mutation_acknowledgements_must_match_complete_request_intent() {
    assert!(matches!(
        decode_ip_create(
            IP_CANCELLED,
            RobotCancellationSchedule::On(date("2028-03-02"))
        ),
        Err(RobotCancellationDecodeError::MutationOutcomeMismatch)
    ));
    assert!(matches!(
        decode_ip_create(IP_AVAILABLE, RobotCancellationSchedule::Immediate),
        Err(RobotCancellationDecodeError::MutationOutcomeMismatch)
    ));
    assert!(
        decode_ip_create(
            IP_CANCELLED,
            RobotCancellationSchedule::On(date("2028-03-01"))
        )
        .is_ok()
    );
    assert!(decode_ip_create(IP_CANCELLED, RobotCancellationSchedule::Immediate).is_ok());
    assert!(matches!(
        decode_subnet_create(
            SUBNET_CANCELLED,
            RobotCancellationSchedule::On(date("2028-03-02"))
        ),
        Err(RobotCancellationDecodeError::MutationOutcomeMismatch)
    ));

    let wrong_reason = RobotCancellationReason::new("price")
        .unwrap_or_else(|_| unreachable!("reason fixture failed"));
    assert!(matches!(
        decode_server_create(
            SERVER_CANCELLED,
            RobotCancellationSchedule::On(date("2028-03-01")),
            Some(wrong_reason),
            RobotLocationReservationIntent::Reserve,
        ),
        Err(RobotCancellationDecodeError::MutationOutcomeMismatch)
    ));
    let reason = RobotCancellationReason::new("migration")
        .unwrap_or_else(|_| unreachable!("reason fixture failed"));
    assert!(
        decode_server_create(
            SERVER_CANCELLED,
            RobotCancellationSchedule::On(date("2028-03-01")),
            Some(reason),
            RobotLocationReservationIntent::Reserve,
        )
        .is_ok()
    );
    let reason = RobotCancellationReason::new("migration")
        .unwrap_or_else(|_| unreachable!("reason fixture failed"));
    assert!(matches!(
        decode_server_create(
            SERVER_CANCELLED,
            RobotCancellationSchedule::On(date("2028-03-01")),
            Some(reason),
            RobotLocationReservationIntent::DoNotReserve,
        ),
        Err(RobotCancellationDecodeError::MutationOutcomeMismatch)
    ));
}

#[test]
fn delete_acknowledgements_are_bound_and_must_be_inactive() {
    let request = RobotServerCancellationDeleteRequest::new(number());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("server revocation preparation failed"));
    assert!(with_empty(prepared, |checked| checked.decode_response()).is_ok());
    assert!(decode_ip_delete(IP_AVAILABLE).is_ok());
    assert!(matches!(
        decode_ip_delete(IP_CANCELLED),
        Err(RobotCancellationDecodeError::MutationOutcomeMismatch)
    ));
    assert!(decode_subnet_delete(SUBNET_AVAILABLE).is_ok());
    assert!(matches!(
        decode_subnet_delete(SUBNET_CANCELLED),
        Err(RobotCancellationDecodeError::MutationOutcomeMismatch)
    ));
}

fn decode_ip_create(
    body: &[u8],
    schedule: RobotCancellationSchedule,
) -> Result<RobotIpCancellation, RobotCancellationDecodeError> {
    let request = RobotIpCancellationCreateRequest::new(ip(), schedule);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("IP cancellation preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_server_create(
    body: &[u8],
    schedule: RobotCancellationSchedule,
    reason: Option<RobotCancellationReason<'_>>,
    reservation: RobotLocationReservationIntent,
) -> Result<RobotServerCancellation, RobotCancellationDecodeError> {
    let request =
        RobotServerCancellationCreateRequest::new(number(), schedule, reason, reservation);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 256];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("server cancellation preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_subnet_create(
    body: &[u8],
    schedule: RobotCancellationSchedule,
) -> Result<RobotSubnetCancellation, RobotCancellationDecodeError> {
    let request = RobotSubnetCancellationCreateRequest::new(subnet(), schedule);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("subnet cancellation preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_ip_delete(body: &[u8]) -> Result<RobotIpCancellation, RobotCancellationDecodeError> {
    let request = RobotIpCancellationDeleteRequest::new(ip());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("IP revocation preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_subnet_delete(
    body: &[u8],
) -> Result<RobotSubnetCancellation, RobotCancellationDecodeError> {
    let request = RobotSubnetCancellationDeleteRequest::new(subnet());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("subnet revocation preparation failed"));
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

fn with_empty<R, O>(
    prepared: PreparedCancellation<'_, '_, R>,
    decode: impl FnOnce(CheckedCancellation<'_, '_, R>) -> O,
) -> O {
    let mut response_storage = [];
    let mut headers = [0_u8; 1];
    let mut response = ResponseBuffer::new(&mut response_storage, 0, &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("response attempt failed"));
    attempt
        .commit(StatusCode::OK, 0, ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("response commit failed"));
    drop(attempt);
    let checked = prepared
        .validate_response(response)
        .unwrap_or_else(|_| unreachable!("response failed"));
    decode(checked)
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
fn date(value: &str) -> RobotCancellationDate {
    RobotCancellationDate::new(value).unwrap_or_else(|_| unreachable!("date fixture failed"))
}
