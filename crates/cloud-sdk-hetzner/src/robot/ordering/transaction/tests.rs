use alloc::{format, vec};

use cloud_sdk::Method;
use cloud_sdk::operation::{
    OperationImpact, PreparationStorage, PrepareOperation, RequestBodySensitivity,
    RequestSemantics, RetryEligibility,
};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::robot::ordering::{RobotOrderRequestError, RobotOrderTransactionId};

const STANDARD: &[u8] =
    include_bytes!("../../../../../../tests/fixtures/robot-transactions/standard.json");
const MARKET: &[u8] =
    include_bytes!("../../../../../../tests/fixtures/robot-transactions/market.json");
const ADDON: &[u8] =
    include_bytes!("../../../../../../tests/fixtures/robot-transactions/addon.json");

const STANDARD_DETAIL: &[u8] = br#"{"transaction":{"id":"B-ready","date":"2026-08-15T12:30:43+02:00","status":"ready","server_number":107239,"server_ip":"188.40.1.1","authorized_key":[],"host_key":[],"comment":null,"product":{"id":"EX40","name":"EX40","description":[],"traffic":"30 TB","dist":"Rescue system","@deprecated arch":"64","lang":"en","location":"FSN1"},"addons":[]}}"#;
const MARKET_DETAIL: &[u8] = br#"{"transaction":{"id":"B-market","date":"2026-08-15T12:30:43Z","status":"in process","server_number":null,"server_ip":null,"authorized_key":[],"host_key":[],"comment":null,"product":{"id":283693,"name":"SB110","description":[],"traffic":"20 TB","dist":"Rescue system","@deprecated arch":"64","lang":"en","cpu":"CPU","cpu_benchmark":8944,"memory_size":24,"hdd_size":1536,"hdd_text":"HDD","hdd_count":2,"datacenter":"FSN1-DC5","network_speed":"1 Gbit/s","fixed_price":true,"next_reduce":0,"next_reduce_date":"2026-08-15 12:30:43"}}}"#;
const ADDON_DETAIL: &[u8] = br#"{"transaction":{"id":"B-addon","date":"2026-08-15T12:30:43Z","status":"ready","server_number":123,"product":{"id":"failover_subnet_ipv4_29","name":"Failover subnet /29","price":{"location":"NBG1","price":{"net":"15.1261","gross":"17.9999","hourly_net":"0.0242","hourly_gross":"0.0288"},"price_setup":{"net":"152.0000","gross":"180.8800"}}},"resources":[{"type":"subnet","id":"10.0.0.0"}]}}"#;

#[test]
fn prepares_all_six_read_only_transaction_operations() {
    let quota = ROBOT_ORDER_TRANSACTION_QUOTA;
    assert_eq!(quota.max_requests(), 500);
    assert_eq!(quota.interval().get(), 3_600);

    let standard_list = RobotStandardTransactionListRequest::new();
    assert_eq!(standard_list.quota(), quota);
    assert_prepared(
        standard_list,
        "/order/server/transaction",
        "robot_list_server_transactions",
        MAX_ROBOT_ORDER_TRANSACTION_LIST_RESPONSE_BYTES,
    );
    let standard_get = RobotStandardTransactionGetRequest::new(transaction_id("B-ready"));
    assert_eq!(standard_get.quota(), quota);
    assert_prepared(
        standard_get,
        "/order/server/transaction/B-ready",
        "robot_get_server_transaction",
        MAX_ROBOT_ORDER_TRANSACTION_ITEM_RESPONSE_BYTES,
    );
    let market_list = RobotMarketTransactionListRequest::new();
    assert_eq!(market_list.quota(), quota);
    assert_prepared(
        market_list,
        "/order/server_market/transaction",
        "robot_list_server_market_transactions",
        MAX_ROBOT_ORDER_TRANSACTION_LIST_RESPONSE_BYTES,
    );
    let market_get = RobotMarketTransactionGetRequest::new(transaction_id("B-market"));
    assert_eq!(market_get.quota(), quota);
    assert_prepared(
        market_get,
        "/order/server_market/transaction/B-market",
        "robot_get_server_market_transaction",
        MAX_ROBOT_ORDER_TRANSACTION_ITEM_RESPONSE_BYTES,
    );
    let addon_list = RobotAddonTransactionListRequest::new();
    assert_eq!(addon_list.quota(), quota);
    assert_prepared(
        addon_list,
        "/order/server_addon/transaction",
        "robot_list_server_addon_transactions",
        MAX_ROBOT_ORDER_TRANSACTION_LIST_RESPONSE_BYTES,
    );
    let addon_get = RobotAddonTransactionGetRequest::new(transaction_id("B-addon"));
    assert_eq!(addon_get.quota(), quota);
    assert_prepared(
        addon_get,
        "/order/server_addon/transaction/B-addon",
        "robot_get_server_addon_transaction",
        MAX_ROBOT_ORDER_TRANSACTION_ITEM_RESPONSE_BYTES,
    );
}

