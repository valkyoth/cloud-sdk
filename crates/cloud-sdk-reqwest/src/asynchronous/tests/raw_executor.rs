use core::future::{Future, poll_fn};
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{
    DeliveryPhase, MediaType, RawResponsePolicy, ResponseBuffer, ResponseMediaPolicy,
    TransportRequest,
};

use super::{run_async_test, test_timeouts};
use crate::asynchronous::{RawAsyncClient, RawAsyncClientBuilder, RawHttpError, UserAgent};
use crate::test_server::{spawn, spawn_concurrent_pair, spawn_raw_response, spawn_raw_split};

mod driver;
mod precommitted;
mod request_body;

pub(super) use driver::RawAsyncTestExt;

fn build_raw_loopback(endpoint: &str) -> Option<RawAsyncClient> {
    let endpoint = crate::asynchronous::HttpsEndpoint::local_http(endpoint).ok()?;
    let user_agent = UserAgent::new("cloud-sdk-raw-test/0.40").ok()?;
    RawAsyncClientBuilder::new(endpoint, user_agent, test_timeouts()?)
        .build_for_loopback()
        .ok()
}

fn policy(limit: u8) -> Option<RawResponsePolicy<'static>> {
    RawResponsePolicy::new(
        16,
        4,
        ResponseMediaPolicy::Required(&[MediaType::JSON]),
        ResponseMediaPolicy::Optional(&[MediaType::JSON]),
        &[],
        limit,
    )
    .ok()
}

#[test]
fn raw_async_streams_directly_into_the_caller_buffer() {
    run_async_test(async {
        let server = spawn(
            "200 OK",
            &[("Content-Type", "application/json")],
            b"{}",
            Duration::ZERO,
        );
        let Ok(server) = server else { return };
        let Some(client) = build_raw_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
            return;
        };
        let Some(policy) = policy(2) else { return };
        let mut body = [0xa5_u8; 16];
        let mut header_storage = [0xa5_u8; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
        assert!(
            client
                .execute_checked(
                    TransportRequest::new(Method::Get, target),
                    policy,
                    response.writer(),
                )
                .await
                .is_ok()
        );
        assert!(
            response
                .with_response(|value| value.body() == b"{}")
                .unwrap_or(false)
        );
    });
}

#[test]
fn raw_async_cancellation_clears_partial_body_and_headers() {
    run_async_test(async {
        let first = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
Content-Length: 6\r\nConnection: close\r\n\r\nsec";
        let server = spawn_raw_split(first, b"ret", Duration::from_millis(1_000));
        let Ok(server) = server else { return };
        let Some(client) = build_raw_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
            return;
        };
        let Some(policy) = policy(2) else { return };
        let mut body = [0xa5_u8; 16];
        let mut header_storage = [0xa5_u8; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
        let cancelled = tokio::time::timeout(
            Duration::from_millis(100),
            client.execute_checked(
                TransportRequest::new(Method::Get, target),
                policy,
                response.writer(),
            ),
        )
        .await;
        assert!(cancelled.is_err());
        assert!(response.writer().headers().is_empty());
        let mut attempt = response
            .writer()
            .begin_attempt()
            .unwrap_or_else(|_| unreachable!());
        assert!(
            attempt
                .body_mut()
                .is_ok_and(|output| output.iter().all(|byte| *byte == 0))
        );
    });
}

#[test]
fn raw_async_enforces_the_informational_response_limit() {
    run_async_test(async {
        let wire = b"HTTP/1.1 103 Early Hints\r\nLink: </a>\r\n\r\n\
HTTP/1.1 103 Early Hints\r\nLink: </b>\r\n\r\n\
HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}";
        let server = spawn_raw_response(wire);
        let Ok(server) = server else { return };
        let Some(client) = build_raw_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
            return;
        };
        let Some(policy) = policy(1) else { return };
        let mut body = [0xa5_u8; 16];
        let mut header_storage = [0xa5_u8; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
        let failure = client
            .execute_checked(
                TransportRequest::new(Method::Get, target),
                policy,
                response.writer(),
            )
            .await;
        assert!(matches!(
            failure,
            Err(error)
                if error.phase() == DeliveryPhase::ResponseStarted
                    && error.error() == &RawHttpError::TooManyInformationalResponses
        ));
    });
}

