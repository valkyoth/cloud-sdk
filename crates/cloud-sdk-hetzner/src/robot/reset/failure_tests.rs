use alloc::{format, vec};

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{
    HeaderSensitivity, ResponseBuffer, ResponseDecodeWorkspace, ResponseMetadata, StatusCode,
    TransportResponse,
};

use super::*;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode, RobotServerNumber};

#[derive(Clone, Copy)]
enum Case {
    List,
    Get,
    Execute,
}

#[test]
fn every_source_locked_reset_failure_is_operation_bound() {
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
            "SERVER_NOT_FOUND",
            RobotProviderErrorCode::ServerNotFound,
        ),
        (
            Case::Get,
            404,
            "RESET_NOT_AVAILABLE",
            RobotProviderErrorCode::ResetNotAvailable,
        ),
        (
            Case::Execute,
            404,
            "SERVER_NOT_FOUND",
            RobotProviderErrorCode::ServerNotFound,
        ),
        (
            Case::Execute,
            404,
            "RESET_NOT_AVAILABLE",
            RobotProviderErrorCode::ResetNotAvailable,
        ),
        (
            Case::Execute,
            409,
            "RESET_MANUAL_ACTIVE",
            RobotProviderErrorCode::ResetManualActive,
        ),
        (
            Case::Execute,
            500,
            "RESET_FAILED",
            RobotProviderErrorCode::ResetFailed,
        ),
    ];
    for (operation, status, code, expected) in cases {
        let failure = decode(operation, status, code)
            .unwrap_or_else(|_| unreachable!("source-locked reset failure was rejected"));
        let RobotFailure::Provider(provider) = failure else {
            unreachable!("reset provider failure changed category")
        };
        assert_eq!(provider.code(), expected);
    }
}

#[test]
fn reset_failure_cross_operation_widening_fails_closed() {
    assert_eq!(
        decode(Case::List, 404, "RESET_NOT_AVAILABLE").err(),
        Some(RobotDecodeError::UnknownCode)
    );
    assert_eq!(
        decode(Case::Get, 409, "RESET_MANUAL_ACTIVE").err(),
        Some(RobotDecodeError::UnsupportedStatus)
    );
    assert_eq!(
        decode(Case::Execute, 500, "MAC_FAILED").err(),
        Some(RobotDecodeError::UnknownCode)
    );
}

#[test]
fn invalid_input_is_admitted_only_for_reset_execution() {
    let body = br#"{"error":{"status":400,"code":"INVALID_INPUT","message":"redacted","missing":null,"invalid":["type"]}}"#;
    assert!(matches!(
        decode_body(Case::Execute, 400, body),
        Ok(RobotFailure::InvalidInput(_))
    ));
    assert_eq!(
        decode_body(Case::Get, 400, body).err(),
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
        Case::List => RobotResetListRequest::new().decode_failure(response, workspace),
        Case::Get => RobotResetGetRequest::new(number()).decode_failure(response, workspace),
        Case::Execute => {
            let request = RobotResetGetRequest::new(number());
            let mut target = [0_u8; 128];
            let mut body = [0_u8; 1];
            let prepared = request
                .prepare_bound(PreparationStorage::new(&mut target, &mut body))
                .unwrap_or_else(|_| unreachable!("reset fixture preparation failed"));
            let bytes = br#"{"reset":{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8::","server_number":321,"type":["hw"],"operating_status":"running"}}"#;
            let reset = with_json(prepared, bytes, |checked| checked.decode_response())
                .unwrap_or_else(|_| unreachable!("reset fixture decoding failed"));
            let execute = RobotResetExecuteRequest::from_checked(
                &reset,
                RobotResetIntent::Execute(RobotResetType::Hardware),
            )
            .unwrap_or_else(|_| unreachable!("checked reset fixture rejected"));
            execute.decode_failure(response, workspace)
        }
    }
}

fn with_json<R, O>(
    prepared: PreparedRobotReset<'_, '_, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotReset<'_, '_, R>) -> O,
) -> O {
    let mut storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut storage, body.len(), &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!());
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!())
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!());
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!())
        .copy_from_slice(body);
    attempt
        .commit(StatusCode::OK, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!());
    drop(attempt);
    decode(
        prepared
            .validate_response(response)
            .unwrap_or_else(|_| unreachable!()),
    )
}

fn number() -> RobotServerNumber {
    RobotServerNumber::new(321).unwrap_or_else(|_| unreachable!("server number failed"))
}
