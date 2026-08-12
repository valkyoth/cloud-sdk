use alloc::{format, vec};

use cloud_sdk::transport::{
    HeaderSensitivity, ResponseBuffer, ResponseDecodeWorkspace, ResponseMetadata, StatusCode,
    TransportResponse,
};

use super::tests::delete_request;
use super::*;
use crate::robot::{
    RobotDecodeError, RobotFailure, RobotMacAddress, RobotProviderErrorCode, RobotSubnetAddress,
};

#[test]
fn every_source_locked_provider_failure_is_operation_bound() {
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
            "SUBNET_NOT_FOUND",
            RobotProviderErrorCode::SubnetNotFound,
        ),
        (
            Case::Update,
            404,
            "SUBNET_NOT_FOUND",
            RobotProviderErrorCode::SubnetNotFound,
        ),
        (
            Case::Update,
            500,
            "TRAFFIC_WARNING_UPDATE_FAILED",
            RobotProviderErrorCode::TrafficWarningUpdateFailed,
        ),
        (
            Case::MacGet,
            404,
            "SUBNET_NOT_FOUND",
            RobotProviderErrorCode::SubnetNotFound,
        ),
        (
            Case::MacGet,
            404,
            "MAC_NOT_AVAILABLE",
            RobotProviderErrorCode::MacNotAvailable,
        ),
        (
            Case::MacSet,
            404,
            "SUBNET_NOT_FOUND",
            RobotProviderErrorCode::SubnetNotFound,
        ),
        (
            Case::MacSet,
            404,
            "MAC_NOT_AVAILABLE",
            RobotProviderErrorCode::MacNotAvailable,
        ),
        (
            Case::MacSet,
            500,
            "MAC_FAILED",
            RobotProviderErrorCode::MacFailed,
        ),
        (
            Case::MacDelete,
            404,
            "SUBNET_NOT_FOUND",
            RobotProviderErrorCode::SubnetNotFound,
        ),
        (
            Case::MacDelete,
            404,
            "MAC_NOT_AVAILABLE",
            RobotProviderErrorCode::MacNotAvailable,
        ),
        (
            Case::MacDelete,
            500,
            "MAC_FAILED",
            RobotProviderErrorCode::MacFailed,
        ),
    ];
    for (operation, status, code, expected) in cases {
        let failure = decode(operation, status, code)
            .unwrap_or_else(|_| unreachable!("source-locked failure was rejected"));
        let RobotFailure::Provider(provider) = failure else {
            unreachable!("provider failure changed category");
        };
        assert_eq!(provider.code(), expected);
        assert!(RobotSubnetFailureCode::from_provider(expected).is_some());
    }
}

#[test]
fn cross_operation_codes_and_statuses_fail_closed() {
    assert_eq!(
        decode(Case::Get, 500, "TRAFFIC_WARNING_UPDATE_FAILED").err(),
        Some(RobotDecodeError::UnsupportedStatus)
    );
    assert_eq!(
        decode(Case::MacGet, 500, "MAC_FAILED").err(),
        Some(RobotDecodeError::UnsupportedStatus)
    );
    assert_eq!(
        decode(Case::List, 404, "SUBNET_NOT_FOUND").err(),
        Some(RobotDecodeError::UnknownCode)
    );
    assert_eq!(
        decode(Case::Update, 500, "MAC_FAILED").err(),
        Some(RobotDecodeError::UnknownCode)
    );
}

#[test]
fn invalid_input_is_admitted_only_for_the_update_operation() {
    let body = br#"{"error":{"status":400,"code":"INVALID_INPUT","message":"redacted","missing":null,"invalid":["traffic_hourly"]}}"#;
    assert!(matches!(
        decode_body(Case::Update, 400, body),
        Ok(RobotFailure::InvalidInput(_))
    ));
    assert_eq!(
        decode_body(Case::List, 400, body).err(),
        Some(RobotDecodeError::UnsupportedStatus)
    );
}

#[derive(Clone, Copy)]
enum Case {
    List,
    Get,
    Update,
    MacGet,
    MacSet,
    MacDelete,
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
    with_response(status, body, |response, workspace| match operation {
        Case::List => RobotSubnetListRequest::all().decode_failure(response, workspace),
        Case::Get => RobotSubnetGetRequest::new(address()).decode_failure(response, workspace),
        Case::Update => {
            RobotSubnetUpdateRequest::new(address(), RobotSubnetTrafficUpdate::warnings(true))
                .decode_failure(response, workspace)
        }
        Case::MacGet => {
            RobotSubnetMacGetRequest::new(address()).decode_failure(response, workspace)
        }
        Case::MacSet => {
            RobotSubnetMacSetRequest::new(address(), mac()).decode_failure(response, workspace)
        }
        Case::MacDelete => delete_request().decode_failure(response, workspace),
    })
}

fn with_response<R>(
    status: u16,
    body: &[u8],
    decode: impl FnOnce(TransportResponse<'_, '_>, &mut ResponseDecodeWorkspace) -> R,
) -> R {
    let mut storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 128];
    let capacity = storage.len();
    let mut response = ResponseBuffer::new(&mut storage, capacity, &mut headers);
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
        .commit(
            StatusCode::new(status).unwrap_or_else(|| unreachable!()),
            body.len(),
            ResponseMetadata::EMPTY,
        )
        .unwrap_or_else(|_| unreachable!());
    drop(attempt);
    let mut workspace = ResponseDecodeWorkspace::new_for_provider();
    response
        .with_response(|response| decode(response, &mut workspace))
        .unwrap_or_else(|_| unreachable!())
}

fn address() -> RobotSubnetAddress {
    RobotSubnetAddress::new("2001:db8::").unwrap_or_else(|_| unreachable!())
}

fn mac() -> RobotMacAddress {
    RobotMacAddress::new("00:21:85:62:3e:9d").unwrap_or_else(|_| unreachable!())
}
