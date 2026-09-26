use super::*;
use crate::{
    discovery::tests::Fixture as _,
    endpoint::OfficialCratesIoEndpoint,
    identifiers::{CrateName, Version},
    query::{Parameter, PerPage},
    wire::JsonResponsePolicy,
};
use alloc::{format, vec};
use cloud_sdk::{
    rate_limit::WallClockTimestamp,
    transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode},
};
use serde_json::{Value, json};

pub(super) const FIXTURES: [&str; 4] = [
    include_str!("fixtures/download_version.json"),
    include_str!("fixtures/get_crate_downloads.json"),
    include_str!("fixtures/get_version_downloads.json"),
    include_str!("fixtures/list_reverse_dependencies.json"),
];
pub(super) fn requests() -> [DownloadRequest<'static>; 4] {
    const PAGE: &[Parameter<'_>] = &[Parameter::PerPage(PerPage::DEFAULT)];
    let name = CrateName::new("serde").fixture("name");
    let version = Version::new("1.0.0").fixture("version");
    [
        DownloadRequest::location(name, version),
        DownloadRequest::crate_counts(name, &[]).fixture("crate counts"),
        DownloadRequest::version_counts(name, version, &[]).fixture("version counts"),
        DownloadRequest::reverse_dependencies(name, PAGE).fixture("reverse"),
    ]
}
fn fixture(index: usize) -> Value {
    serde_json::from_str(FIXTURES.get(index).fixture("fixture")).fixture("json")
}
fn request(index: usize) -> DownloadRequest<'static> {
    *requests().get(index).fixture("request")
}
fn decode(request: DownloadRequest<'_>, wire: &[u8]) -> Result<DownloadResponse, DownloadError> {
    let mut bytes = vec![0xa5; wire.len()];
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
        .fixture("media");
    attempt
        .commit(StatusCode::OK, wire.len(), ResponseMetadata::EMPTY)
        .fixture("commit");
    drop(attempt);
    let result = match JsonResponsePolicy::new(StatusCode::OK, wire.len())
        .fixture("policy")
        .admit(response, WallClockTimestamp::new(0))
    {
        Ok(success) => request.decode(OfficialCratesIoEndpoint::production_api(), success),
        Err(_) => Err(DownloadError::Schema),
    };
    assert!(bytes.iter().all(|b| *b == 0));
    assert!(headers.iter().all(|b| *b == 0));
    result
}
fn changed(request: DownloadRequest<'_>, value: &Value) -> Result<DownloadResponse, DownloadError> {
    decode(request, &serde_json::to_vec(value).fixture("json"))
}
fn put(root: &mut Value, path: &str, value: Value) {
    *root.pointer_mut(path).fixture("path") = value;
}
#[test]
fn source_fixtures_and_atomic_targets() {
    let expected = [
        "/api/v1/crates/serde/1.0.0/download",
        "/api/v1/crates/serde/downloads",
        "/api/v1/crates/serde/1.0.0/downloads",
        "/api/v1/crates/serde/reverse_dependencies?per_page=10",
    ];
    for ((request, wire), expected) in requests().into_iter().zip(FIXTURES).zip(expected) {
        let result = decode(request, wire.as_bytes()).fixture("decode");
        assert!(!format!("{result:?}").contains("serde"));
        request.operation().operation_id().fixture("id");
        for len in 0..expected.len() {
            let mut output = vec![0xa5; len];
            assert!(request.write_target(&mut output).is_err());
            assert!(output.iter().all(|b| *b == 0xa5));
        }
        assert_eq!(
            request
                .write_target(&mut vec![0; expected.len()])
                .fixture("target")
                .as_str(),
            expected
        );
    }
}
#[test]
fn malformed_statistics_and_duplicates_fail_closed() {
    for index in [1, 2] {
        for (key, bad) in [
            ("date", json!("2025-02-29")),
            ("downloads", json!(-1)),
            ("downloads", json!(2147483648_u64)),
            ("version", json!(0)),
        ] {
            let mut value = fixture(index);
            put(&mut value, &format!("/version_downloads/0/{key}"), bad);
            assert!(changed(request(index), &value).is_err());
        }
        let mut value = fixture(index);
        let bucket = value
            .pointer("/version_downloads/0")
            .fixture("bucket")
            .clone();
        value
            .get_mut("version_downloads")
            .fixture("buckets")
            .as_array_mut()
            .fixture("array")
            .push(bucket);
        assert!(changed(request(index), &value).is_err());
    }
    let parameters = [Parameter::BeforeDate(
        crate::identifiers::Date::new("2019-12-12").fixture("date"),
    )];
    let r = DownloadRequest::version_counts(
        CrateName::new("serde").fixture("name"),
        Version::new("1.0.0").fixture("version"),
        &parameters,
    )
    .fixture("request");
    assert!(changed(r, &fixture(2)).is_err());
    let mut value = fixture(2);
    let mut bucket = value
        .pointer("/version_downloads/0")
        .fixture("bucket")
        .clone();
    put(&mut bucket, "/version", json!(43));
    value
        .get_mut("version_downloads")
        .fixture("buckets")
        .as_array_mut()
        .fixture("array")
        .push(bucket);
    assert!(changed(request(2), &value).is_err());
}
#[test]
fn reverse_dependencies_bind_ids_names_and_numbered_pages() {
    for (path, bad) in [
        ("/dependencies/0/crate_id", json!("other")),
        ("/dependencies/0/version_id", json!(41)),
        ("/versions/0/checksum", json!("bad")),
        ("/meta/total", json!(-1)),
    ] {
        let mut value = fixture(3);
        put(&mut value, path, bad);
        assert!(changed(request(3), &value).is_err());
    }
    let name = CrateName::new("serde").fixture("name");
    assert!(DownloadRequest::reverse_dependencies(name, &[]).is_err());
    let seek = [
        Parameter::PerPage(PerPage::DEFAULT),
        Parameter::Seek(crate::query::Seek::new("opaque").fixture("seek")),
    ];
    assert!(DownloadRequest::reverse_dependencies(name, &seek).is_err());
    let mut value = fixture(3);
    put(&mut value, "/meta/total", json!(11));
    let page = [Parameter::PerPage(PerPage::new(1).fixture("size"))];
    let r = DownloadRequest::reverse_dependencies(name, &page).fixture("request");
    let DownloadResponse::ReverseDependencies(result) = changed(r, &value).fixture("page") else {
        unreachable!("wrong response");
    };
    assert_eq!(
        result.next,
        ReverseContinuation::Next(crate::query::Page::new(2).fixture("page"))
    );
    let page = [
        Parameter::PerPage(PerPage::new(1).fixture("size")),
        Parameter::Page(crate::query::Page::new(10).fixture("page")),
    ];
    let r = DownloadRequest::reverse_dependencies(name, &page).fixture("request");
    let DownloadResponse::ReverseDependencies(result) = changed(r, &value).fixture("page") else {
        unreachable!("wrong response");
    };
    assert_eq!(result.next, ReverseContinuation::LimitReached);
}
#[test]
fn duplicate_unknown_json_and_untrusted_locations_are_not_capabilities() {
    for request in requests() {
        assert!(decode(request, br#"{"x":1,"x":2}"#).is_err());
    }
    for bad in [json!(""), json!("https://host/\n"), json!("x".repeat(9000))] {
        assert!(changed(request(0), &json!({"url":bad})).is_err());
    }
    // Inspecting data from staging or a changed source never grants fetch authority.
    let DownloadResponse::Location(location) = changed(
        request(0),
        &json!({"url":"https://untrusted.invalid/object"}),
    )
    .fixture("inert") else {
        unreachable!("location");
    };
    assert!(
        location
            .with_url(|s| s.contains("untrusted"))
            .fixture("url")
    );
}

#[test]
fn date_window_and_crate_version_limits_are_exact() {
    let name = CrateName::new("serde").fixture("name");
    let version = Version::new("1.0.0").fixture("version");
    for (end, first, outside) in [
        ("2024-03-01", "2023-12-03", "2023-12-02"),
        ("2023-03-01", "2022-12-02", "2022-12-01"),
    ] {
        let parameters = [Parameter::BeforeDate(
            crate::identifiers::Date::new(end).fixture("end"),
        )];
        let request =
            DownloadRequest::version_counts(name, version, &parameters).fixture("request");
        let mut value = fixture(2);
        put(&mut value, "/version_downloads/0/date", json!(first));
        assert!(changed(request, &value).is_ok());
        put(&mut value, "/version_downloads/0/date", json!(outside));
        assert!(changed(request, &value).is_err());
        put(&mut value, "/version_downloads/0/date", json!(end));
        assert!(changed(request, &value).is_ok());
    }
    let mut value = fixture(1);
    let mut buckets = alloc::vec::Vec::new();
    for id in 1..=6 {
        buckets.push(json!({"version":id,"downloads":0,"date":"2019-12-13"}));
    }
    put(&mut value, "/version_downloads", json!(buckets));
    assert!(changed(request(1), &value).is_err());
    let include = [crate::query::Include::Versions];
    let parameters = [Parameter::Include(
        crate::query::IncludeSet::new(&include).fixture("include"),
    )];
    let with_versions = DownloadRequest::crate_counts(name, &parameters).fixture("request");
    let mut value = fixture(1);
    assert!(changed(with_versions, &value).is_err());
    let versions = fixture(3).get("versions").fixture("versions").clone();
    value
        .as_object_mut()
        .fixture("object")
        .insert("versions".into(), versions);
    assert!(changed(with_versions, &value).is_ok());
    assert!(changed(request(1), &value).is_err());
    put(&mut value, "/versions/0/id", json!(43));
    assert!(changed(with_versions, &value).is_err());
}
#[cfg(all(feature = "blocking", feature = "async"))]
mod unified;
