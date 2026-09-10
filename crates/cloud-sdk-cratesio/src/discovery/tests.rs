use super::*;
use crate::{
    endpoint::OfficialCratesIoEndpoint,
    query::{Page, Parameter, PerPage, Seek},
    wire::{CratesIoWireError, JsonResponsePolicy},
};
use alloc::{format, vec};
use cloud_sdk::{
    rate_limit::WallClockTimestamp,
    transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode},
};
use serde_json::{Value, json};

mod model_contracts;

pub(crate) trait Fixture<T> {
    fn fixture(self, message: &str) -> T;
}
impl<T, E> Fixture<T> for Result<T, E> {
    fn fixture(self, message: &str) -> T {
        self.unwrap_or_else(|_| unreachable!("{message}"))
    }
}
impl<T> Fixture<T> for Option<T> {
    fn fixture(self, message: &str) -> T {
        self.unwrap_or_else(|| unreachable!("{message}"))
    }
}

pub(super) const FIXTURES: [&str; 7] = [
    include_str!("fixtures/list_categories.json"),
    include_str!("fixtures/find_category.json"),
    include_str!("fixtures/list_category_slugs.json"),
    include_str!("fixtures/list_keywords.json"),
    include_str!("fixtures/find_keyword.json"),
    include_str!("fixtures/get_site_metadata.json"),
    include_str!("fixtures/get_summary.json"),
];
pub(super) fn requests() -> [DiscoveryRequest<'static>; 7] {
    [
        DiscoveryRequest::categories(&[]).fixture("list"),
        DiscoveryRequest::category(
            crate::identifiers::CategorySlug::new("game-development").fixture("slug"),
        ),
        DiscoveryRequest::category_slugs(),
        DiscoveryRequest::keywords(&[]).fixture("list"),
        DiscoveryRequest::keyword(crate::identifiers::Keyword::new("http").fixture("keyword")),
        DiscoveryRequest::site_metadata(),
        DiscoveryRequest::summary(),
    ]
}
fn decode(request: DiscoveryRequest<'_>, wire: &[u8]) -> Result<DiscoveryResponse, DiscoveryError> {
    let mut bytes = vec![0xa5; wire.len().saturating_add(8)];
    let mut headers = [0xa5; 512];
    let mut response = ResponseBuffer::new(&mut bytes, wire.len(), &mut headers);
    let mut attempt = response.writer().begin_attempt().fixture("attempt");
    attempt.body_mut().fixture("body").copy_from_slice(wire);
    attempt
        .headers_mut()
        .fixture("headers")
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .fixture("header");
    attempt
        .commit(StatusCode::OK, wire.len(), ResponseMetadata::EMPTY)
        .fixture("commit");
    drop(attempt);
    let result = {
        let admitted = JsonResponsePolicy::new(StatusCode::OK, wire.len())
            .fixture("policy")
            .admit(response, WallClockTimestamp::new(0));
        match admitted {
            Ok(success) => request.decode(OfficialCratesIoEndpoint::production_api(), success),
            Err(CratesIoWireError::Json | CratesIoWireError::Envelope) => Err(DiscoveryError::Json),
            Err(_) => Err(DiscoveryError::Schema),
        }
    };
    assert!(bytes.iter().all(|v| *v == 0));
    assert!(headers.iter().all(|v| *v == 0));
    result
}
fn changed(
    request: DiscoveryRequest<'_>,
    value: &Value,
) -> Result<DiscoveryResponse, DiscoveryError> {
    decode(request, &serde_json::to_vec(value).fixture("fixture JSON"))
}

#[test]
fn all_seven_source_fixtures_and_unknown_fields_decode() {
    for (request, fixture) in requests().into_iter().zip(FIXTURES) {
        let response = decode(request, fixture.as_bytes()).fixture("source fixture");
        assert!(!format!("{response:?}").contains("Libraries for creating"));
        let mut value: Value = serde_json::from_str(fixture).fixture("fixture");
        put(
            &mut value,
            "",
            "future",
            json!({"secret":"not-for-debug","nested":[true,null,{"number":3.5}]}),
        );
        assert!(changed(request, &value).is_ok());
        assert!(!format!("{:?}", changed(request, &value)).contains("not-for-debug"));
    }
}

#[test]
fn all_required_root_fields_and_nullability_are_checked() {
    for (request, fixture) in requests().into_iter().zip(FIXTURES) {
        let value: Value = serde_json::from_str(fixture).fixture("fixture");
        for key in value.as_object().fixture("object").keys() {
            let mut missing = value.clone();
            missing.as_object_mut().fixture("object").remove(key);
            assert!(changed(request, &missing).is_err(), "missing {key}");
            let mut null = value.clone();
            put(&mut null, "", key, Value::Null);
            assert!(changed(request, &null).is_err(), "null {key}");
        }
    }
}

