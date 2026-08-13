#![no_main]

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    CheckedRobotRdns, PreparedRobotRdns, RobotIpAddress, RobotRdnsGetRequest, RobotRdnsListRequest,
    RobotRdnsName, RobotRdnsSetRequest, RobotRdnsUpdateRequest,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((&selector, body)) = data.split_first() else {
        return;
    };
    match selector % 4 {
        0 => list(body),
        1 => detail(body),
        2 => set(body),
        _ => update(body),
    }
});

fn list(body: &[u8]) {
    let synthetic = synthetic_list_boundary(body).map(|length| vec![b' '; length]);
    let body = synthetic.as_deref().unwrap_or(body);
    let request = RobotRdnsListRequest::all();
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed reverse-DNS list preparation failed"));
    let _ = decode(prepared, StatusCode::OK, body, |checked| {
        checked.decode_response()
    });
}

fn synthetic_list_boundary(body: &[u8]) -> Option<usize> {
    match body {
        b"B-" | b"B-\n" => Some(2_097_151),
        b"B0" | b"B0\n" => Some(2_097_152),
        b"B+" | b"B+\n" => Some(2_097_153),
        _ => None,
    }
}

fn detail(body: &[u8]) {
    let request = RobotRdnsGetRequest::new(address());
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed reverse-DNS detail preparation failed"));
    let _ = decode(prepared, StatusCode::OK, body, |checked| {
        checked.decode_response()
    });
}

fn set(body: &[u8]) {
    let request = RobotRdnsSetRequest::new(address(), ptr());
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 512];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed reverse-DNS set preparation failed"));
    let _ = decode(prepared, StatusCode::CREATED, body, |checked| {
        checked.decode_response()
    });
}

fn update(body: &[u8]) {
    let request = RobotRdnsUpdateRequest::new(address(), ptr());
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 512];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed reverse-DNS update preparation failed"));
    let _ = decode(prepared, StatusCode::OK, body, |checked| {
        checked.decode_response()
    });
}

fn address() -> RobotIpAddress {
    RobotIpAddress::new("192.0.2.50")
        .unwrap_or_else(|_| unreachable!("fixed reverse-DNS address failed"))
}

fn ptr() -> RobotRdnsName {
    RobotRdnsName::new("mail.example.com")
        .unwrap_or_else(|_| unreachable!("fixed reverse-DNS PTR failed"))
}

fn decode<R, O>(
    prepared: PreparedRobotRdns<'_, '_, R>,
    status: StatusCode,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotRdns<'_, '_, R>) -> O,
) -> Option<O> {
    let mut response_storage = body.to_vec();
    let capacity = response_storage.len();
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut response_storage, capacity, &mut headers);
    write_response(&mut response, status, body);
    prepared.validate_response(response).ok().map(decode)
}

fn write_response(response: &mut ResponseBuffer<'_>, status: StatusCode, body: &[u8]) {
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
        .commit(status, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("response commit failed"));
    drop(attempt);
}
