use cloud_sdk_cratesio::{
    endpoint::OfficialCratesIoEndpoint,
    pagination::{Cursor, Direction, PageLink},
    query::{ApiPath, FixedSegment, PathSegment, Query, QueryOperation},
};

pub fn exercise(data: &[u8]) -> bool {
    let Ok(text) = core::str::from_utf8(data) else {
        return false;
    };
    let parts = [PathSegment::Fixed(FixedSegment::Crates)];
    let path = ApiPath::new(&parts).expect("fixed path");
    let query = Query::new(QueryOperation::Crates, &[]).expect("fixed query");
    let endpoint = OfficialCratesIoEndpoint::production_api();
    let Ok(link) = PageLink::new(endpoint, path, query, text, Direction::Next) else {
        return false;
    };
    assert_eq!(link.endpoint(), endpoint);
    if let Cursor::Page(page) = link.cursor() {
        assert!(page.get() > 1 && page.get() <= 10);
    }
    assert_eq!(format!("{link:?}"), "PageLink([redacted])");
    // A continuation admitted for one origin must not bind to another.
    if text.starts_with("https://crates.io/") {
        assert!(
            PageLink::new(
                OfficialCratesIoEndpoint::staging_api(),
                path,
                query,
                text,
                Direction::Next
            )
            .is_err()
        );
    }
    true
}
