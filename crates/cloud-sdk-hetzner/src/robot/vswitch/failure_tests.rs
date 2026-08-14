use alloc::{format, vec};

use cloud_sdk::transport::{
    HeaderSensitivity, ResponseBuffer, ResponseDecodeWorkspace, ResponseMetadata, StatusCode,
    TransportResponse,
};

use super::*;
use crate::robot::{
    RobotCancellationSchedule, RobotDecodeError, RobotFailure, RobotProviderErrorCode,
};

use super::tests::{id, name, selector, vlan};

#[derive(Clone, Copy)]
enum Case {
    List,
    Create,
    Get,
    Update,
    Cancel,
    Add,
    Remove,
}

#[test]
fn every_source_locked_failure_is_operation_bound() {
    let cases = [
        (
            Case::Get,
            404,
            "NOT_FOUND",
            RobotProviderErrorCode::NotFound,
        ),
        (
            Case::Create,
            409,
            "VSWITCH_LIMIT_REACHED",
            RobotProviderErrorCode::VSwitchLimitReached,
        ),
        (
            Case::Update,
            409,
            "VSWITCH_IN_PROCESS",
            RobotProviderErrorCode::VSwitchInProcess,
        ),
        (
            Case::Update,
            409,
            "VSWITCH_VLAN_NOT_UNIQUE",
            RobotProviderErrorCode::VSwitchVlanNotUnique,
        ),
        (
            Case::Cancel,
            409,
            "CONFLICT",
            RobotProviderErrorCode::VSwitchAlreadyCancelled,
        ),
        (
            Case::Add,
            404,
            "SERVER_NOT_FOUND",
            RobotProviderErrorCode::ServerNotFound,
        ),
        (
            Case::Add,
            404,
            "VSWITCH_NOT_AVAILABLE",
            RobotProviderErrorCode::VSwitchNotAvailable,
        ),
        (
            Case::Add,
            409,
            "VSWITCH_SERVER_LIMIT_REACHED",
            RobotProviderErrorCode::VSwitchServerLimitReached,
        ),
        (
            Case::Add,
            409,
            "VSWITCH_PER_SERVER_LIMIT_REACHED",
            RobotProviderErrorCode::VSwitchPerServerLimitReached,
        ),
        (
            Case::Remove,
            409,
            "VSWITCH_IN_PROCESS",
            RobotProviderErrorCode::VSwitchInProcess,
        ),
    ];
    for (operation, status, code, expected) in cases {
        let failure = decode(operation, status, code)
            .unwrap_or_else(|_| unreachable!("source-locked vSwitch failure rejected"));
        let RobotFailure::Provider(provider) = failure else {
            unreachable!("provider failure changed category")
        };
        assert_eq!(provider.code(), expected);
    }
}

#[test]
fn cross_operation_widening_and_invalid_input_fail_closed() {
    assert_eq!(
        decode(Case::List, 404, "NOT_FOUND").err(),
        Some(RobotDecodeError::UnsupportedStatus)
    );
    assert_eq!(
        decode(Case::Remove, 409, "VSWITCH_SERVER_LIMIT_REACHED").err(),
        Some(RobotDecodeError::UnknownCode)
    );
    assert_eq!(
        decode(Case::Cancel, 409, "VSWITCH_IN_PROCESS").err(),
        Some(RobotDecodeError::UnknownCode)
    );
    let invalid = br#"{"error":{"status":400,"code":"INVALID_INPUT","message":"redacted","missing":null,"invalid":["vlan"]}}"#;
    assert!(matches!(
        decode_body(Case::Create, 400, invalid),
        Ok(RobotFailure::InvalidInput(_))
    ));
    assert!(matches!(
        decode_body(Case::Remove, 400, invalid),
        Ok(RobotFailure::InvalidInput(_))
    ));
    assert_eq!(
        decode_body(Case::Get, 400, invalid).err(),
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
    let members = [selector("321")];
    let servers = RobotVSwitchServers::new(&members)
        .unwrap_or_else(|_| unreachable!("membership fixture failed"));
    match operation {
        Case::List => RobotVSwitchListRequest::new().decode_failure(response, workspace),
        Case::Create => RobotVSwitchCreateRequest::new(name("fabric"), vlan(4000))
            .decode_failure(response, workspace),
        Case::Get => RobotVSwitchGetRequest::new(id()).decode_failure(response, workspace),
        Case::Update => {
            RobotVSwitchUpdateRequest::new(id(), RobotVSwitchUpdateIntent::ChangeVlan(vlan(4001)))
                .decode_failure(response, workspace)
        }
        Case::Cancel => RobotVSwitchCancelRequest::new(id(), RobotCancellationSchedule::Immediate)
            .decode_failure(response, workspace),
        Case::Add => {
            RobotVSwitchAddServersRequest::new(id(), servers).decode_failure(response, workspace)
        }
        Case::Remove => {
            RobotVSwitchRemoveServersRequest::new(id(), servers).decode_failure(response, workspace)
        }
    }
}
