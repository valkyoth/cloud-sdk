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
    Create,
    Get,
    Update,
    Delete,
}

#[test]
fn every_source_locked_failure_is_operation_bound() {
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
            "NOT_FOUND",
            RobotProviderErrorCode::NotFound,
        ),
        (
            Case::Create,
            409,
            "KEY_ALREADY_EXISTS",
            RobotProviderErrorCode::SshKeyAlreadyExists,
        ),
        (
            Case::Create,
            500,
            "KEY_CREATE_FAILED",
            RobotProviderErrorCode::SshKeyCreateFailed,
        ),
        (
            Case::Update,
            500,
            "KEY_UPDATE_FAILED",
            RobotProviderErrorCode::SshKeyUpdateFailed,
        ),
        (
            Case::Delete,
            500,
            "KEY_DELETE_FAILED",
            RobotProviderErrorCode::SshKeyDeleteFailed,
        ),
    ];
    for (operation, status, code, expected) in cases {
        let failure = decode(operation, status, code)
            .unwrap_or_else(|_| unreachable!("source-locked SSH-key failure rejected"));
        let RobotFailure::Provider(provider) = failure else {
            unreachable!("provider failure changed category")
        };
        assert_eq!(provider.code(), expected);
    }
}

#[test]
fn cross_operation_widening_and_invalid_input_fail_closed() {
    assert_eq!(
        decode(Case::List, 409, "KEY_ALREADY_EXISTS").err(),
        Some(RobotDecodeError::UnsupportedStatus)
    );
    assert_eq!(
        decode(Case::Delete, 500, "KEY_CREATE_FAILED").err(),
        Some(RobotDecodeError::UnknownCode)
    );
    let invalid = br#"{"error":{"status":400,"code":"INVALID_INPUT","message":"redacted","missing":null,"invalid":["name"]}}"#;
    assert!(matches!(
        decode_body(Case::Create, 400, invalid),
        Ok(RobotFailure::InvalidInput(_))
    ));
    assert!(matches!(
        decode_body(Case::Update, 400, invalid),
        Ok(RobotFailure::InvalidInput(_))
    ));
    assert_eq!(
        decode_body(Case::Delete, 400, invalid).err(),
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
        .unwrap_or_else(|_| unreachable!("failure attempt failed"));
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!("failure headers failed"))
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!("failure content type failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("failure body failed"))
        .copy_from_slice(body);
    attempt
        .commit(
            StatusCode::new(status).unwrap_or_else(|| unreachable!("invalid test status")),
            body.len(),
            ResponseMetadata::EMPTY,
        )
        .unwrap_or_else(|_| unreachable!("failure commit failed"));
    drop(attempt);
    let mut workspace = ResponseDecodeWorkspace::new_for_provider();
    response
        .with_response(|response| dispatch(operation, response, &mut workspace))
        .unwrap_or_else(|_| unreachable!("committed failure unavailable"))
}

fn dispatch(
    operation: Case,
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    use super::tests::{data, fingerprint, name};
    match operation {
        Case::List => RobotSshKeyListRequest::new().decode_failure(response, workspace),
        Case::Create => {
            RobotSshKeyCreateRequest::new(name("key"), data()).decode_failure(response, workspace)
        }
        Case::Get => RobotSshKeyGetRequest::new(fingerprint()).decode_failure(response, workspace),
        Case::Update => RobotSshKeyUpdateRequest::new(fingerprint(), name("key"))
            .decode_failure(response, workspace),
        Case::Delete => {
            RobotSshKeyDeleteRequest::new(fingerprint()).decode_failure(response, workspace)
        }
    }
}
