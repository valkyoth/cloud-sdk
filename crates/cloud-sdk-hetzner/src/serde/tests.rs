use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::*;
use crate::actions::ActionStatus;
use crate::cloud::shared::CloudLabels;
use crate::dns::rrsets::{
    MAX_RECORD_VALUE_BYTES, Record, RecordComment, RecordUpdate, RecordUpdates, RecordValue,
    Records, RrsetAddRecordsRequest, RrsetCreateRequest, RrsetName, RrsetProtectionRequest,
    RrsetReference, RrsetRemoveRecordsRequest, RrsetSetRecordsRequest, RrsetTtl, RrsetTtlRequest,
    RrsetType, RrsetUpdateRecordsRequest, RrsetUpdateRequest,
};
use crate::dns::zones::{ZoneName, ZoneReference, ZoneTtl};
use crate::labels::{LabelKey, LabelValue};
use crate::response::{ApiErrorCode, ErrorCategory};
use cloud_sdk::action_polling::ActionUpdate;
use cloud_sdk_testkit::{AdversarialKind, adversarial_corpus};

macro_rules! valid {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(error) => panic!("expected valid value, got {error:?}"),
        }
    };
}

fn reference() -> RrsetReference<'static> {
    let zone = ZoneReference::Name(valid!(ZoneName::new("example.com")));
    let name = valid!(RrsetName::new("www"));
    RrsetReference::new(zone, name, RrsetType::A)
}

