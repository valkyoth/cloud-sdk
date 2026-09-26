use super::*;
use crate::{
    credentials::{ApiToken, CredentialOrigin},
    discovery::{DiscoveryError as Error, tests::Fixture as _},
    identifiers::{CrateName, Version},
};
use alloc::{format, string::String, vec};
use cloud_sdk::{
    rate_limit::WallClockTimestamp,
    transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode},
};
#[cfg(feature = "blocking")]
mod execution;
#[cfg(all(feature = "blocking", feature = "async"))]
mod unified;
fn name() -> CrateName<'static> {
    CrateName::new("serde").fixture("crate")
}
fn version() -> Version<'static> {
    Version::new("1.0.0").fixture("version")
}
fn token(origin: CredentialOrigin) -> ApiToken {
    use core::sync::atomic::{AtomicU8, Ordering};
    static NEXT: AtomicU8 = AtomicU8::new(0);
    let mut bytes = [b'a'.saturating_add(NEXT.fetch_add(1, Ordering::Relaxed) % 26); 32];
    ApiToken::from_mut_bytes(origin, &mut bytes).fixture("token")
}
fn crate_response() -> serde_json::Value {
    let root: serde_json::Value =
        serde_json::from_str(include_str!("../catalog/fixtures/find_crate.json"))
            .fixture("fixture");
    serde_json::json!({"crate": root.get("crate").fixture("crate").clone()})
}
fn set(root: &mut serde_json::Value, section: &str, field: &str, value: serde_json::Value) {
    root.get_mut(section)
        .fixture("section")
        .as_object_mut()
        .fixture("object")
        .insert(String::from(field), value);
}
fn version_response() -> serde_json::Value {
    serde_json::from_str(include_str!("../versions/fixtures/find_version.json")).fixture("fixture")
}
fn decode(
    request: SettingsRequest<'_>,
    value: &serde_json::Value,
) -> Result<SettingsResponse, Error> {
    let token = token(CredentialOrigin::Production);
    let permit = request.confirm(&token);
    let wire = serde_json::to_vec(value).fixture("JSON");
    let mut bytes = vec![0xa5; wire.len()];
    let mut headers = [0xa5; 512];
    let mut response = ResponseBuffer::new(&mut bytes, wire.len(), &mut headers);
    let mut attempt = response.writer().begin_attempt().fixture("attempt");
    attempt.body_mut().fixture("body").copy_from_slice(&wire);
    attempt
        .headers_mut()
        .fixture("headers")
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .fixture("type");
    attempt
        .commit(StatusCode::OK, wire.len(), ResponseMetadata::EMPTY)
        .fixture("commit");
    drop(attempt);
    let result = match crate::wire::JsonResponsePolicy::new(StatusCode::OK, wire.len())
        .fixture("policy")
        .admit(response, WallClockTimestamp::new(0))
    {
        Ok(success) => permit.decode_response(success),
        Err(_) => Err(Error::Json),
    };
    assert!(bytes.iter().all(|v| *v == 0));
    assert!(headers.iter().all(|v| *v == 0));
    result
}
#[test]
fn exact_patch_bodies_and_atomic_targets() {
    for (request, path, expected) in [
        (
            SettingsRequest::trustpub_only(name(), true),
            "/api/v1/crates/serde",
            r#"{"crate":{"trustpub_only":true}}"#,
        ),
        (
            SettingsRequest::version(
                name(),
                version(),
                Some(true),
                YankMessage::Set("fix \"x\"\n"),
            )
            .fixture("request"),
            "/api/v1/crates/serde/1.0.0",
            r#"{"version":{"yanked":true,"yank_message":"fix \"x\"\n"}}"#,
        ),
        (
            SettingsRequest::version(name(), version(), Some(false), YankMessage::Clear)
                .fixture("request"),
            "/api/v1/crates/serde/1.0.0",
            r#"{"version":{"yanked":false,"yank_message":null}}"#,
        ),
        (
            SettingsRequest::version(name(), version(), None, YankMessage::Clear)
                .fixture("request"),
            "/api/v1/crates/serde/1.0.0",
            r#"{"version":{"yank_message":null}}"#,
        ),
    ] {
        let mut output = [0xa5; 512];
        assert_eq!(
            request.write_target(&mut output).fixture("path").as_str(),
            path
        );
        let mut small = [0xa5; 2];
        assert!(request.write_target(&mut small).is_err());
        assert_eq!(small, [0xa5; 2]);
        request
            .with_json_body(&mut output, |b| assert_eq!(b, expected.as_bytes()))
            .fixture("body");
        assert!(output.iter().all(|v| *v == 0));
        assert!(
            request
                .with_json_body(&mut small, |_| unreachable!("undersized write accepted"))
                .is_err()
        );
        assert_eq!(small, [0; 2]);
        assert_eq!(request.operation().method(), cloud_sdk::Method::Patch);
        assert!(!request.operation().permits_automatic_retry());
        assert_eq!(format!("{request:?}"), "SettingsRequest([redacted])");
    }
}
#[test]
fn message_bounds_conflicts_unicode_and_escaping() {
    assert!(
        SettingsRequest::version(name(), version(), Some(false), YankMessage::Set("")).is_err()
    );
    for text in [String::from("\0"), "x".repeat(MAX_YANK_MESSAGE_BYTES + 1)] {
        assert!(
            SettingsRequest::version(name(), version(), Some(true), YankMessage::Set(&text))
                .is_err()
        );
    }
    for text in [
        String::new(),
        "\t".repeat(MAX_YANK_MESSAGE_BYTES),
        "\u{00e5}".repeat(MAX_YANK_MESSAGE_BYTES / 2),
    ] {
        let request = SettingsRequest::version(name(), version(), None, YankMessage::Set(&text))
            .fixture("request");
        let mut out = vec![0xa5; MAX_SETTINGS_BODY_BYTES];
        request
            .with_json_body(&mut out, |body| {
                let value: serde_json::Value = serde_json::from_slice(body).fixture("body");
                let version = value.get("version").fixture("version");
                assert_eq!(version.get("yank_message").fixture("message"), &text);
                assert!(version.get("yanked").is_none());
            })
            .fixture("encode");
        assert!(out.iter().all(|v| *v == 0));
    }
}
#[test]
fn crate_postconditions_and_bounded_metadata() {
    for enabled in [true, false] {
        let mut root = crate_response();
        set(&mut root, "crate", "trustpub_only", enabled.into());
        assert!(matches!(
            decode(SettingsRequest::trustpub_only(name(), enabled), &root),
            Ok(SettingsResponse::Crate(_))
        ));
        for (field, value) in [
            ("trustpub_only", serde_json::json!(!enabled)),
            ("name", serde_json::json!("other")),
            ("id", serde_json::json!("other")),
            ("description", serde_json::json!("a".repeat(65_537))),
            ("homepage", serde_json::json!("a".repeat(4097))),
        ] {
            let mut bad = root.clone();
            set(&mut bad, "crate", field, value);
            assert!(decode(SettingsRequest::trustpub_only(name(), enabled), &bad).is_err());
        }
    }
}
#[test]
fn version_postconditions_preserve_identity_and_archived_metadata() {
    for (yanked, message) in [(true, Some("reason")), (true, None), (false, None)] {
        let mut root = version_response();
        set(&mut root, "version", "yanked", yanked.into());
        set(&mut root, "version", "yank_message", message.into());
        set(&mut root, "version", "archived", true.into());
        let request = || {
            SettingsRequest::version(
                name(),
                version(),
                Some(yanked),
                message.map(YankMessage::Set).unwrap_or(YankMessage::Clear),
            )
            .fixture("request")
        };
        let SettingsResponse::Version(record) = decode(request(), &root).fixture("response") else {
            unreachable!("variant")
        };
        assert!(
            record
                .fields()
                .required("archived")
                .fixture("archived")
                .boolean()
                .fixture("bool")
        );
        for (field, value) in [
            ("crate", serde_json::json!("other")),
            ("num", serde_json::json!("1.0.0+other")),
            ("yanked", serde_json::json!(!yanked)),
            ("yank_message", serde_json::json!("different")),
            ("id", serde_json::json!(0)),
        ] {
            let mut bad = root.clone();
            set(&mut bad, "version", field, value);
            assert!(decode(request(), &bad).is_err());
        }
        root.get_mut("version")
            .fixture("version")
            .as_object_mut()
            .fixture("object")
            .remove("yank_message");
        assert!(decode(request(), &root).is_err());
    }
    let mut root = version_response();
    set(
        &mut root,
        "version",
        "yank_message",
        "Security vulnerability".into(),
    );
    assert!(
        decode(
            SettingsRequest::version(
                name(),
                version(),
                None,
                YankMessage::Set("Security vulnerability")
            )
            .fixture("request"),
            &root
        )
        .is_err()
    );
    set(&mut root, "version", "yanked", true.into());
    assert!(
        decode(
            SettingsRequest::version(
                name(),
                version(),
                None,
                YankMessage::Set("Security vulnerability")
            )
            .fixture("request"),
            &root
        )
        .is_ok()
    );
}
