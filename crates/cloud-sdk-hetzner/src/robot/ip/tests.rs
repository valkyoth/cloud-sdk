use alloc::{format, vec};

use cloud_sdk::Method;
use cloud_sdk::operation::{
    OperationImpact, PreparationStorage, PrepareOperation, RequestBodySensitivity,
    RequestSemantics, RetryEligibility,
};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::robot::RobotIpAddress;

pub(super) const SUMMARY: &str = r#"{"ip":"192.0.2.10","server_ip":"192.0.2.1","server_number":321,"locked":false,"separate_mac":null,"traffic_warnings":false,"traffic_hourly":50,"traffic_daily":500,"traffic_monthly":8}"#;
pub(super) const DETAIL: &[u8] = br#"{"ip":{"ip":"192.0.2.10","gateway":"192.0.2.1","mask":24,"broadcast":"192.0.2.255","server_ip":"192.0.2.1","server_number":321,"locked":false,"separate_mac":"00:21:85:62:3e:9c","traffic_warnings":true,"traffic_hourly":50,"traffic_daily":500,"traffic_monthly":8}}"#;
pub(super) const MAC_SET: &[u8] = br#"{"mac":{"ip":"192.0.2.10","mac":"00:21:85:62:3e:9c"}}"#;
pub(super) const MAC_DELETED: &[u8] = br#"{"mac":{"ip":"192.0.2.10","mac":null}}"#;

#[test]
fn prepares_all_six_source_locked_operations() {
    assert_prepared(RobotIpListRequest::all(), Method::Get, "/ip", b"");
    assert_prepared(
        RobotIpListRequest::for_server(ip("192.0.2.1"))
            .unwrap_or_else(|_| unreachable!("IPv4 server filter was rejected")),
        Method::Get,
        "/ip?server_ip=192.0.2.1",
        b"",
    );
    assert!(matches!(
        RobotIpListRequest::for_server(ip("2001:db8::1")),
        Err(RobotIpRequestError::InvalidServerAddress)
    ));
    assert_prepared(
        RobotIpGetRequest::new(ip("192.0.2.10")),
        Method::Get,
        "/ip/192.0.2.10",
        b"",
    );
    let update = RobotIpTrafficUpdate::warnings(true)
        .with_hourly(50)
        .with_daily(500)
        .with_monthly(u64::MAX);
    assert_prepared(
        RobotIpUpdateRequest::new(ip("192.0.2.10"), update),
        Method::Post,
        "/ip/192.0.2.10",
        b"traffic_warnings=true&traffic_hourly=50&traffic_daily=500&traffic_monthly=18446744073709551615",
    );
    assert_prepared(
        RobotIpMacGetRequest::new(ip("192.0.2.10")),
        Method::Get,
        "/ip/192.0.2.10/mac",
        b"",
    );
    assert_prepared(
        RobotIpMacSetRequest::new(ip("192.0.2.10")),
        Method::Put,
        "/ip/192.0.2.10/mac",
        b"",
    );
    assert_prepared(
        RobotIpMacDeleteRequest::new(ip("192.0.2.10")),
        Method::Delete,
        "/ip/192.0.2.10/mac",
        b"",
    );
}

#[test]
fn mutation_metadata_and_sensitive_form_are_exact() {
    let request = RobotIpUpdateRequest::new(
        ip("192.0.2.10"),
        RobotIpTrafficUpdate::daily(0).with_warnings(false),
    );
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 256];
    let prepared = request
        .prepare(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("update preparation failed"));
    assert_eq!(prepared.metadata().impact(), OperationImpact::Mutation);
    assert_eq!(
        prepared.metadata().semantics(),
        RequestSemantics::Idempotent
    );
    assert_eq!(
        prepared.metadata().retry_eligibility(),
        RetryEligibility::ExplicitPolicy
    );
    assert_eq!(
        prepared.body_sensitivity(),
        RequestBodySensitivity::Sensitive
    );
    assert_eq!(
        prepared.transport_request().body(),
        b"traffic_warnings=false&traffic_daily=0"
    );
}

