#![no_main]

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    CheckedRobotReset, PreparedRobotReset, RobotResetGetRequest, RobotResetListRequest,
    RobotServerNumber, decode_robot_reset_action,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((&selector, body)) = data.split_first() else {
        return;
    };
    match selector % 3 {
        0 => list(body),
        1 => detail(body),
        _ => action(body),
    }
});

fn list(body: &[u8]) {
    let request = RobotResetListRequest::new();
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed reset list preparation failed"));
    let _ = decode(prepared, body, |checked| checked.decode_response());
}

fn detail(body: &[u8]) {
    let request = RobotResetGetRequest::new(number());
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed reset detail preparation failed"));
    let _ = decode(prepared, body, |checked| checked.decode_response());
}

fn action(body: &[u8]) {
    let request = RobotResetGetRequest::new(number());
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed reset detail preparation failed"));
    let mut response_storage = body.to_vec();
    let capacity = response_storage.len();
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut response_storage, capacity, &mut headers);
    write_response(&mut response, body);
    let _ = prepared
        .as_untyped()
        .validate_response(response)
        .map(|checked| checked.decode_owned_with_workspace(decode_robot_reset_action));
}

fn number() -> RobotServerNumber {
    RobotServerNumber::new(321).unwrap_or_else(|_| unreachable!("fixed reset server number failed"))
}

fn decode<R, O>(
    prepared: PreparedRobotReset<'_, '_, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotReset<'_, '_, R>) -> O,
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
