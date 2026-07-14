use alloc::format;

use super::{ActionEnvelope, ApiErrorEnvelope, PaginationEnvelope, ResponseBytes};

const VALID_ACTION: &str = r#"{"action":{"id":42,"command":"create_rrset","status":"running","progress":50,"started":"2026-07-12T12:00:00Z","finished":null,"resources":[{"id":7,"type":"zone"}],"error":null}}"#;

#[test]
fn malformed_bytes_and_trailing_documents_are_rejected() {
    for input in [
        b"\xff\xfe\xfd".as_slice(),
        br#"{"error":{"code":"forbidden","message":"x"}} trailing"#,
        br#"{"meta":{"pagination":[]}}"#,
        br#"{"action":null}"#,
    ] {
        let admitted = ResponseBytes::new(input);
        assert!(admitted.is_ok());
        assert!(serde_json::from_slice::<ActionEnvelope<'_>>(input).is_err());
        assert!(serde_json::from_slice::<ApiErrorEnvelope<'_>>(input).is_err());
        assert!(serde_json::from_slice::<PaginationEnvelope>(input).is_err());
    }
}

#[test]
fn deeply_nested_unknown_fields_do_not_change_domain_state() {
    let mut nested = "[".repeat(256);
    nested.push('0');
    nested.push_str(&"]".repeat(256));
    let input = with_additive_field(&nested);
    assert!(ResponseBytes::new(input.as_bytes()).is_ok());
    let parsed = serde_json::from_str::<ActionEnvelope<'_>>(&input);
    assert!(parsed.is_ok());
    let Ok(parsed) = parsed else { return };
    assert_eq!(parsed.action().id().get(), 42);
    assert_eq!(parsed.action().command(), "create_rrset");
    assert_eq!(parsed.action().progress(), 50);
}

#[test]
fn every_bounded_action_text_field_rejects_oversized_or_controlled_values() {
    let oversized_command = "c".repeat(257);
    let oversized_timestamp = "2".repeat(65);
    let oversized_resource_type = "r".repeat(129);
    for invalid in [
        VALID_ACTION.replace("create_rrset", &oversized_command),
        VALID_ACTION.replace("2026-07-12T12:00:00Z", &oversized_timestamp),
        VALID_ACTION.replace("\"zone\"", &format!("\"{oversized_resource_type}\"")),
        VALID_ACTION.replace("create_rrset", "create\\u0000rrset"),
        VALID_ACTION.replace("\"zone\"", "\"zo\\u007fne\""),
    ] {
        assert!(serde_json::from_str::<ActionEnvelope<'_>>(&invalid).is_err());
    }
}

#[test]
fn every_bounded_error_field_rejects_empty_oversized_or_controlled_values() {
    let oversized_code = "c".repeat(129);
    let oversized_message = "m".repeat(16_385);
    for input in [
        r#"{"error":{"code":"","message":"x"}}"#.into(),
        r#"{"error":{"code":"forbidden","message":""}}"#.into(),
        format!(r#"{{"error":{{"code":"{oversized_code}","message":"x"}}}}"#),
        format!(r#"{{"error":{{"code":"forbidden","message":"{oversized_message}"}}}}"#),
        r#"{"error":{"code":"forbidden","message":"bad\u0000message"}}"#.into(),
    ] {
        assert!(serde_json::from_str::<ApiErrorEnvelope<'_>>(&input).is_err());
    }
}

#[test]
fn pagination_rejects_duplicates_wrong_types_and_integer_boundaries() {
    for input in [
        r#"{"meta":{"pagination":{"page":1,"page":2,"per_page":25,"previous_page":null,"next_page":null,"last_page":1,"total_entries":1}}}"#,
        r#"{"meta":{"pagination":{"page":"1","per_page":25,"previous_page":null,"next_page":null,"last_page":1,"total_entries":1}}}"#,
        r#"{"meta":{"pagination":{"page":18446744073709551616,"per_page":25,"previous_page":null,"next_page":null,"last_page":1,"total_entries":1}}}"#,
        r#"{"meta":{"pagination":{"page":1,"per_page":65536,"previous_page":null,"next_page":null,"last_page":1,"total_entries":1}}}"#,
        r#"{"meta":{"pagination":{"page":1,"per_page":25,"previous_page":null,"next_page":null,"last_page":1,"total_entries":1.5}}}"#,
    ] {
        assert!(serde_json::from_str::<PaginationEnvelope>(input).is_err());
    }
}

#[test]
fn additive_fields_remain_compatible_when_shallow_and_bounded() {
    let input = with_additive_field(r#"{"accepted":true}"#);
    assert!(serde_json::from_str::<ActionEnvelope<'_>>(&input).is_ok());
}

fn with_additive_field(value: &str) -> alloc::string::String {
    let Some(prefix) = VALID_ACTION.strip_suffix('}') else {
        return VALID_ACTION.into();
    };
    format!(r#"{prefix},"future":{value}}}"#)
}
