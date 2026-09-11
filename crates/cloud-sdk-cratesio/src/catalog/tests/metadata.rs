use super::*;

#[test]
fn every_include_expands_only_the_requested_sections() {
    for name in ["serde", "new"] {
        let source = value(if name == "new" { 2 } else { 1 });
        for include in [
            Include::Full,
            Include::Versions,
            Include::Keywords,
            Include::Categories,
            Include::Badges,
            Include::Downloads,
            Include::DefaultVersion,
        ] {
            let choices = [include];
            let params = [Parameter::Include(
                IncludeSet::new(&choices).fixture("include"),
            )];
            let request =
                CatalogRequest::crate_metadata(CrateName::new(name).fixture("name"), &params)
                    .fixture("request");
            let mut v = source.clone();
            for (key, wanted) in [
                (
                    "versions",
                    matches!(
                        include,
                        Include::Full | Include::Versions | Include::DefaultVersion
                    ),
                ),
                (
                    "keywords",
                    matches!(include, Include::Full | Include::Keywords),
                ),
                (
                    "categories",
                    matches!(include, Include::Full | Include::Categories),
                ),
            ] {
                if !wanted {
                    put(&mut v, "", key, Value::Null);
                }
            }
            assert!(changed(request, &v).is_ok(), "{include:?}");
            put(&mut v, "", "keywords", json!(false));
            assert!(changed(request, &v).is_err());
        }
    }
}
#[test]
fn version_expansion_preserves_future_fields_but_checks_known_schema() {
    let request = *requests().get(1).fixture("request");
    let source = value(1);
    let version = source.pointer("/versions/0").fixture("version");
    for key in version.as_object().fixture("record").keys() {
        let mut v = source.clone();
        v.pointer_mut("/versions/0")
            .fixture("version")
            .as_object_mut()
            .fixture("object")
            .remove(key);
        assert!(changed(request, &v).is_err(), "missing {key}");
    }
    let mut v = source.clone();
    put(&mut v, "/versions/0", "future", json!({"extra":true}));
    let CatalogResponse::Crate(model) = changed(request, &v).fixture("future") else {
        unreachable!("metadata");
    };
    assert!(
        model
            .versions
            .as_ref()
            .fixture("versions")
            .first()
            .fixture("version")
            .fields()
            .get("future")
            .fixture("object")
            .is_some()
    );
    for (key, bad) in [
        ("crate", json!("other")),
        ("num", json!("not-semver")),
        ("checksum", json!("no")),
        ("id", json!(0)),
        ("created_at", json!("2026-02-30T00:00:00Z")),
        ("features", json!({"feature":[false]})),
        ("audit_actions", json!([{}])),
        ("published_by", json!({})),
        ("trustpub_data", json!({"provider":"not-reviewed"})),
    ] {
        let mut v = source.clone();
        put(&mut v, "/versions/0", key, bad);
        assert!(changed(request, &v).is_err(), "{key}");
    }
}
#[test]
fn details_bind_returned_names_and_reject_malformed_crate_links() {
    let source = value(1);
    let request = *requests().get(1).fixture("request");
    for field in ["id", "name"] {
        let mut v = source.clone();
        put(&mut v, "/crate", field, json!("another"));
        assert!(matches!(changed(request, &v), Err(CatalogError::Binding)));
    }
    let mut v = source.clone();
    put(&mut v, "/crate/links", "owners", json!(false));
    assert!(changed(request, &v).is_err());
}

#[test]
fn default_version_include_cannot_substitute_another_valid_version() {
    let choices = [Include::DefaultVersion];
    let params = [Parameter::Include(
        IncludeSet::new(&choices).fixture("include"),
    )];
    let request = CatalogRequest::crate_metadata(CrateName::new("serde").fixture("name"), &params)
        .fixture("request");
    let mut v = value(1);
    put(&mut v, "", "keywords", Value::Null);
    put(&mut v, "", "categories", Value::Null);
    assert!(changed(request, &v).is_ok());
    put(&mut v, "/versions/0", "num", json!("999.0.0"));
    assert!(matches!(changed(request, &v), Err(CatalogError::Binding)));
}
