use super::execution::{Executor, client, stage};
use super::*;
use crate::wire::{TEST_GATE_LOCK, reset_test_gate};
use cloud_sdk::transport::HeaderSensitivity;

#[test]
fn retry_after_blocks_another_permit_without_replay_and_clears_buffers() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    reset_test_gate();
    let token = token();
    let executor = Executor(false);
    let client = client(&executor);
    let mut secret = [0xa5; 1024];
    let mut body = [0xa5; 1024];
    let mut response = [0xa5; 4096];
    let mut headers = [0xa5; 512];
    let result = client.execute(
        PersonalRequest::resend_email(id(7)).confirm(&token),
        PersonalBuffers {
            credential: &mut secret,
            body: &mut body,
            response: &mut response,
            headers: &mut headers,
        },
        |_, _, _, _, writer| {
            let wire = br#"{"errors":[{"detail":"slow down"}]}"#;
            let mut attempt = writer.begin_attempt().fixture("attempt");
            attempt
                .body_mut()
                .fixture("body")
                .get_mut(..wire.len())
                .fixture("range")
                .copy_from_slice(wire);
            let output = attempt.headers_mut().fixture("headers");
            output
                .try_push(
                    "content-type",
                    b"application/json",
                    HeaderSensitivity::Public,
                )
                .fixture("media");
            output
                .try_push("retry-after", b"60", HeaderSensitivity::Public)
                .fixture("delay");
            attempt
                .commit(
                    cloud_sdk::transport::StatusCode::TOO_MANY_REQUESTS,
                    wire.len(),
                    cloud_sdk::transport::ResponseMetadata::EMPTY,
                )
                .fixture("commit");
            Ok::<_, ()>(())
        },
    );
    assert!(matches!(result, Err(PersonalExecutionError::Wire(_))));
    let blocked = client.execute(
        PersonalRequest::resend_email(id(8)).confirm(&token),
        PersonalBuffers {
            credential: &mut secret,
            body: &mut body,
            response: &mut response,
            headers: &mut headers,
        },
        |_, _, _, _, _| -> Result<(), ()> {
            unreachable!("deferred request dispatched");
        },
    );
    assert!(matches!(blocked, Err(PersonalExecutionError::Schedule(_))));
    for bytes in [&secret[..], &body[..], &response[..], &headers[..]] {
        assert!(bytes.iter().all(|b| *b == 0));
    }
    reset_test_gate();
}

#[test]
fn dispatch_keeps_each_user_crate_and_verb_bound_to_its_permit() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let token = token();
    let executor = Executor(false);
    let client = client(&executor);
    let cases = [
        (
            PersonalRequest::follow(CrateName::new("serde").fixture("crate")),
            "/api/v1/crates/serde/follow",
            cloud_sdk::Method::Put,
        ),
        (
            PersonalRequest::unfollow(CrateName::new("tokio").fixture("crate")),
            "/api/v1/crates/tokio/follow",
            cloud_sdk::Method::Delete,
        ),
        (
            PersonalRequest::resend_email(id(31)),
            "/api/v1/users/31/resend",
            cloud_sdk::Method::Put,
        ),
        (
            PersonalRequest::publish_notifications(id(32), true),
            "/api/v1/users/32",
            cloud_sdk::Method::Put,
        ),
    ];
    for (request, path, method) in cases {
        reset_test_gate();
        let mut secret = [0; 1024];
        let mut body = [0; 1024];
        let mut response = [0; 4096];
        let mut headers = [0; 512];
        client
            .execute(
                request.confirm(&token),
                PersonalBuffers {
                    credential: &mut secret,
                    body: &mut body,
                    response: &mut response,
                    headers: &mut headers,
                },
                |_, material, request, _, writer| {
                    assert_eq!(request.method(), method);
                    assert_eq!(request.target().path().as_str(), path);
                    assert!(material.authorization().is_some());
                    assert!(request.headers().get("cookie").is_none());
                    stage(writer, 200, b"application/json", br#"{"ok":true}"#);
                    Ok::<_, ()>(())
                },
            )
            .fixture("request");
    }
    reset_test_gate();
}
