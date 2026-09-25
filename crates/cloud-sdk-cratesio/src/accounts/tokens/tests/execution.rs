use super::*;
#[cfg(feature = "std")]
use crate::std as test_std;
use crate::{
    endpoint::OfficialCratesIoEndpoint,
    wire::{IdentifyingUserAgent, TEST_GATE_LOCK, reset_test_gate},
};
use cloud_sdk::transport::{
    BoundTransport, BoundUserAgent, EndpointIdentity, EndpointIdentityError, ResponseWriter,
};
struct Executor;
impl BoundTransport for Executor {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(OfficialCratesIoEndpoint::production_api()
            .identity()
            .fixture("origin"))
    }
}
impl BoundUserAgent for Executor {
    fn configured_user_agent(&self) -> &[u8] {
        b"tests/1 (tests@example.org)"
    }
}
fn stage(writer: &mut ResponseWriter<'_>, status: u16, body: &[u8], media: Option<&[u8]>) {
    let mut attempt = writer.begin_attempt().fixture("attempt");
    attempt
        .body_mut()
        .fixture("storage")
        .get_mut(..body.len())
        .fixture("range")
        .copy_from_slice(body);
    if let Some(media) = media {
        attempt
            .headers_mut()
            .fixture("headers")
            .try_push("content-type", media, HeaderSensitivity::Public)
            .fixture("type");
    }
    attempt
        .commit(
            StatusCode::new(status).fixture("status"),
            body.len(),
            ResponseMetadata::EMPTY,
        )
        .fixture("commit");
}
#[test]
fn exact_three_exchanges_and_cleanup() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let token = token(CredentialOrigin::Production);
    let client = TokenClient::production(
        &Executor,
        IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("agent"),
        4096,
    )
    .fixture("client");
    for (permit, status, body, media) in [
        (
            TokenPermit::inspect(id(42), &token),
            200,
            RECORD.as_bytes(),
            Some(b"application/json".as_slice()),
        ),
        (
            TokenPermit::confirm_revoke(id(42), &token),
            200,
            b"{}",
            Some(b"application/json"),
        ),
        (TokenPermit::confirm_revoke_current(&token), 204, b"", None),
    ] {
        reset_test_gate();
        let operation = permit.operation();
        let mut secret = [0xa5; 1024];
        let mut bytes = [0xa5; 4096];
        let mut headers = [0xa5; 512];
        let mut calls = 0;
        let result = client.execute(
            permit,
            TokenBuffers {
                credential: &mut secret,
                response: &mut bytes,
                headers: &mut headers,
            },
            |_, material, request, policy, writer| {
                calls += 1;
                assert_eq!(request.method(), operation.method());
                assert_eq!(request.target(), material.target());
                assert!(material.authorization().is_some());
                assert!(request.body().is_empty());
                assert!(request.headers().get("authorization").is_none());
                assert_eq!(policy.max_body_bytes(), 4096);
                stage(writer, status, body, media);
                Ok::<_, ()>(())
            },
        );
        let value = result.fixture("execution");
        assert!(matches!(
            (operation, value),
            (TokenOperation::Inspect, TokenResponse::Metadata(_))
                | (TokenOperation::Revoke, TokenResponse::Revoked)
                | (TokenOperation::RevokeCurrent, TokenResponse::CurrentRevoked)
        ));
        assert_eq!(calls, 1);
        for buffer in [&secret[..], &bytes[..], &headers[..]] {
            assert!(buffer.iter().all(|b| *b == 0));
        }
    }
}
#[test]
fn server_errors_contradictions_and_ambiguous_failures_never_retry() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let token = token(CredentialOrigin::Production);
    let client = TokenClient::production(
        &Executor,
        IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("agent"),
        4096,
    )
    .fixture("client");
    for (status, body, media) in [
        (
            404,
            b"{\"errors\":[{\"detail\":\"stale ID\"}]}".as_slice(),
            Some(b"application/json".as_slice()),
        ),
        (
            403,
            b"{\"errors\":[{\"detail\":\"wrong token\"}]}",
            Some(b"application/json"),
        ),
        (200, b"{}", Some(b"application/json")),
        (204, b"{}", None),
        (204, b"", Some(b"text/html")),
        (205, b"", None),
    ] {
        reset_test_gate();
        let mut secret = [0xa5; 1024];
        let mut bytes = [0xa5; 4096];
        let mut headers = [0xa5; 512];
        let mut calls = 0;
        let result = client.execute(
            TokenPermit::confirm_revoke_current(&token),
            TokenBuffers {
                credential: &mut secret,
                response: &mut bytes,
                headers: &mut headers,
            },
            |_, _, _, _, writer| {
                calls += 1;
                stage(writer, status, body, media);
                Ok::<_, ()>(())
            },
        );
        assert!(result.is_err());
        assert_eq!(calls, 1);
        for buffer in [&secret[..], &bytes[..], &headers[..]] {
            assert!(buffer.iter().all(|b| *b == 0));
        }
    }
}
#[test]
fn wrong_origin_and_short_scratch_do_not_send() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let client = TokenClient::production(
        &Executor,
        IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("agent"),
        4096,
    )
    .fixture("client");
    for (origin, length) in [
        (CredentialOrigin::Staging, 1024),
        (CredentialOrigin::Production, 1),
    ] {
        reset_test_gate();
        let token = token(origin);
        let mut secret = vec![0xa5; length];
        let mut bytes = [0xa5; 4096];
        let mut headers = [0xa5; 512];
        assert!(
            client
                .execute(
                    TokenPermit::confirm_revoke_current(&token),
                    TokenBuffers {
                        credential: &mut secret,
                        response: &mut bytes,
                        headers: &mut headers
                    },
                    |_, _, _, _, _| -> Result<(), ()> { unreachable!("invalid authority sent") }
                )
                .is_err()
        );
        for buffer in [&secret[..], &bytes[..], &headers[..]] {
            assert!(buffer.iter().all(|b| *b == 0));
        }
    }
}
#[test]
fn failure_and_unwind_clear_storage() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let token = token(CredentialOrigin::Production);
    let client = TokenClient::production(
        &Executor,
        IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("agent"),
        4096,
    )
    .fixture("client");
    for panics in [false, true] {
        reset_test_gate();
        let mut secret = [0xa5; 1024];
        let mut bytes = [0xa5; 4096];
        let mut headers = [0xa5; 512];
        let result = test_std::panic::catch_unwind(test_std::panic::AssertUnwindSafe(|| {
            client.execute(
                TokenPermit::confirm_revoke_current(&token),
                TokenBuffers {
                    credential: &mut secret,
                    response: &mut bytes,
                    headers: &mut headers,
                },
                |_, _, _, _, _| -> Result<(), ()> {
                    assert!(!panics, "injected panic");
                    Err(())
                },
            )
        }));
        assert!(matches!(result, Err(_) | Ok(Err(_))));
        for buffer in [&secret[..], &bytes[..], &headers[..]] {
            assert!(buffer.iter().all(|b| *b == 0));
        }
    }
}

