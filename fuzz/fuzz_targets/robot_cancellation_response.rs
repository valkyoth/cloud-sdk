#![no_main]

use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    RobotIpAddress, RobotIpCancellationGetRequest, RobotServerCancellationGetRequest,
    RobotServerNumber, RobotSubnetAddress, RobotSubnetCancellationGetRequest,
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
    decode(request, body, |request, checked| {
        let _ = request.decode_response(checked);
    });
}

fn ip(body: &[u8]) {
    let address =
        RobotIpAddress::new("192.0.2.10").unwrap_or_else(|_| unreachable!("fixed IP failed"));
    let request = RobotIpCancellationGetRequest::new(address);
    decode(request, body, |request, checked| {
        let _ = request.decode_response(checked);
    });
}

fn subnet(body: &[u8]) {
    let address = RobotSubnetAddress::new("2001:db8::")
        .unwrap_or_else(|_| unreachable!("fixed subnet failed"));
    let request = RobotSubnetCancellationGetRequest::new(address);
    decode(request, body, |request, checked| {
        let _ = request.decode_response(checked);
    });
}

fn decode<R>(
    request: R,
    body: &[u8],
    decode: impl FnOnce(R, cloud_sdk::operation::CheckedResponseGuard<'_>),
) where
    R: PrepareOperation,
{
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed cancellation preparation failed"));
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
        decode(request, checked);
    }
}
