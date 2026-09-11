use super::*;
use crate::{
    discovery::tests::Fixture as _,
    endpoint::OfficialCratesIoEndpoint,
    identifiers::CrateName,
    query::{Include, IncludeSet, Page, Parameter, PerPage, SearchQuery, Seek, Sort},
    wire::JsonResponsePolicy,
};
use alloc::{format, vec};
use cloud_sdk::{
    rate_limit::WallClockTimestamp,
    transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode},
};
use serde_json::{Value, json};
mod metadata;
mod pagination;

pub(super) const FIXTURES: [&str; 3] = [
    include_str!("fixtures/list_crates.json"),
    include_str!("fixtures/find_crate.json"),
    include_str!("fixtures/find_new_crate.json"),
];
pub(super) fn requests() -> [CatalogRequest<'static>; 3] {
    [
        CatalogRequest::list(&[]).fixture("list"),
        CatalogRequest::crate_metadata(CrateName::new("serde").fixture("name"), &[])
            .fixture("detail"),
        CatalogRequest::new_crate(&[]).fixture("new"),
    ]
}
fn value(index: usize) -> Value {
    serde_json::from_str(FIXTURES.get(index).fixture("fixture index")).fixture("JSON")
}
fn decode(request: CatalogRequest<'_>, data: &[u8]) -> Result<CatalogResponse, CatalogError> {
    let mut body = vec![0xa5; data.len().saturating_add(8)];
    let mut headers = [0xa5; 512];
    let mut response = ResponseBuffer::new(&mut body, data.len(), &mut headers);
    let mut attempt = response.writer().begin_attempt().fixture("attempt");
    attempt.body_mut().fixture("body").copy_from_slice(data);
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
        .commit(StatusCode::OK, data.len(), ResponseMetadata::EMPTY)
        .fixture("commit");
    drop(attempt);
    let result = {
        let success = JsonResponsePolicy::new(StatusCode::OK, data.len())
            .fixture("policy")
            .admit(response, WallClockTimestamp::new(0));
        match success {
            Ok(success) => request.decode(OfficialCratesIoEndpoint::production_api(), success),
            Err(_) => Err(CatalogError::Json),
        }
    };
    assert!(body.iter().all(|v| *v == 0));
    assert!(headers.iter().all(|v| *v == 0));
    result
}
fn changed(request: CatalogRequest<'_>, value: &Value) -> Result<CatalogResponse, CatalogError> {
    decode(
        request,
        &serde_json::to_vec(value).fixture("encode fixture"),
    )
}
fn put(root: &mut Value, path: &str, name: &str, value: Value) {
    root.pointer_mut(path)
        .fixture("path")
        .as_object_mut()
        .fixture("object")
        .insert(name.into(), value);
}

#[test]
fn all_source_fixtures_decode_and_unknown_fields_are_bounded() {
    for (request, fixture) in requests().into_iter().zip(FIXTURES) {
        let model = decode(request, fixture.as_bytes()).fixture("source fixture");
        assert!(!format!("{model:?}").contains("serialization"));
        let mut v: Value = serde_json::from_str(fixture).fixture("fixture");
        put(
            &mut v,
            "",
            "future",
            json!({"nested":[1,true,null,"untrusted"]}),
        );
        assert!(changed(request, &v).is_ok());
        put(&mut v, "", "future", json!("x".repeat(65_537)));
        assert!(matches!(changed(request, &v), Err(CatalogError::Limit)));
    }
}
#[test]
fn request_routes_metadata_atomicity_and_search_bounds() {
    for ((request, operation), target) in requests()
        .into_iter()
        .zip(["list_crates", "find_crate", "find_new_crate"])
        .zip([
            "/api/v1/crates",
            "/api/v1/crates/serde",
            "/api/v1/crates/new",
        ])
    {
        assert_eq!(request.operation().operation_name(), operation);
        assert_eq!(
            request
                .operation()
                .metadata()
                .fixture("metadata")
                .retry_eligibility(),
            cloud_sdk::operation::RetryEligibility::Never
        );
        for length in 0..target.len() {
            let mut bytes = vec![0xa5; length];
            assert!(request.write_target(&mut bytes).is_err());
            assert!(bytes.iter().all(|v| *v == 0xa5));
        }
        assert_eq!(
            request
                .write_target(&mut [0; 4096])
                .fixture("target")
                .as_str(),
            target
        );
    }
    assert_eq!(
        CatalogRequest::crate_metadata(CrateName::new("new").fixture("name"), &[])
            .fixture("request")
            .operation(),
        CatalogOperation::NewCrate
    );
    assert!(SearchQuery::new("").is_err());
    let text = "x".repeat(1024);
    let params = [Parameter::Search(
        SearchQuery::new(&text).fixture("max query"),
    )];
    assert!(
        CatalogRequest::cargo_search(&params)
            .fixture("search")
            .write_target(&mut [0; 4096])
            .is_ok()
    );
    assert!(SearchQuery::new(&"x".repeat(1025)).is_err());
    assert!(CatalogRequest::cargo_search(&[Parameter::Following]).is_err());
}
#[test]
fn stable_cargo_search_minimum_and_extensions_do_not_require_website_fields() {
    let request = CatalogRequest::cargo_search(&[]).fixture("Cargo search");
    let mut v = json!({"crates":[{"name":"rand", "max_version":"0.6.1", "description":"Random"}], "meta":{"total":119}});
    let CatalogResponse::CargoSearch(page) = changed(request, &v).fixture("stable Cargo fixture")
    else {
        unreachable!("profile");
    };
    assert_eq!(page.total, 119);
    assert_eq!(page.items.first().fixture("item").name, "rand");
    assert!(changed(CatalogRequest::list(&[]).fixture("list"), &v).is_err());
    v.pointer_mut("/crates/0")
        .fixture("item")
        .as_object_mut()
        .fixture("object")
        .remove("description");
    assert!(changed(request, &v).is_ok());
    assert!(changed(request, &value(0)).is_ok());
    put(&mut v, "/crates/0", "max_version", json!(false));
    assert!(changed(request, &v).is_err());
}
#[test]
fn required_fields_duplicates_and_wrong_types_fail_closed() {
    for (request, fixture) in requests().into_iter().zip(FIXTURES) {
        let root: Value = serde_json::from_str(fixture).fixture("fixture");
        for key in root.as_object().fixture("object").keys() {
            let mut missing = root.clone();
            missing.as_object_mut().fixture("object").remove(key);
            assert!(changed(request, &missing).is_err(), "missing {key}");
        }
    }
    assert!(
        decode(
            CatalogRequest::list(&[]).fixture("list"),
            br#"{"crates":[],"meta":{"total":0,"total":1}}"#
        )
        .is_err()
    );
    let mut v = value(0);
    put(&mut v, "/meta", "total", json!(-1));
    assert!(changed(CatalogRequest::list(&[]).fixture("list"), &v).is_err());
}
