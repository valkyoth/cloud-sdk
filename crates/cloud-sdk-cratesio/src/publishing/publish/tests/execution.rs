use super::*;
use crate::{
    credentials::{Api, Credential, CredentialKind, CredentialOrigin, TrustedPublishing},
    endpoint::OfficialCratesIoEndpoint,
    wire::{IdentifyingUserAgent, TEST_GATE_LOCK, reset_test_gate},
};
use cloud_sdk::{
    Method,
    transport::{
        BoundTransport, BoundUserAgent, EndpointIdentity, EndpointIdentityError, HeaderSensitivity,
        ResponseMetadata, StatusCode,
    },
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
fn token<K: CredentialKind>(origin: CredentialOrigin) -> Credential<K> {
    use core::sync::atomic::{AtomicU8, Ordering};
    static NEXT: AtomicU8 = AtomicU8::new(0);
    let mut bytes = [b'a'.saturating_add(NEXT.fetch_add(1, Ordering::Relaxed) % 26); 32];
    Credential::from_mut_bytes(origin, &mut bytes).fixture("token")
}
fn client() -> PublishClient<'static, Executor> {
    PublishClient::production(
        &Executor,
        IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("agent"),
        8192,
    )
    .fixture("client")
}
#[test]
fn both_credentials_publish_once_and_reject_skipped_incomplete_and_ambiguous_uploads() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let api = token::<Api>(CredentialOrigin::Production);
    let trusted = token::<TrustedPublishing>(CredentialOrigin::Production);
    let client = client();
    for temporary in [false, true] {
        for scenario in 0..10 {
            reset_test_gate();
            let permit = if temporary {
                request(5).confirm_trusted(&trusted)
            } else {
                request(5).confirm_api(&api)
            };
            let mut secret = [0xa5; 1024];
            let mut response = [0xa5; 8192];
            let mut headers = [0xa5; 512];
            let mut source = SlicePackage::new(b"crate");
            let mut scratch = [0xa5; 32];
            let mut calls = 0;
            let result = client.execute(
                permit,
                &mut source,
                PublishBuffers {
                    credential: &mut secret,
                    response: &mut response,
                    headers: &mut headers,
                },
                |_, material, req, upload, policy, writer| {
                    calls += 1;
                    assert_eq!(req.method(), Method::Put);
                    assert_eq!(req.target().as_str(), "/api/v1/crates/new");
                    assert!(req.body().is_empty());
                    assert_eq!(material.method(), req.method());
                    assert_eq!(material.target(), req.target());
                    assert_eq!(
                        material
                            .authorization()
                            .fixture("auth")
                            .as_str()
                            .starts_with("Bearer "),
                        temporary
                    );
                    assert!(req.headers().get("content-type").is_some());
                    if scenario != 1 {
                        let mut sink = Sink {
                            fail: scenario == 2,
                            ..Sink::default()
                        };
                        let sent = upload.transfer_to(&mut sink, &mut scratch);
                        if scenario == 2 {
                            assert!(sent.is_err());
                            assert!(sink.aborted);
                        } else {
                            sent.fixture("upload");
                        }
                    }
                    if scenario == 3 {
                        return Err(());
                    } // timeout after all request bytes
                    let mut value: serde_json::Value =
                        serde_json::from_str(include_str!("../success.json")).fixture("fixture");
                    if scenario == 4 {
                        value
                            .as_object_mut()
                            .fixture("object")
                            .insert("errors".into(), serde_json::json!([{"detail":"rejected"}]));
                    }
                    if scenario == 5 {
                        *value.pointer_mut("/crate/name").fixture("name") =
                            serde_json::json!("wrong");
                    }
                    if scenario == 6 {
                        *value.pointer_mut("/warnings/other").fixture("warnings") =
                            serde_json::json!(["x".repeat(4097)]);
                    }
                    let bytes = serde_json::to_vec(&value).fixture("JSON");
                    let mut attempt = writer.begin_attempt().fixture("attempt");
                    attempt
                        .body_mut()
                        .fixture("body")
                        .get_mut(..bytes.len())
                        .fixture("range")
                        .copy_from_slice(&bytes);
                    for (name, value) in [
                        ("content-type", b"application/json".as_slice()),
                        (
                            "content-encoding",
                            if scenario == 7 {
                                b"gzip".as_slice()
                            } else {
                                b"identity"
                            },
                        ),
                    ] {
                        assert!(policy.admits_header(name));
                        attempt
                            .headers_mut()
                            .fixture("headers")
                            .try_push(name, value, HeaderSensitivity::Public)
                            .fixture("header");
                    }
                    if scenario != 8 {
                        attempt
                            .commit(
                                StatusCode::new(if scenario == 9 { 201 } else { 200 })
                                    .fixture("status"),
                                bytes.len(),
                                ResponseMetadata::EMPTY,
                            )
                            .fixture("commit");
                    }
                    Ok(())
                },
            );
            assert_eq!(calls, 1);
            assert_eq!(result.is_ok(), scenario == 0, "scenario {scenario}");
            if scenario == 0 {
                let reply = result.fixture("publication acknowledgement");
                assert_eq!(reply.krate().name, "serde");
                for field in ["invalid_categories", "invalid_badges", "other"] {
                    reply
                        .warnings()
                        .required(field)
                        .fixture("warning field")
                        .array()
                        .fixture("warning array");
                }
                assert!(!format!("{reply:?}").contains("serde"));
            }
            for b in [&secret[..], &response[..], &headers[..]] {
                assert!(b.iter().all(|v| *v == 0));
            }
            if scenario != 1 {
                assert_eq!(scratch, [0; 32]);
            }
        }
    }
    reset_test_gate();
}
#[test]
fn wrong_origin_and_small_credential_buffer_fail_before_dispatch() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    for staging in [true, false] {
        reset_test_gate();
        let token = token::<Api>(if staging {
            CredentialOrigin::Staging
        } else {
            CredentialOrigin::Production
        });
        let mut credential = [0xa5; 1];
        let mut response = [0xa5; 8192];
        let mut headers = [0xa5; 512];
        let mut source = SlicePackage::new(b"crate");
        let result = client().execute(
            request(5).confirm_api(&token),
            &mut source,
            PublishBuffers {
                credential: &mut credential,
                response: &mut response,
                headers: &mut headers,
            },
            |_, _, _, _, _, _| -> Result<(), ()> { unreachable!("invalid publish dispatched") },
        );
        assert!(result.is_err());
        for b in [&credential[..], &response[..], &headers[..]] {
            assert!(b.iter().all(|v| *v == 0));
        }
    }
    reset_test_gate();
}
