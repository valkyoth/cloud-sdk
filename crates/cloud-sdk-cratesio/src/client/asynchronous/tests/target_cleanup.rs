use super::*;
use crate::{
    accounts::personal::invitations::{Filter, InvitationPermit},
    credentials::CredentialOrigin,
    discovery::DiscoveryClient,
    query::{MAX_TARGET_BYTES, PerPage, Seek},
    wire::TEST_GATE_LOCK,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Exit {
    Success,
    Error,
    Unwind,
    Cancel,
}

#[test]
fn invitation_runners_clear_the_entire_target_after_dispatch() {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let token = operations::token(CredentialOrigin::Production);
    for mode in 0..3 {
        for exit in [Exit::Success, Exit::Error, Exit::Unwind, Exit::Cancel] {
            if mode == 0 && exit == Exit::Cancel {
                continue; // Blocking dispatch cannot suspend.
            }
            reset_test_gate();
            let mut fixture = Fixture::new(
                Method::Get,
                "/api/v1/crate_owner_invitations?crate_name=serde&per_page=10&seek=cursor_1",
                b"",
                br#"{"crate_owner_invitations":[],"users":[],"meta":{"next_page":null}}"#,
                200,
            );
            fixture.fail = exit == Exit::Error;
            fixture.pending = exit == Exit::Cancel;
            fixture.unwind = exit == Exit::Unwind;
            let local = Local(fixture, Rc::new(()));
            let permit = InvitationPermit::list(
                &token,
                Filter::Crate(operations::name()),
                PerPage::DEFAULT,
                Some(Seek::new("cursor_1").fixture("cursor")),
            );
            let mut scratch = [0xa5; MAX_TARGET_BYTES];
            let mut buffers = Buffers::new();
            let outcome = catch_unwind(AssertUnwindSafe(|| {
                if mode == 0 {
                    let c =
                        RegistryClient::production(&local.0, identity(), 16384).fixture("client");
                    return Some(crate::client::blocking::run_invitation(
                        permit,
                        &c,
                        buffers.parts(),
                        &mut scratch,
                    ));
                }
                macro_rules! poll_runner {
                    ($transport:expr, $run:ident) => {{
                        let c = DiscoveryClient::production($transport, identity(), 16384)
                            .fixture("client");
                        let mut f = core::pin::pin!(super::super::runner::$run(
                            &c,
                            permit,
                            buffers.parts(),
                            &mut scratch
                        ));
                        match f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
                            Poll::Ready(r) => Some(r),
                            Poll::Pending => None,
                        }
                    }};
                }
                if mode == 1 {
                    poll_runner!(&local.0, execute_async)
                } else {
                    poll_runner!(&local, execute_local)
                }
            }));
            match exit {
                Exit::Success => assert!(matches!(outcome, Ok(Some(Ok(_))))),
                Exit::Error => assert!(matches!(
                    outcome,
                    Ok(Some(Err(DiscoveryExecutionError::Transport(()))))
                )),
                Exit::Cancel => assert!(matches!(outcome, Ok(None))),
                Exit::Unwind => assert_eq!(
                    outcome
                        .err()
                        .and_then(|p| p.downcast_ref::<&str>().copied()),
                    Some("fixture transport unwind")
                ),
            }
            assert_eq!(local.0.calls.load(Ordering::SeqCst), 1);
            assert!(scratch.iter().all(|&b| b == 0));
        }
    }
    reset_test_gate();
}