#[test]
fn raw_async_rechecks_informational_rejection_after_final_response() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build();
    assert!(runtime.is_ok());
    let Ok(runtime) = runtime else { return };
    runtime.block_on(async {
        for _ in 0..32 {
            let wire = b"HTTP/1.1 103 Early Hints\r\nLink: </a>\r\n\r\n\
HTTP/1.1 103 Early Hints\r\nLink: </b>\r\n\r\n\
HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
Content-Length: 2\r\nConnection: close\r\n\r\n{}";
            let server = spawn_raw_response(wire);
            let server_error = server.as_ref().err();
            assert!(
                server_error.is_none(),
                "failed to create loopback server: {server_error:?}"
            );
            let Ok(server) = server else { return };
            let client = build_raw_loopback(&server.endpoint);
            assert!(client.is_some());
            let Some(client) = client else { return };
            let target = cloud_sdk::transport::RequestTarget::new("/servers");
            assert!(target.is_ok());
            let Ok(target) = target else { return };
            let Some(policy) = policy(1) else { return };
            let mut body = [0xa5_u8; 16];
            let mut header_storage = [0xa5_u8; 128];
            let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
            let result = client
                .execute_checked(
                    TransportRequest::new(Method::Get, target),
                    policy,
                    response.writer(),
                )
                .await;
            assert!(matches!(
                result,
                Err(error)
                    if error.phase() == DeliveryPhase::ResponseStarted
                        && error.error() == &RawHttpError::TooManyInformationalResponses
            ));
        }
    });
}

#[test]
fn raw_async_aborts_before_a_rejected_informational_stream_finishes() {
    run_async_test(async {
        let informational = b"HTTP/1.1 103 Early Hints\r\nLink: </a>\r\n\r\n\
HTTP/1.1 103 Early Hints\r\nLink: </b>\r\n\r\n";
        let final_response = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
Content-Length: 2\r\nConnection: close\r\n\r\n{}";
        let server = spawn_raw_split(informational, final_response, Duration::from_millis(1_500));
        let Ok(server) = server else { return };
        let Some(client) = build_raw_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
            return;
        };
        let Some(policy) = policy(1) else { return };
        let mut body = [0xa5_u8; 16];
        let mut header_storage = [0xa5_u8; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
        let result = tokio::time::timeout(
            Duration::from_millis(750),
            client.execute_checked(
                TransportRequest::new(Method::Get, target),
                policy,
                response.writer(),
            ),
        )
        .await;
        assert!(matches!(
            result,
            Ok(Err(error))
                if error.phase() == DeliveryPhase::ResponseStarted
                    && error.error() == &RawHttpError::TooManyInformationalResponses
        ));
    });
}

#[test]
fn raw_async_rejects_switching_protocols_and_observed_trailers() {
    for (wire, expected) in [
        (
            &b"HTTP/1.1 101 Switching Protocols\r\nConnection: upgrade\r\n\
Upgrade: websocket\r\n\r\n"[..],
            RawHttpError::SwitchingProtocols,
        ),
        (
            &b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
Transfer-Encoding: chunked\r\n\r\n2\r\n{}\r\n0\r\nX-Checksum: secret\r\n\r\n"[..],
            RawHttpError::ResponseTrailersRejected,
        ),
    ] {
        run_async_test(async {
            let Ok(server) = spawn_raw_response(wire) else {
                return;
            };
            let Some(client) = build_raw_loopback(&server.endpoint) else {
                return;
            };
            let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
                return;
            };
            let Some(policy) = policy(2) else { return };
            let mut body = [0xa5_u8; 16];
            let mut header_storage = [0xa5_u8; 128];
            let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
            assert!(matches!(
                client
                    .execute_checked(
                        TransportRequest::new(Method::Get, target),
                        policy,
                        response.writer(),
                    )
                    .await,
                Err(error) if error.error() == &expected
            ));
        });
    }
}

#[test]
fn raw_async_enforces_streamed_bytes_without_content_length() {
    run_async_test(async {
        let wire = b"HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\n\
Connection: close\r\n\r\n12345";
        let Ok(server) = spawn_raw_response(wire) else {
            return;
        };
        let Some(client) = build_raw_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
            return;
        };
        let Some(policy) = policy(2) else { return };
        let mut body = [0xa5_u8; 16];
        let mut header_storage = [0xa5_u8; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
        assert!(matches!(
            client
                .execute_checked(
                    TransportRequest::new(Method::Get, target),
                    policy,
                    response.writer(),
                )
                .await,
            Err(error)
                if error.phase() == DeliveryPhase::ResponseStarted
                    && error.error() == &RawHttpError::ResponseTooLarge
        ));
    });
}

