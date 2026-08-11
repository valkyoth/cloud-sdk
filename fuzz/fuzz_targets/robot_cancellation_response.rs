#![no_main]

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    CheckedCancellation, PreparedCancellation, RobotIpAddress, RobotIpCancellationGetRequest,
    RobotServerCancellationGetRequest, RobotServerNumber, RobotSubnetAddress,
    RobotSubnetCancellationGetRequest,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((&selector, body)) = data.split_first() else {
        return;
    };
    match selector % 3 {
        0 => server(body),
        1 => ip(body),
        _ => subnet(body),
    }
});

fn server(body: &[u8]) {
    let number =
        RobotServerNumber::new(321).unwrap_or_else(|_| unreachable!("fixed server number failed"));
    let request = RobotServerCancellationGetRequest::new(number);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed cancellation preparation failed"));
    decode(prepared, body, |checked| {
        let _ = checked.decode_response();
    });
}

fn ip(body: &[u8]) {
    let address =
        RobotIpAddress::new("192.0.2.10").unwrap_or_else(|_| unreachable!("fixed IP failed"));
    let request = RobotIpCancellationGetRequest::new(address);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed cancellation preparation failed"));
    decode(prepared, body, |checked| {
        let _ = checked.decode_response();
    });
}

fn subnet(body: &[u8]) {
    let address = RobotSubnetAddress::new("2001:db8::")
        .unwrap_or_else(|_| unreachable!("fixed subnet failed"));
    let request = RobotSubnetCancellationGetRequest::new(address);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed cancellation preparation failed"));
    decode(prepared, body, |checked| {
        let _ = checked.decode_response();
    });
}

fn decode<R, O>(
    prepared: PreparedCancellation<'_, '_, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedCancellation<'_, '_, R>) -> O,
) {
    let mut response_storage = body.to_vec();
    let capacity = response_storage.len();
    let mut headers = [0_u8; 64];
    let mut response = ResponseBuffer::new(&mut response_storage, capacity, &mut headers);
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
        .unwrap_or_else(|_| unreachable!("response commit failed"));
    drop(attempt);
    if let Ok(checked) = prepared.validate_response(response) {
        decode(checked);
    }
}
