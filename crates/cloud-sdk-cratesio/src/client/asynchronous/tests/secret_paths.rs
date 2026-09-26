use super::*;
use crate::{
    accounts::personal::PersonalPermit,
    credentials::{CredentialOrigin, EmailConfirmationToken, OwnerInvitationToken},
    identifiers::NumericId,
    wire::TEST_GATE_LOCK,
};
use alloc::{format, string::ToString};

fn permit(invitation: bool, byte: u8) -> PersonalPermit<'static> {
    permit_with_state(invitation, byte, false)
}
fn permit_with_state(invitation: bool, byte: u8, cleared: bool) -> PersonalPermit<'static> {
    let mut source = [byte; 24];
    let permit = if invitation {
        let mut token =
            OwnerInvitationToken::from_mut_bytes(CredentialOrigin::Production, &mut source)
                .fixture("token");
        if cleared {
            token.clear();
        }
        PersonalPermit::accept_invitation_token(token, NumericId::new(42).fixture("crate id"))
    } else {
        let mut token =
            EmailConfirmationToken::from_mut_bytes(CredentialOrigin::Production, &mut source)
                .fixture("token");
        if cleared {
            token.clear();
        }
        PersonalPermit::confirm_email(token)
    };
    assert!(source.iter().all(|b| *b == 0));
    permit
}

#[test]
fn secret_path_short_storage_and_cleared_tokens_never_dispatch() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    for invitation in [false, true] {
        for mode in 0..3 {
            for cleared in [false, true] {
                reset_test_gate();
                let byte = token_byte();
                let target = target(invitation, byte);
                let fixture = fixture(invitation, &target);
                let client =
                    RegistryClient::production(&fixture, identity(), 16384).fixture("client");
                let mut buffers = Buffers::new();
                let mut tiny = [0xa5; 1];
                let mut parts = buffers.parts();
                if !cleared {
                    parts.credential = &mut tiny;
                }
                let permit = permit_with_state(invitation, byte, cleared);
                let result = match mode {
                    0 => client.execute(permit, parts),
                    1 => ready(client.execute_local(permit, parts)),
                    2 => ready(send(client.execute_async(permit, parts))),
                    _ => unreachable!("mode"),
                };
                assert!(result.is_err());
                assert_eq!(fixture.calls.load(Ordering::SeqCst), 0);
                if cleared {
                    buffers.cleared();
                } else {
                    assert_eq!(tiny, [0]);
                    assert!(
                        buffers
                            .body
                            .iter()
                            .chain(&buffers.response)
                            .chain(&buffers.headers)
                            .all(|b| *b == 0)
                    );
                    assert!(buffers.credential.iter().all(|b| *b == 0xa5));
                }
            }
        }
    }
    reset_test_gate();
}
fn token_byte() -> u8 {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    b'a'.saturating_add(u8::try_from(NEXT.fetch_add(1, Ordering::Relaxed) % 26).fixture("byte"))
}
fn target(invitation: bool, byte: u8) -> alloc::string::String {
    let prefix = if invitation {
        "/api/v1/me/crate_owner_invitations/accept/"
    } else {
        "/api/v1/confirm/"
    };
    format!("{prefix}{}", char::from(byte).to_string().repeat(24))
}
fn fixture(invitation: bool, target: &str) -> Fixture<'_> {
    let mut fixture = Fixture::new(
        Method::Put,
        target,
        b"",
        if invitation {
            br#"{"crate_owner_invitation":{"crate_id":42,"accepted":true}}"#
        } else {
            br#"{"ok":true}"#
        },
        200,
    );
    fixture.auth = false;
    fixture
}

#[test]
fn secret_path_permits_execute_exactly_once_in_all_modes_without_authorization() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    for invitation in [false, true] {
        let byte = token_byte();
        let target = target(invitation, byte);
        parity(|| permit(invitation, byte), fixture(invitation, &target));
    }
}

#[test]
fn secret_path_cancellation_clears_all_scratch_without_retry() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    for invitation in [false, true] {
        for local_mode in [false, true] {
            reset_test_gate();
            let byte = token_byte();
            let target = target(invitation, byte);
            let mut fixture = fixture(invitation, &target);
            fixture.pending = true;
            let local = Local(fixture, Rc::new(()));
            let client = RegistryClient::production(&local.0, identity(), 16384).fixture("client");
            let local_client =
                RegistryClient::production(&local, identity(), 16384).fixture("client");
            let mut buffers = Buffers::new();
            if local_mode {
                drop(local_client.execute_local(permit(invitation, byte), buffers.parts()));
            } else {
                drop(send(
                    client.execute_async(permit(invitation, byte), buffers.parts()),
                ));
            }
            buffers.cleared();
            assert_eq!(local.0.calls.load(Ordering::SeqCst), 0);
            if local_mode {
                let mut future = core::pin::pin!(
                    local_client.execute_local(permit(invitation, byte), buffers.parts())
                );
                assert!(
                    future
                        .as_mut()
                        .poll(&mut Context::from_waker(Waker::noop()))
                        .is_pending()
                );
            } else {
                let mut future = core::pin::pin!(send(
                    client.execute_async(permit(invitation, byte), buffers.parts())
                ));
                assert!(
                    future
                        .as_mut()
                        .poll(&mut Context::from_waker(Waker::noop()))
                        .is_pending()
                );
            }
            buffers.cleared();
            assert_eq!(local.0.calls.load(Ordering::SeqCst), 1);
        }
    }
    reset_test_gate();
}

#[test]
fn secret_path_response_failure_is_not_success_or_retried() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    for invitation in [false, true] {
        for local_mode in [false, true] {
            for wire in [
                b"{}".as_slice(),
                br#"{"ok":false}"#,
                br#"{"crate_owner_invitation":{"crate_id":43,"accepted":true}}"#,
            ] {
                reset_test_gate();
                let byte = token_byte();
                let target = target(invitation, byte);
                let mut fixture = fixture(invitation, &target);
                fixture.wire = wire;
                let client =
                    RegistryClient::production(&fixture, identity(), 16384).fixture("client");
                let mut buffers = Buffers::new();
                let result = if local_mode {
                    ready(client.execute_local(permit(invitation, byte), buffers.parts()))
                } else {
                    ready(send(
                        client.execute_async(permit(invitation, byte), buffers.parts()),
                    ))
                };
                assert!(result.is_err());
                buffers.cleared();
                assert_eq!(fixture.calls.load(Ordering::SeqCst), 1);
            }
        }
    }
    reset_test_gate();
}
