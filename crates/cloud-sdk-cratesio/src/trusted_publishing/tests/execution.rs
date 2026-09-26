use super::*;
#[cfg(feature = "std")]
use crate::std as test_std;
use crate::{
    endpoint::OfficialCratesIoEndpoint,
    wire::{IdentifyingUserAgent, TEST_GATE_LOCK, reset_test_gate},
};
use alloc::vec::Vec;
use cloud_sdk::transport::{
    BoundTransport, BoundUserAgent, EndpointIdentity, EndpointIdentityError, ResponseWriter,
};
pub(super) struct Executor;
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
pub(super) fn client() -> TrustedPublishingClient<'static, Executor> {
    TrustedPublishingClient::production(
        &Executor,
        IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("agent"),
        8192,
    )
    .fixture("client")
}
pub(super) fn now() -> u64 {
    test_std::time::SystemTime::now()
        .duration_since(test_std::time::UNIX_EPOCH)
        .fixture("time")
        .as_secs()
}
pub(super) fn stage(
    writer: &mut ResponseWriter<'_>,
    status: u16,
    body: &[u8],
    media: Option<&[u8]>,
    encoding: Option<&[u8]>,
    delay: Option<&[u8]>,
) {
    let mut attempt = writer.begin_attempt().fixture("attempt");
    attempt
        .body_mut()
        .fixture("storage")
        .get_mut(..body.len())
        .fixture("range")
        .copy_from_slice(body);
    for (name, value) in [
        ("content-type", media),
        ("content-encoding", encoding),
        ("retry-after", delay),
    ] {
        if let Some(value) = value {
            attempt
                .headers_mut()
                .fixture("headers")
                .try_push(name, value, HeaderSensitivity::Public)
                .fixture("header");
        }
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
fn all_eight_operations_send_once_and_erase_storage() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let token = api(CredentialOrigin::Production);
    let client = client();
    let time = now();
    let params = [Parameter::Crate(CrateName::new("regex").fixture("crate"))];
    let mut cases = Vec::new();
    for p in [Publisher::GitHub, Publisher::GitLab] {
        let q = Query::new(p.query(), &params).fixture("query");
        let provider = if p == Publisher::GitHub {
            "github"
        } else {
            "gitlab"
        };
        let mut list = fixture(&format!("list_trustpub_{provider}_configs"));
        *list.pointer_mut("/meta/next_page").fixture("next") = Value::Null;
        cases.push((
            TrustedPublishingPermit::list(p, q, &token).fixture("list"),
            200,
            serde_json::to_vec(&list).fixture("body"),
        ));
        cases.push((
            TrustedPublishingPermit::confirm_create(config(p), &token),
            200,
            serde_json::to_vec(&fixture(&format!("create_trustpub_{provider}_config")))
                .fixture("body"),
        ));
        cases.push((
            TrustedPublishingPermit::confirm_delete(p, id(42), &token),
            204,
            vec![],
        ));
    }
    cases.push((
        TrustedPublishingPermit::confirm_exchange(
            assertion(Publisher::GitHub, time, CredentialOrigin::Production),
            policy(Publisher::GitHub, time),
        ),
        200,
        serde_json::to_vec(&token_response()).fixture("token response"),
    ));
    cases.push((
        TrustedPublishingPermit::confirm_revoke(temporary(time)),
        204,
        vec![],
    ));
    for (permit, status, wire) in cases {
        reset_test_gate();
        let op = permit.operation();
        let mut calls = 0;
        let mut secret = [0xa5; 32768];
        let mut body = [0xa5; 4096];
        let mut bytes = [0xa5; 8192];
        let mut headers = [0xa5; 512];
        let value = client
            .execute(
                permit,
                TrustedPublishingBuffers {
                    credential: &mut secret,
                    body: &mut body,
                    response: &mut bytes,
                    headers: &mut headers,
                },
                |_, material, request, policy, writer| {
                    calls += 1;
                    assert_eq!(request.method(), op.method());
                    assert_eq!(request.target(), material.target());
                    assert_eq!(
                        material.authorization().is_none(),
                        op == TrustedPublishingOperation::Exchange
                    );
                    assert!(request.headers().get("authorization").is_none());
                    assert!(policy.admits_header("content-encoding"));
                    assert_eq!(
                        request.body().is_empty(),
                        !matches!(
                            op,
                            TrustedPublishingOperation::Create(_)
                                | TrustedPublishingOperation::Exchange
                        )
                    );
                    if op == TrustedPublishingOperation::Exchange {
                        assert_eq!(Some(request.body()), material.json_body());
                        assert!(
                            serde_json::from_slice::<Value>(request.body())
                                .fixture("body")
                                .get("jwt")
                                .fixture("jwt")
                                .is_string()
                        );
                    }
                    stage(
                        writer,
                        status,
                        &wire,
                        if status == 204 {
                            None
                        } else {
                            Some(b"application/json")
                        },
                        None,
                        None,
                    );
                    Ok::<_, ()>(())
                },
            )
            .fixture("execute");
        assert!(matches!(
            (op, value),
            (
                TrustedPublishingOperation::List(_),
                TrustedPublishingResponse::Configurations(_)
            ) | (
                TrustedPublishingOperation::Create(_),
                TrustedPublishingResponse::Created(_)
            ) | (
                TrustedPublishingOperation::Delete(_),
                TrustedPublishingResponse::Deleted
            ) | (
                TrustedPublishingOperation::Exchange,
                TrustedPublishingResponse::Exchanged(_)
            ) | (
                TrustedPublishingOperation::Revoke,
                TrustedPublishingResponse::Revoked
            )
        ));
        assert_eq!(calls, 1);
        for buffer in [&secret[..], &body[..], &bytes[..], &headers[..]] {
            assert!(buffer.iter().all(|b| *b == 0));
        }
    }
}
#[test]
fn invalid_origin_assertion_and_insufficient_scratch_never_send() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let client = client();
    let time = now();
    let staging = api(CredentialOrigin::Staging);
    let production = api(CredentialOrigin::Production);
    for (permit, capacity) in [
        (
            TrustedPublishingPermit::confirm_create(config(Publisher::GitHub), &staging),
            32768,
        ),
        (
            TrustedPublishingPermit::confirm_create(config(Publisher::GitHub), &production),
            1,
        ),
        (
            TrustedPublishingPermit::confirm_exchange(
                assertion(Publisher::GitLab, time, CredentialOrigin::Production),
                policy(Publisher::GitHub, time),
            ),
            32768,
        ),
    ] {
        reset_test_gate();
        let mut secret = vec![0xa5; capacity];
        let mut body = [0xa5; 4096];
        let mut bytes = [0xa5; 8192];
        let mut headers = [0xa5; 512];
        assert!(
            client
                .execute(
                    permit,
                    TrustedPublishingBuffers {
                        credential: &mut secret,
                        body: &mut body,
                        response: &mut bytes,
                        headers: &mut headers
                    },
                    |_, _, _, _, _| -> Result<(), ()> {
                        unreachable!("invalid preflight dispatched")
                    }
                )
                .is_err()
        );
        for buffer in [&secret[..], &body[..], &bytes[..], &headers[..]] {
            assert!(buffer.iter().all(|b| *b == 0));
        }
    }
}
#[test]
fn empty_acknowledgements_are_strict_and_errors_never_retry() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let client = client();
    for (status, wire, media, encoding) in [
        (
            200,
            b"{}".as_slice(),
            Some(b"application/json".as_slice()),
            None,
        ),
        (204, b"x", None, None),
        (204, b"", Some(b"application/json"), None),
        (204, b"", None, Some(b"identity".as_slice())),
        (
            403,
            br#"{"errors":[{"detail":"denied"}]}"#,
            Some(b"application/json"),
            None,
        ),
        (205, b"", None, None),
    ] {
        reset_test_gate();
        let mut secret = [0xa5; 1024];
        let mut body = [0xa5; 4096];
        let mut bytes = [0xa5; 8192];
        let mut headers = [0xa5; 512];
        let mut calls = 0;
        assert!(
            client
                .execute(
                    TrustedPublishingPermit::confirm_revoke(temporary(now())),
                    TrustedPublishingBuffers {
                        credential: &mut secret,
                        body: &mut body,
                        response: &mut bytes,
                        headers: &mut headers
                    },
                    |_, _, _, _, writer| {
                        calls += 1;
                        stage(writer, status, wire, media, encoding, None);
                        Ok::<_, ()>(())
                    }
                )
                .is_err()
        );
        assert_eq!(calls, 1);
        for buffer in [&secret[..], &body[..], &bytes[..], &headers[..]] {
            assert!(buffer.iter().all(|b| *b == 0));
        }
    }
}
#[test]
fn transport_failure_unwind_and_rate_deferral_preserve_cleanup() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let client = client();
    for unwind in [false, true] {
        reset_test_gate();
        let mut secret = [0xa5; 32768];
        let mut body = [0xa5; 4096];
        let mut bytes = [0xa5; 8192];
        let mut headers = [0xa5; 512];
        let time = now();
        let result = test_std::panic::catch_unwind(test_std::panic::AssertUnwindSafe(|| {
            client.execute(
                TrustedPublishingPermit::confirm_exchange(
                    assertion(Publisher::GitHub, time, CredentialOrigin::Production),
                    policy(Publisher::GitHub, time),
                ),
                TrustedPublishingBuffers {
                    credential: &mut secret,
                    body: &mut body,
                    response: &mut bytes,
                    headers: &mut headers,
                },
                |_, _, _, _, writer| {
                    stage(
                        writer,
                        200,
                        &serde_json::to_vec(&token_response()).fixture("body"),
                        Some(b"application/json"),
                        None,
                        None,
                    );
                    assert!(!unwind, "injected unwind");
                    Err::<(), _>(())
                },
            )
        }));
        assert!(matches!(result, Err(_) | Ok(Err(_))));
        for buffer in [&secret[..], &body[..], &bytes[..], &headers[..]] {
            assert!(buffer.iter().all(|b| *b == 0));
        }
    }
    reset_test_gate();
    let mut secret = [0; 1024];
    let mut body = [0; 4096];
    let mut bytes = [0; 8192];
    let mut headers = [0; 512];
    client
        .execute(
            TrustedPublishingPermit::confirm_revoke(temporary(now())),
            TrustedPublishingBuffers {
                credential: &mut secret,
                body: &mut body,
                response: &mut bytes,
                headers: &mut headers,
            },
            |_, _, _, _, writer| {
                stage(writer, 204, b"", None, None, Some(b"60"));
                Ok::<_, ()>(())
            },
        )
        .fixture("revocation");
    assert!(matches!(
        client.execute(
            TrustedPublishingPermit::confirm_revoke(temporary(now())),
            TrustedPublishingBuffers {
                credential: &mut secret,
                body: &mut body,
                response: &mut bytes,
                headers: &mut headers
            },
            |_, _, _, _, _| -> Result<(), ()> { unreachable!("deferred gate sent") }
        ),
        Err(crate::discovery::DiscoveryExecutionError::Schedule(_))
    ));
    reset_test_gate();
}
