#![no_main]

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    RobotIpAddress, RobotSubnetAddress, RobotTrafficGranularity, RobotTrafficInterval,
    RobotTrafficRequest, RobotTrafficTarget,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Some((&selector, body)) = data.split_first() else {
        return;
    };
    let single = selector & 1 != 0;
    let target = if selector & 2 == 0 {
        RobotTrafficTarget::ip(
            RobotIpAddress::new("192.0.2.10")
                .unwrap_or_else(|_| unreachable!("fixed traffic IP failed")),
        )
    } else {
        RobotTrafficTarget::subnet(
            RobotSubnetAddress::new("2001:db8::")
                .unwrap_or_else(|_| unreachable!("fixed traffic subnet failed")),
        )
    };
    let interval =
        RobotTrafficInterval::new(RobotTrafficGranularity::Month, "2026-07-01", "2026-07-31")
            .unwrap_or_else(|_| unreachable!("fixed traffic interval failed"));
    let request = RobotTrafficRequest::new(interval, vec![target], single)
        .unwrap_or_else(|_| unreachable!("fixed traffic request failed"));
    let mut target_storage = [0_u8; 32];
    let mut request_body = [0_u8; 256];
    let prepared = request
        .prepare_bound(PreparationStorage::new(
            &mut target_storage,
            &mut request_body,
        ))
        .unwrap_or_else(|_| unreachable!("fixed traffic preparation failed"));

    let synthetic = match body {
        b"B-" | b"B-\n" => Some(8_388_607),
        b"B0" | b"B0\n" => Some(8_388_608),
        b"B+" | b"B+\n" => Some(8_388_609),
        _ => None,
    }
    .map(|length| vec![b' '; length]);
    let body = synthetic.as_deref().unwrap_or(body);
    let mut response_storage = body.to_vec();
    let capacity = response_storage.len();
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut response_storage, capacity, &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("traffic response attempt failed"));
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!("traffic response headers failed"))
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!("traffic content type failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("traffic response body failed"))
        .copy_from_slice(body);
    attempt
        .commit(StatusCode::OK, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("traffic response commit failed"));
    drop(attempt);
    if let Ok(checked) = prepared.validate_response(response) {
        let _ = checked.decode_response();
    }
});
