use alloc::{format, string::String, vec};

use cloud_sdk::Method;
use cloud_sdk::operation::{
    OperationImpact, PreparationStorage, PrepareOperation, RequestBodySensitivity,
    RequestSemantics, RetryEligibility,
};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::robot::{RobotCancellationDate, RobotCancellationSchedule};

pub(super) const DETAIL: &[u8] = br#"{"id":4321,"name":"my vSwitch","vlan":4000,"cancelled":false,"server":[{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8:1::","server_number":321,"status":"ready"},{"server_ip":"192.0.2.11","server_ipv6_net":"2001:db8:2::","server_number":421,"status":"in process"}],"subnet":[{"ip":"198.51.100.0","mask":29,"gateway":"198.51.100.1"}],"cloud_network":[{"id":123,"ip":"10.0.2.0","mask":24,"gateway":"10.0.2.1"}]}"#;
const CREATED: &[u8] = br#"{"id":4321,"name":"my vSwitch","vlan":4000,"cancelled":false,"server":[],"subnet":[],"cloud_network":[]}"#;
const SUMMARY: &str = r#"{"id":4321,"name":"my vSwitch","vlan":4000,"cancelled":false}"#;

#[test]
fn values_are_bounded_canonical_unique_and_redacted() {
    assert!(RobotVSwitchId::new(0).is_err());
    assert!(RobotVlanId::new(0).is_err());
    assert!(RobotVlanId::new(4094).is_ok());
    assert!(RobotVlanId::new(4095).is_err());

    let name = RobotVSwitchName::new("private fabric")
        .unwrap_or_else(|_| unreachable!("name fixture failed"));
    assert_eq!(format!("{name:?}"), "RobotVSwitchName([redacted])");
    for invalid in ["", " leading", "trailing ", "hidden\nline", "a\u{202e}b"] {
        assert!(RobotVSwitchName::new(invalid).is_err());
    }

    for valid in ["321", "18446744073709551615", "192.0.2.10", "2001:db8::1"] {
        assert!(RobotVSwitchServerIdentifier::new(valid).is_ok());
    }
    for invalid in ["0", "0321", "18446744073709551616", "192.000.2.1", "host"] {
        assert!(RobotVSwitchServerIdentifier::new(invalid).is_err());
    }
    let duplicate = [selector("321"), selector("321")];
    assert_eq!(
        RobotVSwitchServers::new(&duplicate).err(),
        Some(RobotVSwitchValueError::DuplicateServer)
    );
    assert_eq!(
        RobotVSwitchServers::new(&[]).err(),
        Some(RobotVSwitchValueError::InvalidServerCount)
    );
}

#[test]
fn prepares_all_seven_source_locked_operations() {
    assert_prepared(
        RobotVSwitchListRequest::new(),
        Method::Get,
        "/vswitch",
        b"",
        "robot_list_vswitches",
        OperationImpact::ReadOnly,
        RequestBodySensitivity::Public,
        MAX_ROBOT_VSWITCH_LIST_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotVSwitchCreateRequest::new(name("my vSwitch"), vlan(4000)),
        Method::Post,
        "/vswitch",
        b"name=my+vSwitch&vlan=4000",
        "robot_create_vswitch",
        OperationImpact::Mutation,
        RequestBodySensitivity::Sensitive,
        MAX_ROBOT_VSWITCH_ITEM_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotVSwitchGetRequest::new(id()),
        Method::Get,
        "/vswitch/4321",
        b"",
        "robot_get_vswitch",
        OperationImpact::ReadOnly,
        RequestBodySensitivity::Public,
        MAX_ROBOT_VSWITCH_ITEM_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotVSwitchUpdateRequest::new(
            id(),
            RobotVSwitchUpdateIntent::RenameAndChangeVlan {
                name: name("new fabric"),
                vlan: vlan(4001),
            },
        ),
        Method::Post,
        "/vswitch/4321",
        b"name=new+fabric&vlan=4001",
        "robot_update_vswitch",
        OperationImpact::Mutation,
        RequestBodySensitivity::Sensitive,
        0,
    );
    assert_prepared(
        RobotVSwitchCancelRequest::new(id(), RobotCancellationSchedule::Immediate),
        Method::Delete,
        "/vswitch/4321",
        b"cancellation_date=now",
        "robot_cancel_vswitch",
        OperationImpact::Destructive,
        RequestBodySensitivity::Sensitive,
        0,
    );
    let members = [selector("321"), selector("192.0.2.11")];
    let servers = RobotVSwitchServers::new(&members)
        .unwrap_or_else(|_| unreachable!("membership fixture failed"));
    assert_prepared(
        RobotVSwitchAddServersRequest::new(id(), servers),
        Method::Post,
        "/vswitch/4321/server",
        b"server%5B%5D=321&server%5B%5D=192.0.2.11",
        "robot_add_vswitch_servers",
        OperationImpact::Mutation,
        RequestBodySensitivity::Sensitive,
        0,
    );
    assert_prepared(
        RobotVSwitchRemoveServersRequest::new(id(), servers),
        Method::Delete,
        "/vswitch/4321/server",
        b"server%5B%5D=321&server%5B%5D=192.0.2.11",
        "robot_remove_vswitch_servers",
        OperationImpact::Destructive,
        RequestBodySensitivity::Sensitive,
        0,
    );
}

