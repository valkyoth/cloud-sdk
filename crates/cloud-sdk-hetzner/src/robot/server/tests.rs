use alloc::vec;

use cloud_sdk::Method;
use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::{
    RobotServerDecodeError, RobotServerGetRequest, RobotServerListRequest, RobotServerName,
    RobotServerNumber, RobotServerUpdateRequest,
};

const SUMMARY: &str = r#"{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8:1::","server_number":321,"server_name":"server-1","product":"AX42","dc":"FSN1-DC10","traffic":"unlimited","status":"ready","cancelled":false,"paid_until":"2028-02-29","ip":["192.0.2.10","2001:db8:1::1"],"subnet":null}"#;
const DETAIL: &str = r#"{"server":{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8:1::","server_number":321,"server_name":"server-1","product":"AX42","dc":"FSN1-DC10","traffic":"unlimited","status":"in process","cancelled":false,"paid_until":"2028-02-29","ip":["192.0.2.10"],"subnet":[{"ip":"2001:db8:2::","mask":"64"}],"reset":true,"rescue":true,"vnc":false,"windows":true,"plesk":false,"cpanel":false,"wol":true,"hot_swap":true,"linked_storagebox":42}}"#;

#[test]
fn prepares_canonical_list_get_and_rename_requests() {
    let list = prepare(RobotServerListRequest::new());
    assert_eq!(list.0, Method::Get);
    assert_eq!(list.1, "/server");
    assert!(list.2.is_empty());

    let get = prepare(RobotServerGetRequest::new(server_number(321)));
    assert_eq!(get.0, Method::Get);
    assert_eq!(get.1, "/server/321");

    let name =
        RobotServerName::new("renamed-1").unwrap_or_else(|_| unreachable!("fixture name failed"));
    let update = prepare(RobotServerUpdateRequest::rename(server_number(321), name));
    assert_eq!(update.0, Method::Post);
    assert_eq!(update.1, "/server/321");
    assert_eq!(update.2, b"server_name=renamed-1");
    assert_eq!(
        update.3.as_deref(),
        Some("application/x-www-form-urlencoded")
    );
}

#[test]
fn canonical_identity_and_name_validation_fail_closed() {
    assert!(RobotServerNumber::new(0).is_none());
    for invalid in ["", "-server", "server-", "server name", "server.example"] {
        assert!(RobotServerName::new(invalid).is_err());
    }
    assert!(RobotServerName::new("server-01").is_ok());
}

#[test]
fn preparation_failures_clear_target_and_form_storage() {
    let number =
        RobotServerNumber::new(321).unwrap_or_else(|| unreachable!("fixture number failed"));
    let name =
        RobotServerName::new("renamed-1").unwrap_or_else(|_| unreachable!("fixture name failed"));
    let request = RobotServerUpdateRequest::rename(number, name);

    let mut short_target = [0x5a_u8; 4];
    let mut body = [0x5a_u8; 64];
    assert!(
        request
            .prepare(PreparationStorage::new(&mut short_target, &mut body))
            .is_err()
    );
    assert_eq!(short_target, [0; 4]);
    assert_eq!(body, [0; 64]);

    let mut target = [0x5a_u8; 64];
    let mut short_body = [0x5a_u8; 4];
    assert!(
        request
            .prepare(PreparationStorage::new(&mut target, &mut short_body))
            .is_err()
    );
    assert_eq!(target, [0; 64]);
    assert_eq!(short_body, [0; 4]);
}

#[test]
fn decodes_list_nullability_and_detailed_capabilities() {
    let list_body = alloc::format!("[{{\"server\":{SUMMARY}}}]");
    let request = RobotServerListRequest::new();
    let list = decode_list(request, list_body.as_bytes());
    let Ok(list) = list else {
        unreachable!("valid list did not decode")
    };
    assert_eq!(list.len(), 1);
    let Some(summary) = list.as_slice().first() else {
        unreachable!("decoded list lost its server")
    };
    assert_eq!(summary.number().with_number(|number| number), 321);
    assert!(summary.status().is_ready());
    assert_eq!(summary.subnets(), None);
    assert_eq!(
        summary
            .paid_until()
            .with_date(|year, month, day| (year, month, day)),
        (2028, 2, 29)
    );
    assert_eq!(summary.try_with_name(|name| name == "server-1"), Ok(true));

    let number =
        RobotServerNumber::new(321).unwrap_or_else(|| unreachable!("fixture number failed"));
    let server = decode_detail(RobotServerGetRequest::new(number), DETAIL.as_bytes());
    let Ok(server) = server else {
        unreachable!("valid detail did not decode")
    };
    assert!(server.summary().status().is_in_process());
    assert_eq!(server.summary().subnets().map(<[_]>::len), Some(1));
    assert!(server.capabilities().wake_on_lan());
    assert_eq!(
        server
            .linked_storage_box()
            .map(|value| value.with_number(|number| number)),
        Some(42)
    );
}

