#![no_main]

use core::fmt::Write;

use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    RobotServerDecodeError, RobotServerGetRequest, RobotServerListRequest, RobotServerNumber,
};
use libfuzzer_sys::fuzz_target;

const EXACT_LIST: &[u8] = b"exact-list-bound";
const EXACT_COLLECTIONS: &[u8] = b"exact-collections";
const DUPLICATE_LAST: &[u8] = b"duplicate-last";

fuzz_target!(|data: &[u8]| {
    let marker = data.strip_suffix(b"\n").unwrap_or(data);
    if marker == EXACT_LIST {
        let payload = exact_list_payload();
        assert_eq!(decode_list(&payload), Some(Ok(())));
        return;
    }
    if marker == EXACT_COLLECTIONS {
        let payload = collection_payload(false);
        assert_eq!(decode_detail(&payload), Some(Ok(())));
        return;
    }
    if marker == DUPLICATE_LAST {
        let payload = collection_payload(true);
        assert_eq!(
            decode_detail(&payload),
            Some(Err(RobotServerDecodeError::DuplicateIdentity))
        );
        return;
    }
    let Some((&selector, payload)) = data.split_first() else {
        return;
    };
    if selector & 1 == 0 {
        decode_list(payload);
    } else {
        decode_detail(payload);
    }
});

fn decode_list(body: &[u8]) -> Option<Result<(), RobotServerDecodeError>> {
    let request = RobotServerListRequest::new();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 64];
    let prepared = request
        .prepare(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed list preparation failed"));
    with_response(prepared, body, |checked| {
        request.decode_response(checked).map(|_| ())
    })
}

fn decode_detail(body: &[u8]) -> Option<Result<(), RobotServerDecodeError>> {
    let number =
        RobotServerNumber::new(321).unwrap_or_else(|_| unreachable!("fixed server number failed"));
    let request = RobotServerGetRequest::new(number);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 64];
    let prepared = request
        .prepare(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed detail preparation failed"));
    with_response(prepared, body, |checked| {
        request.decode_response(checked).map(|_| ())
    })
}

fn with_response<R>(
    prepared: cloud_sdk::operation::PreparedRequest<'_>,
    body: &[u8],
    decode: impl FnOnce(cloud_sdk::operation::CheckedResponseGuard<'_>) -> R,
) -> Option<R> {
    let mut response_storage = body.to_vec();
    let capacity = response_storage.len();
    let mut header_storage = [0_u8; 256];
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
    prepared.validate_response(response).ok().map(decode)
}

fn exact_list_payload() -> Vec<u8> {
    let mut output = String::with_capacity(1_200_000);
    output.push('[');
    for number in 1..=4_096_u64 {
        if number != 1 {
            output.push(',');
        }
        write!(output, "{{\"server\":")
            .unwrap_or_else(|_| unreachable!("string formatting failed"));
        write_summary(&mut output, number, "[\"192.0.2.10\"]", "null");
        output.push('}');
    }
    output.push(']');
    output.into_bytes()
}

fn collection_payload(duplicate_last: bool) -> Vec<u8> {
    let mut addresses = String::with_capacity(100_000);
    addresses.push_str("[\"192.0.2.10\"");
    let upper = if duplicate_last { 4_095 } else { 4_096 };
    for value in 1..upper {
        write!(addresses, ",\"2001:db8::{value:x}\"")
            .unwrap_or_else(|_| unreachable!("string formatting failed"));
    }
    if duplicate_last {
        addresses.push_str(",\"2001:db8::ffe\"");
    }
    addresses.push(']');

    let mut subnets = String::with_capacity(180_000);
    subnets.push('[');
    for value in 0..4_096_u16 {
        if value != 0 {
            subnets.push(',');
        }
        write!(
            subnets,
            "{{\"ip\":\"2001:db8::{value:x}\",\"mask\":\"128\"}}"
        )
        .unwrap_or_else(|_| unreachable!("string formatting failed"));
    }
    subnets.push(']');

    let mut output = String::with_capacity(addresses.len() + subnets.len() + 512);
    output.push_str("{\"server\":");
    write_summary(&mut output, 321, &addresses, &subnets);
    let closed_summary = output.pop();
    debug_assert_eq!(closed_summary, Some('}'));
    output.push_str(",\"reset\":true,\"rescue\":true,\"vnc\":false,\"windows\":true,\"plesk\":false,\"cpanel\":false,\"wol\":true,\"hot_swap\":true}}");
    output.into_bytes()
}

fn write_summary(output: &mut String, number: u64, addresses: &str, subnets: &str) {
    write!(
        output,
        "{{\"server_ip\":\"192.0.2.10\",\"server_ipv6_net\":\"2001:db8:1::\",\"server_number\":{number},\"server_name\":\"server-1\",\"product\":\"AX42\",\"dc\":\"FSN1-DC10\",\"traffic\":\"unlimited\",\"status\":\"ready\",\"cancelled\":false,\"paid_until\":\"2028-02-29\",\"ip\":{addresses},\"subnet\":{subnets}}}"
    )
    .unwrap_or_else(|_| unreachable!("string formatting failed"));
}
