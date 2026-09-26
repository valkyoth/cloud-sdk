use super::operations::{token, yank};
use super::*;
use crate::{
    credentials::CredentialOrigin,
    discovery::DiscoveryError,
    wire::{OfficialApiGate, ScheduleError, TEST_GATE_LOCK},
};

fn fixture() -> Fixture<'static> {
    Fixture::new(
        Method::Delete,
        "/api/v1/crates/serde/1.0.0/yank",
        b"",
        br#"{"ok":true}"#,
        200,
    )
}
#[test]
fn unpolled_and_in_flight_cancellation_clear_all_regions_for_both_modes() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let token = token(CredentialOrigin::Production);
    for local_mode in [false, true] {
        reset_test_gate();
        let mut fixture = fixture();
        fixture.pending = true;
        let local = Local(fixture, Rc::new(()));
        let client = RegistryClient::production(&local.0, identity(), 16384).fixture("client");
        let local_client = RegistryClient::production(&local, identity(), 16384).fixture("client");
        let mut buffers = Buffers::new();
        if local_mode {
            drop(local_client.execute_local(yank().confirm(&token), buffers.parts()));
        } else {
            drop(send(
                client.execute_async(yank().confirm(&token), buffers.parts()),
            ));
        }
        buffers.cleared();
        assert_eq!(local.0.calls.load(Ordering::SeqCst), 0);
        if local_mode {
            let mut future = core::pin::pin!(
                local_client.execute_local(yank().confirm(&token), buffers.parts())
            );
            assert!(
                future
                    .as_mut()
                    .poll(&mut Context::from_waker(Waker::noop()))
                    .is_pending()
            );
            assert!(matches!(
                OfficialApiGate::new(identity()).begin(),
                Err(ScheduleError::Unavailable)
            ));
        } else {
            let mut future = core::pin::pin!(send(
                client.execute_async(yank().confirm(&token), buffers.parts())
            ));
            assert!(
                future
                    .as_mut()
                    .poll(&mut Context::from_waker(Waker::noop()))
                    .is_pending()
            );
            assert!(matches!(
                OfficialApiGate::new(identity()).begin(),
                Err(ScheduleError::Unavailable)
            ));
        }
        buffers.cleared();
        assert_eq!(local.0.calls.load(Ordering::SeqCst), 1);
        assert!(matches!(
            OfficialApiGate::new(identity()).begin(),
            Err(ScheduleError::Wait(_))
        ));
    }
    reset_test_gate();
}
#[test]
fn async_preflight_transport_and_wire_failures_cannot_leave_scratch_or_retry() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    for local_mode in [false, true] {
        for case in 0..10 {
            reset_test_gate();
            let mut fixture = fixture();
            match case {
                3 => fixture.fail = true,
                4 => fixture.wire = br#"{"ok":false}"#,
                5 => fixture.status = 302,
                6 => fixture.encoding = Some(b"gzip"),
                7 => fixture.retry = Some(b"86401"),
                8 => {
                    fixture.status = 403;
                    fixture.wire = br#"{"errors":[{"detail":"denied"}]}"#;
                }
                9 => fixture.media = false,
                _ => {}
            }
            let local = Local(fixture, Rc::new(()));
            let client = RegistryClient::production(&local.0, identity(), 16384).fixture("client");
            let local_client =
                RegistryClient::production(&local, identity(), 16384).fixture("client");
            local.0.changed.store(case == 0, Ordering::SeqCst);
            let token = token(if case == 1 {
                CredentialOrigin::Staging
            } else {
                CredentialOrigin::Production
            });
            let occupied = if case == 2 {
                Some(OfficialApiGate::new(identity()).begin().fixture("gate"))
            } else {
                None
            };
            let mut buffers = Buffers::new();
            let result = if local_mode {
                ready(local_client.execute_local(yank().confirm(&token), buffers.parts()))
            } else {
                ready(send(
                    client.execute_async(yank().confirm(&token), buffers.parts()),
                ))
            };
            assert!(result.is_err());
            buffers.cleared();
            assert_eq!(local.0.calls.load(Ordering::SeqCst), usize::from(case >= 3));
            if case == 7 {
                assert!(matches!(
                    result,
                    Err(DiscoveryExecutionError::Schedule(ScheduleError::Overflow))
                ));
            }
            drop(occupied);
        }
    }
    reset_test_gate();
}
#[test]
fn invalid_material_and_unqualified_secret_paths_fail_before_admission() {
    use crate::{
        accounts::personal::PersonalPermit,
        credentials::{EmailConfirmationToken, OwnerInvitationToken},
        identifiers::NumericId,
    };
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let fixture = fixture();
    let client = RegistryClient::production(&fixture, identity(), 16384).fixture("client");
    for invitation in [false, true] {
        for local in [false, true] {
            reset_test_gate();
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let mut source = [b'a'.saturating_add(
                u8::try_from(NEXT.fetch_add(1, Ordering::Relaxed) % 26).fixture("token variation"),
            ); 24];
            let permit = if invitation {
                PersonalPermit::accept_invitation_token(
                    OwnerInvitationToken::from_mut_bytes(CredentialOrigin::Production, &mut source)
                        .fixture("token"),
                    NumericId::new(42).fixture("id"),
                )
            } else {
                PersonalPermit::confirm_email(
                    EmailConfirmationToken::from_mut_bytes(
                        CredentialOrigin::Production,
                        &mut source,
                    )
                    .fixture("token"),
                )
            };
            let mut buffers = Buffers::new();
            let result = if local {
                ready(client.execute_local(permit, buffers.parts()))
            } else {
                ready(send(client.execute_async(permit, buffers.parts())))
            };
            assert!(matches!(
                result,
                Err(DiscoveryExecutionError::Model(DiscoveryError::Binding))
            ));
            buffers.cleared();
            assert!(source.iter().all(|b| *b == 0));
            assert_eq!(fixture.calls.load(Ordering::SeqCst), 0);
            let _unused = OfficialApiGate::new(identity())
                .begin()
                .fixture("no admission consumed");
        }
    }
    reset_test_gate();
}

#[test]
fn cleared_credentials_and_short_scratch_fail_before_admission() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    for local in [false, true] {
        for short in [false, true] {
            reset_test_gate();
            let mut token = token(CredentialOrigin::Production);
            if !short {
                token.clear();
            }
            let fixture = fixture();
            let client = RegistryClient::production(&fixture, identity(), 16384).fixture("client");
            let mut buffers = Buffers::new();
            let mut tiny = [0xa5; 1];
            let mut parts = buffers.parts();
            if short {
                parts.credential = &mut tiny;
            }
            let result = if local {
                ready(client.execute_local(yank().confirm(&token), parts))
            } else {
                ready(send(client.execute_async(yank().confirm(&token), parts)))
            };
            assert!(matches!(
                result,
                Err(DiscoveryExecutionError::Model(DiscoveryError::Binding))
            ));
            if short {
                assert_eq!(tiny, [0]);
                // This buffer was intentionally not passed to the runner.
                assert!(buffers.credential.iter().all(|b| *b == 0xa5));
                cloud_sdk_sanitization::sanitize_bytes(&mut buffers.credential);
            }
            buffers.cleared();
            assert_eq!(fixture.calls.load(Ordering::SeqCst), 0);
            let _unused = OfficialApiGate::new(identity())
                .begin()
                .fixture("unoccupied gate");
        }
    }
    reset_test_gate();
}