#[test]
fn transaction_identifiers_are_protected_and_target_failures_clear_storage() {
    let id = transaction_id("B-secret-value");
    assert_eq!(format!("{id:?}"), "RobotOrderTransactionId([redacted])");
    let request = RobotStandardTransactionGetRequest::new(id);
    let mut target = [0x5a_u8; 8];
    let mut body = [0x5a_u8; 8];
    assert_eq!(
        request
            .prepare(PreparationStorage::new(&mut target, &mut body))
            .err(),
        Some(RobotOrderRequestError::Target)
    );
    assert!(target.iter().all(|byte| *byte == 0));
    assert!(body.iter().all(|byte| *byte == 0));
}

#[test]
fn decodes_official_standard_market_and_addon_snapshots() {
    let standard = decode_standard_list(STANDARD)
        .unwrap_or_else(|_| unreachable!("standard transaction fixture failed"));
    let [standard_pending, standard_ready] = standard.transactions() else {
        unreachable!("standard transaction fixture count changed");
    };
    assert_eq!(
        standard_pending.status(),
        RobotOrderTransactionStatus::InProcess
    );
    assert!(standard_pending.server_number().is_none());
    assert_eq!(standard_ready.status(), RobotOrderTransactionStatus::Ready);
    assert_eq!(standard_ready.authorized_keys().len(), 1);
    assert_eq!(standard_ready.host_keys().len(), 1);
    assert_eq!(standard_ready.product().architecture(), 64);
    assert_eq!(
        format!("{standard_ready:?}"),
        "RobotStandardTransaction([redacted])"
    );

    let market = decode_market_list(MARKET)
        .unwrap_or_else(|_| unreachable!("market transaction fixture failed"));
    let [market_pending, market_ready] = market.transactions() else {
        unreachable!("market transaction fixture count changed");
    };
    assert_eq!(market_pending.product().cpu_benchmark(), 8_944);
    assert!(market_pending.product().fixed_price());
    assert_eq!(market_ready.product().hdd_count(), 7);

    let addon = decode_addon_list(ADDON)
        .unwrap_or_else(|_| unreachable!("addon transaction fixture failed"));
    let [addon_pending, addon_ready] = addon.transactions() else {
        unreachable!("addon transaction fixture count changed");
    };
    assert!(addon_pending.resources().is_empty());
    assert_eq!(addon_ready.resources().len(), 1);
    assert!(addon_ready.product().price().hourly().is_some());
}