#[test]
fn raw_async_rejects_truncated_declared_body() {
    run_async_test(async {
        let wire = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
Content-Length: 5\r\nConnection: close\r\n\r\n{}";
        let Ok(server) = spawn_raw_response(wire) else {
            return;
        };
        let Some(client) = build_raw_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
            return;
        };
        let Some(policy) = policy(2) else { return };
        let mut body = [0xa5_u8; 16];
        let mut header_storage = [0xa5_u8; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
        assert!(matches!(
            client
                .execute_checked(
                    TransportRequest::new(Method::Get, target),
                    policy,
                    response.writer(),
                )
                .await,
            Err(error)
                if error.phase() == DeliveryPhase::ResponseStarted
                    && error.error() == &RawHttpError::ResponseReadFailed
        ));
    });
}

#[test]
fn raw_async_applies_head_and_no_content_body_rules() {
    run_async_test(async {
        let head = b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
Content-Length: 2\r\nConnection: close\r\n\r\n{}";
        let Ok(server) = spawn_raw_response(head) else {
            return;
        };
        let Some(client) = build_raw_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
            return;
        };
        let Some(policy) = policy(2) else { return };
        let mut body = [0xa5_u8; 16];
        let mut header_storage = [0xa5_u8; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
        assert!(
            client
                .execute_checked(
                    TransportRequest::new(Method::Head, target),
                    policy,
                    response.writer(),
                )
                .await
                .is_ok()
        );
        assert!(
            response
                .with_response(|value| value.body().is_empty())
                .unwrap_or(false)
        );

        let no_content = b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\
Connection: close\r\n\r\n";
        let Ok(server) = spawn_raw_response(no_content) else {
            return;
        };
        let Some(client) = build_raw_loopback(&server.endpoint) else {
            return;
        };
        let mut body = [0xa5_u8; 16];
        let mut header_storage = [0xa5_u8; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
        assert!(matches!(
            client
                .execute_checked(
                    TransportRequest::new(Method::Get, target),
                    policy,
                    response.writer(),
                )
                .await,
            Err(error) if error.error() == &RawHttpError::InvalidNoBodyFraming
        ));
    });
}

#[test]
fn raw_async_client_is_clone_send_sync_and_concurrent() {
    fn assert_traits<T: Clone + Send + Sync>() {}
    assert_traits::<RawAsyncClient>();

    run_async_test(async {
        let Ok(server) = spawn_concurrent_pair("200 OK", b"{}") else {
            return;
        };
        let Some(client) = build_raw_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
            return;
        };
        let Ok(policy) = RawResponsePolicy::new(
            16,
            4,
            ResponseMediaPolicy::Optional(&[MediaType::JSON]),
            ResponseMediaPolicy::Optional(&[MediaType::JSON]),
            &[],
            2,
        ) else {
            return;
        };
        let first = client.clone();
        let second = client.clone();
        let mut first_body = [0xa5_u8; 16];
        let mut first_headers = [0xa5_u8; 128];
        let mut first_response = ResponseBuffer::new(&mut first_body, 16, &mut first_headers);
        let mut second_body = [0xa5_u8; 16];
        let mut second_headers = [0xa5_u8; 128];
        let mut second_response = ResponseBuffer::new(&mut second_body, 16, &mut second_headers);
        let first_future = first.execute_checked(
            TransportRequest::new(Method::Get, target),
            policy,
            first_response.writer(),
        );
        let second_future = second.execute_checked(
            TransportRequest::new(Method::Get, target),
            policy,
            second_response.writer(),
        );
        let mut first_future = core::pin::pin!(first_future);
        let mut second_future = core::pin::pin!(second_future);
        let mut first_done = false;
        let mut second_done = false;
        poll_fn(|context| {
            if !first_done
                && let core::task::Poll::Ready(result) =
                    Future::poll(first_future.as_mut(), context)
            {
                assert!(result.is_ok());
                first_done = true;
            }
            if !second_done
                && let core::task::Poll::Ready(result) =
                    Future::poll(second_future.as_mut(), context)
            {
                assert!(result.is_ok());
                second_done = true;
            }
            if first_done && second_done {
                core::task::Poll::Ready(())
            } else {
                core::task::Poll::Pending
            }
        })
        .await;
    });
}
