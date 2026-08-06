use std::string::String;
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::authentication::{CredentialLifetime, CredentialTimestamp};
use cloud_sdk::transport::{
    BoundTransport, RequestTarget, ResponseStorageSanitizer, TransportRequest,
};
use cloud_sdk_sanitization::SecretBuffer;

use super::{BearerToken, build_expiring_loopback, build_loopback};
use crate::test_server::{spawn_concurrent_pair, spawn_sequence_with_first_delay};

#[test]
fn prepared_cleanup_contract_clears_the_complete_caller_buffer() {
    let client = build_loopback("http://127.0.0.1:1/v1");
    assert!(client.is_some());
    let Some(client) = client else {
        unreachable!("security fixture construction failed")
    };
    let mut output = [0xA5_u8; 64];
    client.sanitize_response_storage(&mut output);
    assert_eq!(output, [0_u8; 64]);
}

#[test]
fn blocking_client_is_clone_send_sync_and_endpoint_bound() {
    fn assert_shared<T: Clone + Send + Sync>() {}
    assert_shared::<super::super::BlockingClient>();

    let Some(client) = build_loopback("http://127.0.0.1:9/v1") else {
        unreachable!("security fixture construction failed");
    };
    let identity = client.endpoint_identity();
    assert!(identity.is_ok());
    let Ok(identity) = identity else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(identity.host(), "127.0.0.1");
    assert_eq!(identity.effective_port(), 9);
    assert_eq!(identity.base_path(), "/v1");

    let before = client.endpoint_identity();
    let replacement = BearerToken::new("replacement-token");
    assert!(replacement.is_ok());
    let Ok(replacement) = replacement else {
        unreachable!("security fixture construction failed");
    };
    assert!(client.rotate_bearer_token(replacement).is_ok());

    assert_eq!(client.endpoint_identity(), before);
}

#[test]
fn blocking_shared_handle_supports_overlapping_caller_bounded_requests() {
    let server = spawn_concurrent_pair("200 OK", b"ok");
    let Ok(server) = server else {
        unreachable!("security fixture construction failed")
    };
    let Some(client) = build_loopback(&server.endpoint) else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = RequestTarget::new("/concurrent") else {
        unreachable!("security fixture construction failed");
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
    let Ok(server) = server else {
        unreachable!("security fixture construction failed")
    };
    let Some(client) = build_loopback(&server.endpoint) else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = RequestTarget::new("/rotation") else {
        unreachable!("security fixture construction failed");
    };

    std::thread::scope(|scope| {
        let first = scope.spawn(|| send_once(&client, target));
        let first_request = server.request.recv_timeout(Duration::from_secs(2));
        assert!(first_request.is_ok());
        let Ok(first_request) = first_request else {
            unreachable!("security fixture construction failed");
        };
        assert!(has_authorization(&first_request.bytes, "test-token"));

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
    let Ok(second_request) = second_request else {
        unreachable!("security fixture construction failed");
    };
    assert!(has_authorization(&second_request.bytes, "rotated-token"));
    assert!(!has_authorization(&second_request.bytes, "test-token"));
}

#[test]
fn blocking_guarded_rotation_clears_source_and_is_shared_by_clones() {
    let Some(client) = build_loopback("http://127.0.0.1:9/v1") else {
        unreachable!("security fixture construction failed");
    };
    let clone = client.clone();
    let mut source = *b"guarded-token";
    let result = clone.rotate_bearer_token_from_secret_buffer(SecretBuffer::new(&mut source));
    assert!(result.is_ok());
    assert_eq!(source, [0; 13]);
    assert_eq!(client.endpoint_identity(), clone.endpoint_identity());
}

#[test]
fn blocking_expiring_refresh_updates_token_and_lifetime_for_every_clone() {
    let initial =
        CredentialLifetime::from_expires_in(CredentialTimestamp::from_seconds(1_000), 3_599, 300)
            .unwrap_or_else(|_| unreachable!("security fixture construction failed"));
    let Some(client) = build_expiring_loopback("http://127.0.0.1:9/v1", initial) else {
        unreachable!("security fixture construction failed");
    };
    let clone = client.clone();
    let snapshot = clone
        .credential_snapshot()
        .unwrap_or_else(|_| unreachable!("security fixture construction failed"));
    let handoff = snapshot
        .refresh_handoff_at(CredentialTimestamp::from_seconds(4_300))
        .unwrap_or_else(|_| unreachable!("security fixture construction failed"));
    let replacement_lifetime =
        CredentialLifetime::from_expires_in(CredentialTimestamp::from_seconds(4_300), 3_599, 300)
            .unwrap_or_else(|_| unreachable!("security fixture construction failed"));
    let mut replacement = *b"expiring-token";

    assert!(
        clone
            .refresh_bearer_token_from_mut_bytes_with_lifetime(
                handoff,
                &mut replacement,
                replacement_lifetime,
            )
            .is_ok()
    );
    assert_eq!(replacement, [0; 14]);
    assert_eq!(
        client
            .credential_snapshot()
            .map(|snapshot| snapshot.lifetime()),
        Ok(Some(replacement_lifetime))
    );
}

fn send_once(client: &super::super::BlockingClient, target: RequestTarget<'_>) -> bool {
    let mut output = [0xa5_u8; 8];
    let response = super::send_test(
        client,
        TransportRequest::new(Method::Get, target),
        &mut output,
    );
    response.is_ok_and(|response| response.status().is_success() && response.body() == b"ok")
}

fn has_authorization(request: &[u8], token: &str) -> bool {
    let wire = String::from_utf8_lossy(request).to_ascii_lowercase();
    wire.contains(&std::format!("authorization: bearer {token}\r\n"))
}
