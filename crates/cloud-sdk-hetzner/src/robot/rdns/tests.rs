use alloc::{format, string::String, vec};
use core::fmt::Write as _;

use cloud_sdk::Method;
use cloud_sdk::operation::{
    CheckedResponse, ContentTypePolicy, OperationImpact, PreparationStorage, PrepareOperation,
    RequestBodySensitivity, RequestIdPolicy, RequestSemantics, ResponseBodyPolicy, ResponsePolicy,
    RetryEligibility,
};
use cloud_sdk::transport::{
    HeaderSensitivity, MediaType, ResponseBuffer, ResponseDecodeWorkspace, ResponseMetadata,
    StatusCode,
};

use super::prepare::{MAX_ROBOT_RDNS_ITEM_RESPONSE_BYTES, MAX_ROBOT_RDNS_LIST_RESPONSE_BYTES};
use super::*;
use crate::robot::RobotIpAddress;

pub(super) const ENTRY: &[u8] = br#"{"rdns":{"ip":"192.0.2.50","ptr":"mail.example.com"}}"#;
const IPV6_ENTRY: &[u8] = br#"{"rdns":{"ip":"2001:db8::50","ptr":"mail-v6.example.com"}}"#;
const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];

#[test]
fn prepares_all_source_locked_operations_and_policies() {
    assert_prepared(
        RobotRdnsListRequest::all(),
        Method::Get,
        "/rdns",
        b"",
        "robot_list_rdns",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Public,
        2_097_152,
    );
    assert_prepared(
        RobotRdnsListRequest::for_server(ip("192.0.2.10"))
            .unwrap_or_else(|_| unreachable!("server filter fixture failed")),
        Method::Get,
        "/rdns?server_ip=192.0.2.10",
        b"",
        "robot_list_rdns",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Public,
        2_097_152,
    );
    assert_prepared(
        RobotRdnsGetRequest::new(ip("2001:db8::50")),
        Method::Get,
        "/rdns/2001:db8::50",
        b"",
        "robot_get_rdns",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Public,
        16_384,
    );
    assert_prepared(
        RobotRdnsSetRequest::new(ip("192.0.2.50"), ptr("mail.example.com")),
        Method::Put,
        "/rdns/192.0.2.50",
        b"ptr=mail.example.com",
        "robot_set_rdns",
        OperationImpact::Mutation,
        RequestSemantics::NonIdempotent,
        RetryEligibility::Never,
        RequestBodySensitivity::Sensitive,
        16_384,
    );
    assert_prepared(
        RobotRdnsUpdateRequest::new(ip("192.0.2.50"), ptr("new.example.com")),
        Method::Post,
        "/rdns/192.0.2.50",
        b"ptr=new.example.com",
        "robot_update_rdns",
        OperationImpact::Mutation,
        RequestSemantics::Idempotent,
        RetryEligibility::Never,
        RequestBodySensitivity::Sensitive,
        16_384,
    );
    assert_prepared(
        RobotRdnsDeleteRequest::new(ip("192.0.2.50")),
        Method::Delete,
        "/rdns/192.0.2.50",
        b"",
        "robot_delete_rdns",
        OperationImpact::Destructive,
        RequestSemantics::Idempotent,
        RetryEligibility::Never,
        RequestBodySensitivity::Public,
        0,
    );
}

#[test]
fn list_filter_rejects_ipv6_main_server_address() {
    assert!(matches!(
        RobotRdnsListRequest::for_server(ip("2001:db8::10")),
        Err(RobotRdnsRequestError::InvalidServerAddress)
    ));
}

#[test]
fn decodes_ipv4_ipv6_and_redacts_values() {
    let v4 = decode_get("192.0.2.50", ENTRY)
        .unwrap_or_else(|_| unreachable!("IPv4 reverse-DNS fixture failed"));
    assert!(v4.with_address(|value| value.is_ipv4()));
    assert_eq!(
        v4.ptr()
            .try_with_text(|value| String::from(value))
            .unwrap_or_else(|_| unreachable!("PTR fixture lost UTF-8")),
        "mail.example.com"
    );
    assert!(!format!("{v4:?}").contains("mail.example.com"));

    let v6 = decode_get("2001:db8::50", IPV6_ENTRY)
        .unwrap_or_else(|_| unreachable!("IPv6 reverse-DNS fixture failed"));
    assert!(v6.with_address(|value| value.is_ipv6()));
}

