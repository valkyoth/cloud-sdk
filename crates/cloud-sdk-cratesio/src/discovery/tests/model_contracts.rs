use super::*;

#[test]
fn summary_nullable_fields_accept_typed_values_and_reject_other_types() {
    let source: Value = serde_json::from_str(FIXTURES.get(6).fixture("summary")).fixture("JSON");
    let fields = [
        ("versions", json!([42]), json!([0])),
        ("keywords", json!(["http"]), json!([null])),
        ("categories", json!(["game-development"]), json!(42)),
        ("recent_downloads", json!(123), json!(1.5)),
        ("default_version", json!("1.0.0"), json!(false)),
        ("max_stable_version", json!("1.0.0"), json!([])),
        ("description", json!("description"), json!({})),
        ("homepage", json!("https://example.org"), json!(1)),
        (
            "documentation",
            json!("https://example.org/docs"),
            json!([]),
        ),
        (
            "repository",
            json!("https://example.org/repo"),
            json!(false),
        ),
    ];
    for (name, valid, invalid) in fields {
        for value in [Value::Null, valid] {
            let mut root = source.clone();
            put(&mut root, "/new_crates/0", name, value);
            assert!(
                changed(DiscoveryRequest::summary(), &root).is_ok(),
                "{name}"
            );
        }
        let mut root = source.clone();
        put(&mut root, "/new_crates/0", name, invalid);
        assert!(
            changed(DiscoveryRequest::summary(), &root).is_err(),
            "{name}"
        );
    }
    for name in [
        "versions",
        "version_downloads",
        "owners",
        "owner_team",
        "owner_user",
        "reverse_dependencies",
    ] {
        let mut root = source.clone();
        root.pointer_mut("/new_crates/0/links")
            .fixture("links")
            .as_object_mut()
            .fixture("object")
            .remove(name);
        assert!(
            changed(DiscoveryRequest::summary(), &root).is_err(),
            "{name}"
        );
        let mut root = source.clone();
        put(&mut root, "/new_crates/0/links", name, Value::Null);
        assert_eq!(
            changed(DiscoveryRequest::summary(), &root).is_ok(),
            name == "versions"
        );
    }
}

#[test]
fn discovery_records_bind_details_and_preserve_counts_and_timestamps() {
    let mut value: Value = serde_json::from_str(FIXTURES.get(4).fixture("keyword")).fixture("JSON");
    put(
        &mut value,
        "/keyword",
        "created_at",
        json!("2026-09-10T12:34:56.123-02:00"),
    );
    put(&mut value, "/keyword", "crates_cnt", json!(123));
    let request = *requests().get(4).fixture("request");
    let DiscoveryResponse::Keyword(record) = changed(request, &value).fixture("keyword") else {
        unreachable!("wrong operation model");
    };
    assert_eq!(record.keyword, "http");
    assert_eq!(record.crates_cnt, 123);
    assert_eq!(record.created_at.as_str(), "2026-09-10T12:34:56.123-02:00");
    put(&mut value, "/keyword", "keyword", json!("different"));
    assert!(matches!(
        changed(request, &value),
        Err(DiscoveryError::Binding)
    ));
    let mut value: Value =
        serde_json::from_str(FIXTURES.get(1).fixture("category")).fixture("JSON");
    put(&mut value, "/category", "slug", json!("different"));
    assert!(matches!(
        changed(*requests().get(1).fixture("request"), &value),
        Err(DiscoveryError::Binding)
    ));
}

#[test]
fn taxonomy_required_field_matrix_and_wrong_types_fail_closed() {
    for (index, pointer) in [
        (0, "/categories/0"),
        (1, "/category"),
        (2, "/category_slugs/0"),
        (3, "/keywords/0"),
        (4, "/keyword"),
    ] {
        let root: Value =
            serde_json::from_str(FIXTURES.get(index).fixture("source")).fixture("JSON");
        let object = root
            .pointer(pointer)
            .fixture("record")
            .as_object()
            .fixture("object");
        for key in object.keys() {
            let mut missing = root.clone();
            missing
                .pointer_mut(pointer)
                .fixture("record")
                .as_object_mut()
                .fixture("object")
                .remove(key);
            assert!(
                changed(*requests().get(index).fixture("request"), &missing).is_err(),
                "{key}"
            );
            let mut invalid = root.clone();
            put(&mut invalid, pointer, key, Value::Null);
            assert!(
                changed(*requests().get(index).fixture("request"), &invalid).is_err(),
                "{key}"
            );
        }
    }
}

#[test]
fn bounded_unknown_object_and_value_structure_cannot_bypass_limits() {
    let source: Value = serde_json::from_str(FIXTURES.get(5).fixture("site")).fixture("JSON");
    for count in [64, 65] {
        let mut root = source.clone();
        let mut fields = serde_json::Map::new();
        for i in 0..count {
            fields.insert(format!("key{i}"), Value::Null);
        }
        put(&mut root, "", "future", Value::Object(fields));
        assert_eq!(
            changed(DiscoveryRequest::site_metadata(), &root).is_ok(),
            count == 64
        );
    }
    for length in [256, 257] {
        let mut root = source.clone();
        put(&mut root, "", &"k".repeat(length), Value::Null);
        assert_eq!(
            changed(DiscoveryRequest::site_metadata(), &root).is_ok(),
            length == 256
        );
    }
    for count in [15, 16] {
        let mut root = source.clone();
        put(
            &mut root,
            "",
            "future",
            json!(vec![vec![Value::Null; 1024]; count]),
        );
        assert_eq!(
            changed(DiscoveryRequest::site_metadata(), &root).is_ok(),
            count == 15
        );
    }
}
