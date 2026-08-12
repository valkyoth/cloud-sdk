use alloc::{format, vec};
use core::net::{IpAddr, Ipv4Addr};

use cloud_sdk::Method;
use cloud_sdk::operation::{
    OperationImpact, PreparationStorage, PrepareOperation, RequestBodySensitivity,
    RequestSemantics, RetryEligibility,
};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::robot::{RobotIpAddress, RobotMacAddress, RobotSubnetAddress};

pub(super) const SUBNET: &str = r#"{"ip":"192.0.2.10","mask":24,"gateway":"192.0.2.1","server_ip":"192.0.2.1","server_number":321,"failover":false,"locked":false,"traffic_warnings":true,"traffic_hourly":50,"traffic_daily":500,"traffic_monthly":8}"#;
const NULL_SUBNET: &str = r#"{"ip":"198.51.100.127","mask":24,"gateway":"198.51.100.1","server_ip":null,"server_number":421,"failover":false,"locked":false,"traffic_warnings":false,"traffic_hourly":100,"traffic_daily":500,"traffic_monthly":2}"#;
pub(super) const DETAIL: &[u8] = br#"{"subnet":{"ip":"192.0.2.10","mask":24,"gateway":"192.0.2.1","server_ip":"192.0.2.1","server_number":321,"failover":false,"locked":false,"traffic_warnings":true,"traffic_hourly":50,"traffic_daily":500,"traffic_monthly":8}}"#;
pub(super) const MAC_SET: &[u8] = br#"{"mac":{"ip":"2001:db8::","mask":"64","mac":"00:21:85:62:3e:9d","possible_mac":{"192.0.2.1":"00:21:85:62:3e:9c","192.0.2.2":"00:21:85:62:3e:9d"}}}"#;
pub(super) const MAC_DELETED: &[u8] = br#"{"mac":{"ip":"2001:db8::","mask":"64","mac":"00:21:85:62:3e:9c","possible_mac":{"192.0.2.1":"00:21:85:62:3e:9c","192.0.2.2":"00:21:85:62:3e:9d"}}}"#;

#[test]
fn prepares_all_six_source_locked_operations() {
    assert_prepared(RobotSubnetListRequest::all(), Method::Get, "/subnet", b"");
    assert_prepared(
        RobotSubnetListRequest::for_server(server_ip("192.0.2.1"))
            .unwrap_or_else(|_| unreachable!("IPv4 server filter was rejected")),
        Method::Get,
        "/subnet?server_ip=192.0.2.1",
        b"",
    );
    assert!(matches!(
        RobotSubnetListRequest::for_server(server_ip("2001:db8::1")),
        Err(RobotSubnetRequestError::InvalidServerAddress)
    ));
    assert_prepared(
        RobotSubnetGetRequest::new(subnet("192.0.2.10")),
        Method::Get,
        "/subnet/192.0.2.10",
        b"",
    );
    let update = RobotSubnetTrafficUpdate::warnings(true)
        .with_hourly(50)
        .with_daily(500)
        .with_monthly(u64::MAX);
    assert_prepared(
        RobotSubnetUpdateRequest::new(subnet("192.0.2.10"), update),
        Method::Post,
        "/subnet/192.0.2.10",
        b"traffic_warnings=true&traffic_hourly=50&traffic_daily=500&traffic_monthly=18446744073709551615",
    );
    assert_prepared(
        RobotSubnetMacGetRequest::new(subnet("2001:db8::")),
        Method::Get,
        "/subnet/2001:db8::/mac",
        b"",
    );
    assert_prepared(
        RobotSubnetMacSetRequest::new(subnet("2001:db8::"), mac("00:21:85:62:3e:9d")),
        Method::Put,
        "/subnet/2001:db8::/mac",
        b"mac=00%3A21%3A85%3A62%3A3e%3A9d",
    );
    assert_prepared(
        RobotSubnetMacDeleteRequest::new(subnet("2001:db8::")),
        Method::Delete,
        "/subnet/2001:db8::/mac",
        b"",
    );
}