#[test]
fn rejects_identity_conflicts_unknown_state_and_noncanonical_subnets() {
    let duplicate = alloc::format!("[{{\"server\":{SUMMARY}}},{{\"server\":{SUMMARY}}}]");
    assert!(matches!(
        decode_list(RobotServerListRequest::new(), duplicate.as_bytes()),
        Err(RobotServerDecodeError::DuplicateIdentity)
    ));

    let number =
        RobotServerNumber::new(999).unwrap_or_else(|| unreachable!("fixture number failed"));
    assert!(matches!(
        decode_detail(RobotServerGetRequest::new(number), DETAIL.as_bytes()),
        Err(RobotServerDecodeError::ResponseIdentityMismatch)
    ));

    let unknown = DETAIL.replace("in process", "future-state");
    assert!(matches!(
        decode_detail(
            RobotServerGetRequest::new(server_number(321)),
            unknown.as_bytes()
        ),
        Err(RobotServerDecodeError::UnknownStatus)
    ));

    let host_bits = DETAIL.replace("2001:db8:2::", "2001:db8:2::1");
    assert!(matches!(
        decode_detail(
            RobotServerGetRequest::new(server_number(321)),
            host_bits.as_bytes()
        ),
        Err(RobotServerDecodeError::InvalidSubnet)
    ));
}

#[test]
fn rejects_invalid_dates_extra_fields_and_missing_main_address() {
    let invalid_date = DETAIL.replace("2028-02-29", "2027-02-29");
    assert!(matches!(
        decode_detail(
            RobotServerGetRequest::new(server_number(321)),
            invalid_date.as_bytes()
        ),
        Err(RobotServerDecodeError::InvalidDate)
    ));
    let extra = DETAIL.replacen("\"server_ip\"", "\"future\":true,\"server_ip\"", 1);
    assert!(matches!(
        decode_detail(
            RobotServerGetRequest::new(server_number(321)),
            extra.as_bytes()
        ),
        Err(RobotServerDecodeError::InvalidEnvelope)
    ));
    let missing_main = DETAIL.replace("[\"192.0.2.10\"]", "[\"192.0.2.11\"]");
    assert!(matches!(
        decode_detail(
            RobotServerGetRequest::new(server_number(321)),
            missing_main.as_bytes()
        ),
        Err(RobotServerDecodeError::InvalidAddress)
    ));
}

#[test]
fn every_server_diagnostic_is_static_and_redacted() {
    let list_body = alloc::format!("[{{\"server\":{SUMMARY}}}]");
    let list = decode_list(RobotServerListRequest::new(), list_body.as_bytes())
        .unwrap_or_else(|_| unreachable!("valid list did not decode"));
    let server = decode_detail(
        RobotServerGetRequest::new(server_number(321)),
        DETAIL.as_bytes(),
    )
    .unwrap_or_else(|_| unreachable!("valid detail did not decode"));
    let name =
        RobotServerName::new("renamed-1").unwrap_or_else(|_| unreachable!("fixture name failed"));
    let diagnostics = alloc::format!(
        "{list:?} {server:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
        server.summary(),
        server.summary().number(),
        server.summary().status(),
        server.summary().paid_until(),
        server.summary().addresses().first(),
        server.summary().subnets().and_then(|values| values.first()),
        server.capabilities(),
        server.linked_storage_box(),
        RobotServerGetRequest::new(server_number(321)),
        RobotServerUpdateRequest::rename(server_number(321), name),
    );
    for secret in [
        "321",
        "42",
        "192.0.2.10",
        "2001:db8",
        "2028",
        "ready",
        "process",
        "renamed-1",
    ] {
        assert!(
            !diagnostics.contains(secret),
            "diagnostics exposed {secret}"
        );
    }
}

fn server_number(value: u64) -> RobotServerNumber {
    RobotServerNumber::new(value).unwrap_or_else(|| unreachable!("fixture number failed"))
}

fn prepare<O>(
    operation: O,
) -> (
    Method,
    alloc::string::String,
    alloc::vec::Vec<u8>,
    Option<alloc::string::String>,
)
where
    O: PrepareOperation<Error = super::RobotServerRequestError>,
{
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 256];
    let prepared = operation
        .prepare(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("request preparation failed"));
    let request = prepared.transport_request();
    let content_type = request
        .headers()
        .as_slice()
        .iter()
        .find(|header| header.name().as_str() == "content-type")
        .map(|header| alloc::string::String::from(header.value().as_str()));
    (
        request.method(),
        request.target().as_str().into(),
        request.body().to_vec(),
        content_type,
    )
}

fn decode_list(
    request: RobotServerListRequest,
    body: &[u8],
) -> Result<super::RobotServerList, RobotServerDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 64];
    let prepared = request
        .prepare(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("list preparation failed"));
    with_checked(prepared, body, |checked| request.decode_response(checked))
}

fn decode_detail(
    request: RobotServerGetRequest,
    body: &[u8],
) -> Result<super::RobotServer, RobotServerDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 64];
    let prepared = request
        .prepare(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("get preparation failed"));
    with_checked(prepared, body, |checked| request.decode_response(checked))
}

fn with_checked<R>(
    prepared: cloud_sdk::operation::PreparedRequest<'_>,
    body: &[u8],
    decode: impl FnOnce(cloud_sdk::operation::CheckedResponseGuard<'_>) -> R,
) -> R {
    let mut response_storage = vec![0_u8; body.len()];
    let mut header_storage = [0_u8; 256];
    let mut response = ResponseBuffer::new(&mut response_storage, body.len(), &mut header_storage);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("response attempt failed"));
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
        .unwrap_or_else(|_| unreachable!("policy rejected fixture"));
    decode(checked)
}
