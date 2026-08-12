#![no_main]

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    CheckedRobotSubnet, PreparedRobotSubnet, RobotMacAddress, RobotSubnetAddress,
    RobotSubnetGetRequest, RobotSubnetListRequest, RobotSubnetMacDeleteRequest,
    RobotSubnetMacGetRequest, RobotSubnetMacSetRequest,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((&selector, body)) = data.split_first() else {
        return;
    };
    match selector % 5 {
        0 => list(body),
        1 => detail(body),
        2 => mac(body),
        3 => set_mac(body),
        _ => delete_mac(body),
    }
});

fn list(body: &[u8]) {
    let request = RobotSubnetListRequest::all();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed subnet list preparation failed"));
    decode(prepared, body, |checked| {
        let _ = checked.decode_response();
    });
}

fn detail(body: &[u8]) {
    let request = RobotSubnetGetRequest::new(address());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed subnet detail preparation failed"));
    decode(prepared, body, |checked| {
        let _ = checked.decode_response();
    });
}

fn mac(body: &[u8]) {
    let request = RobotSubnetMacGetRequest::new(address());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed subnet MAC preparation failed"));
    decode(prepared, body, |checked| {
        let _ = checked.decode_response();
    });
}

fn set_mac(body: &[u8]) {
    let request = RobotSubnetMacSetRequest::new(address(), selected_mac());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed subnet MAC set preparation failed"));
    decode(prepared, body, |checked| {
        let _ = checked.decode_response();
    });
}

fn delete_mac(body: &[u8]) {
    let request = RobotSubnetMacDeleteRequest::new(address());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed subnet MAC delete preparation failed"));
    decode(prepared, body, |checked| {
        let _ = checked.decode_response();
    });
}

fn address() -> RobotSubnetAddress {
    RobotSubnetAddress::new("2001:db8::")
        .unwrap_or_else(|_| unreachable!("fixed subnet address failed"))
}

fn selected_mac() -> RobotMacAddress {
    RobotMacAddress::new("00:21:85:62:3e:9d")
        .unwrap_or_else(|_| unreachable!("fixed subnet MAC failed"))
}

fn decode<R, O>(
    prepared: PreparedRobotSubnet<'_, '_, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotSubnet<'_, '_, R>) -> O,
) {
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