fn records<'a>(values: &'a [Record<'a>]) -> Records<'a> {
    valid!(Records::new(values))
}

#[test]
fn serde_rrset_bodies_match_source_locked_json_shapes() {
    let zone = ZoneReference::Name(valid!(ZoneName::new("example.com")));
    let name = valid!(RrsetName::new("www"));
    let value = valid!(RecordValue::new("192.0.2.1"));
    let comment = valid!(RecordComment::new("primary"));
    let entries = [Record::new(value).with_comment(comment)];
    let entries = records(&entries);
    let label_entries = [(
        valid!(LabelKey::new("environment")),
        valid!(LabelValue::new("prod")),
    )];
    let labels = valid!(CloudLabels::new(&label_entries));

    let create = RrsetCreateRequest::new(zone, name, RrsetType::A, entries)
        .with_ttl(RrsetTtl::InheritZoneDefault)
        .with_labels(labels);
    let body = valid!(RrsetRequestBody::create(create));
    let json = valid!(serde_json::to_string(&body));
    assert_eq!(
        json,
        r#"{"name":"www","type":"A","ttl":null,"records":[{"value":"192.0.2.1","comment":"primary"}],"labels":{"environment":"prod"}}"#
    );
    assert!(json.len() <= body.size_upper_bound());
    assert!(format!("{body:?}").contains("[redacted]"));

    let empty_labels = valid!(CloudLabels::new(&[]));
    let update = valid!(RrsetRequestBody::update(
        RrsetUpdateRequest::new(reference()).with_labels(empty_labels)
    ));
    assert_eq!(valid!(serde_json::to_string(&update)), r#"{"labels":{}}"#);

    let protection = valid!(RrsetRequestBody::protection(RrsetProtectionRequest::new(
        reference(),
        true
    )));
    assert_eq!(
        valid!(serde_json::to_string(&protection)),
        r#"{"change":true}"#
    );

    let ttl = valid!(ZoneTtl::new(3600));
    let change_ttl = valid!(RrsetRequestBody::change_ttl(RrsetTtlRequest::new(
        reference(),
        RrsetTtl::Explicit(ttl)
    )));
    assert_eq!(
        valid!(serde_json::to_string(&change_ttl)),
        r#"{"ttl":3600}"#
    );

    let set = valid!(RrsetRequestBody::set_records(RrsetSetRecordsRequest::new(
        reference(),
        entries
    )));
    assert_eq!(
        valid!(serde_json::to_string(&set)),
        r#"{"records":[{"value":"192.0.2.1","comment":"primary"}]}"#
    );

    let add = valid!(RrsetRequestBody::add_records(
        RrsetAddRecordsRequest::new(reference(), entries).with_ttl(RrsetTtl::InheritZoneDefault)
    ));
    assert_eq!(
        valid!(serde_json::to_string(&add)),
        r#"{"records":[{"value":"192.0.2.1","comment":"primary"}],"ttl":null}"#
    );

    let remove = valid!(RrsetRequestBody::remove_records(
        RrsetRemoveRecordsRequest::new(reference(), entries)
    ));
    assert_eq!(
        valid!(serde_json::to_string(&remove)),
        r#"{"records":[{"value":"192.0.2.1","comment":"primary"}]}"#
    );

    let updates = [RecordUpdate::new(value, comment)];
    let updates = valid!(RecordUpdates::new(&updates));
    let update_records = valid!(RrsetRequestBody::update_records(
        RrsetUpdateRecordsRequest::new(reference(), updates)
    ));
    assert_eq!(
        valid!(serde_json::to_string(&update_records)),
        r#"{"records":[{"value":"192.0.2.1","comment":"primary"}]}"#
    );

    let unicode = valid!(RecordValue::new("quoted \"snowman \u{2603}\""));
    let unicode_records = [Record::new(unicode)];
    let unicode_records = records(&unicode_records);
    let unicode_body = valid!(RrsetRequestBody::set_records(RrsetSetRecordsRequest::new(
        reference(),
        unicode_records
    )));
    let unicode_json = valid!(serde_json::to_string(&unicode_body));
    assert!(unicode_json.len() <= unicode_body.size_upper_bound());
}

#[test]
fn serde_rrset_body_limit_is_checked_before_serialization() {
    let mut values: Vec<String> = Vec::new();
    for suffix in b'a'..=b'i' {
        let mut value = "\\".repeat(MAX_RECORD_VALUE_BYTES - 1);
        value.push(char::from(suffix));
        values.push(value);
    }

    let mut record_entries = Vec::new();
    for value in &values {
        let value = valid!(RecordValue::new(value));
        record_entries.push(Record::new(value));
    }
    let records = valid!(Records::new(&record_entries));
    assert!(matches!(
        RrsetRequestBody::set_records(RrsetSetRecordsRequest::new(reference(), records)),
        Err(RrsetBodyError::BodyTooLarge)
    ));
}

#[test]
fn serde_error_envelope_borrows_and_ignores_additive_fields() {
    let json = r#"{
        "error": {
            "code": "rate_limit_exceeded",
            "message": "slow \"down\"",
            "details": {"retry_after": 1},
            "future": true
        },
        "request_id": "abc"
    }"#;
    let envelope: ApiErrorEnvelope<'_> = valid!(serde_json::from_str(json));
    assert_eq!(envelope.error().code(), ApiErrorCode::RateLimitExceeded);
    assert_eq!(envelope.error().code().category(), ErrorCategory::RateLimit);
    assert_eq!(envelope.error().message(), "slow \"down\"");
    let debug = format!("{envelope:?}");
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("slow"));
    let display = format!("{}", envelope.error());
    assert_eq!(display, "Hetzner API returned an error response");
    assert!(!display.contains("slow"));

    let duplicate = r#"{"error":{"code":"forbidden","code":"not_found","message":"x"}}"#;
    assert!(serde_json::from_str::<ApiErrorEnvelope<'_>>(duplicate).is_err());
    let missing = r#"{"error":{"code":"forbidden"}}"#;
    assert!(serde_json::from_str::<ApiErrorEnvelope<'_>>(missing).is_err());

    let long_message = "x".repeat(MAX_API_ERROR_MESSAGE_BYTES.saturating_add(1));
    let oversized = format!(r#"{{"error":{{"code":"forbidden","message":"{long_message}"}}}}"#);
    assert!(serde_json::from_str::<ApiErrorEnvelope<'_>>(&oversized).is_err());
}

#[test]
fn serde_action_envelope_validates_security_relevant_fields() {
    let json = r#"{
        "action": {
            "id": 42,
            "command": "create_\u0072rset",
            "status": "running",
            "progress": 50,
            "started": "2026-07-12T12:00:00Z",
            "finished": null,
            "resources": [{"id": 7, "type": "zone", "future": true}],
            "error": null,
            "future": "accepted"
        },
        "future": true
    }"#;
    let envelope: ActionEnvelope<'_> = valid!(serde_json::from_str(json));
    assert_eq!(envelope.action().id().get(), 42);
    assert_eq!(envelope.action().command(), "create_rrset");
    assert_eq!(envelope.action().status(), ActionStatus::Running);
    assert_eq!(envelope.action().polling_update(), ActionUpdate::Running);
    assert_eq!(envelope.action().progress(), 50);
    assert_eq!(envelope.action().resources().len(), 1);
    let resource = envelope.action().resources().first();
    assert!(resource.is_some());
    let Some(resource) = resource else {
        return;
    };
    assert_eq!(resource.id().get(), 7);
    assert_eq!(resource.resource_type(), "zone");

    for invalid in [
        json.replace(r#""id": 42"#, r#""id": 0"#),
        json.replace(r#""status": "running""#, r#""status": "future""#),
        json.replace(r#""progress": 50"#, r#""progress": 101"#),
        json.replace(r#""id": 7"#, r#""id": 0"#),
    ] {
        assert!(serde_json::from_str::<ActionEnvelope<'_>>(&invalid).is_err());
    }

    let duplicate = json.replace(
        r#""status": "running""#,
        r#""status": "running", "status": "success""#,
    );
    assert!(serde_json::from_str::<ActionEnvelope<'_>>(&duplicate).is_err());
    let missing = json.replace(r#""status": "running","#, "");
    assert!(serde_json::from_str::<ActionEnvelope<'_>>(&missing).is_err());
    let missing_finished = json.replace(r#""finished": null,"#, "");
    assert!(serde_json::from_str::<ActionEnvelope<'_>>(&missing_finished).is_err());
    let missing_error = json.replace(r#""error": null,"#, "");
    assert!(serde_json::from_str::<ActionEnvelope<'_>>(&missing_error).is_err());
    let controlled = json.replace(r#""type": "zone""#, r#""type": "zo\u0000ne""#);
    assert!(serde_json::from_str::<ActionEnvelope<'_>>(&controlled).is_err());

    let resources =
        vec![r#"{"id":7,"type":"zone"}"#; MAX_ACTION_RESPONSE_RESOURCES.saturating_add(1)]
            .join(",");
    let oversized = format!(
        r#"{{"action":{{"id":42,"command":"test","status":"running","progress":1,"started":"2026-07-12T12:00:00Z","finished":null,"resources":[{resources}],"error":null}}}}"#
    );
    assert!(serde_json::from_str::<ActionEnvelope<'_>>(&oversized).is_err());
}

#[test]
fn serde_action_error_is_classified_without_exposing_raw_unknown_code() {
    let json = r#"{
        "action": {
            "id": 42,
            "command": "change_ttl",
            "status": "error",
            "progress": 100,
            "started": "2026-07-12T12:00:00Z",
            "finished": "2026-07-12T12:00:01Z",
            "resources": [],
            "error": {"code": "future_error", "message": "failed"}
        }
    }"#;
    let envelope: ActionEnvelope<'_> = valid!(serde_json::from_str(json));
    let error = envelope.action().error();
    assert!(error.is_some());
    let Some(error) = error else {
        return;
    };
    assert_eq!(error.code(), ApiErrorCode::Unknown);
    assert_eq!(error.message(), "failed");
    assert_eq!(
        envelope.action().polling_update(),
        ActionUpdate::Failed(Some(error))
    );
}

#[test]
fn serde_response_bytes_are_bounded_before_parser_use() {
    let secret = ResponseBytes::new(b"secret response");
    assert!(secret.is_ok());
    if let Ok(secret) = secret {
        let debug = format!("{secret:?}");
        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains("secret response"));
    }

    let accepted = vec![0_u8; MAX_SERDE_RESPONSE_BYTES];
    let admitted = ResponseBytes::new(&accepted);
    assert!(admitted.is_ok());
    assert_eq!(
        admitted.map(ResponseBytes::as_slice),
        Ok(accepted.as_slice())
    );

    let rejected = vec![0_u8; MAX_SERDE_RESPONSE_BYTES.saturating_add(1)];
    assert_eq!(
        ResponseBytes::new(&rejected),
        Err(ResponseSizeError::TooLarge)
    );
}

#[test]
fn serde_boundary_consumes_provider_neutral_adversarial_corpus() {
    let corpus = valid!(adversarial_corpus());
    for fixture in corpus {
        match fixture.kind() {
            AdversarialKind::MalformedJson | AdversarialKind::MissingRequiredFields => {
                let bytes = valid!(fixture.body().as_bytes().ok_or(ResponseSizeError::TooLarge));
                assert!(serde_json::from_slice::<ActionEnvelope<'_>>(bytes).is_err());
            }
            AdversarialKind::UnknownFields => {
                let bytes = valid!(fixture.body().as_bytes().ok_or(ResponseSizeError::TooLarge));
                assert!(serde_json::from_slice::<ActionEnvelope<'_>>(bytes).is_ok());
            }
            AdversarialKind::OversizedResponse => {
                let bytes = vec![0_u8; fixture.body().len()];
                assert_eq!(ResponseBytes::new(&bytes), Err(ResponseSizeError::TooLarge));
            }
            AdversarialKind::InvalidPagination => {
                let bytes = valid!(fixture.body().as_bytes().ok_or(ResponseSizeError::TooLarge));
                assert!(serde_json::from_slice::<serde_json::Value>(bytes).is_ok());
            }
            AdversarialKind::InvalidActionState => {
                let bytes = valid!(fixture.body().as_bytes().ok_or(ResponseSizeError::TooLarge));
                assert!(serde_json::from_slice::<ActionEnvelope<'_>>(bytes).is_err());
            }
        }
    }
}
