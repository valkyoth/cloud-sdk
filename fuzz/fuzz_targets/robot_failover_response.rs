#![no_main]

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    CheckedRobotFailover, PreparedRobotFailover, RobotFailoverDeleteRouteRequest,
    RobotFailoverGetRequest, RobotFailoverListRequest, RobotFailoverRerouteRequest, RobotIpAddress,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((&selector, body)) = data.split_first() else {
        return;
    };
    match selector % 4 {
        0 => list(body),
        1 => detail(body),
        2 => reroute(body),
        _ => delete(body),
    }
});

fn list(body: &[u8]) {
    let request = RobotFailoverListRequest::new();
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed failover list preparation failed"));
    let _ = decode(prepared, body, |checked| checked.decode_response());
}

fn detail(body: &[u8]) {
    let request = RobotFailoverGetRequest::new(route());
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed failover detail preparation failed"));
    let _ = decode(prepared, body, |checked| checked.decode_response());
}

fn reroute(body: &[u8]) {
    let request = RobotFailoverRerouteRequest::new(route(), destination())
        .unwrap_or_else(|_| unreachable!("fixed reroute request failed"));
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 64];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed reroute preparation failed"));
    let _ = decode(prepared, body, |checked| checked.decode_response());
}

fn delete(body: &[u8]) {
    let request = RobotFailoverDeleteRouteRequest::new(route());
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed delete preparation failed"));
    let _ = decode(prepared, body, |checked| checked.decode_response());
}

fn route() -> RobotIpAddress {
    RobotIpAddress::new("192.0.2.50")
        .unwrap_or_else(|_| unreachable!("fixed failover route failed"))
}

fn destination() -> RobotIpAddress {
    RobotIpAddress::new("192.0.2.11")
        .unwrap_or_else(|_| unreachable!("fixed failover destination failed"))
}

fn decode<R, O>(
    prepared: PreparedRobotFailover<'_, '_, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotFailover<'_, '_, R>) -> O,
) -> Option<O> {
    let mut response_storage = body.to_vec();
    let capacity = response_storage.len();
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut response_storage, capacity, &mut headers);
    write_response(&mut response, body);
    prepared.validate_response(response).ok().map(decode)
}

fn write_response(response: &mut ResponseBuffer<'_>, body: &[u8]) {
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
}
