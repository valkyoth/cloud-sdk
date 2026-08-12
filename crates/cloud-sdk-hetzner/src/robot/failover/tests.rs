use alloc::{format, string::String, vec};
use core::fmt::Write as _;

use cloud_sdk::Method;
use cloud_sdk::operation::{
    OperationImpact, PreparationStorage, PrepareOperation, RequestBodySensitivity,
    RequestSemantics, RetryEligibility,
};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::robot::RobotIpAddress;

pub(super) const ACTIVE: &[u8] = br#"{"failover":{"ip":"192.0.2.50","netmask":"255.255.255.255","server_ip":"192.0.2.10","server_ipv6_net":"2001:db8:1::","server_number":321,"active_server_ip":"192.0.2.11"}}"#;
pub(super) const DELETED: &[u8] = br#"{"failover":{"ip":"192.0.2.50","netmask":"255.255.255.255","server_ip":"192.0.2.10","server_ipv6_net":"2001:db8:1::","server_number":321,"active_server_ip":null}}"#;
const IPV6: &[u8] = br#"{"failover":{"ip":"2001:db8:2::","netmask":"ffff:ffff:ffff:ffff::","server_ip":"192.0.2.10","server_ipv6_net":"2001:db8:1::","server_number":321,"active_server_ip":"2001:db8:3::"}}"#;

#[test]
fn prepares_all_source_locked_operations_and_policies() {
    assert_prepared(
        RobotFailoverListRequest::new(),
        Method::Get,
        "/failover",
        b"",
        "robot_list_failovers",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Public,
        2_097_152,
    );
    assert_prepared(
        RobotFailoverGetRequest::new(ip("192.0.2.50")),
        Method::Get,
        "/failover/192.0.2.50",
        b"",
        "robot_get_failover",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Public,
        16_384,
    );
    assert_prepared(
        RobotFailoverRerouteRequest::new(ip("192.0.2.50"), ip("192.0.2.11"))
            .unwrap_or_else(|_| unreachable!("reroute fixture failed")),
        Method::Post,
        "/failover/192.0.2.50",
        b"active_server_ip=192.0.2.11",
        "robot_reroute_failover",
        OperationImpact::Mutation,
        RequestSemantics::NonIdempotent,
        RetryEligibility::Never,
        RequestBodySensitivity::Sensitive,
        16_384,
    );
    assert_prepared(
        RobotFailoverDeleteRouteRequest::new(ip("192.0.2.50")),
        Method::Delete,
        "/failover/192.0.2.50",
        b"",
        "robot_delete_failover_route",
        OperationImpact::Destructive,
        RequestSemantics::NonIdempotent,
        RetryEligibility::Never,
        RequestBodySensitivity::Public,
        16_384,
    );
}

#[test]
fn reroute_rejects_cross_family_destination_before_preparation() {
    assert!(matches!(
        RobotFailoverRerouteRequest::new(ip("192.0.2.50"), ip("2001:db8::")),
        Err(RobotFailoverRequestError::AddressFamilyMismatch)
    ));
}

#[test]
fn decodes_ipv4_ipv6_and_null_route_acknowledgements() {
    let v4 = decode_get("192.0.2.50", ACTIVE)
        .unwrap_or_else(|_| unreachable!("IPv4 failover fixture failed"));
    assert_eq!(v4.prefix(), 32);
    assert_eq!(v4.server_number().with_number(|value| value), 321);
    assert!(v4.with_active_server(|value| value.is_some()));
    assert!(!format!("{v4:?}").contains("192.0.2"));

    let v6 = decode_get("2001:db8:2::", IPV6)
        .unwrap_or_else(|_| unreachable!("IPv6 failover fixture failed"));
    assert_eq!(v6.prefix(), 64);
    assert!(v6.with_route(|value| value.is_ipv6()));
    assert!(v6.with_active_server(|value| value.is_some_and(|value| value.is_ipv6())));

    let deleted = decode_get("192.0.2.50", DELETED)
        .unwrap_or_else(|_| unreachable!("delete acknowledgement failed"));
    assert!(deleted.with_active_server(|value| value.is_none()));
}

#[test]
fn list_is_bounded_distinct_and_strict() {
    let active = text(ACTIVE);
    let list = format!("[{active}]");
    let decoded = decode_list(list.as_bytes())
        .unwrap_or_else(|_| unreachable!("failover list fixture failed"));
    assert_eq!(decoded.len(), 1);
    assert!(!decoded.is_empty());

    let duplicate = format!("[{active},{active}]");
    assert_eq!(
        decode_list(duplicate.as_bytes()).err(),
        Some(RobotFailoverDecodeError::InvalidList)
    );
    let extra = active.replace("\"active_server_ip\"", "\"future\":1,\"active_server_ip\"");
    assert_eq!(
        decode_get("192.0.2.50", extra.as_bytes()).err(),
        Some(RobotFailoverDecodeError::InvalidEnvelope)
    );
}

#[test]
fn list_boundary_accepts_4096_and_rejects_4097() {
    for count in [4_095, 4_096] {
        let body = list_fixture(count);
        let decoded = decode_list(body.as_bytes())
            .unwrap_or_else(|_| unreachable!("admitted failover list boundary failed"));
        assert_eq!(decoded.len(), count);
    }
    let body = list_fixture(4_097);
    assert!(decode_list(body.as_bytes()).is_err());
}

