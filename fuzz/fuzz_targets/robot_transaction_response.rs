#![no_main]

use cloud_sdk::operation::PreparationStorageGuard;
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};
use cloud_sdk_hetzner::robot::{
    RobotAddonTransactionGetRequest, RobotAddonTransactionListRequest,
    RobotMarketTransactionGetRequest, RobotMarketTransactionListRequest, RobotOrderTransactionId,
    RobotStandardTransactionGetRequest, RobotStandardTransactionListRequest,
};
use libfuzzer_sys::fuzz_target;

macro_rules! decode {
    ($request:expr, $body:expr) => {{
        let request = $request;
        let mut target = [0_u8; 4_096];
        let mut request_body = [0_u8; 1];
        let mut storage = PreparationStorageGuard::new(&mut target, &mut request_body);
        let prepared = request
            .prepare_bound(&mut storage)
            .unwrap_or_else(|_| unreachable!("fixed Robot transaction preparation failed"));
        with_response($body, |response| {
            if let Ok(checked) = prepared.validate_response(response) {
                let _ = checked.decode_response();
            }
        });
    }};
}

fuzz_target!(|body: &[u8]| {
    decode!(RobotStandardTransactionListRequest::new(), body);
    decode!(RobotStandardTransactionGetRequest::new(id("B-fuzz")), body);
    decode!(RobotMarketTransactionListRequest::new(), body);
    decode!(RobotMarketTransactionGetRequest::new(id("B-fuzz")), body);
    decode!(RobotAddonTransactionListRequest::new(), body);
    decode!(RobotAddonTransactionGetRequest::new(id("B-fuzz")), body);
});

fn id(value: &str) -> RobotOrderTransactionId {
    RobotOrderTransactionId::new(value)
        .unwrap_or_else(|_| unreachable!("fixed Robot transaction ID failed"))
}

fn with_response(body: &[u8], inspect: impl FnOnce(ResponseBuffer<'_>)) {
    let mut storage = body.to_vec();
    let capacity = storage.len();
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut storage, capacity, &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("transaction response attempt failed"));
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!("transaction response headers failed"))
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!("transaction content type failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("transaction response body failed"))
        .copy_from_slice(body);
    attempt
        .commit(StatusCode::OK, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("transaction response commit failed"));
    drop(attempt);
    inspect(response);
}
