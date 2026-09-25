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
fn client() -> YankClient<'static, Executor> {
    YankClient::production(
        &Executor,
        IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("agent"),
        4096,
    )
    .fixture("client")
}
#[test]
fn cargo_exchanges_are_bodyless_single_attempt_and_clear_every_exit() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let token = token(CredentialOrigin::Production);
    let client = client();
    for yanked in [true, false] {
        for case in 0..22 {
            let scenario = case % 11;
            let (version, encoded) = if case < 11 {
                ("1.0.0", "1.0.0")
            } else {
                ("1.2.3-alpha.1+build.2", "1.2.3-alpha.1%2Bbuild.2")
            };
            let name = CrateName::new("serde").fixture("name");
            let version = Version::new(version).fixture("version");
            let intent = if yanked {
                YankRequest::yank(name, version)
            } else {
                YankRequest::unyank(name, version)
            };
            reset_test_gate();
            let mut secret = [0xa5; 1024];
            let mut response = [0xa5; 4096];
            let mut headers = [0xa5; 512];
            let mut calls = 0;
            let result = client.execute(
                intent.confirm(&token),
                YankBuffers {
                    credential: &mut secret,
                    response: &mut response,
                    headers: &mut headers,
                },
                |_, material, request, policy, writer| {
                    calls += 1;
                    assert_eq!(
                        request.method(),
                        if yanked {
                            cloud_sdk::Method::Delete
                        } else {
                            cloud_sdk::Method::Put
                        }
                    );
                    assert_eq!(material.method(), request.method());
                    assert_eq!(material.target(), request.target());
                    assert_eq!(
                        request.target().as_str(),
                        format!(
                            "/api/v1/crates/serde/{encoded}/{}",
                            if yanked { "yank" } else { "unyank" }
                        )
                    );
                    assert!(request.body().is_empty());
                    assert!(request.headers().get("content-type").is_none());
                    assert!(request.headers().get("accept").is_some());
                    let wire = match scenario {
                        1 | 2 => br#"{"errors":[{"detail":"denied"}]}"#.as_slice(),
                        8 => br#"{"ok":false}"#,
                        9 => br#"{"ok":true,"ok":false}"#,
                        _ => br#"{"ok":true}"#,
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
                                    10 => 302,
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
            assert_eq!(calls, 1);
            assert_eq!(result.is_ok(), scenario == 0);
            if matches!(scenario, 1 | 2) {
                assert!(matches!(
                    result,
                    Err(YankExecutionError::Wire(
                        crate::wire::CratesIoWireError::Provider(_)
                    ))
                ));
            }
            for b in [&secret[..], &response[..], &headers[..]] {
                assert!(b.iter().all(|v| *v == 0));
            }
        }
    }
    reset_test_gate();
}
#[test]
fn invalid_origin_capacity_and_occupied_gate_never_dispatch() {
    let _lock = TEST_GATE_LOCK.lock().fixture("lock");
    let client = client();
    for scenario in 0..3 {
        reset_test_gate();
        let token = token(if scenario == 0 {
            CredentialOrigin::Staging
        } else {
            CredentialOrigin::Production
        });
        let mut secret = vec![0xa5; if scenario == 1 { 1 } else { 1024 }];
        let mut response = [0xa5; 4096];
        let mut headers = [0xa5; 512];
        let occupied = if scenario == 2 {
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
        assert!(
            client
                .execute(
                    request(true).confirm(&token),
                    YankBuffers {
                        credential: &mut secret,
                        response: &mut response,
                        headers: &mut headers,
                    },
                    |_, _, _, _, _| -> Result<(), ()> {
                        unreachable!("rejected request dispatched")
                    }
                )
                .is_err()
        );
        drop(occupied);
        for b in [&secret[..], &response[..], &headers[..]] {
            assert!(b.iter().all(|v| *v == 0));
        }
    }
    reset_test_gate();
}
