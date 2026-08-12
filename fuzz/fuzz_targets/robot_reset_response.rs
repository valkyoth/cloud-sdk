#![no_main]

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    CheckedRobotReset, PreparedRobotReset, RobotResetExecuteRequest, RobotResetGetRequest,
    RobotResetIntent, RobotResetListRequest, RobotResetType, RobotServerNumber,
};
use libfuzzer_sys::fuzz_target;

const DETAIL: &[u8] = br#"{"reset":{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8::","server_number":321,"type":["sw","hw","man"],"operating_status":"not supported"}}"#;

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
    let detail_request = RobotResetGetRequest::new(number());
    let mut detail_target = [0_u8; 64];
    let mut detail_body = [0_u8; 1];
    let detail_prepared = detail_request
        .prepare_bound(PreparationStorage::new(
            &mut detail_target,
            &mut detail_body,
        ))
        .unwrap_or_else(|_| unreachable!("fixed reset preflight preparation failed"));
    let Some(reset) =
        decode(detail_prepared, DETAIL, |checked| checked.decode_response()).and_then(Result::ok)
    else {
        unreachable!("fixed checked reset state failed");
    };
    let request = RobotResetExecuteRequest::from_checked(
        &reset,
        RobotResetIntent::Execute(RobotResetType::Hardware),
    )
    .unwrap_or_else(|_| unreachable!("advertised reset capability failed"));
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 32];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed reset action preparation failed"));
    let _ = decode(prepared, body, |checked| checked.decode_response());
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
    prepared.validate_response(response).ok().map(decode)
}
