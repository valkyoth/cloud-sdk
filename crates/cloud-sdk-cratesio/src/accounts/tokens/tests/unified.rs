use super::*;
use crate::{
    client::RegistryClient,
    client::async_test_support::{Buffers, Fixture, identity, parity, ready, send},
    wire::{OfficialApiGate, ScheduleError, TEST_GATE_LOCK, reset_test_gate},
};
use cloud_sdk::Method;
#[test]
fn token_operations_have_three_mode_parity() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let token = token(CredentialOrigin::Production);
    parity(
        || TokenPermit::inspect(id(42), &token),
        Fixture::new(
            Method::Get,
            "/api/v1/me/tokens/42",
            b"",
            RECORD.as_bytes(),
            200,
        ),
    );
    parity(
        || TokenPermit::confirm_revoke(id(42), &token),
        Fixture::new(Method::Delete, "/api/v1/me/tokens/42", b"", b"{}", 200),
    );
    parity(
        || TokenPermit::confirm_revoke_current(&token),
        Fixture::new(Method::Delete, "/api/v1/tokens/current", b"", b"", 204),
    );
}
#[test]
fn async_empty_success_rejects_framing_errors_and_applies_rate_delay() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let token = token(CredentialOrigin::Production);
    for local in [false, true] {
        for case in 0..5 {
            reset_test_gate();
            let mut fixture = Fixture::new(Method::Delete, "/api/v1/tokens/current", b"", b"", 204);
            match case {
                0 => fixture.media = true,
                1 => fixture.wire = b"x",
                2 => fixture.encoding = Some(b"identity"),
                3 => fixture.retry = Some(b"86401"),
                _ => fixture.retry = Some(b"60"),
            }
            let client = RegistryClient::production(&fixture, identity(), 16384).fixture("client");
            let mut buffers = Buffers::new();
            let result =
                if local {
                    ready(client.execute_local(
                        TokenPermit::confirm_revoke_current(&token),
                        buffers.parts(),
                    ))
                } else {
                    ready(send(client.execute_async(
                        TokenPermit::confirm_revoke_current(&token),
                        buffers.parts(),
                    )))
                };
            assert_eq!(result.is_ok(), case == 4);
            buffers.cleared();
            if case == 4 {
                assert!(
                    matches!(OfficialApiGate::new(identity()).begin(), Err(ScheduleError::Wait(d)) if d.as_secs() >= 59)
                );
            }
        }
    }
    reset_test_gate();
}
