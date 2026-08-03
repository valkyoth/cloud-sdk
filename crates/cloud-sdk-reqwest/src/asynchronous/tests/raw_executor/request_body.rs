use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{
    ContentType, DeliveryPhase, RequestHeader, RequestHeaders, ResponseBuffer, TransportRequest,
};

use super::{RawAsyncTestExt, build_raw_loopback, policy};
use crate::asynchronous::{MAX_RAW_REQUEST_BODY_BYTES, RawHttpError};
use crate::test_server::spawn;

#[test]
fn public_path_accepts_exact_and_rejects_plus_one_request_body() {
    super::super::run_async_test(async {
        let Ok(server) = spawn(
            "200 OK",
            &[("Content-Type", "application/json")],
            b"{}",
            Duration::ZERO,
        ) else {
            return;
        };
        let Some(client) = build_raw_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
            return;
        };
        let header_values = [RequestHeader::content_type(ContentType::JSON)];
        let Ok(headers) = RequestHeaders::new(&header_values) else {
            return;
        };
        let Some(policy) = policy(2) else { return };
        let exact = std::vec![0x5a; MAX_RAW_REQUEST_BODY_BYTES];
        let mut body = [0xa5_u8; 16];
        let mut header_storage = [0xa5_u8; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
        assert!(
            client
                .execute_checked(
                    TransportRequest::new(Method::Post, target)
                        .with_headers(headers)
                        .with_body(&exact),
                    policy,
                    response.writer(),
                )
                .await
                .is_ok()
        );
        drop(exact);

        let Some(client) = build_raw_loopback("http://127.0.0.1:9/v1") else {
            return;
        };
        let oversized = std::vec![0x5a; MAX_RAW_REQUEST_BODY_BYTES.saturating_add(1)];
        let mut body = [0xa5_u8; 16];
        let mut header_storage = [0xa5_u8; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
        assert!(matches!(
            client
                .execute_checked(
                    TransportRequest::new(Method::Post, target)
                        .with_headers(headers)
                        .with_body(&oversized),
                    policy,
                    response.writer(),
                )
                .await,
            Err(error)
                if error.phase() == DeliveryPhase::NotSent
                    && error.error() == &RawHttpError::RequestBodyTooLarge
        ));
    });
}