#[test]
fn compiled_operation_policies_are_exact() {
    assert_policy(
        RobotSubnetListRequest::all(),
        Method::Get,
        "robot_list_subnets",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Public,
    );
    assert_policy(
        RobotSubnetGetRequest::new(subnet("192.0.2.10")),
        Method::Get,
        "robot_get_subnet",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Public,
    );
    assert_policy(
        RobotSubnetUpdateRequest::new(
            subnet("192.0.2.10"),
            RobotSubnetTrafficUpdate::warnings(true),
        ),
        Method::Post,
        "robot_update_subnet",
        OperationImpact::Mutation,
        RequestSemantics::Idempotent,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Sensitive,
    );
    assert_policy(
        RobotSubnetMacGetRequest::new(subnet("2001:db8::")),
        Method::Get,
        "robot_get_subnet_mac",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Public,
    );
    assert_policy(
        RobotSubnetMacSetRequest::new(subnet("2001:db8::"), mac("00:21:85:62:3e:9d")),
        Method::Put,
        "robot_set_subnet_mac",
        OperationImpact::Mutation,
        RequestSemantics::NonIdempotent,
        RetryEligibility::Never,
        RequestBodySensitivity::Sensitive,
    );
    assert_policy(
        RobotSubnetMacDeleteRequest::new(subnet("2001:db8::")),
        Method::Delete,
        "robot_delete_subnet_mac",
        OperationImpact::Destructive,
        RequestSemantics::Idempotent,
        RetryEligibility::Never,
        RequestBodySensitivity::Public,
    );
}

