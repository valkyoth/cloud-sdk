use core::future::{Future, poll_fn};
use core::task::Poll;
use std::string::String;
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{AsyncTransport, BoundTransport, RequestTarget, TransportRequest};

use super::{BearerToken, build_loopback, run_async_test};
use crate::test_server::{spawn_concurrent_pair, spawn_sequence_with_first_delay};

#[test]
fn async_client_is_clone_send_sync_and_endpoint_bound() {
    fn assert_shared<T: Clone + Send + Sync>() {}
    assert_shared::<super::super::AsyncClient>();

    let Some(client) = build_loopback("http://127.0.0.1:9/v1") else {
        return;
    };
    let before = client.endpoint_identity();
    assert!(before.is_ok());
    let replacement = BearerToken::new("replacement-token");
    assert!(replacement.is_ok());
    if let Ok(replacement) = replacement {
        assert!(client.rotate_bearer_token(replacement).is_ok());
    }
    assert_eq!(client.endpoint_identity(), before);
}

#[test]
fn async_shared_handle_supports_overlapping_caller_bounded_requests() {
    run_async_test(async {
        let server = spawn_concurrent_pair("200 OK", b"ok");
        let Ok(server) = server else { return };
        let Some(client) = build_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = RequestTarget::new("/concurrent") else {
            return;
        };

        let results = join_two(send_once(&client, target), send_once(&client, target)).await;
        assert_eq!(results, (true, true));

        assert!(server.request.recv_timeout(Duration::from_secs(2)).is_ok());
        assert!(server.request.recv_timeout(Duration::from_secs(2)).is_ok());
    });
}

#[test]
fn async_rotation_from_mutable_bytes_is_visible_to_clones() {
    run_async_test(async {
        let server = spawn_concurrent_pair("200 OK", b"ok");
        let Ok(server) = server else { return };
        let Some(client) = build_loopback(&server.endpoint) else {
            return;
        };
        let clone = client.clone();
        let mut replacement = *b"rotated-token";
        assert!(
            clone
                .rotate_bearer_token_from_mut_bytes(&mut replacement)
                .is_ok()
        );
        assert_eq!(replacement, [0; 13]);
        let Ok(target) = RequestTarget::new("/rotation") else {
            return;
        };

        let results = join_two(send_once(&client, target), send_once(&clone, target)).await;
        assert_eq!(results, (true, true));
    });
}

#[test]
fn async_rotation_does_not_change_an_in_flight_token_snapshot() {
    run_async_test(async {
        let server = spawn_sequence_with_first_delay("200 OK", b"ok", Duration::from_millis(150));
        let Ok(server) = server else { return };
        let Some(client) = build_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = RequestTarget::new("/rotation") else {
            return;
        };
        let first = send_once(&client, target);
        let mut first = core::pin::pin!(first);

        let first_request = loop {
            match server.request.try_recv() {
                Ok(request) => break request,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => return,
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    let state = poll_fn(|context| Poll::Ready(first.as_mut().poll(context))).await;
                    assert!(matches!(state, Poll::Pending));
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            }
        };
        assert!(has_authorization(&first_request.bytes, "test-token"));

        let mut replacement = *b"rotated-token";
        assert!(
            client
                .rotate_bearer_token_from_mut_bytes(&mut replacement)
                .is_ok()
        );
        assert_eq!(replacement, [0; 13]);
        assert!(first.as_mut().await);

        assert!(send_once(&client, target).await);
        let second_request = server.request.recv_timeout(Duration::from_secs(2));
        assert!(second_request.is_ok());
        if let Ok(second_request) = second_request {
            assert!(has_authorization(&second_request.bytes, "rotated-token"));
            assert!(!has_authorization(&second_request.bytes, "test-token"));
        }
    });
}

async fn send_once(client: &super::super::AsyncClient, target: RequestTarget<'_>) -> bool {
    let mut output = [0xa5_u8; 8];
    let response = AsyncTransport::send(
        client,
        TransportRequest::new(Method::Get, target),
        &mut output,
    )
    .await;
    response.is_ok_and(|response| response.status().is_success() && response.body() == b"ok")
}

async fn join_two(
    first: impl Future<Output = bool>,
    second: impl Future<Output = bool>,
) -> (bool, bool) {
    let mut first = core::pin::pin!(first);
    let mut second = core::pin::pin!(second);
    let mut first_result = None;
    let mut second_result = None;
    poll_fn(|context| {
        if first_result.is_none()
            && let Poll::Ready(value) = first.as_mut().poll(context)
        {
            first_result = Some(value);
        }
        if second_result.is_none()
            && let Poll::Ready(value) = second.as_mut().poll(context)
        {
            second_result = Some(value);
        }
        match (first_result, second_result) {
            (Some(first), Some(second)) => Poll::Ready((first, second)),
            _ => Poll::Pending,
        }
    })
    .await
}

fn has_authorization(request: &[u8], token: &str) -> bool {
    let wire = String::from_utf8_lossy(request).to_ascii_lowercase();
    wire.contains(&std::format!("authorization: bearer {token}\r\n"))
}