#[test]
fn update_variants_and_calendar_cancellation_are_exact() {
    assert_eq!(
        prepared_body(&RobotVSwitchUpdateRequest::new(
            id(),
            RobotVSwitchUpdateIntent::Rename(name("renamed")),
        )),
        "name=renamed"
    );
    assert_eq!(
        prepared_body(&RobotVSwitchUpdateRequest::new(
            id(),
            RobotVSwitchUpdateIntent::ChangeVlan(vlan(4002)),
        )),
        "vlan=4002"
    );
    let date = RobotCancellationDate::new("2028-03-01")
        .unwrap_or_else(|_| unreachable!("date fixture failed"));
    assert_eq!(
        prepared_body(&RobotVSwitchCancelRequest::new(
            id(),
            RobotCancellationSchedule::On(date),
        )),
        "cancellation_date=2028-03-01"
    );
}

#[test]
fn detail_decode_is_strict_bounded_and_redacted() {
    let result = decode_get(id(), DETAIL).unwrap_or_else(|_| unreachable!("detail fixture failed"));
    assert_eq!(result.id(), id());
    assert_eq!(result.vlan(), vlan(4000));
    assert!(!result.cancelled());
    assert_eq!(result.servers().len(), 2);
    let first_server = result
        .servers()
        .first()
        .unwrap_or_else(|| unreachable!("server fixture became empty"));
    assert_eq!(first_server.status(), RobotVSwitchServerStatus::Ready);
    assert_eq!(first_server.number().with_number(|value| value), 321);
    let first_subnet = result
        .subnets()
        .first()
        .unwrap_or_else(|| unreachable!("subnet fixture became empty"));
    assert_eq!(first_subnet.prefix(), 29);
    let first_network = result
        .cloud_networks()
        .first()
        .unwrap_or_else(|| unreachable!("Cloud Network fixture became empty"));
    assert_eq!(first_network.id(), 123);
    assert!(!format!("{result:?}").contains("192.0.2"));

    let extra = text(DETAIL).replace("\"cancelled\"", "\"future\":1,\"cancelled\"");
    assert_eq!(
        decode_get(id(), extra.as_bytes()).err(),
        Some(RobotVSwitchDecodeError::InvalidEnvelope)
    );
    assert_eq!(
        decode_get(
            RobotVSwitchId::new(999).unwrap_or_else(|_| unreachable!("ID fixture failed")),
            DETAIL,
        )
        .err(),
        Some(RobotVSwitchDecodeError::ResponseIdentityMismatch)
    );
}