#[test]
fn list_is_bounded_distinct_and_strict() {
    let entry = text(ENTRY);
    let decoded = decode_list(format!("[{entry}]").as_bytes())
        .unwrap_or_else(|_| unreachable!("reverse-DNS list fixture failed"));
    assert_eq!(decoded.len(), 1);
    assert!(!decoded.is_empty());

    assert_eq!(
        decode_list(format!("[{entry},{entry}]").as_bytes()).err(),
        Some(RobotRdnsDecodeError::InvalidList)
    );
    let extra = entry.replace("\"ptr\"", "\"future\":1,\"ptr\"");
    assert_eq!(
        decode_get("192.0.2.50", extra.as_bytes()).err(),
        Some(RobotRdnsDecodeError::InvalidEnvelope)
    );
}

#[test]
fn list_boundary_accepts_4096_and_rejects_4097() {
    for count in [4_095, 4_096] {
        let body = list_fixture(count);
        let decoded = decode_list(body.as_bytes())
            .unwrap_or_else(|_| unreachable!("admitted reverse-DNS list boundary failed"));
        assert_eq!(decoded.len(), count);
    }
    let body = list_fixture(4_097);
    assert!(decode_list(body.as_bytes()).is_err());
}

#[test]
fn strict_models_reject_noncanonical_values_and_wrong_identity() {
    let uppercase = text(ENTRY).replace("mail.example.com", "Mail.example.com");
    assert_eq!(
        decode_get("192.0.2.50", uppercase.as_bytes()).err(),
        Some(RobotRdnsDecodeError::InvalidPtr)
    );
    let noncanonical_ip = text(ENTRY).replace("192.0.2.50", "192.000.2.50");
    assert_eq!(
        decode_get("192.0.2.50", noncanonical_ip.as_bytes()).err(),
        Some(RobotRdnsDecodeError::InvalidAddress)
    );
    assert_eq!(
        decode_get("192.0.2.51", ENTRY).err(),
        Some(RobotRdnsDecodeError::ResponseIdentityMismatch)
    );
}

#[test]
fn mutation_acknowledgements_require_exact_ptr_and_status() {
    let set = RobotRdnsSetRequest::new(ip("192.0.2.50"), ptr("mail.example.com"));
    assert!(decode_set(&set, StatusCode::CREATED, ENTRY).is_ok());
    assert!(!set_policy_accepts(&set, StatusCode::OK, ENTRY));

    let update = RobotRdnsUpdateRequest::new(ip("192.0.2.50"), ptr("new.example.com"));
    assert_eq!(
        decode_update(&update, StatusCode::OK, ENTRY).err(),
        Some(RobotRdnsDecodeError::MutationOutcomeMismatch)
    );
    let updated = text(ENTRY).replace("mail.example.com", "new.example.com");
    assert!(decode_update(&update, StatusCode::OK, updated.as_bytes()).is_ok());
    assert!(decode_update(&update, StatusCode::CREATED, updated.as_bytes()).is_ok());
}

#[test]
fn delete_requires_an_empty_ok_response() {
    let request = RobotRdnsDeleteRequest::new(ip("192.0.2.50"));
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("delete preparation failed"));
    with_response(StatusCode::OK, b"", None, |response| {
        assert!(
            prepared
                .validate_response(response)
                .unwrap_or_else(|_| unreachable!("empty delete response failed"))
                .decode_response()
                .is_ok()
        );
    });

    let mut target = [0_u8; 128];
    let mut body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("delete preparation failed"));
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
fn free_decoders_enforce_independent_operation_limits() {
    let item = vec![b' '; MAX_ROBOT_RDNS_ITEM_RESPONSE_BYTES + 1];
    let expected = ip("192.0.2.50");
    assert_eq!(
        with_wide_json(&item, |checked, workspace| {
            decode_robot_rdns(checked, &expected, workspace)
        })
        .err(),
        Some(RobotRdnsDecodeError::ResponseTooLarge)
    );
    let list = vec![b' '; MAX_ROBOT_RDNS_LIST_RESPONSE_BYTES + 1];
    assert_eq!(
        with_wide_json(&list, decode_robot_rdns_list).err(),
        Some(RobotRdnsDecodeError::ResponseTooLarge)
    );
}

