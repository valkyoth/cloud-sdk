//! Cloud action, metrics, and composite response regressions.

use alloc::format;
use alloc::string::String;

use cloud_sdk::transport::StatusCode;

use super::checked_fixtures::{action_value, resource_value};
use super::checked_test_support::{decode_response, prepared, response};
use super::{HetznerDecodeError, HetznerSuccess, ResponseModelError};
use crate::actions::MAX_ACTION_ID;
use crate::identity::CLOUD_SERVICE_ID;
use crate::response::ApiErrorCode;

#[test]
fn composite_preserves_singular_follow_up_and_nullable_secret_states() {
    let body = serde_json::json!({
        "server": resource_value("server"),
        "action": action_value(),
        "next_actions": [action_value()],
        "root_password": null,
    });
    let body = serde_json::to_vec(&body).unwrap_or_default();
    let decoded = decode_response(
        prepared("create_server", CLOUD_SERVICE_ID, StatusCode::CREATED),
        response(StatusCode::CREATED, &body),
    );
    let Ok(decoded) = decoded else {
        unreachable!("composite fixture failed to decode")
    };
    let HetznerSuccess::Composite(composite) = decoded.success() else {
        unreachable!("composite fixture selected the wrong model")
    };
    assert!(composite.action().is_some());
    assert!(composite.actions().is_empty());
    assert_eq!(composite.next_actions().len(), 1);
    assert!(matches!(composite.secret("root_password"), Some(None)));
    assert_eq!(composite.secret("password"), None);
}

#[test]
fn composite_rejects_null_for_nonnullable_secret_outputs() {
    let console = serde_json::json!({
        "action": action_value(),
        "password": null,
        "wss_url": "wss://console.example.invalid",
    });
    let console = serde_json::to_vec(&console).unwrap_or_default();
    assert_eq!(
        decode_response(
            prepared(
                "request_server_console",
                CLOUD_SERVICE_ID,
                StatusCode::CREATED,
            ),
            response(StatusCode::CREATED, &console),
        ),
        Err(HetznerDecodeError::Model(ResponseModelError::WrongType))
    );
}

#[test]
fn checked_actions_enforce_source_ids_utc_and_unknown_error_retention() {
    let mut action = action_value();
    let Some(fields) = action.as_object_mut() else {
        unreachable!("action fixture is not an object")
    };
    fields.insert("status".into(), serde_json::json!("error"));
    fields.insert("progress".into(), serde_json::json!(100));
    fields.insert("finished".into(), serde_json::json!("2026-08-08T00:00:01Z"));
    fields.insert(
        "error".into(),
        serde_json::json!({"code":"future_error","message":"private failure"}),
    );
    let body = serde_json::to_vec(&serde_json::json!({"action":action})).unwrap_or_default();
    let decoded = decode_response(
        prepared("get_action", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, &body),
    );
    let Ok(decoded) = decoded else {
        unreachable!("action fixture failed to decode")
    };
    let HetznerSuccess::Action(action) = decoded.success() else {
        unreachable!("action fixture selected the wrong model")
    };
    let Some(error) = action.error() else {
        unreachable!("action fixture lost its error")
    };
    assert_eq!(error.code(), ApiErrorCode::Unknown);
    assert_eq!(error.code_text(), "future_error");
    let debug = format!("{action:?}");
    assert!(!debug.contains("future_error"));
    assert!(!debug.contains("private failure"));

    for invalid in [
        String::from(
            r#"{"action":{"id":1,"command":"x","status":"running","progress":1,"started":"2026-08-08T00:00:00+00:00","finished":null,"resources":[],"error":null}}"#,
        ),
        format!(
            r#"{{"action":{{"id":1,"command":"x","status":"running","progress":1,"started":"2026-08-08T00:00:00Z","finished":null,"resources":[{{"id":{},"type":"server"}}],"error":null}}}}"#,
            MAX_ACTION_ID + 1
        ),
    ] {
        assert!(matches!(
            decode_response(
                prepared("get_action", CLOUD_SERVICE_ID, StatusCode::OK),
                response(StatusCode::OK, invalid.as_bytes()),
            ),
            Err(HetznerDecodeError::Model(_))
        ));
    }
}

#[test]
fn checked_provider_errors_retain_unknown_codes_with_redacted_diagnostics() {
    let Some(bad_request) = StatusCode::new(400) else {
        unreachable!("HTTP 400 must be a valid status")
    };
    let decoded = decode_response(
        prepared("get_action", CLOUD_SERVICE_ID, StatusCode::OK),
        response(
            bad_request,
            br#"{"error":{"code":"future_provider_error","message":"private failure"}}"#,
        ),
    );
    let Err(HetznerDecodeError::Provider(error)) = decoded else {
        unreachable!("provider error fixture selected the wrong result")
    };
    assert_eq!(error.code(), ApiErrorCode::Unknown);
    assert_eq!(error.code_text(), "future_provider_error");
    let debug = format!("{error:?}");
    assert!(!debug.contains("future_provider_error"));
    assert!(!debug.contains("private failure"));
}