#[test]
fn route_and_membership_invariants_fail_closed() {
    let host_bits = text(DETAIL).replace("198.51.100.0", "198.51.100.1");
    assert_eq!(
        decode_get(id(), host_bits.as_bytes()).err(),
        Some(RobotVSwitchDecodeError::InvalidNetwork)
    );
    let outside = text(DETAIL).replace("198.51.100.1", "198.51.101.1");
    assert_eq!(
        decode_get(id(), outside.as_bytes()).err(),
        Some(RobotVSwitchDecodeError::InvalidNetwork)
    );
    let wrong_family = text(DETAIL).replace("2001:db8:1::", "192.0.2.12");
    assert_eq!(
        decode_get(id(), wrong_family.as_bytes()).err(),
        Some(RobotVSwitchDecodeError::InvalidServer)
    );
    let duplicate = text(DETAIL).replace("\"server_number\":421", "\"server_number\":321");
    assert_eq!(
        decode_get(id(), duplicate.as_bytes()).err(),
        Some(RobotVSwitchDecodeError::InvalidList)
    );
}

#[test]
fn list_and_create_are_request_bound() {
    let list = format!("[{SUMMARY}]");
    let decoded =
        decode_list(list.as_bytes()).unwrap_or_else(|_| unreachable!("list fixture failed"));
    assert_eq!(decoded.len(), 1);
    assert!(!decoded.is_empty());
    let duplicate = format!("[{SUMMARY},{SUMMARY}]");
    assert_eq!(
        decode_list(duplicate.as_bytes()).err(),
        Some(RobotVSwitchDecodeError::InvalidList)
    );
    let second_id = SUMMARY.replace("\"id\":4321", "\"id\":4322");
    let duplicate_vlan = format!("[{SUMMARY},{second_id}]");
    assert_eq!(
        decode_list(duplicate_vlan.as_bytes()).err(),
        Some(RobotVSwitchDecodeError::InvalidList)
    );

    let request = RobotVSwitchCreateRequest::new(name("my vSwitch"), vlan(4000));
    assert!(decode_create(&request, CREATED).is_ok());
    let mismatch = text(CREATED).replace("\"vlan\":4000", "\"vlan\":4001");
    assert_eq!(
        decode_create(&request, mismatch.as_bytes()).err(),
        Some(RobotVSwitchDecodeError::MutationOutcomeMismatch)
    );
}

#[test]
fn empty_acknowledgements_forbid_body_and_content_type() {
    let request =
        RobotVSwitchUpdateRequest::new(id(), RobotVSwitchUpdateIntent::ChangeVlan(vlan(4001)));
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("update preparation failed"));
    with_response(StatusCode::OK, b"", None, |response| {
        let checked = prepared
            .validate_response(response)
            .unwrap_or_else(|_| unreachable!("empty response failed"));
        assert!(checked.decode_response().is_ok());
    });

    let mut target = [0_u8; 128];
    let mut body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("update preparation failed"));
    with_response(
        StatusCode::OK,
        b"{}",
        Some("application/json"),
        |response| {
            assert!(prepared.validate_response(response).is_err());
        },
    );
}

#[test]
fn failed_preparation_clears_all_caller_storage() {
    let request = RobotVSwitchCreateRequest::new(name("my vSwitch"), vlan(4000));
    let mut target = [0xa5_u8; 2];
    let mut body = [0x5a_u8; 2];
    assert!(
        request
            .prepare_bound(PreparationStorage::new(&mut target, &mut body))
            .is_err()
    );
    assert_eq!(target, [0_u8; 2]);
    assert_eq!(body, [0_u8; 2]);
}

#[allow(clippy::too_many_arguments)]
fn assert_prepared<O>(
    operation: O,
    method: Method,
    target: &str,
    body: &[u8],
    operation_id: &str,
    impact: OperationImpact,
    sensitivity: RequestBodySensitivity,
    maximum: usize,
) where
    O: PrepareOperation<Error = RobotVSwitchRequestError>,
{
    let mut target_storage = [0_u8; 128];
    let mut body_storage = [0_u8; 4_096];
    let prepared = operation
        .prepare(PreparationStorage::new(
            &mut target_storage,
            &mut body_storage,
        ))
        .unwrap_or_else(|_| unreachable!("vSwitch preparation failed"));
    assert_eq!(prepared.transport_request().method(), method);
    assert_eq!(prepared.transport_request().target().as_str(), target);
    assert_eq!(prepared.transport_request().body(), body);
    assert_eq!(
        prepared.operation_id().map(|value| value.as_str()),
        Some(operation_id)
    );
    assert_eq!(prepared.metadata().impact(), impact);
    assert_eq!(
        prepared.metadata().semantics(),
        if impact == OperationImpact::ReadOnly {
            RequestSemantics::Safe
        } else {
            RequestSemantics::NonIdempotent
        }
    );
    assert_eq!(
        prepared.metadata().retry_eligibility(),
        if impact == OperationImpact::ReadOnly {
            RetryEligibility::ExplicitPolicy
        } else {
            RetryEligibility::Never
        }
    );
    assert_eq!(prepared.body_sensitivity(), sensitivity);
    assert_eq!(prepared.response_policy().max_body_bytes(), maximum);
}

