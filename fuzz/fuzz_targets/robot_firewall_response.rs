#![no_main]

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    RobotFirewallGetRequest, RobotFirewallTemplateGetRequest, RobotFirewallTemplateId,
    RobotFirewallTemplateListRequest, RobotServerNumber,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &[u8]| {
    let Some((&selector, body)) = input.split_first() else {
        return;
    };
    match selector % 3 {
        0 => fuzz_firewall(body),
        1 => fuzz_template(body),
        _ => fuzz_template_list(body),
    }
});

fn fuzz_firewall(body: &[u8]) {
    let request = RobotFirewallGetRequest::new(server());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed firewall preparation failed"));
    with_response(body, |response| {
        if let Ok(checked) = prepared.validate_response(response) {
            let _ = checked.decode_response();
        }
    });
}

fn fuzz_template(body: &[u8]) {
    let id = RobotFirewallTemplateId::new(17)
        .unwrap_or_else(|_| unreachable!("fixed template identity failed"));
    let request = RobotFirewallTemplateGetRequest::new(id);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed template preparation failed"));
    with_response(body, |response| {
        if let Ok(checked) = prepared.validate_response(response) {
            let _ = checked.decode_response();
        }
    });
}

fn fuzz_template_list(body: &[u8]) {
    let request = RobotFirewallTemplateListRequest::new();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("fixed template-list preparation failed"));
    with_response(body, |response| {
        if let Ok(checked) = prepared.validate_response(response) {
            let _ = checked.decode_response();
        }
    });
}

fn server() -> RobotServerNumber {
    RobotServerNumber::new(321).unwrap_or_else(|_| unreachable!("fixed server identity failed"))
}

fn with_response(body: &[u8], inspect: impl FnOnce(ResponseBuffer<'_>)) {
    let mut storage = body.to_vec();
    let capacity = storage.len();
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut storage, capacity, &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("firewall response attempt failed"));
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!("firewall response headers failed"))
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!("firewall content type failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("firewall response body failed"))
        .copy_from_slice(body);
    attempt
        .commit(StatusCode::OK, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("firewall response commit failed"));
    drop(attempt);
    inspect(response);
}
