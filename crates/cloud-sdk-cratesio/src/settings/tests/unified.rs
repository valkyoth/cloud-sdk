use super::*;
use crate::{
    client::async_test_support::{Fixture, parity},
    wire::TEST_GATE_LOCK,
};
#[test]
fn settings_three_mode_wire_and_postcondition_parity() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let token = token(CredentialOrigin::Production);
    let mut value = crate_response();
    set(&mut value, "crate", "trustpub_only", true.into());
    let wire = serde_json::to_vec(&value).fixture("response");
    parity(
        || SettingsRequest::trustpub_only(name(), true).confirm(&token),
        Fixture::new(
            cloud_sdk::Method::Patch,
            "/api/v1/crates/serde",
            br#"{"crate":{"trustpub_only":true}}"#,
            &wire,
            200,
        ),
    );
    let mut value = version_response();
    set(&mut value, "version", "yanked", true.into());
    set(&mut value, "version", "yank_message", "reason".into());
    let wire = serde_json::to_vec(&value).fixture("response");
    parity(
        || {
            SettingsRequest::version(name(), version(), Some(true), YankMessage::Set("reason"))
                .fixture("request")
                .confirm(&token)
        },
        Fixture::new(
            cloud_sdk::Method::Patch,
            "/api/v1/crates/serde/1.0.0",
            br#"{"version":{"yanked":true,"yank_message":"reason"}}"#,
            &wire,
            200,
        ),
    );
}
