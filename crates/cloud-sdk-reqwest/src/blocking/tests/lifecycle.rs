use std::string::String;
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{BlockingTransport, BoundTransport, RequestTarget, TransportRequest};
use cloud_sdk_sanitization::SecretBuffer;

use super::{BearerToken, build_loopback};
use crate::test_server::{spawn_concurrent_pair, spawn_sequence_with_first_delay};

#[test]
fn blocking_client_is_clone_send_sync_and_endpoint_bound() {
    fn assert_shared<T: Clone + Send + Sync>() {}
    assert_shared::<super::super::BlockingClient>();

    let Some(client) = build_loopback("http://127.0.0.1:9/v1") else {
        return;
    };
    let identity = client.endpoint_identity();
    assert!(identity.is_ok());
    if let Ok(identity) = identity {
        assert_eq!(identity.host(), "127.0.0.1");
        assert_eq!(identity.effective_port(), 9);
        assert_eq!(identity.base_path(), "/v1");
    }
    let before = client.endpoint_identity();
    let replacement = BearerToken::new("replacement-token");
    assert!(replacement.is_ok());
    if let Ok(replacement) = replacement {
        assert!(client.rotate_bearer_token(replacement).is_ok());
    }
    assert_eq!(client.endpoint_identity(), before);
}

#[test]
fn blocking_shared_handle_supports_overlapping_caller_bounded_requests() {
    let server = spawn_concurrent_pair("200 OK", b"ok");
    let Ok(server) = server else { return };
    let Some(client) = build_loopback(&server.endpoint) else {
        return;
    };
    let Ok(target) = RequestTarget::new("/concurrent") else {
        return;
    };

    std::thread::scope(|scope| {
        let first = scope.spawn(|| send_once(&client, target));
        let second = scope.spawn(|| send_once(&client, target));
        assert!(matches!(first.join(), Ok(true)));
        assert!(matches!(second.join(), Ok(true)));
    });

    assert!(server.request.recv_timeout(Duration::from_secs(2)).is_ok());
    assert!(server.request.recv_timeout(Duration::from_secs(2)).is_ok());
}

#[test]
fn blocking_rotation_keeps_in_flight_snapshot_and_changes_new_requests() {
    let server = spawn_sequence_with_first_delay("200 OK", b"ok", Duration::from_millis(150));
    let Ok(server) = server else { return };
    let Some(client) = build_loopback(&server.endpoint) else {
        return;
    };
    let Ok(target) = RequestTarget::new("/rotation") else {
        return;
    };

    std::thread::scope(|scope| {
        let first = scope.spawn(|| send_once(&client, target));
        let first_request = server.request.recv_timeout(Duration::from_secs(2));
        assert!(first_request.is_ok());
        if let Ok(first_request) = first_request {
            assert!(has_authorization(&first_request.bytes, "test-token"));
        }

        let mut replacement = *b"rotated-token";
        assert!(
            client
                .rotate_bearer_token_from_mut_bytes(&mut replacement)
                .is_ok()
        );
        assert_eq!(replacement, [0; 13]);
        assert!(matches!(first.join(), Ok(true)));
    });

    assert!(send_once(&client, target));
    let second_request = server.request.recv_timeout(Duration::from_secs(2));
    assert!(second_request.is_ok());
    if let Ok(second_request) = second_request {
        assert!(has_authorization(&second_request.bytes, "rotated-token"));
        assert!(!has_authorization(&second_request.bytes, "test-token"));
    }
}

#[test]
fn blocking_guarded_rotation_clears_source_and_is_shared_by_clones() {
    let Some(client) = build_loopback("http://127.0.0.1:9/v1") else {
        return;
    };
    let clone = client.clone();
    let mut source = *b"guarded-token";
    let result = clone.rotate_bearer_token_from_secret_buffer(SecretBuffer::new(&mut source));
    assert!(result.is_ok());
    assert_eq!(source, [0; 13]);
    assert_eq!(client.endpoint_identity(), clone.endpoint_identity());
}

fn send_once(client: &super::super::BlockingClient, target: RequestTarget<'_>) -> bool {
    let mut output = [0xa5_u8; 8];
    let response = client.send(TransportRequest::new(Method::Get, target), &mut output);
    response.is_ok_and(|response| response.status().is_success() && response.body() == b"ok")
}

fn has_authorization(request: &[u8], token: &str) -> bool {
    let wire = String::from_utf8_lossy(request).to_ascii_lowercase();
    wire.contains(&std::format!("authorization: bearer {token}\r\n"))
}