#[test]
fn preparation_failures_clear_target_and_form_storage() {
    let request = RobotSubnetUpdateRequest::new(
        subnet("192.0.2.10"),
        RobotSubnetTrafficUpdate::warnings(true).with_monthly(8),
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

    let set = RobotSubnetMacSetRequest::new(subnet("2001:db8::"), mac("00:21:85:62:3e:9d"));
    let mut target = [0x5a_u8; 128];
    let mut body = [0x5a_u8; 4];
    assert!(
        set.prepare(PreparationStorage::new(&mut target, &mut body))
            .is_err()
    );
    assert_eq!(target, [0; 128]);
    assert_eq!(body, [0; 4]);
}

#[test]
fn decodes_host_bits_nullable_assignment_boundaries_and_mac_choices() {
    let list_body = format!("[{{\"subnet\":{SUBNET}}},{{\"subnet\":{NULL_SUBNET}}}]");
    let list = decode_list(RobotSubnetListRequest::all(), list_body.as_bytes())
        .unwrap_or_else(|_| unreachable!("subnet list fixture failed"));
    assert_eq!(list.len(), 2);
    let first = list
        .as_slice()
        .first()
        .unwrap_or_else(|| unreachable!("list lost entry"));
    assert_eq!(first.prefix(), 24);
    assert_eq!(first.server_number().with_number(|value| value), 321);
    assert!(first.traffic().enabled());
    assert!(first.with_network_address(|value| value == IpAddr::V4(Ipv4Addr::new(192, 0, 2, 0))));
    assert_eq!(
        first.with_broadcast(|value| value),
        Some(Ipv4Addr::new(192, 0, 2, 255))
    );
    let second = list
        .as_slice()
        .get(1)
        .unwrap_or_else(|| unreachable!("list lost null entry"));
    assert!(second.with_server_address(|value| value.is_none()));

    let response = decode_mac_get(RobotSubnetMacGetRequest::new(subnet("2001:db8::")), MAC_SET)
        .unwrap_or_else(|_| unreachable!("subnet MAC fixture failed"));
    assert_eq!(response.prefix(), 64);
    assert_eq!(response.possible().len(), 2);
    assert_eq!(response.mac(), &mac("00:21:85:62:3e:9d"));
}

#[test]
fn rejects_duplicates_identity_network_and_envelope_conflicts() {
    let duplicate = format!("[{{\"subnet\":{SUBNET}}},{{\"subnet\":{SUBNET}}}]");
    assert_eq!(
        decode_list(RobotSubnetListRequest::all(), duplicate.as_bytes()).err(),
        Some(RobotSubnetDecodeError::InvalidList)
    );
    assert_eq!(
        decode_get(RobotSubnetGetRequest::new(subnet("192.0.2.11")), DETAIL).err(),
        Some(RobotSubnetDecodeError::ResponseIdentityMismatch)
    );
    let wrong_gateway =
        text(DETAIL).replace("\"gateway\":\"192.0.2.1\"", "\"gateway\":\"198.51.100.1\"");
    assert_eq!(
        decode_get(
            RobotSubnetGetRequest::new(subnet("192.0.2.10")),
            wrong_gateway.as_bytes()
        )
        .err(),
        Some(RobotSubnetDecodeError::InvalidNetwork)
    );
    let invalid_prefix = text(DETAIL).replace("\"mask\":24", "\"mask\":33");
    assert_eq!(
        decode_get(
            RobotSubnetGetRequest::new(subnet("192.0.2.10")),
            invalid_prefix.as_bytes()
        )
        .err(),
        Some(RobotSubnetDecodeError::InvalidNetwork)
    );
    let extra = text(DETAIL).replace("\"mask\":24", "\"mask\":24,\"extra\":true");
    assert_eq!(
        decode_get(
            RobotSubnetGetRequest::new(subnet("192.0.2.10")),
            extra.as_bytes()
        )
        .err(),
        Some(RobotSubnetDecodeError::InvalidEnvelope)
    );
}

#[test]
fn request_association_rejects_filter_update_and_mac_conflicts() {
    let list_body = format!("[{{\"subnet\":{SUBNET}}}]");
    let filtered = RobotSubnetListRequest::for_server(server_ip("192.0.2.99"))
        .unwrap_or_else(|_| unreachable!("IPv4 server filter was rejected"));
    assert_eq!(
        decode_list(filtered, list_body.as_bytes()).err(),
        Some(RobotSubnetDecodeError::ResponseIdentityMismatch)
    );
    let update = RobotSubnetUpdateRequest::new(
        subnet("192.0.2.10"),
        RobotSubnetTrafficUpdate::warnings(false),
    );
    assert_eq!(
        decode_update(update, DETAIL).err(),
        Some(RobotSubnetDecodeError::MutationOutcomeMismatch)
    );
    let set = RobotSubnetMacSetRequest::new(subnet("2001:db8::"), mac("00:21:85:62:3e:9c"));
    assert_eq!(
        decode_mac_set(set, MAC_SET).err(),
        Some(RobotSubnetDecodeError::MutationOutcomeMismatch)
    );
}

#[test]
fn mac_shape_and_diagnostics_fail_closed() {
    let absent_current = text(MAC_SET).replace(
        "00:21:85:62:3e:9d\",\"possible_mac",
        "00:21:85:62:3e:9e\",\"possible_mac",
    );
    assert_eq!(
        decode_mac_get(
            RobotSubnetMacGetRequest::new(subnet("2001:db8::")),
            absent_current.as_bytes()
        )
        .err(),
        Some(RobotSubnetDecodeError::InvalidMac)
    );
    let empty = text(MAC_SET).replace(
        "{\"192.0.2.1\":\"00:21:85:62:3e:9c\",\"192.0.2.2\":\"00:21:85:62:3e:9d\"}",
        "{}",
    );
    assert_eq!(
        decode_mac_get(
            RobotSubnetMacGetRequest::new(subnet("2001:db8::")),
            empty.as_bytes()
        )
        .err(),
        Some(RobotSubnetDecodeError::InvalidList)
    );
    let leading_zero = text(MAC_SET).replace("\"mask\":\"64\"", "\"mask\":\"064\"");
    assert_eq!(
        decode_mac_get(
            RobotSubnetMacGetRequest::new(subnet("2001:db8::")),
            leading_zero.as_bytes()
        )
        .err(),
        Some(RobotSubnetDecodeError::InvalidNetwork)
    );
    let detail = decode_get(RobotSubnetGetRequest::new(subnet("192.0.2.10")), DETAIL)
        .unwrap_or_else(|_| unreachable!("detail fixture failed"));
    let diagnostics = format!(
        "{detail:?} {:?} {:?}",
        RobotSubnetMacGetRequest::new(subnet("2001:db8::")),
        mac("00:21:85:62:3e:9c")
    );
    for secret in ["192.0.2", "2001:db8", "00:21", "321"] {
        assert!(!diagnostics.contains(secret));
    }
}

fn subnet(value: &str) -> RobotSubnetAddress {
    RobotSubnetAddress::new(value).unwrap_or_else(|_| unreachable!("subnet fixture failed"))
}

fn server_ip(value: &str) -> RobotIpAddress {
    RobotIpAddress::new(value).unwrap_or_else(|_| unreachable!("server IP fixture failed"))
}

fn mac(value: &str) -> RobotMacAddress {
    RobotMacAddress::new(value).unwrap_or_else(|_| unreachable!("MAC fixture failed"))
}

fn assert_prepared<O>(operation: O, method: Method, target: &str, body: &[u8])
where
    O: PrepareOperation<Error = RobotSubnetRequestError>,
{
    let mut target_storage = [0_u8; 128];
    let mut body_storage = [0_u8; 256];
    let prepared = operation
        .prepare(PreparationStorage::new(
            &mut target_storage,
            &mut body_storage,
        ))
        .unwrap_or_else(|_| unreachable!("request preparation failed"));
    assert_eq!(prepared.transport_request().method(), method);
    assert_eq!(prepared.transport_request().target().as_str(), target);
    assert_eq!(prepared.transport_request().body(), body);
}

fn assert_policy<O>(
    operation: O,
    method: Method,
    operation_id: &'static str,
    impact: OperationImpact,
    semantics: RequestSemantics,
    retry: RetryEligibility,
    sensitivity: RequestBodySensitivity,
) where
    O: PrepareOperation<Error = RobotSubnetRequestError>,
{
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 256];
    let prepared = operation
        .prepare(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("request preparation failed"));
    assert_eq!(prepared.transport_request().method(), method);
    assert_eq!(
        prepared.operation_id().map(|value| value.as_str()),
        Some(operation_id)
    );
    assert_eq!(prepared.metadata().impact(), impact);
    assert_eq!(prepared.metadata().semantics(), semantics);
    assert_eq!(prepared.metadata().retry_eligibility(), retry);
    assert_eq!(prepared.body_sensitivity(), sensitivity);
}

