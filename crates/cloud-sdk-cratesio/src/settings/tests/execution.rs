use super::*;
#[cfg(feature = "std")]
use crate::std as test_std;
use crate::{
    endpoint::OfficialCratesIoEndpoint,
    wire::{IdentifyingUserAgent, TEST_GATE_LOCK, reset_test_gate},
};
use cloud_sdk::transport::{
    BoundTransport, BoundUserAgent, EndpointIdentity, EndpointIdentityError,
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
fn client() -> SettingsClient<'static, Executor> {
    SettingsClient::production(
        &Executor,
        IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("agent"),
        16_384,
    )
    .fixture("client")
}
#[test]
fn exact_exchanges_error_classification_cleanup_and_no_retry() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let token = token(CredentialOrigin::Production);
    let client = client();
    for is_version in [false, true] {
        for status in [200, 201, 403, 429, 500] {
            reset_test_gate();
            let request = if is_version {
                SettingsRequest::version(name(), version(), Some(false), YankMessage::Clear)
                    .fixture("request")
            } else {
                SettingsRequest::trustpub_only(name(), true)
            };
            let mut value = if is_version {
                version_response()
            } else {
                crate_response()
            };
            if is_version {
                set(
                    &mut value,
                    "version",
                    "yank_message",
                    serde_json::Value::Null,
                );
            } else {
                set(&mut value, "crate", "trustpub_only", true.into());
            }
            if status >= 400 {
                value = serde_json::json!({"errors":[{"detail":"denied"}]});
            }
            let wire = serde_json::to_vec(&value).fixture("wire");
            let mut credential = [0xa5; 1024];
            let mut body = [0xa5; 1024];
            let mut response = [0xa5; 16_384];
            let mut headers = [0xa5; 1024];
            let mut calls = 0;
            let result = client.execute(
                request.confirm(&token),
                SettingsBuffers {
                    credential: &mut credential,
                    body: &mut body,
                    response: &mut response,
                    headers: &mut headers,
                },
                |_, material, request, policy, writer| {
                    calls += 1;
                    assert_eq!(material.method(), cloud_sdk::Method::Patch);
                    assert_eq!(request.method(), material.method());
                    assert_eq!(
                        request.target().as_str(),
                        if is_version {
                            "/api/v1/crates/serde/1.0.0"
                        } else {
                            "/api/v1/crates/serde"
                        }
                    );
                    assert!(request.headers().get("content-type").is_some());
                    let json: serde_json::Value =
                        serde_json::from_slice(request.body()).fixture("payload");
                    assert_eq!(
                        json,
                        if is_version {
                            serde_json::json!({"version":{"yanked":false,"yank_message":null}})
                        } else {
                            serde_json::json!({"crate":{"trustpub_only":true}})
                        }
                    );
                    let mut attempt = writer.begin_attempt().fixture("attempt");
                    attempt
                        .body_mut()
                        .fixture("body")
                        .get_mut(..wire.len())
                        .fixture("range")
                        .copy_from_slice(&wire);
                    for (name, value) in [
                        ("content-type", b"application/json".as_slice()),
                        ("content-encoding", b"identity"),
                    ] {
                        assert!(policy.admits_header(name));
                        if policy.admits_header(name) {
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
                            wire.len(),
                            ResponseMetadata::EMPTY,
                        )
                        .fixture("commit");
                    Ok::<_, ()>(())
                },
            );
            assert_eq!(calls, 1);
            match status {
                200 => assert!(result.is_ok()),
                201 => assert!(matches!(
                    result,
                    Err(SettingsExecutionError::Wire(
                        crate::wire::CratesIoWireError::UnexpectedStatus
                    ))
                )),
                _ => assert!(matches!(
                    result,
                    Err(SettingsExecutionError::Wire(
                        crate::wire::CratesIoWireError::Provider(_)
                    ))
                )),
            }
            for buffer in [&credential[..], &body[..], &response[..], &headers[..]] {
                assert!(buffer.iter().all(|v| *v == 0));
            }
        }
    }
}
#[test]
fn wrong_origin_capacity_and_transport_failure_clear_every_buffer() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let client = client();
    for scenario in 0..4 {
        reset_test_gate();
        let token = token(if scenario == 0 {
            CredentialOrigin::Staging
        } else {
            CredentialOrigin::Production
        });
        let mut credential = vec![0xa5; if scenario == 1 { 1 } else { 1024 }];
        let mut body = vec![0xa5; if scenario == 2 { 1 } else { 1024 }];
        let mut response = [0xa5; 4096];
        let mut headers = [0xa5; 512];
        let mut calls = 0;
        let result = client.execute(
            SettingsRequest::trustpub_only(name(), true).confirm(&token),
            SettingsBuffers {
                credential: &mut credential,
                body: &mut body,
                response: &mut response,
                headers: &mut headers,
            },
            |_, _, _, _, _| {
                calls += 1;
                Err::<(), _>(())
            },
        );
        assert!(result.is_err());
        assert_eq!(calls, usize::from(scenario == 3));
        for buffer in [&credential[..], &body[..], &response[..], &headers[..]] {
            assert!(buffer.iter().all(|v| *v == 0));
        }
    }
}