#[test]
fn full_summary_required_nullable_fields_and_badges_are_retained() {
    let mut root: Value =
        serde_json::from_str(FIXTURES.get(6).fixture("fixture index")).fixture("fixture");
    let record = root.pointer("/new_crates/0").fixture("fixture path");
    for key in record.as_object().fixture("record").keys() {
        let mut missing = root.clone();
        missing
            .pointer_mut("/new_crates/0")
            .fixture("fixture path")
            .as_object_mut()
            .fixture("record")
            .remove(key);
        assert!(
            changed(DiscoveryRequest::summary(), &missing).is_err(),
            "missing crate {key}"
        );
    }
    put(
        &mut root,
        "/new_crates/0",
        "badges",
        json!([{"future":{"text":"untrusted","n":1.25}}]),
    );
    let response = changed(DiscoveryRequest::summary(), &root).fixture("badge");
    let DiscoveryResponse::Summary(summary) = response else {
        unreachable!("summary");
    };
    let badge = summary
        .new_crates
        .first()
        .fixture("record")
        .badges
        .first()
        .fixture("badge");
    assert_eq!(
        badge
            .get("future")
            .fixture("object")
            .fixture("future")
            .get("text")
            .fixture("object")
            .fixture("text")
            .with_text(|v| v == "untrusted"),
        Ok(true)
    );
    put(&mut root, "/new_crates/0", "recent_downloads", json!(-1));
    assert!(changed(DiscoveryRequest::summary(), &root).is_err());
}

#[test]
fn timestamps_accept_offsets_and_reject_invalid_calendars() {
    let mut value: Value =
        serde_json::from_str(FIXTURES.get(4).fixture("fixture index")).fixture("fixture");
    for text in [
        "2024-02-29T23:59:59Z",
        "2026-09-10T12:00:00.123456789+02:00",
        "2026-09-10t12:00:00z",
    ] {
        put(&mut value, "/keyword", "created_at", json!(text));
        assert!(
            changed(*requests().get(4).fixture("request index"), &value).is_ok(),
            "{text}"
        );
    }
    for text in [
        "2026-02-29T00:00:00Z",
        "2026-02-30T00:00:00Z",
        "0000-01-01T00:00:00Z",
        "2026-01-01T25:00:00Z",
        "2026-01-01T00:00:60Z",
        "2026-01-01T00:00:00+24:00",
        "2026-01-01T00:00:00+00:60",
        "2026-01-01T00:00:00.1234567890Z",
        "2026-01-01T00:00:00Z\n",
    ] {
        put(&mut value, "/keyword", "created_at", json!(text));
        assert!(
            changed(*requests().get(4).fixture("request index"), &value).is_err(),
            "{text}"
        );
    }
}

#[test]
fn optional_taxonomy_and_banner_fields_preserve_omission_but_reject_null() {
    let mut category: Value =
        serde_json::from_str(FIXTURES.get(1).fixture("fixture index")).fixture("fixture");
    put(&mut category, "/category", "subcategories", json!([]));
    put(&mut category, "/category", "parent_categories", json!([]));
    assert!(changed(*requests().get(1).fixture("request index"), &category).is_ok());
    put(&mut category, "/category", "subcategories", Value::Null);
    assert!(changed(*requests().get(1).fixture("request index"), &category).is_err());
    let mut site: Value =
        serde_json::from_str(FIXTURES.get(5).fixture("fixture index")).fixture("fixture");
    put(&mut site, "", "banner_message", json!("<untrusted>"));
    assert!(changed(*requests().get(5).fixture("request index"), &site).is_ok());
    put(&mut site, "", "banner_message", Value::Null);
    assert!(changed(*requests().get(5).fixture("request index"), &site).is_err());
}

