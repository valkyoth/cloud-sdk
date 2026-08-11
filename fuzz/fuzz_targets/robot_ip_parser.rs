#![no_main]

use core::net::IpAddr;
use core::str::FromStr;

use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::RobotServerListRequest;
use libfuzzer_sys::fuzz_target;

const MAX_CANDIDATE_BYTES: usize = 128;

fuzz_target!(|data: &[u8]| {
    let data = data.strip_suffix(b"\n").unwrap_or(data);
    if data.len() > MAX_CANDIDATE_BYTES {
        return;
    }
    let Ok(candidate) = core::str::from_utf8(data) else {
        return;
    };
    let oracle = IpAddr::from_str(candidate).ok();
    assert_eq!(
        decode_candidate(candidate, AddressField::Ipv4),
        oracle.filter(IpAddr::is_ipv4)
    );
    assert_eq!(
        decode_candidate(candidate, AddressField::Ipv6),
        oracle.filter(IpAddr::is_ipv6)
    );
});

#[derive(Clone, Copy)]
enum AddressField {
    Ipv4,
    Ipv6,
}

fn decode_candidate(candidate: &str, field: AddressField) -> Option<IpAddr> {
    let escaped = serde_json::to_string(candidate)
        .unwrap_or_else(|_| unreachable!("bounded UTF-8 candidate did not serialize"));
    let (ipv4, ipv6, addresses) = match field {
        AddressField::Ipv4 => (escaped.as_str(), "\"2001:db8::\"", escaped.as_str()),
        AddressField::Ipv6 => ("\"192.0.2.10\"", escaped.as_str(), "\"192.0.2.10\""),
    };
    let body = format!(
        "[{{\"server\":{{\"server_ip\":{ipv4},\"server_ipv6_net\":{ipv6},\
         \"server_number\":1,\"server_name\":\"server-1\",\"product\":\"AX42\",\
         \"dc\":\"FSN1-DC10\",\"traffic\":\"unlimited\",\"status\":\"ready\",\
         \"cancelled\":false,\"paid_until\":\"2028-02-29\",\"ip\":[{addresses}],\
         \"subnet\":null}}}}]"
    );
    decode_list(body.as_bytes(), field)
}

fn decode_list(body: &[u8], field: AddressField) -> Option<IpAddr> {
    let request = RobotServerListRequest::new();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed Robot list preparation failed"));
    let mut response_storage = body.to_vec();
    let capacity = response_storage.len();
    let mut header_storage = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut response_storage, capacity, &mut header_storage);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("fresh response attempt failed"));
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!("fresh response headers failed"))
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!("fixed content type failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("fresh response body failed"))
        .copy_from_slice(body);
    attempt
        .commit(StatusCode::OK, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("bounded response commit failed"));
    drop(attempt);
    let checked = prepared.validate_response(response).ok()?;
    let list = request.decode_response(checked).ok()?;
    let summary = list.as_slice().first()?;
    Some(match field {
        AddressField::Ipv4 => IpAddr::V4(summary.with_main_ipv4(|address| address)),
        AddressField::Ipv6 => IpAddr::V6(summary.with_main_ipv6_network(|address| address)),
    })
}
