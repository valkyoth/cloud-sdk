use super::*;
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
fn client() -> OwnerChangeClient<'static, Executor> {
    OwnerChangeClient::production(
        &Executor,
        IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("agent"),
        4096,
    )
    .fixture("client")
}
#[test]
fn exact_cargo_exchanges_and_failure_paths_never_retry() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let token = token(CredentialOrigin::Production);
    let client = client();
    let owners = [selector("crates.io:alice"), selector("github:org:team")];
    for remove in [false, true] {
        for scenario in 0..8 {
            reset_test_gate();
            let permit = if remove {
                OwnerChangeRequest::remove(name(), &owners)
                    .fixture("request")
                    .confirm_removal(&token)
            } else {
                OwnerChangeRequest::add(name(), &owners)
                    .fixture("request")
                    .confirm_add(&token)
            }
            .fixture("permit");
            let mut secret = [0xa5; 1024];
            let mut body = [0xa5; 1024];
            let mut response = [0xa5; 4096];
            let mut headers = [0xa5; 512];
            let mut calls = 0;
            let result = client.execute(
                permit,
                OwnerChangeBuffers {
                    credential: &mut secret,
                    body: &mut body,
                    response: &mut response,
                    headers: &mut headers,
                },
                |_, material, request, policy, writer| {
                    calls += 1;
                    assert_eq!(
                        request.method(),
                        if remove {
                            cloud_sdk::Method::Delete
                        } else {
                            cloud_sdk::Method::Put
                        }
                    );
                    assert_eq!(material.method(), request.method());
                    assert_eq!(request.target().as_str(), "/api/v1/crates/example/owners");
                    assert_eq!(
                        request.body(),
                        br#"{"users":["crates.io:alice","github:org:team"]}"#
                    );
                    assert!(request.headers().get("content-type").is_some());
                    let wire = if matches!(scenario, 1 | 2) {
                        br#"{"errors":[{"detail":"partial or rejected"}]}"#.as_slice()
                    } else {
                        br#"{"ok":true,"msg":"acknowledged"}"#
                    };
                    let mut attempt = writer.begin_attempt().fixture("attempt");
                    attempt
                        .body_mut()
                        .fixture("body")
                        .get_mut(..wire.len())
                        .fixture("range")
                        .copy_from_slice(wire);
                    for (name, value) in [
                        ("content-type", b"application/json".as_slice()),
                        (
                            "content-encoding",
                            if scenario == 4 {
                                b"gzip".as_slice()
                            } else {
                                b"identity"
                            },
                        ),
                    ] {
                        assert!(policy.admits_header(name));
                        if policy.admits_header(name) && scenario != 5 {
                            attempt
                                .headers_mut()
                                .fixture("headers")
                                .try_push(name, value, HeaderSensitivity::Public)
                                .fixture("header");
                        }
                    }
                    if scenario == 6 {
                        return Err(());
                    }
                    if scenario != 7 {
                        attempt
                            .commit(
                                StatusCode::new(match scenario {
                                    2 => 403,
                                    3 => 201,
                                    _ => 200,
                                })
                                .fixture("status"),
                                wire.len(),
                                ResponseMetadata::EMPTY,
                            )
                            .fixture("commit");
                    }
                    Ok(())
                },
            );
            assert_eq!(result.is_ok(), scenario == 0);
            assert_eq!(calls, 1);
            if matches!(scenario, 1 | 2) {
                assert!(matches!(
                    result,
                    Err(OwnerChangeExecutionError::Wire(
                        crate::wire::CratesIoWireError::Provider(_)
                    ))
                ));
            }
            for buffer in [&secret[..], &body[..], &response[..], &headers[..]] {
                assert!(buffer.iter().all(|v| *v == 0));
            }
        }
    }
}
#[test]
fn origin_capacity_and_shared_gate_rejections_clear_scratch() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let client = client();
    let owners = [selector("alice")];
    for scenario in 0..4 {
        reset_test_gate();
        let token = token(if scenario == 0 {
            CredentialOrigin::Staging
        } else {
            CredentialOrigin::Production
        });
        let mut secret = vec![0xa5; if scenario == 1 { 1 } else { 1024 }];
        let mut body = vec![0xa5; if scenario == 2 { 1 } else { 1024 }];
        let mut response = [0xa5; 4096];
        let mut headers = [0xa5; 512];
        let occupied = if scenario == 3 {
            Some(
                crate::wire::OfficialApiGate::new(
                    IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("agent"),
                )
                .begin()
                .fixture("gate"),
            )
        } else {
            None
        };
        let result = client.execute(
            OwnerChangeRequest::add(name(), &owners)
                .fixture("request")
                .confirm_add(&token)
                .fixture("permit"),
            OwnerChangeBuffers {
                credential: &mut secret,
                body: &mut body,
                response: &mut response,
                headers: &mut headers,
            },
            |_, _, _, _, _| -> Result<(), ()> { unreachable!("rejected request dispatched") },
        );
        assert!(result.is_err());
        drop(occupied);
        for buffer in [&secret[..], &body[..], &response[..], &headers[..]] {
            assert!(buffer.iter().all(|v| *v == 0));
        }
    }
    reset_test_gate();
}