#[test]
fn failed_preparation_clears_complete_caller_storage() {
    let request = RobotRdnsSetRequest::new(ip("192.0.2.50"), ptr("mail.example.com"));
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
    O: PrepareOperation<Error = RobotRdnsRequestError>,
{
    let mut target_storage = [0_u8; 128];
    let mut body_storage = [0_u8; 512];
    let prepared = operation
        .prepare(PreparationStorage::new(
            &mut target_storage,
            &mut body_storage,
        ))
        .unwrap_or_else(|_| unreachable!("reverse-DNS preparation failed"));
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
}

fn decode_list(body: &[u8]) -> Result<RobotRdnsList, RobotRdnsDecodeError> {
    let request = RobotRdnsListRequest::all();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("list preparation failed"));
    with_json(prepared, StatusCode::OK, body, |checked| {
        checked.decode_response()
    })
}

fn decode_get(address: &str, body: &[u8]) -> Result<RobotRdns, RobotRdnsDecodeError> {
    let request = RobotRdnsGetRequest::new(ip(address));
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("get preparation failed"));
    with_json(prepared, StatusCode::OK, body, |checked| {
        checked.decode_response()
    })
}

fn set_policy_accepts(request: &RobotRdnsSetRequest, status: StatusCode, body: &[u8]) -> bool {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 512];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("set preparation failed"));
    with_response(status, body, Some("application/json"), |response| {
        prepared.validate_response(response).is_ok()
    })
}

fn decode_set(
    request: &RobotRdnsSetRequest,
    status: StatusCode,
    body: &[u8],
) -> Result<RobotRdns, RobotRdnsDecodeError> {
    let mut result = None;
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 512];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("set preparation failed"));
    with_response(status, body, Some("application/json"), |response| {
        result = Some(
            prepared
                .validate_response(response)
                .unwrap_or_else(|_| unreachable!("set response policy failed"))
                .decode_response(),
        );
    });
    result.unwrap_or_else(|| unreachable!("set response was not decoded"))
}

fn decode_update(
    request: &RobotRdnsUpdateRequest,
    status: StatusCode,
    body: &[u8],
) -> Result<RobotRdns, RobotRdnsDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 512];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("update preparation failed"));
    with_json(prepared, status, body, |checked| checked.decode_response())
}

fn with_json<'request, R, O>(
    prepared: PreparedRobotRdns<'_, 'request, R>,
    status: StatusCode,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotRdns<'_, 'request, R>) -> O,
) -> O {
    let mut result = None;
    with_response(status, body, Some("application/json"), |response| {
        let checked = prepared
            .validate_response(response)
            .unwrap_or_else(|_| unreachable!("response policy failed"));
        result = Some(decode(checked));
    });
    result.unwrap_or_else(|| unreachable!("response was not decoded"))
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

fn with_wide_json<R, E>(
    body: &[u8],
    decode: impl for<'response> FnOnce(
        CheckedResponse<'response>,
        &mut ResponseDecodeWorkspace,
    ) -> Result<R, E>,
) -> Result<R, E> {
    let policy = ResponsePolicy::new(
        OK,
        ContentTypePolicy::Required(JSON),
        ResponseBodyPolicy::Required,
        body.len(),
    )
    .unwrap_or_else(|_| unreachable!("wide response policy failed"));
    let mut result = None;
    with_response(StatusCode::OK, body, Some("application/json"), |response| {
        result = Some(
            policy
                .validate(response, RequestIdPolicy::Protected)
                .unwrap_or_else(|_| unreachable!("wide response validation failed"))
                .decode_owned_with_workspace(decode),
        );
    });
    result.unwrap_or_else(|| unreachable!("wide response was not decoded"))
}

pub(super) fn ip(value: &str) -> RobotIpAddress {
    RobotIpAddress::new(value).unwrap_or_else(|_| unreachable!("IP fixture failed"))
}

pub(super) fn ptr(value: &str) -> RobotRdnsName {
    RobotRdnsName::new(value).unwrap_or_else(|_| unreachable!("PTR fixture failed"))
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
            "{{\"rdns\":{{\"ip\":\"10.{second}.{third}.{fourth}\",\"ptr\":\"host-{index}.example.com\"}}}}"
        )
        .unwrap_or_else(|_| unreachable!("String fixture write failed"));
    }
    body.push(']');
    body
}

fn text(value: &[u8]) -> &str {
    core::str::from_utf8(value).unwrap_or_else(|_| unreachable!("fixture lost UTF-8"))
}
