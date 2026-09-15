use super::*;
use crate::{
    discovery::tests::Fixture as _,
    endpoint::OfficialCratesIoEndpoint,
    identifiers::{CrateName, Version},
    wire::JsonResponsePolicy,
};
use alloc::{format, vec};
use cloud_sdk::{
    rate_limit::WallClockTimestamp,
    transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode},
};
use serde_json::{Value, json};

pub(super) const FIXTURES: [&str; 5] = [
    include_str!("fixtures/list_versions.json"),
    include_str!("fixtures/find_version.json"),
    include_str!("fixtures/get_version_dependencies.json"),
    r#"{"users":[],"meta":{"names":[]}}"#,
    include_str!("fixtures/get_version_readme.json"),
];
pub(super) fn requests() -> [VersionRequest<'static>; 5] {
    const PARAMETERS: &[crate::query::Parameter<'_>] = &[crate::query::Parameter::PerPage(
        crate::query::PerPage::DEFAULT,
    )];
    let name = CrateName::new("serde").fixture("name");
    let version = Version::new("1.0.0").fixture("version");
    [
        VersionRequest::list(name, PARAMETERS).fixture("list"),
        VersionRequest::detail(name, version),
        VersionRequest::dependencies(name, version),
        VersionRequest::authors(name, version),
        VersionRequest::readme(name, version),
    ]
}
fn decode(
    request: VersionRequest<'_>,
    wire: &[u8],
    media: &[u8],
    status: u16,
) -> Result<VersionResponse, VersionError> {
    let mut bytes = vec![0xa5; wire.len().checked_add(8).fixture("buffer size")];
    let mut headers = [0xa5; 512];
    let mut response = ResponseBuffer::new(&mut bytes, wire.len(), &mut headers);
    let mut attempt = response.writer().begin_attempt().fixture("attempt");
    attempt.body_mut().fixture("body").copy_from_slice(wire);
    attempt
        .headers_mut()
        .fixture("headers")
        .try_push("content-type", media, HeaderSensitivity::Public)
        .fixture("media");
    attempt
        .commit(
            StatusCode::new(status).fixture("status"),
            wire.len(),
            ResponseMetadata::EMPTY,
        )
        .fixture("commit");
    drop(attempt);
    let result = match JsonResponsePolicy::new(StatusCode::OK, wire.len())
        .fixture("policy")
        .admit(response, WallClockTimestamp::new(0))
    {
        Ok(success) => request.decode(OfficialCratesIoEndpoint::production_api(), success),
        Err(_) => Err(VersionError::Schema),
    };
    assert!(bytes.iter().all(|b| *b == 0));
    assert!(headers.iter().all(|b| *b == 0));
    result
}
fn changed(request: VersionRequest<'_>, value: &Value) -> Result<VersionResponse, VersionError> {
    decode(
        request,
        &serde_json::to_vec(value).fixture("json"),
        b"application/json",
        200,
    )
}
fn fixture(index: usize) -> Value {
    serde_json::from_str(FIXTURES.get(index).fixture("fixture")).fixture("json")
}
fn request(index: usize) -> VersionRequest<'static> {
    *requests().get(index).fixture("request")
}
fn put(root: &mut Value, parent: &str, key: &str, value: Value) {
    root.pointer_mut(parent)
        .fixture("parent")
        .as_object_mut()
        .fixture("object")
        .insert(key.into(), value);
}
#[test]
fn all_source_fixtures_decode_with_redacted_future_fields() {
    for (i, request) in requests().into_iter().enumerate() {
        let mut value = fixture(i);
        put(&mut value, "", "future", json!({"secret":"not-in-debug"}));
        let response = changed(request, &value).fixture("response");
        assert!(!format!("{response:?}").contains("not-in-debug"));
    }
}
#[test]
fn exact_paths_metadata_and_atomic_buffers() {
    let expected = [
        "/api/v1/crates/serde/versions?per_page=10",
        "/api/v1/crates/serde/1.0.0",
        "/api/v1/crates/serde/1.0.0/dependencies",
        "/api/v1/crates/serde/1.0.0/authors",
        "/api/v1/crates/serde/1.0.0/readme",
    ];
    for (request, expected) in requests().into_iter().zip(expected) {
        assert_eq!(
            request
                .operation()
                .metadata()
                .fixture("metadata")
                .retry_eligibility(),
            cloud_sdk::operation::RetryEligibility::Never
        );
        request.operation().operation_id().fixture("operation id");
        for len in 0..expected.len() {
            let mut out = vec![0xa5; len];
            assert!(request.write_target(&mut out).is_err());
            assert!(out.iter().all(|b| *b == 0xa5));
        }
        let mut out = vec![0; expected.len()];
        assert_eq!(
            request.write_target(&mut out).fixture("target").as_str(),
            expected
        );
    }
    let req = VersionRequest::detail(
        CrateName::new("serde").fixture("name"),
        Version::new("1.0.0+build.01").fixture("version"),
    );
    let mut out = [0; 128];
    assert_eq!(
        req.write_target(&mut out).fixture("target").as_str(),
        "/api/v1/crates/serde/1.0.0%2Bbuild.01"
    );
}
#[test]
fn version_schema_and_identity_are_not_lossy() {
    let base = fixture(1);
    for key in base
        .pointer("/version")
        .fixture("version")
        .as_object()
        .fixture("object")
        .keys()
    {
        let mut value = base.clone();
        value
            .pointer_mut("/version")
            .fixture("version")
            .as_object_mut()
            .fixture("object")
            .remove(key);
        assert!(
            changed(request(1), &value).is_err(),
            "missing required field"
        );
    }
    for (key, value) in [
        ("id", json!(0)),
        ("crate", json!("other")),
        ("num", json!("1.0.1")),
        ("num", json!("1.0.0+other")),
        ("num", json!("01.0.0")),
        ("checksum", json!("a".repeat(63))),
        ("checksum", json!("g".repeat(64))),
        ("created_at", json!("2026-02-30T00:00:00Z")),
        ("has_lib", json!(1)),
    ] {
        let mut changed_value = base.clone();
        put(&mut changed_value, "/version", key, value);
        assert!(changed(request(1), &changed_value).is_err());
    }
    for key in [
        "published_by",
        "trustpub_data",
        "bin_names",
        "linecounts",
        "license",
        "rust_version",
    ] {
        let mut value = base.clone();
        put(&mut value, "/version", key, Value::Null);
        assert!(changed(request(1), &value).is_ok());
    }
}
#[test]
fn dependencies_preserve_requirements_unknown_kind_and_reject_duplicate_ids() {
    for requirement in ["*", "^1", ">=1.0, <2", "=1.0.0-alpha.1", "~0.2", "1.*"] {
        semver::VersionReq::parse(requirement).fixture("requirement oracle");
        let mut value = fixture(2);
        put(&mut value, "/dependencies/0", "req", json!(requirement));
        put(&mut value, "/dependencies/0", "kind", json!("future-kind"));
        let VersionResponse::Dependencies(values) = changed(request(2), &value).fixture("deps")
        else {
            unreachable!("deps")
        };
        let record = values.first().fixture("record");
        assert_eq!(record.kind().fixture("kind"), DependencyKind::Unknown);
        assert!(
            record
                .fields()
                .get("req")
                .fixture("field")
                .fixture("req")
                .with_text(|s| s == requirement)
                .fixture("text")
        );
    }
    let mut value = fixture(2);
    let mut second = value
        .pointer("/dependencies/0")
        .fixture("dependency")
        .clone();
    value
        .pointer_mut("/dependencies")
        .fixture("deps")
        .as_array_mut()
        .fixture("array")
        .push(second.clone());
    assert!(changed(request(2), &value).is_err());
    put(&mut second, "", "id", json!(170));
    put(&mut second, "", "kind", json!("dev"));
    *value.pointer_mut("/dependencies/1").fixture("second") = second;
    assert!(changed(request(2), &value).is_ok());
    put(&mut value, "/dependencies/1", "version_id", json!(999));
    assert!(changed(request(2), &value).is_err());
    for (key, bad) in [
        ("req", json!("x".repeat(4097))),
        ("target", json!("x".repeat(4097))),
        ("features", json!(["x".repeat(257)])),
        ("target", json!(false)),
        ("id", json!(-1)),
    ] {
        let mut value = fixture(2);
        put(&mut value, "/dependencies/0", key, bad);
        assert!(changed(request(2), &value).is_err());
    }
}
#[test]
fn authors_are_empty_and_readme_is_json_not_rendered_html() {
    assert!(changed(request(3), &json!({"users":[{}],"meta":{"names":[]}})).is_err());
    assert!(changed(request(3), &json!({"users":[],"meta":{"names":["name"]}})).is_err());
    assert!(decode(request(4), b"<h1>README</h1>", b"text/html", 200).is_err());
    for status in [201, 302, 404, 500] {
        assert!(
            decode(
                request(4),
                FIXTURES.get(4).fixture("fixture").as_bytes(),
                b"application/json",
                status
            )
            .is_err()
        );
    }
    for bad in [
        Value::Null,
        json!("x".repeat(8193)),
        json!("https://example.org/\n"),
    ] {
        assert!(changed(request(4), &json!({"url":bad})).is_err());
    }
    let VersionResponse::Readme(location) = changed(request(4), &fixture(4)).fixture("readme")
    else {
        unreachable!("readme")
    };
    assert!(location.with_url(|s| s.ends_with(".html")).fixture("url"));
}
#[test]
fn duplicate_json_and_unknown_field_limits_apply_to_every_operation() {
    for request in requests() {
        for bytes in [
            br#"{"future":1,"future":2}"#.as_slice(),
            br#"{}{}"#,
            br#"{"future":{"key":1,"key":2}}"#,
        ] {
            assert!(decode(request, bytes, b"application/json", 200).is_err());
        }
    }
    let mut value = fixture(1);
    put(&mut value, "", "future", json!("x".repeat(65_537)));
    assert!(changed(request(1), &value).is_err());
}

mod pagination;
