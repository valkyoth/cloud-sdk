use alloc::{format, string::String, vec};
use core::fmt::Write;

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::robot::{RobotIpList, RobotIpListRequest};

const ENTRY: &[u8] = br#"[{"rdns":{"ip":"192.0.2.50","ptr":"mail.example.com"}}]"#;
const EMPTY: &[u8] = b"[]";

#[test]
fn filtered_lists_require_matching_checked_ip_inventory() {
    let inventory = decode_ip_inventory("192.0.2.50", "192.0.2.10");
    assert_eq!(
        decode_filtered_list(None).err(),
        Some(RobotRdnsDecodeError::UnverifiableServerFilter)
    );
    let result = decode_filtered_list(Some(&inventory))
        .unwrap_or_else(|_| unreachable!("matching IP inventory was rejected"));
    assert_eq!(result.len(), 1);

    assert_eq!(
        decode_filtered_body(EMPTY, Some(&inventory)).err(),
        Some(RobotRdnsDecodeError::UnverifiableServerFilter)
    );

    let wrong_address = decode_ip_inventory("192.0.2.51", "192.0.2.10");
    assert_eq!(
        decode_filtered_list(Some(&wrong_address)).err(),
        Some(RobotRdnsDecodeError::ResponseIdentityMismatch)
    );
    let wrong_server = decode_ip_inventory("192.0.2.50", "192.0.2.11");
    assert_eq!(
        decode_filtered_list(Some(&wrong_server)).err(),
        Some(RobotRdnsDecodeError::ResponseIdentityMismatch)
    );
}

#[test]
fn maximum_filtered_list_uses_the_bounded_assignment_index() {
    let mut inventory_body = String::from("[");
    let mut rdns_body = String::from("[");
    for index in 0..MAX_ROBOT_RDNS_LIST_ITEMS {
        let separator = if index == 0 { "" } else { "," };
        let third = index / 256;
        let fourth = index % 256;
        write!(
            inventory_body,
            "{separator}{{\"ip\":{{\"ip\":\"198.18.{third}.{fourth}\",\"server_ip\":\"192.0.2.10\",\"server_number\":321,\"locked\":false,\"separate_mac\":null,\"traffic_warnings\":false,\"traffic_hourly\":50,\"traffic_daily\":500,\"traffic_monthly\":8}}}}"
        )
        .unwrap_or_else(|_| unreachable!("inventory fixture write failed"));
        write!(
            rdns_body,
            "{separator}{{\"rdns\":{{\"ip\":\"198.18.{third}.{fourth}\",\"ptr\":\"mail.example.com\"}}}}"
        )
        .unwrap_or_else(|_| unreachable!("reverse-DNS fixture write failed"));
    }
    inventory_body.push(']');
    rdns_body.push(']');

    let inventory = decode_ip_inventory_body(inventory_body.as_bytes());
    let result = decode_filtered_body(rdns_body.as_bytes(), Some(&inventory))
        .unwrap_or_else(|_| unreachable!("maximum filtered response was rejected"));
    assert_eq!(result.len(), MAX_ROBOT_RDNS_LIST_ITEMS);
}

fn decode_filtered_list(
    inventory: Option<&RobotIpList>,
) -> Result<RobotRdnsFilteredMembership, RobotRdnsDecodeError> {
    decode_filtered_body(ENTRY, inventory)
}

fn decode_filtered_body(
    body: &[u8],
    inventory: Option<&RobotIpList>,
) -> Result<RobotRdnsFilteredMembership, RobotRdnsDecodeError> {
    let request = RobotRdnsListRequest::for_server(ip("192.0.2.10"))
        .unwrap_or_else(|_| unreachable!("filtered request fixture failed"));
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("filtered list preparation failed"));
    with_json(body, |response| {
        let checked = prepared
            .validate_response(response)
            .unwrap_or_else(|_| unreachable!("filtered response policy failed"));
        match inventory {
            Some(inventory) => checked.decode_response_with_inventory(inventory),
            None => match checked.decode_response() {
                Err(error) => Err(error),
                Ok(_) => unreachable!("filtered response decoded without inventory"),
            },
        }
    })
}

fn decode_ip_inventory(address: &str, server_address: &str) -> RobotIpList {
    let body = format!(
        "[{{\"ip\":{{\"ip\":\"{address}\",\"server_ip\":\"{server_address}\",\"server_number\":321,\"locked\":false,\"separate_mac\":null,\"traffic_warnings\":false,\"traffic_hourly\":50,\"traffic_daily\":500,\"traffic_monthly\":8}}}}]"
    );
    decode_ip_inventory_body(body.as_bytes())
}

fn decode_ip_inventory_body(body: &[u8]) -> RobotIpList {
    let request = RobotIpListRequest::all();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("IP inventory preparation failed"));
    with_json(body, |response| {
        prepared
            .validate_response(response)
            .unwrap_or_else(|_| unreachable!("IP inventory policy failed"))
            .decode_response()
            .unwrap_or_else(|_| unreachable!("IP inventory decode failed"))
    })
}

fn with_json<R>(body: &[u8], inspect: impl FnOnce(ResponseBuffer<'_>) -> R) -> R {
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
    inspect(response)
}

fn ip(value: &str) -> crate::robot::RobotIpAddress {
    crate::robot::RobotIpAddress::new(value).unwrap_or_else(|_| unreachable!("IP fixture failed"))
}
