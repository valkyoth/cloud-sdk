use super::*;
use crate::query::{Include, IncludeSet, Parameter, PerPage};

#[test]
fn pagination_binds_links_and_explicit_release_tracks() {
    let name = CrateName::new("serde").fixture("name");
    let params = [Parameter::PerPage(PerPage::new(1).fixture("per page"))];
    let request = VersionRequest::list(name, &params).fixture("request");
    let mut value = fixture(0);
    put(&mut value, "/meta", "total", json!(2));
    assert!(changed(request, &value).is_err());
    put(
        &mut value,
        "/meta",
        "next_page",
        json!("?seek=abc&per_page=1"),
    );
    let VersionResponse::Versions(page) = changed(request, &value).fixture("page") else {
        unreachable!("page")
    };
    assert!(matches!(page.next, VersionContinuation::Next(_)));
    for link in [
        "?page=3&per_page=1",
        "?page=2&per_page=100",
        "https://other.invalid/?page=2",
        "?page=2&page=2",
        "?page=2&per_page=1&sort=date",
    ] {
        put(&mut value, "/meta", "next_page", json!(link));
        assert!(changed(request, &value).is_err());
    }
    let mut value = fixture(0);
    let tracks = [Include::ReleaseTracks];
    let params = [
        Parameter::PerPage(PerPage::DEFAULT),
        Parameter::Include(IncludeSet::new(&tracks).fixture("include")),
    ];
    let request = VersionRequest::list(name, &params).fixture("request");
    assert!(changed(request, &value).is_err());
    put(
        &mut value,
        "/meta",
        "release_tracks",
        json!({"1": {"highest":"1.0.0"}}),
    );
    assert!(changed(request, &value).is_ok());
    assert!(changed(super::request(0), &value).is_err());
    put(
        &mut value,
        "/meta/release_tracks/1",
        "highest",
        json!("bad"),
    );
    assert!(changed(request, &value).is_err());
}

#[test]
fn list_enforces_page_bound_unique_ids_and_exact_filters() {
    let name = CrateName::new("serde").fixture("name");
    assert!(VersionRequest::list(name, &[]).is_err());
    let pages = [
        Parameter::PerPage(PerPage::DEFAULT),
        Parameter::Page(crate::query::Page::new(1).fixture("page")),
    ];
    assert!(VersionRequest::list(name, &pages).is_err());
    let mut value = fixture(0);
    let duplicate = value.pointer("/versions/0").fixture("version").clone();
    value
        .pointer_mut("/versions")
        .fixture("versions")
        .as_array_mut()
        .fixture("array")
        .push(duplicate);
    assert!(changed(request(0), &value).is_err());
    let nums = [Version::new("2.0.0").fixture("version")];
    let params = [
        Parameter::PerPage(PerPage::DEFAULT),
        Parameter::Versions(&nums),
    ];
    let request =
        VersionRequest::list(CrateName::new("serde").fixture("name"), &params).fixture("request");
    assert!(changed(request, &fixture(0)).is_err());
}

#[test]
fn full_final_page_still_requires_a_seek_link_and_empty_tail_completes() {
    let name = CrateName::new("serde").fixture("name");
    let parameters = [Parameter::PerPage(PerPage::new(1).fixture("size"))];
    let request = VersionRequest::list(name, &parameters).fixture("request");
    let mut value = fixture(0);
    assert!(changed(request, &value).is_err());
    put(
        &mut value,
        "/meta",
        "next_page",
        json!("?seek=abc&per_page=1"),
    );
    assert!(changed(request, &value).is_ok());
    let parameters = [
        Parameter::PerPage(PerPage::new(1).fixture("size")),
        Parameter::Seek(crate::query::Seek::new("abc").fixture("seek")),
    ];
    let request = VersionRequest::list(name, &parameters).fixture("request");
    let empty = json!({"versions":[],"meta":{"total":0,"next_page":null}});
    assert!(changed(request, &empty).is_ok());
    put(
        &mut value,
        "/meta",
        "next_page",
        json!("?seek=abc&per_page=1"),
    );
    assert!(changed(request, &value).is_err());
}