#[test]
fn every_operation_has_exact_compiled_security_policy() {
    assert_policy(
        RobotIpListRequest::all(),
        Method::Get,
        "robot_list_ips",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Public,
    );
    assert_policy(
        RobotIpGetRequest::new(ip("192.0.2.10")),
        Method::Get,
        "robot_get_ip",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Public,
    );
    assert_policy(
        RobotIpUpdateRequest::new(ip("192.0.2.10"), RobotIpTrafficUpdate::warnings(true)),
        Method::Post,
        "robot_update_ip",
        OperationImpact::Mutation,
        RequestSemantics::Idempotent,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Sensitive,
    );
    assert_policy(
        RobotIpMacGetRequest::new(ip("192.0.2.10")),
        Method::Get,
        "robot_get_ip_mac",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Public,
    );
    assert_policy(
        RobotIpMacSetRequest::new(ip("192.0.2.10")),
        Method::Put,
        "robot_set_ip_mac",
        OperationImpact::Mutation,
        RequestSemantics::NonIdempotent,
        RetryEligibility::Never,
        RequestBodySensitivity::Public,
    );
    assert_policy(
        RobotIpMacDeleteRequest::new(ip("192.0.2.10")),
        Method::Delete,
        "robot_delete_ip_mac",
        OperationImpact::Destructive,
        RequestSemantics::Idempotent,
        RetryEligibility::Never,
        RequestBodySensitivity::Public,
    );
}

