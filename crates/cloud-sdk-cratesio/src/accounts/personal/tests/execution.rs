use super::*;
#[cfg(feature = "std")]
use crate::std as test_std;
use crate::{
    credentials::{EmailConfirmationToken, OwnerInvitationToken},
    endpoint::OfficialCratesIoEndpoint,
    wire::{IdentifyingUserAgent, TEST_GATE_LOCK, reset_test_gate},
};
use alloc::string::ToString;
use cloud_sdk::{
    Method,
    transport::{
        BoundTransport, BoundUserAgent, EndpointIdentity, EndpointIdentityError, HeaderSensitivity,
        ResponseMetadata, ResponseWriter, StatusCode,
    },
};
pub(super) struct Executor(pub(super) bool);
impl BoundTransport for Executor {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        if self.0 {
            Ok(OfficialCratesIoEndpoint::staging_api()
                .identity()
                .fixture("endpoint"))
        } else {
            Ok(OfficialCratesIoEndpoint::production_api()
                .identity()
                .fixture("endpoint"))
        }
    }
}
impl BoundUserAgent for Executor {
    fn configured_user_agent(&self) -> &[u8] {
        b"tests/1 (tests@example.org)"
    }
}
pub(super) fn client(executor: &Executor) -> PersonalClient<'_, Executor> {
    let identity = IdentifyingUserAgent::new("tests/1 (tests@example.org)").fixture("agent");
    if executor.0 {
        PersonalClient::staging(executor, identity, 4096).fixture("client")
    } else {
        PersonalClient::production(executor, identity, 4096).fixture("client")
    }
}
pub(super) fn stage(writer: &mut ResponseWriter<'_>, status: u16, media: &[u8], body: &[u8]) {
    let mut attempt = writer.begin_attempt().fixture("attempt");
    attempt
        .body_mut()
        .fixture("body")
        .get_mut(..body.len())
        .fixture("range")
        .copy_from_slice(body);
    attempt
        .headers_mut()
        .fixture("headers")
        .try_push("content-type", media, HeaderSensitivity::Public)
        .fixture("type");
    attempt
        .commit(
            StatusCode::new(status).fixture("status"),
            body.len(),
            ResponseMetadata::EMPTY,
        )
        .fixture("commit");
}
#[test]
fn executes_exact_permitted_request_once_and_clears_every_buffer() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    reset_test_gate();
    let token = token();
    let executor = Executor(false);
    let client = client(&executor);
    let mut credential = [0xa5; 1024];
    let mut body = [0xa5; 4096];
    let mut response = [0xa5; 4096];
    let mut headers = [0xa5; 512];
    let mut calls = 0;
    let result = client.execute(
        PersonalRequest::decline_invitation(id(42)).confirm(&token),
        PersonalBuffers {
            credential: &mut credential,
            body: &mut body,
            response: &mut response,
            headers: &mut headers,
        },
        |_, material, request, policy, writer| {
            calls += 1;
            assert_eq!(request.method(), Method::Put);
            assert_eq!(
                request.target().path().as_str(),
                "/api/v1/me/crate_owner_invitations/42"
            );
            assert_eq!(request.target(), material.target());
            assert!(material.authorization().is_some());
            assert!(request.headers().get("authorization").is_none());
            assert_eq!(
                request
                    .headers()
                    .get("content-type")
                    .fixture("type")
                    .value()
                    .as_str(),
                "application/json"
            );
            assert_eq!(
                request.body(),
                br#"{"crate_owner_invite":{"crate_id":42,"accepted":false}}"#
            );
            assert_eq!(policy.max_body_bytes(), 4096);
            stage(
                writer,
                200,
                b"application/json",
                br#"{"crate_owner_invitation":{"crate_id":42,"accepted":false}}"#,
            );
            Ok::<_, ()>(())
        },
    );
    assert_eq!(
        result.fixture("execution"),
        PersonalResponse::Invitation {
            crate_id: id(42),
            accepted: false
        }
    );
    assert_eq!(calls, 1);
    for buffer in [&credential[..], &body[..], &response[..], &headers[..]] {
        assert!(buffer.iter().all(|v| *v == 0));
    }
}
#[test]
fn wrong_origin_capacity_and_admission_never_dispatch() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let token = token();
    for (staging, capacity) in [(true, 1024), (false, 1)] {
        reset_test_gate();
        let executor = Executor(staging);
        let client = client(&executor);
        let mut secret = vec![0xa5; capacity];
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
            |_, _, _, _, _| -> Result<(), ()> { unreachable!("invalid context dispatched") },
        );
        assert!(result.is_err());
        for buffer in [&secret[..], &body[..], &response[..], &headers[..]] {
            assert!(buffer.iter().all(|v| *v == 0));
        }
    }
}
#[test]
fn secret_path_tokens_have_no_authorization_and_redacted_diagnostics() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    for invitation in [false, true] {
        reset_test_gate();
        let byte = runtime_token_byte();
        let spelling = char::from(byte).to_string().repeat(24);
        let mut source = [byte; 24];
        let permit = if invitation {
            PersonalPermit::accept_invitation_token(
                OwnerInvitationToken::from_mut_bytes(CredentialOrigin::Production, &mut source)
                    .fixture("token"),
                id(42),
            )
        } else {
            PersonalPermit::confirm_email(
                EmailConfirmationToken::from_mut_bytes(CredentialOrigin::Production, &mut source)
                    .fixture("token"),
            )
        };
        assert!(source.iter().all(|v| *v == 0));
        let executor = Executor(false);
        let client = client(&executor);
        let mut secret = [0xa5; 1024];
        let mut body = [0xa5; 1024];
        let mut response = [0xa5; 4096];
        let mut headers = [0xa5; 512];
        let result = client.execute(
            permit,
            PersonalBuffers {
                credential: &mut secret,
                body: &mut body,
                response: &mut response,
                headers: &mut headers,
            },
            |_, material, request, _, writer| {
                assert!(material.authorization().is_none());
                assert!(request.body().is_empty());
                assert!(!format!("{request:?} {material:?}").contains(&spelling));
                let prefix = if invitation {
                    "/api/v1/me/crate_owner_invitations/accept/"
                } else {
                    "/api/v1/confirm/"
                };
                assert_eq!(
                    request.target().path().as_str(),
                    format!("{prefix}{spelling}")
                );
                stage(
                    writer,
                    200,
                    b"application/json",
                    if invitation {
                        br#"{"crate_owner_invitation":{"crate_id":42,"accepted":true}}"#
                    } else {
                        br#"{"ok":true}"#
                    },
                );
                Ok::<_, ()>(())
            },
        );
        assert!(result.is_ok());
        for buffer in [&secret[..], &body[..], &response[..], &headers[..]] {
            assert!(buffer.iter().all(|v| *v == 0));
        }
    }
}
#[test]
fn expired_repeated_wrong_user_and_malformed_responses_fail_without_retries() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let token = token();
    let executor = Executor(false);
    let client = client(&executor);
    for (status, media, wire) in [
        (
            400,
            b"application/json".as_slice(),
            br#"{"errors":[{"detail":"wrong user"}]}"#.as_slice(),
        ),
        (
            403,
            b"application/json",
            br#"{"errors":[{"detail":"email not verified"}]}"#,
        ),
        (
            410,
            b"application/json",
            br#"{"errors":[{"detail":"expired"}]}"#,
        ),
        (
            404,
            b"application/json",
            br#"{"errors":[{"detail":"already accepted"}]}"#,
        ),
        (200, b"text/html", br#"{"ok":true}"#),
        (201, b"application/json", br#"{"ok":true}"#),
        (200, b"application/json", br#"{"ok":false}"#),
    ] {
        reset_test_gate();
        let mut secret = [0xa5; 1024];
        let mut body = [0xa5; 1024];
        let mut response = [0xa5; 4096];
        let mut headers = [0xa5; 512];
        let mut calls = 0;
        let result = client.execute(
            PersonalRequest::resend_email(id(7)).confirm(&token),
            PersonalBuffers {
                credential: &mut secret,
                body: &mut body,
                response: &mut response,
                headers: &mut headers,
            },
            |_, _, _, _, writer| {
                calls += 1;
                stage(writer, status, media, wire);
                Ok::<_, ()>(())
            },
        );
        assert!(result.is_err());
        assert_eq!(calls, 1);
        for buffer in [&secret[..], &body[..], &response[..], &headers[..]] {
            assert!(buffer.iter().all(|v| *v == 0));
        }
    }
}
#[test]
fn adapter_failure_or_unwind_clears_serialized_email_and_credentials() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let token = token();
    let executor = Executor(false);
    let client = client(&executor);
    for panics in [false, true] {
        reset_test_gate();
        let mut secret = [0xa5; 1024];
        let mut body = [0xa5; 1024];
        let mut response = [0xa5; 4096];
        let mut headers = [0xa5; 512];
        let result = test_std::panic::catch_unwind(test_std::panic::AssertUnwindSafe(|| {
            client.execute(
                PersonalRequest::update_email(
                    id(7),
                    EmailAddress::new("private@example.org").fixture("email"),
                )
                .confirm(&token),
                PersonalBuffers {
                    credential: &mut secret,
                    body: &mut body,
                    response: &mut response,
                    headers: &mut headers,
                },
                |_, _, _, _, _| -> Result<(), ()> {
                    assert!(!panics, "injected adapter panic");
                    Err(())
                },
            )
        }));
        assert!(matches!(result, Err(_) | Ok(Err(_))));
        for buffer in [&secret[..], &body[..], &response[..], &headers[..]] {
            assert!(buffer.iter().all(|v| *v == 0));
        }
    }
}