#[test]
fn limits_apply_to_known_and_unknown_fields() {
    let mut root: Value =
        serde_json::from_str(FIXTURES.get(6).fixture("fixture index")).fixture("fixture");
    let record = root.pointer("/new_crates/0").fixture("crate").clone();
    put(&mut root, "", "new_crates", json!(vec![record; 11]));
    assert!(changed(DiscoveryRequest::summary(), &root).is_err());
    let mut site: Value =
        serde_json::from_str(FIXTURES.get(5).fixture("fixture index")).fixture("fixture");
    put(&mut site, "", "future", json!(vec![null_value(); 1025]));
    assert!(changed(DiscoveryRequest::site_metadata(), &site).is_err());
    put(&mut site, "", "future", json!("x".repeat(65_537)));
    assert!(changed(DiscoveryRequest::site_metadata(), &site).is_err());
    put(&mut site, "", "future", json!("x".repeat(65_536)));
    assert!(changed(DiscoveryRequest::site_metadata(), &site).is_ok());
    let mut category: Value =
        serde_json::from_str(FIXTURES.get(1).fixture("fixture index")).fixture("fixture");
    let base = category
        .pointer_mut("/category")
        .fixture("fixture path")
        .clone();
    let mut nested = base.clone();
    for _ in 0..9 {
        let mut parent = base.clone();
        put(&mut parent, "", "subcategories", json!([nested]));
        nested = parent;
    }
    put(&mut category, "", "category", nested);
    assert!(changed(*requests().get(1).fixture("request index"), &category).is_err());
}
fn null_value() -> Value {
    Value::Null
}

#[test]
fn duplicates_and_trailing_json_cannot_enter_models() {
    for input in [
        br#"{"read_only":false,"read_only":true}"#.as_slice(),
        br#"{"future":{"key":1,"key":2}}"#,
        br#"{}{}"#,
        br#"{"future":1e9999999}"#,
    ] {
        assert!(decode(DiscoveryRequest::site_metadata(), input).is_err());
    }
}

#[test]
fn page_counts_and_links_cannot_change_origin_or_filters() {
    let params = [Parameter::PerPage(PerPage::new(1).fixture("page size"))];
    let request = DiscoveryRequest::keywords(&params).fixture("query");
    let mut value: Value =
        serde_json::from_str(FIXTURES.get(3).fixture("fixture index")).fixture("fixture");
    let DiscoveryResponse::Keywords(page) = changed(request, &value).fixture("page") else {
        unreachable!("keywords");
    };
    assert_eq!(
        page.next,
        PageContinuation::Page(Page::new(2).fixture("page"))
    );
    put(
        &mut value,
        "/meta",
        "next_page",
        json!("?page=2&per_page=1"),
    );
    assert!(changed(request, &value).is_ok());
    for link in [
        "https://other.example/api/v1/keywords?page=2&per_page=1",
        "?page=3&per_page=1",
        "?page=2&per_page=100",
        "?page=2&per_page=1&sort=crates",
        "?page=2&page=2&per_page=1",
    ] {
        put(&mut value, "/meta", "next_page", json!(link));
        assert!(changed(request, &value).is_err());
    }
    value
        .pointer_mut("/meta")
        .fixture("fixture path")
        .as_object_mut()
        .fixture("meta")
        .remove("next_page");
    let params = [Parameter::Page(Page::new(10).fixture("page"))];
    let request = DiscoveryRequest::keywords(&params).fixture("query");
    let DiscoveryResponse::Keywords(page) = changed(request, &value).fixture("page") else {
        unreachable!("keywords");
    };
    assert_eq!(page.next, PageContinuation::LimitReached);
    assert!(
        DiscoveryRequest::keywords(&[Parameter::Seek(Seek::new("abc").fixture("seek"))]).is_err()
    );
}

#[test]
fn requests_are_atomic_gets_with_exact_metadata() {
    let expected = [
        "/api/v1/categories",
        "/api/v1/categories/game-development",
        "/api/v1/category_slugs",
        "/api/v1/keywords",
        "/api/v1/keywords/http",
        "/api/v1/site_metadata",
        "/api/v1/summary",
    ];
    for (request, expected) in requests().into_iter().zip(expected) {
        assert_eq!(
            request.operation().metadata().fixture("metadata").impact(),
            cloud_sdk::operation::OperationImpact::ReadOnly
        );
        assert_eq!(
            request
                .operation()
                .metadata()
                .fixture("metadata")
                .retry_eligibility(),
            cloud_sdk::operation::RetryEligibility::Never
        );
        for length in 0..expected.len() {
            let mut buffer = vec![0xa5; length];
            assert!(request.write_target(&mut buffer).is_err());
            assert!(buffer.iter().all(|v| *v == 0xa5));
        }
        let mut buffer = vec![0; expected.len()];
        assert_eq!(
            request.write_target(&mut buffer).fixture("target").as_str(),
            expected
        );
    }
}

fn put(root: &mut Value, parent: &str, key: &str, value: Value) {
    root.pointer_mut(parent)
        .fixture("fixture parent")
        .as_object_mut()
        .fixture("fixture object")
        .insert(key.into(), value);
}
