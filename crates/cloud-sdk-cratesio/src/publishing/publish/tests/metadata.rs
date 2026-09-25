use super::*;
use serde_json::{Value, json};
fn changed(field: &str, value: Value) -> Vec<u8> {
    let mut metadata: Value = serde_json::from_slice(META).fixture("JSON");
    metadata
        .as_object_mut()
        .fixture("object")
        .insert(field.into(), value);
    serde_json::to_vec(&metadata).fixture("JSON")
}
fn dependency() -> Value {
    json!({"name":"example","version_req":"^1.0", "features":[], "optional":false,
    "default_features":true, "kind":"normal", "target":null, "registry":null, "explicit_name_in_toml":null})
}
#[test]
fn full_metadata_and_renamed_target_specific_dependencies_are_validated() {
    let metadata = PublishMetadata::from_json(META).fixture("metadata");
    assert_eq!(format!("{metadata:?}"), "PublishMetadata([redacted])");
    for field in [
        "name",
        "vers",
        "deps",
        "features",
        "authors",
        "description",
        "documentation",
        "homepage",
        "readme",
        "readme_file",
        "keywords",
        "categories",
        "license",
        "license_file",
        "repository",
        "badges",
        "links",
        "rust_version",
    ] {
        assert!(metadata.fields().get(field).fixture("field").is_some());
    }
    let a = dependency();
    let mut b = a.clone();
    assert!(PublishMetadata::from_json(&changed("deps", json!([a, b]))).is_err());
    b.as_object_mut()
        .fixture("object")
        .insert("kind".into(), Value::Null);
    assert!(PublishMetadata::from_json(&changed("deps", json!([a, b]))).is_err());
    b.as_object_mut()
        .fixture("object")
        .insert("kind".into(), json!("dev"));
    assert!(PublishMetadata::from_json(&changed("deps", json!([a, b]))).is_ok());
    b.as_object_mut()
        .fixture("object")
        .insert("kind".into(), json!("normal"));
    b.as_object_mut()
        .fixture("object")
        .insert("explicit_name_in_toml".into(), json!("_renamed"));
    assert!(PublishMetadata::from_json(&changed("deps", json!([a, b]))).is_ok());
    for target in [
        "cfg(unix)",
        "cfg(all(unix, target_arch = \"x86_64\"))",
        "cfg(not(windows))",
        "cfg(any())",
        "x86_64-unknown-linux-gnu",
    ] {
        let mut dep = dependency();
        dep.as_object_mut()
            .fixture("object")
            .insert("target".into(), json!(target));
        assert!(PublishMetadata::from_json(&changed("deps", json!([dep]))).is_ok());
    }
}
#[test]
fn malformed_metadata_and_bounded_semantic_fields_fail_closed() {
    for (field, value) in [
        ("vers", json!("1.2")),
        ("name", json!("../bad")),
        ("license", json!("not-a-license")),
        ("license", json!("MIT AND")),
        ("rust_version", json!("^1.92")),
        ("rust_version", json!("1.92-beta")),
        ("readme_file", json!("../README")),
        ("license_file", json!("/LICENSE")),
        ("homepage", json!("https://token@host/")),
        ("repository", json!("javascript:alert(1)")),
        ("links", json!("lib\nfoo")),
        ("features", json!({"bad/name":[]})),
        ("features", json!({"-bad":[]})),
        ("features", json!({"ok":["dep:1bad"]})),
        ("features", json!({"ok":["dep+bad/feature"]})),
        ("features", json!({"ok":["dep:"]})),
        ("keywords", json!(["a", "b", "c", "d", "e", "f"])),
        ("future", json!(true)),
    ] {
        assert!(
            PublishMetadata::from_json(&changed(field, value)).is_err(),
            "{field}"
        );
    }
    for (field, value) in [
        ("version_req", json!("not a requirement")),
        ("target", json!("cfg(not(a,b))")),
        ("target", json!("cfg(all(unix)")),
        ("target", json!("cfg(unix)evil")),
        ("target", json!("cfg(target_os = \"\u{e9}\")")),
        ("optional", json!(1)),
        ("features", json!(["bad/path"])),
        ("explicit_name_in_toml", json!("../bad")),
    ] {
        let mut dep = dependency();
        dep.as_object_mut()
            .fixture("object")
            .insert(field.into(), value);
        assert!(PublishMetadata::from_json(&changed("deps", json!([dep]))).is_err());
    }
    assert!(PublishMetadata::from_json(br#"{"name":"a","name":"b"}"#).is_err());
    assert!(
        PublishMetadata::from_json(&vec![b' '; MAX_PUBLISH_METADATA_BYTES.saturating_add(1)])
            .is_err()
    );
    let deep = format!("cfg({}unix{})", "not(".repeat(10), ")".repeat(10));
    assert!(!super::super::target::valid(&deep));
}

#[test]
fn metadata_profile_accepts_content_and_rejects_field_bound_overruns() {
    for (field, value) in [
        ("authors", json!(["Example Author"])),
        ("homepage", json!("https://example.com/project")),
        ("readme_file", json!("docs/README.md")),
        ("license_file", json!("LICENSE.txt")),
        ("rust_version", json!("1.92.0")),
        (
            "badges",
            json!({"maintenance":{"status":"actively-developed"}}),
        ),
        (
            "features",
            json!({"default":["dep:optional", "dep/feature", "dep?/weak"]}),
        ),
        ("readme", json!("# Title\n\nText")),
        (
            "features",
            json!({"name+more":["0feature", "_private.v1", "dep:_optional"]}),
        ),
    ] {
        assert!(
            PublishMetadata::from_json(&changed(field, value)).is_ok(),
            "{field}"
        );
    }
    for (field, value) in [
        ("description", json!("x".repeat(1001))),
        ("authors", json!(vec!["a"; 65])),
        ("features", json!({"default":vec!["feature"; 301]})),
        ("readme", json!("x".repeat(65_537))),
        ("rust_version", json!("1.92.0.1")),
        ("rust_version", json!("1.092")),
        ("license", json!("MIT OR ".repeat(200))),
    ] {
        assert!(
            PublishMetadata::from_json(&changed(field, value)).is_err(),
            "{field}"
        );
    }
    let dependencies: Vec<_> = (0..257)
        .map(|i| {
            let mut d = dependency();
            d.as_object_mut()
                .fixture("object")
                .insert("name".into(), json!(format!("dep{i}")));
            d
        })
        .collect();
    assert!(PublishMetadata::from_json(&changed("deps", json!(dependencies))).is_err());
    assert!(!super::super::target::valid(&format!(
        "cfg(all({}))",
        vec!["unix"; 65].join(",")
    )));
}