#[test]
fn self_revocation_retry_after_updates_shared_gate_without_replaying() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let token = token(CredentialOrigin::Production);
    let client = TokenClient::production(
        &Executor,
        IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("agent"),
        4096,
    )
    .fixture("client");
    for delay in [b"60".as_slice(), b"invalid", b"86401"] {
        reset_test_gate();
        let mut secret = [0xa5; 1024];
        let mut bytes = [0xa5; 4096];
        let mut headers = [0xa5; 512];
        let mut calls = 0;
        let result = client.execute(
            TokenPermit::confirm_revoke_current(&token),
            TokenBuffers {
                credential: &mut secret,
                response: &mut bytes,
                headers: &mut headers,
            },
            |_, _, _, _, writer| {
                calls += 1;
                let mut attempt = writer.begin_attempt().fixture("attempt");
                attempt
                    .headers_mut()
                    .fixture("headers")
                    .try_push("retry-after", delay, HeaderSensitivity::Public)
                    .fixture("delay");
                attempt
                    .commit(
                        StatusCode::new(204).fixture("status"),
                        0,
                        ResponseMetadata::EMPTY,
                    )
                    .fixture("commit");
                Ok::<_, ()>(())
            },
        );
        assert_eq!(calls, 1);
        assert_eq!(result.is_ok(), delay == b"60");
        for buffer in [&secret[..], &bytes[..], &headers[..]] {
            assert!(buffer.iter().all(|b| *b == 0));
        }
        if delay == b"60" {
            assert!(matches!(
                client.execute(
                    TokenPermit::inspect(id(42), &token),
                    TokenBuffers {
                        credential: &mut secret,
                        response: &mut bytes,
                        headers: &mut headers
                    },
                    |_, _, _, _, _| -> Result<(), ()> { unreachable!("deferred gate dispatched") }
                ),
                Err(crate::discovery::DiscoveryExecutionError::Schedule(_))
            ));
        }
    }
}

