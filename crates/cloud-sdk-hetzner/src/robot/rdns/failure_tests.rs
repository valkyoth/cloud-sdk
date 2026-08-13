use alloc::{format, vec};

use cloud_sdk::transport::{
    HeaderSensitivity, ResponseBuffer, ResponseDecodeWorkspace, ResponseMetadata, StatusCode,
    TransportResponse,
};

use super::*;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

#[derive(Clone, Copy)]
enum Case {
    List,
    Get,
    Set,
    Update,
    Delete,
}

#[test]
fn every_source_locked_rdns_failure_is_operation_bound() {
    let cases = [
        (
            Case::List,
            404,
            "NOT_FOUND",
            RobotProviderErrorCode::NotFound,
        ),
        (
            Case::Get,
            404,
            "IP_NOT_FOUND",
            RobotProviderErrorCode::IpNotFound,
        ),
        (
            Case::Get,
            404,
            "RDNS_NOT_FOUND",
            RobotProviderErrorCode::RdnsNotFound,
        ),
        (
            Case::Set,
            409,
            "RDNS_ALREADY_EXISTS",
            RobotProviderErrorCode::RdnsAlreadyExists,
        ),
        (
            Case::Set,
            500,
            "RDNS_CREATE_FAILED",
            RobotProviderErrorCode::RdnsCreateFailed,
        ),
        (
            Case::Update,
            500,
            "RDNS_UPDATE_FAILED",
            RobotProviderErrorCode::RdnsUpdateFailed,
        ),
        (
            Case::Delete,
            500,
            "RDNS_DELETE_FAILED",
            RobotProviderErrorCode::RdnsDeleteFailed,
        ),
    ];
    for (operation, status, code, expected) in cases {
        let failure = decode(operation, status, code)
            .unwrap_or_else(|_| unreachable!("source-locked reverse-DNS failure was rejected"));
        let RobotFailure::Provider(provider) = failure else {
            unreachable!("reverse-DNS provider failure changed category")
        };
        assert_eq!(provider.code(), expected);
    }
}

#[test]
fn cross_operation_failure_widening_fails_closed() {
    assert_eq!(
        decode(Case::List, 409, "RDNS_ALREADY_EXISTS").err(),
        Some(RobotDecodeError::UnsupportedStatus)
    );
    assert_eq!(
        decode(Case::Get, 404, "RDNS_CREATE_FAILED").err(),
        Some(RobotDecodeError::UnknownCode)
    );
    assert_eq!(
        decode(Case::Delete, 500, "RDNS_CREATE_FAILED").err(),
        Some(RobotDecodeError::UnknownCode)
    );
}

#[test]
fn invalid_input_is_admitted_only_for_ptr_writes() {
    let body = br#"{"error":{"status":400,"code":"INVALID_INPUT","message":"redacted","missing":null,"invalid":["ptr"]}}"#;
    assert!(matches!(
        decode_body(Case::Set, 400, body),
        Ok(RobotFailure::InvalidInput(_))
    ));
    assert!(matches!(
        decode_body(Case::Update, 400, body),
        Ok(RobotFailure::InvalidInput(_))
    ));
    assert_eq!(
        decode_body(Case::Delete, 400, body).err(),
        Some(RobotDecodeError::UnsupportedStatus)
    );
}

fn decode(operation: Case, status: u16, code: &str) -> Result<RobotFailure, RobotDecodeError> {
    let body = format!(r#"{{"error":{{"status":{status},"code":"{code}","message":"redacted"}}}}"#);
    decode_body(operation, status, body.as_bytes())
}

fn decode_body(
    operation: Case,
    status: u16,
    body: &[u8],
) -> Result<RobotFailure, RobotDecodeError> {
    let mut storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 256];
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
            StatusCode::new(status).unwrap_or_else(|| unreachable!("invalid test status")),
            body.len(),
            ResponseMetadata::EMPTY,
        )
        .unwrap_or_else(|_| unreachable!("failure response commit failed"));
    drop(attempt);
    let mut workspace = ResponseDecodeWorkspace::new_for_provider();
    response
        .with_response(|response| dispatch(operation, response, &mut workspace))
        .unwrap_or_else(|_| unreachable!("committed failure response unavailable"))
}

fn dispatch(
    operation: Case,
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    match operation {
        Case::List => RobotRdnsListRequest::all().decode_failure(response, workspace),
        Case::Get => RobotRdnsGetRequest::new(ip()).decode_failure(response, workspace),
        Case::Set => RobotRdnsSetRequest::new(ip(), ptr()).decode_failure(response, workspace),
        Case::Update => {
            RobotRdnsUpdateRequest::new(ip(), ptr()).decode_failure(response, workspace)
        }
        Case::Delete => RobotRdnsDeleteRequest::new(ip()).decode_failure(response, workspace),
    }
}

fn ip() -> crate::robot::RobotIpAddress {
    crate::robot::RobotIpAddress::new("192.0.2.50")
        .unwrap_or_else(|_| unreachable!("IP fixture failed"))
}

fn ptr() -> RobotRdnsName {
    RobotRdnsName::new("mail.example.com").unwrap_or_else(|_| unreachable!("PTR fixture failed"))
}
