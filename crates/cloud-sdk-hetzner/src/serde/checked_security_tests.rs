//! Security regressions for the checked response decoder.

use alloc::string::String;

use cloud_sdk::operation::ResponsePolicyError;
use cloud_sdk::transport::StatusCode;

use super::checked_test_support::{
    assert_decode_error, decode_response, decode_response_with_request_id, prepared, response,
};
use super::strict_json::MAX_JSON_NODES;
use super::{HetznerDecodeError, HetznerSuccess};
use crate::identity::CLOUD_SERVICE_ID;

#[test]
fn rejects_aggregate_json_amplification_and_protects_failure_path_strings() {
    let mut amplified = String::from(r#"{"server":{"id":1},"future":["#);
    let inner = "[null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null]";
    let containers = MAX_JSON_NODES / 17 + 1;
    for index in 0..containers {
        if index != 0 {
            amplified.push(',');
        }
        amplified.push_str(inner);
    }
    amplified.push_str("]}");

    assert!(containers < 4096);
    assert!(amplified.len() < 1_000_000);
    assert_decode_error(
        decode_response(
            prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
            response(StatusCode::OK, amplified.as_bytes()),
        ),
        HetznerDecodeError::MalformedPayload,
    );

    for malformed in [
        br#"{"root_password":"first","root_password":"second"}"#.as_slice(),
        br#"{"root_password":"temporary"} trailing"#,
    ] {
        assert_decode_error(
            decode_response(
                prepared("create_server", CLOUD_SERVICE_ID, StatusCode::CREATED),
                response(StatusCode::CREATED, malformed),
            ),
            HetznerDecodeError::MalformedPayload,
        );
    }
    assert!(matches!(
        decode_response(
            prepared("create_server", CLOUD_SERVICE_ID, StatusCode::CREATED),
            response(StatusCode::CREATED, br#"{"root_password":"temporary"}"#),
        ),
        Err(HetznerDecodeError::Model(_))
    ));
}

#[test]
fn escaped_provider_and_action_errors_remain_in_protected_models() -> Result<(), &'static str> {
    let provider = decode_response(
        prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
        response(
            StatusCode::TOO_MANY_REQUESTS,
            br#"{"error":{"code":"invalid_input","message":"secret: \"\u2603\""}}"#,
        ),
    );
    let provider = match provider {
        Err(HetznerDecodeError::Provider(error)) => error,
        _ => return Err("provider error was not decoded"),
    };
    assert_eq!(
        provider.try_with_message(|message| message == "secret: \"☃\""),
        Ok(true)
    );

    let action = br#"{"action":{"id":1,"command":"create_server","status":"error","progress":100,"started":"2026-07-16T00:00:00Z","finished":"2026-07-16T00:00:01Z","resources":[],"error":{"code":"action_failed","message":"secret: \"\u2603\""}}}"#;
    let decoded = decode_response(
        prepared("get_action", CLOUD_SERVICE_ID, StatusCode::OK),
        response(StatusCode::OK, action),
    )
    .map_err(|_| "action response was not decoded")?;
    let HetznerSuccess::Action(action) = decoded.success() else {
        return Err("action response returned the wrong model");
    };
    let error = action.error().ok_or("action error was missing")?;
    assert_eq!(
        error.try_with_message(|message| message == "secret: \"☃\""),
        Ok(true)
    );
    Ok(())
}

#[test]
fn provider_errors_apply_the_operation_request_id_policy() {
    let decoded = decode_response_with_request_id(
        prepared("get_server", CLOUD_SERVICE_ID, StatusCode::OK),
        response(
            StatusCode::TOO_MANY_REQUESTS,
            br#"{"error":{"code":"rate_limit_exceeded","message":"slow down"}}"#,
        ),
        b"",
    );
    assert_decode_error(
        decoded,
        HetznerDecodeError::ResponsePolicy(ResponsePolicyError::InvalidRequestId),
    );
}
