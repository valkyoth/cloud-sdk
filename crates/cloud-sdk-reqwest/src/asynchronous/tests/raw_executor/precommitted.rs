use cloud_sdk::Method;
use cloud_sdk::transport::{
    ResponseBuffer, ResponseMetadata, StatusCode, TransportFailure, TransportRequest,
};

use super::{RawAsyncTestExt, build_raw_loopback, policy};
use crate::asynchronous::RawHttpError;

#[test]
fn raw_async_precommitted_writer_fails_before_network_access() {
    super::super::run_async_test(async {
        let Some(client) = build_raw_loopback("http://127.0.0.1:1/v1") else {
            return;
        };
        let Ok(target) = cloud_sdk::transport::RequestTarget::new("/precommitted") else {
            return;
        };
        let Some(policy) = policy(2) else { return };
        let mut body = [0xa5_u8; 8];
        let mut headers = [0xa5_u8; 128];
        let mut response = ResponseBuffer::new(&mut body, 8, &mut headers);
        let mut attempt = response
            .writer()
            .begin_attempt()
            .unwrap_or_else(|_| unreachable!());
        assert!(
            attempt
                .commit(StatusCode::OK, 0, ResponseMetadata::EMPTY)
                .is_ok()
        );
        drop(attempt);
        assert_eq!(
            client
                .execute_checked(
                    TransportRequest::new(Method::Get, target),
                    policy,
                    response.writer(),
                )
                .await,
            Err(TransportFailure::not_sent(
                RawHttpError::ResponseAlreadyCommitted
            ))
        );
    });
}