macro_rules! decode_bound {
    ($name:ident, $request:ty, $result:ty, $body_len:expr) => {
        fn $name(request: $request, body: &[u8]) -> Result<$result, RobotSubnetDecodeError> {
            let mut target = [0_u8; 128];
            let mut request_body = [0_u8; $body_len];
            let prepared = request
                .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
                .unwrap_or_else(|_| unreachable!("bound preparation failed"));
            with_json(prepared, body, |checked| checked.decode_response())
        }
    };
}

decode_bound!(decode_list, RobotSubnetListRequest, RobotSubnetList, 1);
decode_bound!(decode_get, RobotSubnetGetRequest, RobotSubnet, 1);
decode_bound!(decode_update, RobotSubnetUpdateRequest, RobotSubnet, 256);
decode_bound!(decode_mac_get, RobotSubnetMacGetRequest, RobotSubnetMac, 1);
decode_bound!(
    decode_mac_set,
    RobotSubnetMacSetRequest,
    RobotSubnetMac,
    128
);

fn with_json<R, O>(
    prepared: PreparedRobotSubnet<'_, '_, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotSubnet<'_, '_, R>) -> O,
) -> O {
    let mut response_storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut response_storage, body.len(), &mut headers);
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
        .unwrap_or_else(|_| unreachable!("content type failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("response body failed"))
        .copy_from_slice(body);
    attempt
        .commit(StatusCode::OK, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("response commit failed"));
    drop(attempt);
    let checked = prepared
        .validate_response(response)
        .unwrap_or_else(|_| unreachable!("response policy failed"));
    decode(checked)
}

fn text(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).unwrap_or_else(|_| unreachable!("fixture lost UTF-8"))
}