#[test]
fn policy_honoring_adapter_retains_encoding_for_all_token_operations() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let token = token(CredentialOrigin::Production);
    let client = TokenClient::production(
        &Executor,
        IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("agent"),
        4096,
    )
    .fixture("client");
    for operation in [
        TokenOperation::Inspect,
        TokenOperation::Revoke,
        TokenOperation::RevokeCurrent,
    ] {
        for encoding in [
            None,
            Some(b"identity".as_slice()),
            Some(b"Identity"),
            Some(b"gzip"),
            Some(b"br"),
            Some(b""),
        ] {
            reset_test_gate();
            let permit = match operation {
                TokenOperation::Inspect => TokenPermit::inspect(id(42), &token),
                TokenOperation::Revoke => TokenPermit::confirm_revoke(id(42), &token),
                TokenOperation::RevokeCurrent => TokenPermit::confirm_revoke_current(&token),
            };
            let (status, body) = match operation {
                TokenOperation::Inspect => (200, RECORD.as_bytes()),
                TokenOperation::Revoke => (200, b"{}".as_slice()),
                TokenOperation::RevokeCurrent => (204, b"".as_slice()),
            };
            let mut secret = [0xa5; 1024];
            let mut bytes = [0xa5; 4096];
            let mut headers = [0xa5; 512];
            let mut calls = 0;
            let result = client.execute(
                permit,
                TokenBuffers {
                    credential: &mut secret,
                    response: &mut bytes,
                    headers: &mut headers,
                },
                |_, _, _, policy, writer| {
                    calls += 1;
                    assert!(policy.admits_header("content-encoding"));
                    assert!(policy.admits_header("Content-Encoding"));
                    let mut attempt = writer.begin_attempt().fixture("attempt");
                    attempt
                        .body_mut()
                        .fixture("body")
                        .get_mut(..body.len())
                        .fixture("range")
                        .copy_from_slice(body);
                    if status == 200 {
                        attempt
                            .headers_mut()
                            .fixture("headers")
                            .try_push(
                                "content-type",
                                b"application/json",
                                HeaderSensitivity::Public,
                            )
                            .fixture("type");
                    }
                    match encoding {
                        // Model the adapter's admitted-header filtering, not direct injection.
                        Some(value) if policy.admits_header("Content-Encoding") => {
                            attempt
                                .headers_mut()
                                .fixture("headers")
                                .try_push("Content-Encoding", value, HeaderSensitivity::Public)
                                .fixture("encoding");
                        }
                        Some(_) => unreachable!("encoding header was not admitted"),
                        None => assert!(encoding.is_none()),
                    }
                    attempt
                        .commit(
                            StatusCode::new(status).fixture("status"),
                            body.len(),
                            ResponseMetadata::EMPTY,
                        )
                        .fixture("commit");
                    Ok::<_, ()>(())
                },
            );
            let accepted = encoding.is_none()
                || (status == 200 && encoding.is_some_and(|v| v.eq_ignore_ascii_case(b"identity")));
            if accepted {
                assert!(result.is_ok());
            } else {
                assert!(matches!(
                    result,
                    Err(crate::discovery::DiscoveryExecutionError::Wire(
                        crate::wire::CratesIoWireError::ContentType
                    ))
                ));
            }
            assert_eq!(calls, 1);
            for buffer in [&secret[..], &bytes[..], &headers[..]] {
                assert!(buffer.iter().all(|b| *b == 0));
            }
        }
    }
}