#[test]
fn route_validation_rejects_family_mask_and_host_bit_conflicts() {
    let cross_family = text(ACTIVE).replace("255.255.255.255", "ffff:ffff:ffff:ffff::");
    assert_eq!(
        decode_get("192.0.2.50", cross_family.as_bytes()).err(),
        Some(RobotFailoverDecodeError::InvalidRoute)
    );
    let noncontiguous = text(ACTIVE).replace("255.255.255.255", "255.0.255.0");
    assert_eq!(
        decode_get("192.0.2.50", noncontiguous.as_bytes()).err(),
        Some(RobotFailoverDecodeError::InvalidRoute)
    );
    let host_bits = text(ACTIVE).replace("255.255.255.255", "255.255.255.0");
    assert_eq!(
        decode_get("192.0.2.50", host_bits.as_bytes()).err(),
        Some(RobotFailoverDecodeError::InvalidRoute)
    );
    let wrong_active = text(ACTIVE).replace("\"192.0.2.11\"", "\"2001:db8:3::\"");
    assert_eq!(
        decode_get("192.0.2.50", wrong_active.as_bytes()).err(),
        Some(RobotFailoverDecodeError::InvalidAddress)
    );
}

#[test]
fn exact_identity_and_mutation_outcome_are_enforced() {
    assert_eq!(
        decode_get("192.0.2.51", ACTIVE).err(),
        Some(RobotFailoverDecodeError::ResponseIdentityMismatch)
    );
    let reroute = RobotFailoverRerouteRequest::new(ip("192.0.2.50"), ip("192.0.2.12"))
        .unwrap_or_else(|_| unreachable!("reroute fixture failed"));
    assert_eq!(
        decode_reroute(&reroute, ACTIVE).err(),
        Some(RobotFailoverDecodeError::MutationOutcomeMismatch)
    );
    let delete = RobotFailoverDeleteRouteRequest::new(ip("192.0.2.50"));
    assert_eq!(
        decode_delete(&delete, ACTIVE).err(),
        Some(RobotFailoverDecodeError::MutationOutcomeMismatch)
    );
    assert!(decode_delete(&delete, DELETED).is_ok());
}

#[test]
fn failed_preparation_clears_complete_caller_storage() {
    let request = RobotFailoverRerouteRequest::new(ip("192.0.2.50"), ip("192.0.2.11"))
        .unwrap_or_else(|_| unreachable!("reroute fixture failed"));
    let mut target = [0xa5_u8; 4];
    let mut body = [0x5a_u8; 4];
    assert!(
        request
            .prepare_bound(PreparationStorage::new(&mut target, &mut body))
            .is_err()
    );
    assert_eq!(target, [0_u8; 4]);
    assert_eq!(body, [0_u8; 4]);
}

#[allow(clippy::too_many_arguments)]
fn assert_prepared<O>(
    operation: O,
    method: Method,
    target: &str,
    body: &[u8],
    operation_id: &str,
    impact: OperationImpact,
    semantics: RequestSemantics,
    retry: RetryEligibility,
    sensitivity: RequestBodySensitivity,
    maximum: usize,
) where
    O: PrepareOperation<Error = RobotFailoverRequestError>,
{
    let mut target_storage = [0_u8; 128];
    let mut body_storage = [0_u8; 128];
    let prepared = operation
        .prepare(PreparationStorage::new(
            &mut target_storage,
            &mut body_storage,
        ))
        .unwrap_or_else(|_| unreachable!("failover preparation failed"));
    assert_eq!(prepared.transport_request().method(), method);
    assert_eq!(prepared.transport_request().target().as_str(), target);
    assert_eq!(prepared.transport_request().body(), body);
    assert_eq!(
        prepared.operation_id().map(|value| value.as_str()),
        Some(operation_id)
    );
    assert_eq!(prepared.metadata().impact(), impact);
    assert_eq!(prepared.metadata().semantics(), semantics);
    assert_eq!(prepared.metadata().retry_eligibility(), retry);
    assert_eq!(prepared.body_sensitivity(), sensitivity);
    assert_eq!(prepared.response_policy().max_body_bytes(), maximum);
    assert_eq!(
        prepared.raw_response_policy().body_limit(StatusCode::OK),
        maximum
    );
}

fn decode_list(body: &[u8]) -> Result<RobotFailoverList, RobotFailoverDecodeError> {
    let request = RobotFailoverListRequest::new();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("list preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_get(route: &str, body: &[u8]) -> Result<RobotFailover, RobotFailoverDecodeError> {
    let request = RobotFailoverGetRequest::new(ip(route));
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("get preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_reroute(
    request: &RobotFailoverRerouteRequest,
    body: &[u8],
) -> Result<RobotFailover, RobotFailoverDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("reroute preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_delete(
    request: &RobotFailoverDeleteRouteRequest,
    body: &[u8],
) -> Result<RobotFailover, RobotFailoverDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("delete preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn with_json<R, O>(
    prepared: PreparedRobotFailover<'_, '_, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotFailover<'_, '_, R>) -> O,
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

pub(super) fn ip(value: &str) -> RobotIpAddress {
    RobotIpAddress::new(value).unwrap_or_else(|_| unreachable!("IP fixture failed"))
}

fn list_fixture(count: usize) -> String {
    let mut body = String::from("[");
    for index in 0..count {
        if index != 0 {
            body.push(',');
        }
        let value =
            u32::try_from(index).unwrap_or_else(|_| unreachable!("fixture index exceeded u32"));
        let [_, second, third, fourth] = value.to_be_bytes();
        write!(
            body,
            "{{\"failover\":{{\"ip\":\"10.{second}.{third}.{fourth}\",\"netmask\":\"255.255.255.255\",\"server_ip\":\"192.0.2.10\",\"server_ipv6_net\":\"2001:db8:1::\",\"server_number\":321,\"active_server_ip\":\"192.0.2.11\"}}}}"
        )
        .unwrap_or_else(|_| unreachable!("String fixture write failed"));
    }
    body.push(']');
    body
}

fn text(value: &[u8]) -> &str {
    core::str::from_utf8(value).unwrap_or_else(|_| unreachable!("fixture lost UTF-8"))
}
