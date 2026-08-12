#![no_main]

use cloud_sdk::operation::{PermitTimestamp, PreparationStorage};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    CheckedRobotSubnet, PreparedRobotSubnet, RobotMacAddress, RobotSubnetAddress,
    RobotSubnetGetRequest, RobotSubnetListRequest, RobotSubnetMacDeleteRequest,
    RobotSubnetMacGetRequest, RobotSubnetMacSetRequest, RobotSubnetMutationLease,
    RobotSubnetObservationWindow,
};
use libfuzzer_sys::fuzz_target;

const DELETE_SUBNET: &[u8] = br#"{"subnet":{"ip":"2001:db8::","mask":64,"gateway":"2001:db8::1","server_ip":"192.0.2.1","server_number":321,"failover":false,"locked":false,"traffic_warnings":true,"traffic_hourly":50,"traffic_daily":500,"traffic_monthly":8}}"#;
const DELETE_MAC: &[u8] = br#"{"mac":{"ip":"2001:db8::","mask":"64","mac":"00:21:85:62:3e:9d","possible_mac":{"192.0.2.1":"00:21:85:62:3e:9c","192.0.2.2":"00:21:85:62:3e:9d"}}}"#;

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
    let _ = decode(prepared, body, |checked| {
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
    let _ = decode(prepared, body, |checked| {
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
    let _ = decode(prepared, body, |checked| {
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
    let _ = decode(prepared, body, |checked| {
        let _ = checked.decode_response();
    });
}

fn delete_mac(body: &[u8]) {
    let Some(request) = delete_request() else {
        unreachable!("fixed default MAC evidence failed");
    };
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed subnet MAC delete preparation failed"));
    let _ = decode(prepared, body, |checked| {
        let _ = checked.decode_response();
    });
}

fn delete_request() -> Option<RobotSubnetMacDeleteRequest> {
    let detail_request = RobotSubnetGetRequest::new(address());
    let mut detail_target = [0_u8; 128];
    let mut detail_body = [0_u8; 1];
    let prepared = detail_request
        .prepare_bound(PreparationStorage::new(
            &mut detail_target,
            &mut detail_body,
        ))
        .ok()?;
    let subnet = decode(prepared, DELETE_SUBNET, |checked| checked.decode_response())?.ok()?;

    let mac_request = RobotSubnetMacGetRequest::new(address());
    let mut mac_target = [0_u8; 128];
    let mut mac_body = [0_u8; 1];
    let prepared = mac_request
        .prepare_bound(PreparationStorage::new(&mut mac_target, &mut mac_body))
        .ok()?;
    let mac_state = decode(prepared, DELETE_MAC, |checked| checked.decode_response())?.ok()?;
    let observations = RobotSubnetObservationWindow::new(
        PermitTimestamp::from_seconds(1),
        PermitTimestamp::from_seconds(2),
    )
    .ok()?;
    let lease = RobotSubnetMutationLease::new(
        address(),
        b"fuzz-lock-generation-0001",
        PermitTimestamp::from_seconds(31),
    )
    .ok()?;
    RobotSubnetMacDeleteRequest::from_checked(subnet, mac_state, observations, lease).ok()
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
    prepared.validate_response(response).ok().map(decode)
}
