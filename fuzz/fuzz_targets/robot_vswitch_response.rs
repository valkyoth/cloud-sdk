#![no_main]

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    RobotVSwitchCreateRequest, RobotVSwitchGetRequest, RobotVSwitchId, RobotVSwitchListRequest,
    RobotVSwitchName, RobotVlanId,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &[u8]| {
    let Some((&selector, body)) = input.split_first() else {
        return;
    };
    match selector % 3 {
        0 => fuzz_detail(body),
        1 => fuzz_list(body),
        _ => fuzz_create(body),
    }
});

fn fuzz_detail(body: &[u8]) {
    let request = RobotVSwitchGetRequest::new(id());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed vSwitch detail preparation failed"));
    with_response(StatusCode::OK, body, |response| {
        if let Ok(checked) = prepared.validate_response(response) {
            let _ = checked.decode_response();
        }
    });
}

fn fuzz_list(body: &[u8]) {
    let request = RobotVSwitchListRequest::new();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed vSwitch list preparation failed"));
    with_response(StatusCode::OK, body, |response| {
        if let Ok(checked) = prepared.validate_response(response) {
            let _ = checked.decode_response();
        }
    });
}

fn fuzz_create(body: &[u8]) {
    let name = RobotVSwitchName::new("fuzz fabric")
        .unwrap_or_else(|_| unreachable!("fixed vSwitch name failed"));
    let vlan = RobotVlanId::new(4000).unwrap_or_else(|_| unreachable!("fixed vSwitch VLAN failed"));
    let request = RobotVSwitchCreateRequest::new(name, vlan);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed vSwitch create preparation failed"));
    with_response(StatusCode::CREATED, body, |response| {
        if let Ok(checked) = prepared.validate_response(response) {
            let _ = checked.decode_response();
        }
    });
}

fn id() -> RobotVSwitchId {
    RobotVSwitchId::new(4321).unwrap_or_else(|_| unreachable!("fixed vSwitch ID failed"))
}

fn with_response(status: StatusCode, body: &[u8], inspect: impl FnOnce(ResponseBuffer<'_>)) {
    let mut storage = body.to_vec();
    let capacity = storage.len();
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut storage, capacity, &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("vSwitch response attempt failed"));
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!("vSwitch response headers failed"))
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!("vSwitch content type failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("vSwitch response body failed"))
        .copy_from_slice(body);
    attempt
        .commit(status, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("vSwitch response commit failed"));
    drop(attempt);
    inspect(response);
}