fn prepared_body<O>(operation: &O) -> String
where
    O: PrepareOperation<Error = RobotVSwitchRequestError>,
{
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 4_096];
    let prepared = operation
        .prepare(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("form preparation failed"));
    String::from(
        core::str::from_utf8(prepared.transport_request().body())
            .unwrap_or_else(|_| unreachable!("form lost UTF-8")),
    )
}

fn decode_list(body: &[u8]) -> Result<RobotVSwitchList, RobotVSwitchDecodeError> {
    let request = RobotVSwitchListRequest::new();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("list preparation failed"));
    with_json(prepared, StatusCode::OK, body, |checked| {
        checked.decode_response()
    })
}

fn decode_get(id: RobotVSwitchId, body: &[u8]) -> Result<RobotVSwitch, RobotVSwitchDecodeError> {
    let request = RobotVSwitchGetRequest::new(id);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("get preparation failed"));
    with_json(prepared, StatusCode::OK, body, |checked| {
        checked.decode_response()
    })
}

fn decode_create(
    request: &RobotVSwitchCreateRequest,
    body: &[u8],
) -> Result<RobotVSwitch, RobotVSwitchDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("create preparation failed"));
    with_json(prepared, StatusCode::CREATED, body, |checked| {
        checked.decode_response()
    })
}

fn with_json<R, O>(
    prepared: PreparedRobotVSwitch<'_, '_, R>,
    status: StatusCode,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotVSwitch<'_, '_, R>) -> O,
) -> O {
    with_response(status, body, Some("application/json"), |response| {
        let checked = prepared
            .validate_response(response)
            .unwrap_or_else(|_| unreachable!("response policy failed"));
        decode(checked)
    })
}

fn with_response<R>(
    status: StatusCode,
    body: &[u8],
    content_type: Option<&str>,
    inspect: impl FnOnce(ResponseBuffer<'_>) -> R,
) -> R {
    let mut storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut storage, body.len(), &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("response attempt failed"));
    if let Some(content_type) = content_type {
        attempt
            .headers_mut()
            .unwrap_or_else(|_| unreachable!("response headers failed"))
            .try_push(
                "content-type",
                content_type.as_bytes(),
                HeaderSensitivity::Public,
            )
            .unwrap_or_else(|_| unreachable!("content type failed"));
    }
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("response body failed"))
        .copy_from_slice(body);
    attempt
        .commit(status, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("response commit failed"));
    drop(attempt);
    inspect(response)
}

pub(super) fn id() -> RobotVSwitchId {
    RobotVSwitchId::new(4321).unwrap_or_else(|_| unreachable!("ID fixture failed"))
}

pub(super) fn name(value: &str) -> RobotVSwitchName {
    RobotVSwitchName::new(value).unwrap_or_else(|_| unreachable!("name fixture failed"))
}

pub(super) fn vlan(value: u16) -> RobotVlanId {
    RobotVlanId::new(value).unwrap_or_else(|_| unreachable!("VLAN fixture failed"))
}

pub(super) fn selector(value: &str) -> RobotVSwitchServerIdentifier<'_> {
    RobotVSwitchServerIdentifier::new(value)
        .unwrap_or_else(|_| unreachable!("selector fixture failed"))
}

fn text(value: &[u8]) -> &str {
    core::str::from_utf8(value).unwrap_or_else(|_| unreachable!("fixture lost UTF-8"))
}
