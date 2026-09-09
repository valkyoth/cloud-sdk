use super::*;
use crate::identifiers::CrateName;
use crate::query::*;
use alloc::format;
use cloud_sdk::pagination::PaginationLimits;

fn valid<T, E>(result: Result<T, E>) -> T {
    result.unwrap_or_else(|_| unreachable!("pagination fixture construction failed"))
}
const PARTS: &[PathSegment<'static>] = &[PathSegment::Fixed(FixedSegment::Crates)];

#[test]
fn next_and_previous_links_bind_origin_path_filters_and_size() {
    let endpoint = OfficialCratesIoEndpoint::production_api();
    let path = valid(ApiPath::new(PARTS));
    let parameters = [
        Parameter::Search(valid(SearchQuery::new("a b+c"))),
        Parameter::Page(valid(Page::new(2))),
        Parameter::PerPage(valid(PerPage::new(10))),
    ];
    let query = valid(Query::new(QueryOperation::Crates, &parameters));
    for link in [
        "?q=a+b%2bc&page=3&per_page=10",
        "/api/v1/crates?page=3&per_page=10&q=a%20b%2Bc",
        "https://crates.io/api/v1/crates?page=3&per_page=10&q=a+b%2Bc",
    ] {
        let accepted = valid(PageLink::new(endpoint, path, query, link, Direction::Next));
        assert_eq!(accepted.cursor(), Cursor::Page(valid(Page::new(3))));
        assert_eq!(accepted.endpoint(), endpoint);
        let mut output = [0; 256];
        let len = valid(accepted.encode_target(&mut output));
        let target = valid(core::str::from_utf8(
            output
                .get(..len)
                .unwrap_or_else(|| unreachable!("encoded target")),
        ));
        assert!(target.starts_with("/api/v1/crates?"));
        assert!(!format!("{accepted:?}").contains("a+b"));
    }
    let meta = valid(MetaLinks::new(
        endpoint,
        path,
        query,
        Some("?page=3&per_page=10&q=a+b%2Bc"),
        Some("?page=1&per_page=10&q=a+b%2Bc"),
    ));
    assert!(meta.next().is_some());
    assert!(meta.previous().is_some());
    for invalid in [
        "http://crates.io/api/v1/crates?page=3",
        "https://crates.io.evil/api/v1/crates?page=3",
        "https://user@crates.io/api/v1/crates?page=3",
        "https://crates.io:443/api/v1/crates?page=3",
        "https://static.crates.io/api/v1/crates?page=3",
        "//crates.io/api/v1/crates?page=3",
        "/api/v1/keywords?page=3&per_page=10&q=a+b%2Bc",
        "?page=3&per_page=11&q=a+b%2Bc",
        "?page=3&q=a+b%2Bc",
        "?page=3&per_page=10&q=changed",
        "?page=3&per_page=10&q=a+b%2Bc&token=oops",
        "?page=3&page=3&per_page=10&q=a+b%2Bc",
        "?page=3&seek=OTg&per_page=10&q=a+b%2Bc",
        "?page=3&per_page=10&q=a+b%2Bc#fragment",
        "?page=3&per_page=10&q=%FF",
        "?page=03&per_page=10&q=a+b%2Bc",
        "?page=2&per_page=10&q=a+b%2Bc",
        "?page=4&per_page=10&q=a+b%2Bc",
        "?page=3&per_page=10&q=a+b%2Bc&q=a+b%2Bc",
        "?%70age=3&per_page=10&q=a+b%2Bc",
    ] {
        assert!(
            PageLink::new(endpoint, path, query, invalid, Direction::Next).is_err(),
            "accepted {invalid}"
        );
    }
    assert!(
        MetaLinks::new(
            endpoint,
            path,
            query,
            None,
            Some("?page=3&per_page=10&q=a+b%2Bc")
        )
        .is_err()
    );
}

#[test]
fn seek_state_stays_opaque_and_numbered_modes_cannot_be_mixed() {
    let endpoint = OfficialCratesIoEndpoint::staging_api();
    let path = valid(ApiPath::new(PARTS));
    let query = valid(Query::new(QueryOperation::Crates, &[]));
    for link in [
        "?seek=OTg",
        "https://staging.crates.io/api/v1/crates?seek=OTg",
    ] {
        let link = valid(PageLink::new(endpoint, path, query, link, Direction::Next));
        assert_eq!(link.cursor(), Cursor::Seek(valid(Seek::new("OTg"))));
    }
    for link in [
        "?seek=",
        "?seek=a/b",
        "?seek=%4F%54g",
        "?seek=OTg=",
        "?seek=OTg&seek=OTg",
        "?seek=OTg&%73eek=OTg",
        "?seek=OTg&page=2",
        "https://crates.io/api/v1/crates?seek=OTg",
    ] {
        assert!(PageLink::new(endpoint, path, query, link, Direction::Next).is_err());
    }
    let parameters = [Parameter::Seek(valid(Seek::new("OTg")))];
    let query = valid(Query::new(QueryOperation::Crates, &parameters));
    assert!(PageLink::new(endpoint, path, query, "?seek=OTg", Direction::Next).is_err());
    assert!(PageLink::new(endpoint, path, query, "?page=2", Direction::Next).is_err());
    assert!(PageLink::new(endpoint, path, query, "?seek=OTk", Direction::Next).is_ok());
    assert!(
        PageLink::new(
            OfficialCratesIoEndpoint::static_downloads(),
            path,
            query,
            "?seek=OTk",
            Direction::Next
        )
        .is_err()
    );
}

#[test]
fn repeated_array_pairs_are_matched_once_and_cannot_be_changed() {
    let names = [
        valid(CrateName::new("serde")),
        valid(CrateName::new("tokio")),
    ];
    let params = [Parameter::Ids(&names)];
    let query = valid(Query::new(QueryOperation::Crates, &params));
    let endpoint = OfficialCratesIoEndpoint::production_api();
    let path = valid(ApiPath::new(PARTS));
    let raw = "?ids%5B%5D=tokio&page=2&ids%5b%5d=serde";
    let link = valid(PageLink::new(endpoint, path, query, raw, Direction::Next));
    let expected = "/api/v1/crates?ids%5B%5D=tokio&page=2&ids%5b%5d=serde";
    let mut output = [0xA5; 256];
    assert_eq!(valid(link.encode_target(&mut output)), expected.len());
    assert_eq!(output.get(..expected.len()), Some(expected.as_bytes()));
    for capacity in 0..expected.len() {
        let mut output = [0xA5; 256];
        assert!(
            link.encode_target(
                output
                    .get_mut(..capacity)
                    .unwrap_or_else(|| unreachable!("capacity"))
            )
            .is_err()
        );
        assert_eq!(output, [0xA5; 256]);
    }
    for raw in [
        "?ids[]=tokio&page=2&ids[]=tokio",
        "?ids[]=serde&page=2",
        "?ids[]=serde&ids[]=tokio&page=2&ids[]=tokio",
    ] {
        assert!(PageLink::new(endpoint, path, query, raw, Direction::Next).is_err());
    }
}

#[test]
fn traversal_enforces_budgets_for_meta_and_legacy_responses() {
    let one = valid(Page::new(1));
    assert_eq!(valid(legacy_next(one, true)), Some(valid(Page::new(2))));
    assert_eq!(valid(legacy_next(valid(Page::new(10)), false)), None);
    assert!(legacy_next(valid(Page::new(10)), true).is_err());
    let limits = valid(PaginationLimits::new(2, 20, 128));
    let mut budget = Traversal::new(limits, valid(PerPage::new(10)));
    assert!(budget.admit_legacy(11, one, true).is_err());
    assert_eq!(
        valid(budget.admit_legacy(10, one, true)),
        Some(valid(Page::new(2)))
    );
    assert!(budget.admit_legacy(10, valid(Page::new(2)), true).is_err());
    assert_eq!(
        valid(budget.admit_legacy(10, valid(Page::new(2)), false)),
        None
    );
    assert!(budget.admit_legacy(0, valid(Page::new(2)), false).is_err());
    let endpoint = OfficialCratesIoEndpoint::production_api();
    let path = valid(ApiPath::new(PARTS));
    let query = valid(Query::new(QueryOperation::Crates, &[]));
    let next = valid(PageLink::new(
        endpoint,
        path,
        query,
        "?page=2",
        Direction::Next,
    ));
    let mut budget = Traversal::new(
        valid(PaginationLimits::new(2, 20, 2)),
        valid(PerPage::new(10)),
    );
    assert!(budget.admit(10, Some(&next)).is_err());
    assert!(budget.admit(10, None).is_ok());
}

#[test]
fn malformed_links_and_encoded_control_properties_fail_closed() {
    let endpoint = OfficialCratesIoEndpoint::production_api();
    let path = valid(ApiPath::new(PARTS));
    let query = valid(Query::new(QueryOperation::Crates, &[]));
    for input in [
        "",
        "?",
        "?page=0",
        "?page=11",
        "?page=4294967296",
        "?page=2&",
        "?page=2%",
        "?page=2%00",
        "?page=2&unknown=1",
        "/api/v1/../crates?page=2",
        "https://crates.io?seek=OTg",
    ] {
        assert!(PageLink::new(endpoint, path, query, input, Direction::Next).is_err());
    }
    for b in 0_u8..=255 {
        let raw = format!("?page=2&x=%{b:02X}");
        assert!(PageLink::new(endpoint, path, query, &raw, Direction::Next).is_err());
    }
}