#[test]
fn rejected_media_uncommitted_partial_and_panic_paths_clear_scratch() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let token = token(CredentialOrigin::Production);
    let client = client();
    for scenario in 0..6 {
        reset_test_gate();
        let mut credential = [0xa5; 1024];
        let mut body = [0xa5; 1024];
        let mut response = [0xa5; 16_384];
        let mut headers = [0xa5; 1024];
        let mut calls = 0;
        let mut value = crate_response();
        set(&mut value, "crate", "trustpub_only", true.into());
        let wire = serde_json::to_vec(&value).fixture("wire");
        let outcome = test_std::panic::catch_unwind(test_std::panic::AssertUnwindSafe(|| {
            client.execute(
                SettingsRequest::trustpub_only(name(), true).confirm(&token),
                SettingsBuffers {
                    credential: &mut credential,
                    body: &mut body,
                    response: &mut response,
                    headers: &mut headers,
                },
                |_, _, _, policy, writer| {
                    calls += 1;
                    let mut attempt = writer.begin_attempt().fixture("attempt");
                    attempt
                        .body_mut()
                        .fixture("body")
                        .get_mut(..wire.len())
                        .fixture("range")
                        .copy_from_slice(&wire);
                    if scenario == 4 {
                        return Err(());
                    }
                    if scenario == 5 {
                        unreachable!("injected adapter panic");
                    }
                    if scenario != 0 && policy.admits_header("content-type") {
                        let media = if scenario == 1 {
                            b"text/plain".as_slice()
                        } else {
                            b"application/json"
                        };
                        attempt
                            .headers_mut()
                            .fixture("headers")
                            .try_push("content-type", media, HeaderSensitivity::Public)
                            .fixture("type");
                    }
                    if scenario == 2 && policy.admits_header("content-encoding") {
                        attempt
                            .headers_mut()
                            .fixture("headers")
                            .try_push("content-encoding", b"gzip", HeaderSensitivity::Public)
                            .fixture("encoding");
                    }
                    if scenario != 3 {
                        attempt
                            .commit(StatusCode::OK, wire.len(), ResponseMetadata::EMPTY)
                            .fixture("commit");
                    }
                    Ok(())
                },
            )
        }));
        if scenario == 5 {
            assert!(outcome.is_err());
        } else {
            assert!(outcome.fixture("no panic").is_err());
        }
        assert_eq!(calls, 1);
        for buffer in [&credential[..], &body[..], &response[..], &headers[..]] {
            assert!(buffer.iter().all(|v| *v == 0));
        }
    }
    reset_test_gate();
}