#[test]
fn detail_decoding_is_bound_to_each_requested_transaction_identity() {
    assert!(decode_standard_get("B-ready", STANDARD_DETAIL).is_ok());
    assert_eq!(
        decode_standard_get("B-other", STANDARD_DETAIL).err(),
        Some(RobotOrderTransactionDecodeError::ResponseIdentityMismatch)
    );
    assert!(decode_market_get("B-market", MARKET_DETAIL).is_ok());
    assert_eq!(
        decode_market_get("B-other", MARKET_DETAIL).err(),
        Some(RobotOrderTransactionDecodeError::ResponseIdentityMismatch)
    );
    assert!(decode_addon_get("B-addon", ADDON_DETAIL).is_ok());
    assert_eq!(
        decode_addon_get("B-other", ADDON_DETAIL).err(),
        Some(RobotOrderTransactionDecodeError::ResponseIdentityMismatch)
    );
}

#[test]
fn strict_decoding_rejects_state_timestamp_and_duplicate_drift() {
    let ready_without_server = text(STANDARD_DETAIL)
        .replace("\"server_number\":107239", "\"server_number\":null")
        .replace("\"server_ip\":\"188.40.1.1\"", "\"server_ip\":null");
    assert_eq!(
        decode_standard_get("B-ready", ready_without_server.as_bytes()).err(),
        Some(RobotOrderTransactionDecodeError::InvalidStatus)
    );
    let processing_with_server =
        text(STANDARD_DETAIL).replace("\"status\":\"ready\"", "\"status\":\"in process\"");
    assert_eq!(
        decode_standard_get("B-ready", processing_with_server.as_bytes()).err(),
        Some(RobotOrderTransactionDecodeError::InvalidStatus)
    );
    let lowercase_separator = text(STANDARD_DETAIL).replace("T12:30:43", "t12:30:43");
    assert_eq!(
        decode_standard_get("B-ready", lowercase_separator.as_bytes()).err(),
        Some(RobotOrderTransactionDecodeError::InvalidTimestamp)
    );
    let malformed_legacy =
        text(MARKET_DETAIL).replace("2026-08-15 12:30:43", "2026-08-15T12:30:43");
    assert_eq!(
        decode_market_get("B-market", malformed_legacy.as_bytes()).err(),
        Some(RobotOrderTransactionDecodeError::InvalidTimestamp)
    );
    let duplicate = format!("[{},{}]", text(STANDARD_DETAIL), text(STANDARD_DETAIL));
    assert_eq!(
        decode_standard_list(duplicate.as_bytes()).err(),
        Some(RobotOrderTransactionDecodeError::InvalidList)
    );

    let zero_metrics = text(MARKET_DETAIL)
        .replace("\"cpu_benchmark\":8944", "\"cpu_benchmark\":0")
        .replace("\"memory_size\":24", "\"memory_size\":0")
        .replace("\"hdd_size\":1536", "\"hdd_size\":0")
        .replace("\"hdd_count\":2", "\"hdd_count\":0");
    let transaction = decode_market_get("B-market", zero_metrics.as_bytes())
        .unwrap_or_else(|_| unreachable!("documented unsigned metrics rejected zero"));
    assert_eq!(transaction.product().cpu_benchmark(), 0);
    assert_eq!(transaction.product().memory_size(), 0);
    assert_eq!(transaction.product().hdd_size(), 0);
    assert_eq!(transaction.product().hdd_count(), 0);
}

#[test]
fn strict_decoding_rejects_duplicate_keys_resources_and_price_pairs() {
    let key = r#"{"key":{"name":"key1","fingerprint":"same","type":"ED25519","size":256}}"#;
    let duplicate_keys = text(STANDARD_DETAIL).replace(
        "\"authorized_key\":[]",
        &format!("\"authorized_key\":[{key},{key}]"),
    );
    assert_eq!(
        decode_standard_get("B-ready", duplicate_keys.as_bytes()).err(),
        Some(RobotOrderTransactionDecodeError::InvalidKey)
    );
    let resource = r#"{"type":"subnet","id":"10.0.0.0"}"#;
    let duplicate_resources = text(ADDON_DETAIL).replace(
        &format!("\"resources\":[{resource}]"),
        &format!("\"resources\":[{resource},{resource}]"),
    );
    assert_eq!(
        decode_addon_get("B-addon", duplicate_resources.as_bytes()).err(),
        Some(RobotOrderTransactionDecodeError::InvalidResource)
    );
    let incomplete_hourly =
        text(ADDON_DETAIL).replace("\"hourly_gross\":\"0.0288\"", "\"hourly_gross\":null");
    assert_eq!(
        decode_addon_get("B-addon", incomplete_hourly.as_bytes()).err(),
        Some(RobotOrderTransactionDecodeError::InvalidPrice)
    );
}

