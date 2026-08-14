#![no_main]

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    RobotAddonProductListRequest, RobotMarketProductGetRequest, RobotMarketProductId,
    RobotMarketProductListRequest, RobotOrderCurrencyRequest, RobotOrderProductId,
    RobotServerNumber, RobotStandardProductGetRequest, RobotStandardProductListRequest,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: &[u8]| {
    let Some((&selector, body)) = input.split_first() else {
        return;
    };
    match selector % 6 {
        0 => standard_list(body),
        1 => standard_get(body),
        2 => market_list(body),
        3 => market_get(body),
        4 => addon_list(body),
        _ => currency(body),
    }
});

macro_rules! decode {
    ($request:expr, $body:expr) => {{
        let request = $request;
        let mut target = [0_u8; 4_096];
        let mut request_body = [0_u8; 1];
        let prepared = request
            .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
            .unwrap_or_else(|_| unreachable!("fixed Robot catalog preparation failed"));
        with_response($body, |response| {
            if let Ok(checked) = prepared.validate_response(response) {
                let _ = checked.decode_response();
            }
        });
    }};
}

fn standard_list(body: &[u8]) {
    decode!(RobotStandardProductListRequest::default(), body);
}

fn standard_get(body: &[u8]) {
    let id = RobotOrderProductId::new("EX40")
        .unwrap_or_else(|_| unreachable!("fixed standard product ID failed"));
    decode!(RobotStandardProductGetRequest::new(id), body);
}

fn market_list(body: &[u8]) {
    decode!(RobotMarketProductListRequest::new(), body);
}

fn market_get(body: &[u8]) {
    let id = RobotMarketProductId::new(282_323)
        .unwrap_or_else(|_| unreachable!("fixed market product ID failed"));
    decode!(RobotMarketProductGetRequest::new(id), body);
}

fn addon_list(body: &[u8]) {
    let server =
        RobotServerNumber::new(321).unwrap_or_else(|_| unreachable!("fixed addon server failed"));
    decode!(RobotAddonProductListRequest::new(server), body);
}

fn currency(body: &[u8]) {
    decode!(RobotOrderCurrencyRequest::new(), body);
}

fn with_response(body: &[u8], inspect: impl FnOnce(ResponseBuffer<'_>)) {
    let mut storage = body.to_vec();
    let capacity = storage.len();
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut storage, capacity, &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("catalog response attempt failed"));
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!("catalog response headers failed"))
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!("catalog content type failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("catalog response body failed"))
        .copy_from_slice(body);
    attempt
        .commit(StatusCode::OK, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("catalog response commit failed"));
    drop(attempt);
    inspect(response);
}