#[test]
fn preparation_failures_clear_target_and_form_storage() {
    let request = RobotIpUpdateRequest::new(
        ip("192.0.2.10"),
        RobotIpTrafficUpdate::warnings(true).with_monthly(8),
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

    let mut target = [0x5a_u8; 128];
    let mut body = [0x5a_u8; 4];
    assert!(
        request
            .prepare(PreparationStorage::new(&mut target, &mut body))
            .is_err()
    );
    assert_eq!(target, [0; 128]);
    assert_eq!(body, [0; 4]);
}

#[test]
fn decodes_list_detail_and_mac_shapes() {
    let empty = decode_list(RobotIpListRequest::all(), b"[]")
        .unwrap_or_else(|_| unreachable!("empty list fixture failed"));
    assert!(empty.is_empty());

    let list_body = format!("[{{\"ip\":{SUMMARY}}}]");
    let list = decode_list(RobotIpListRequest::all(), list_body.as_bytes())
        .unwrap_or_else(|_| unreachable!("list fixture failed"));
    assert_eq!(list.len(), 1);
    let entry = list
        .as_slice()
        .first()
        .unwrap_or_else(|| unreachable!("list lost entry"));
    assert_eq!(entry.server_number().with_number(|value| value), 321);
    assert!(!entry.is_locked());
    assert!(entry.separate_mac().is_none());
    assert_eq!(entry.traffic().daily_megabytes(), 500);

    let detail = decode_get(RobotIpGetRequest::new(ip("192.0.2.10")), DETAIL)
        .unwrap_or_else(|_| unreachable!("detail fixture failed"));
    assert_eq!(detail.prefix(), 24);
    assert!(detail.summary().traffic().enabled());
    assert!(detail.summary().separate_mac().is_some());
    assert!(detail.with_gateway(|value| value == "192.0.2.1".parse().unwrap_or(value)));

    let mac = decode_mac_get(RobotIpMacGetRequest::new(ip("192.0.2.10")), MAC_SET)
        .unwrap_or_else(|_| unreachable!("MAC fixture failed"));
    assert!(mac.mac().is_some());
    let deleted = decode_mac_delete(RobotIpMacDeleteRequest::new(ip("192.0.2.10")), MAC_DELETED)
        .unwrap_or_else(|_| unreachable!("MAC delete fixture failed"));
    assert!(deleted.mac().is_none());
}

#[test]
fn rejects_duplicates_identity_network_fields_and_mutation_conflicts() {
    let duplicate = format!("[{{\"ip\":{SUMMARY}}},{{\"ip\":{SUMMARY}}}]");
    assert_eq!(
        decode_list(RobotIpListRequest::all(), duplicate.as_bytes()).err(),
        Some(RobotIpDecodeError::InvalidList)
    );
    assert_eq!(
        decode_get(RobotIpGetRequest::new(ip("192.0.2.11")), DETAIL).err(),
        Some(RobotIpDecodeError::ResponseIdentityMismatch)
    );
    let bad_broadcast = text(DETAIL).replace("192.0.2.255", "192.0.2.254");
    assert_eq!(
        decode_get(
            RobotIpGetRequest::new(ip("192.0.2.10")),
            bad_broadcast.as_bytes()
        )
        .err(),
        Some(RobotIpDecodeError::InvalidNetwork)
    );
    let extra = text(DETAIL).replace("\"mask\":24", "\"mask\":24,\"extra\":true");
    assert_eq!(
        decode_get(RobotIpGetRequest::new(ip("192.0.2.10")), extra.as_bytes()).err(),
        Some(RobotIpDecodeError::InvalidEnvelope)
    );
    let request =
        RobotIpUpdateRequest::new(ip("192.0.2.10"), RobotIpTrafficUpdate::warnings(false));
    assert_eq!(
        decode_update(request, DETAIL).err(),
        Some(RobotIpDecodeError::MutationOutcomeMismatch)
    );
    assert_eq!(
        decode_mac_delete(RobotIpMacDeleteRequest::new(ip("192.0.2.10")), MAC_SET).err(),
        Some(RobotIpDecodeError::MutationOutcomeMismatch)
    );
}

#[test]
fn list_filter_and_diagnostics_fail_closed() {
    let list_body = format!("[{{\"ip\":{SUMMARY}}}]");
    assert_eq!(
        decode_list(
            RobotIpListRequest::for_server(ip("192.0.2.99"))
                .unwrap_or_else(|_| unreachable!("IPv4 server filter was rejected")),
            list_body.as_bytes()
        )
        .err(),
        Some(RobotIpDecodeError::ResponseIdentityMismatch)
    );
    let detail = decode_get(RobotIpGetRequest::new(ip("192.0.2.10")), DETAIL)
        .unwrap_or_else(|_| unreachable!("detail fixture failed"));
    let diagnostics = format!(
        "{detail:?} {:?} {:?} {:?}",
        detail.summary(),
        RobotIpMacGetRequest::new(ip("192.0.2.10")),
        RobotMacAddress::new("00:21:85:62:3e:9c")
            .unwrap_or_else(|_| unreachable!("MAC fixture failed"))
    );
    for secret in ["192.0.2", "00:21", "321"] {
        assert!(!diagnostics.contains(secret));
    }
}

fn ip(value: &str) -> RobotIpAddress {
    RobotIpAddress::new(value).unwrap_or_else(|_| unreachable!("IP fixture failed"))
}

fn assert_prepared<O>(operation: O, method: Method, target: &str, body: &[u8])
where
    O: PrepareOperation<Error = RobotIpRequestError>,
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
    O: PrepareOperation<Error = RobotIpRequestError>,
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

fn decode_list(
    request: RobotIpListRequest,
    body: &[u8],
) -> Result<RobotIpList, RobotIpDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("list preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_get(request: RobotIpGetRequest, body: &[u8]) -> Result<RobotIp, RobotIpDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("get preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_update(
    request: RobotIpUpdateRequest,
    body: &[u8],
) -> Result<RobotIp, RobotIpDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 256];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("update preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_mac_get(
    request: RobotIpMacGetRequest,
    body: &[u8],
) -> Result<RobotIpMac, RobotIpDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("MAC get preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_mac_delete(
    request: RobotIpMacDeleteRequest,
    body: &[u8],
) -> Result<RobotIpMac, RobotIpDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("MAC delete preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn with_json<R, O>(
    prepared: PreparedRobotIp<'_, '_, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotIp<'_, '_, R>) -> O,
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