fn assert_prepared<O>(operation: O, target: &str, id: &str, maximum: usize)
where
    O: PrepareOperation<Error = RobotOrderRequestError>,
{
    let mut target_storage = [0_u8; 4_096];
    let mut body_storage = [0_u8; 1];
    let prepared = operation
        .prepare(PreparationStorage::new(
            &mut target_storage,
            &mut body_storage,
        ))
        .unwrap_or_else(|_| unreachable!("transaction preparation failed"));
    assert_eq!(prepared.transport_request().method(), Method::Get);
    assert_eq!(prepared.transport_request().target().as_str(), target);
    assert!(prepared.transport_request().body().is_empty());
    assert_eq!(
        prepared.operation_id().map(|value| value.as_str()),
        Some(id)
    );
    assert_eq!(prepared.metadata().impact(), OperationImpact::ReadOnly);
    assert_eq!(prepared.metadata().semantics(), RequestSemantics::Safe);
    assert_eq!(
        prepared.metadata().retry_eligibility(),
        RetryEligibility::ExplicitPolicy
    );
    assert_eq!(prepared.body_sensitivity(), RequestBodySensitivity::Public);
    assert_eq!(prepared.response_policy().max_body_bytes(), maximum);
}

fn decode_standard_list(
    body: &[u8],
) -> Result<RobotStandardTransactionList, RobotOrderTransactionDecodeError> {
    let request = RobotStandardTransactionListRequest::new();
    let mut target = [0_u8; 256];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("standard list preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_market_list(
    body: &[u8],
) -> Result<RobotMarketTransactionList, RobotOrderTransactionDecodeError> {
    let request = RobotMarketTransactionListRequest::new();
    let mut target = [0_u8; 256];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("market list preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_addon_list(
    body: &[u8],
) -> Result<RobotAddonTransactionList, RobotOrderTransactionDecodeError> {
    let request = RobotAddonTransactionListRequest::new();
    let mut target = [0_u8; 256];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("addon list preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

macro_rules! detail_decoder {
    ($name:ident, $request:ty, $output:ty) => {
        fn $name(id: &str, body: &[u8]) -> Result<$output, RobotOrderTransactionDecodeError> {
            let request = <$request>::new(transaction_id(id));
            let mut target = [0_u8; 256];
            let mut request_body = [0_u8; 1];
            let prepared = request
                .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
                .unwrap_or_else(|_| unreachable!("transaction detail preparation failed"));
            with_json(prepared, body, |checked| checked.decode_response())
        }
    };
}

detail_decoder!(
    decode_standard_get,
    RobotStandardTransactionGetRequest,
    RobotStandardTransaction
);
detail_decoder!(
    decode_market_get,
    RobotMarketTransactionGetRequest,
    RobotMarketTransaction
);
detail_decoder!(
    decode_addon_get,
    RobotAddonTransactionGetRequest,
    RobotAddonTransaction
);

fn with_json<'request, R, O>(
    prepared: PreparedRobotOrderTransaction<'_, 'request, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotOrderTransaction<'_, 'request, R>) -> O,
) -> O {
    let mut storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut storage, body.len(), &mut headers);
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

fn transaction_id(value: &str) -> RobotOrderTransactionId {
    RobotOrderTransactionId::new(value)
        .unwrap_or_else(|_| unreachable!("transaction ID fixture failed"))
}

fn text(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).unwrap_or_else(|_| unreachable!("fixture lost UTF-8"))
}
