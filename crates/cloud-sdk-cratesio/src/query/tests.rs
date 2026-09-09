use super::*;
use crate::identifiers::*;
use alloc::{format, vec};

fn valid<T, E>(result: Result<T, E>) -> T {
    result.unwrap_or_else(|_| unreachable!("query fixture construction failed"))
}

#[test]
fn query_values_enforce_bounds_and_explicit_selectors() {
    for n in [0, 11, u32::MAX] {
        assert!(Page::new(n).is_err());
    }
    for n in [0, 101, u32::MAX] {
        assert!(PerPage::new(n).is_err());
    }
    for n in 1..=10 {
        assert!(Page::new(n).is_ok());
    }
    for n in 1..=100 {
        assert!(PerPage::new(n).is_ok());
    }
    for s in ["01", "+1", "-1", "1.0", "4294967296", "", "\u{e9}"] {
        assert!(Page::parse(s).is_err());
    }
    for s in ["", "a\n", "a\0", "a\\b", "a#b", "a\u{85}"] {
        assert!(SearchQuery::new(s).is_err());
    }
    assert!(SearchQuery::new(&"x".repeat(1024)).is_ok());
    assert!(SearchQuery::new(&"x".repeat(1025)).is_err());
    for s in ["", "a=b", "a+b", "a/b", "%2F", "a\n"] {
        assert!(Seek::new(s).is_err());
    }
    assert!(Seek::new("WyJmb28iLDQyXQ").is_ok());
    assert!(Seek::new(&"a".repeat(1024)).is_ok());
    assert!(Seek::new(&"a".repeat(1025)).is_err());
    for value in ["unknown", "Versions", "", "full,versions"] {
        assert!(Include::parse(value).is_err());
    }
    assert!(Sort::parse("unknown").is_err());
    assert!(IncludeSet::new(&[]).is_err());
    assert!(IncludeSet::new(&[Include::Versions, Include::Versions]).is_err());
    assert!(IncludeSet::new(&[Include::Versions, Include::DefaultVersion]).is_err());
    assert!(IncludeSet::new(&[Include::Full, Include::Keywords]).is_err());
}

#[test]
fn query_groups_reject_duplicates_shadowed_filters_and_wrong_operations() {
    let page = Parameter::Page(valid(Page::new(1)));
    let seek = Parameter::Seek(valid(Seek::new("OTg")));
    assert!(matches!(
        Query::new(QueryOperation::Crates, &[page, page]),
        Err(QueryError::Duplicate)
    ));
    assert!(matches!(
        Query::new(QueryOperation::Crates, &[page, seek]),
        Err(QueryError::Conflict)
    ));
    assert!(Query::new(QueryOperation::Crate, &[page]).is_err());
    assert!(Query::new(QueryOperation::Categories, &[Parameter::Sort(Sort::Semver)]).is_err());
    assert!(Query::new(QueryOperation::Versions, &[Parameter::Sort(Sort::Semver)]).is_ok());
    assert!(
        Query::new(
            QueryOperation::Categories,
            &[Parameter::Include(valid(IncludeSet::new(&[Include::Full])))]
        )
        .is_err()
    );
    assert!(
        Query::new(
            QueryOperation::Downloads,
            &[Parameter::Include(valid(IncludeSet::new(&[
                Include::Versions
            ])))]
        )
        .is_ok()
    );
    let keyword = valid(Keyword::new("test"));
    let name = valid(CrateName::new("test"));
    let id = valid(NumericId::new(1));
    let filters = [
        Parameter::Keyword(keyword),
        Parameter::AllKeywords(&[keyword]),
        Parameter::Letter('a'),
        Parameter::UserId(id),
        Parameter::TeamId(id),
        Parameter::Following,
        Parameter::Ids(&[name]),
    ];
    for (i, left) in filters.iter().enumerate() {
        for right in filters.iter().skip(i.saturating_add(1)) {
            assert!(matches!(
                Query::new(QueryOperation::Crates, &[*left, *right]),
                Err(QueryError::Conflict)
            ));
        }
    }
    assert!(Query::new(QueryOperation::Crates, &[Parameter::Letter('\u{e9}')]).is_err());
    assert!(Query::new(QueryOperation::Crates, &[Parameter::Ids(&[name, name])]).is_err());
    assert!(Query::new(QueryOperation::Crates, &[Parameter::Ids(&[])]).is_err());
    assert!(Query::new(QueryOperation::Crates, &vec![page; 17]).is_err());
}

#[test]
fn exact_transactional_query_encoding_and_path_binding() {
    let name = valid(CrateName::new("serde"));
    let path = valid(ApiPath::new(&[PathSegment::Fixed(FixedSegment::Crates)]));
    let params = [
        Parameter::Search(valid(SearchQuery::new("rust + \u{e9}&a=b%2F"))),
        Parameter::PerPage(valid(PerPage::new(100))),
        Parameter::Page(valid(Page::new(1))),
    ];
    let query = valid(Query::new(QueryOperation::Crates, &params));
    let expected = "/api/v1/crates?page=1&per_page=100&q=rust%20%2B%20%C3%A9%26a%3Db%252F";
    let mut output = [0xA5; 256];
    assert_eq!(
        valid(query.write_target(path, &mut output)).as_str(),
        expected
    );
    assert!(
        output
            .get(expected.len()..)
            .is_some_and(|v| v.iter().all(|b| *b == 0xA5))
    );
    for capacity in 0..expected.len() {
        let mut out = [0xA5; 256];
        assert!(
            query
                .write_target(
                    path,
                    out.get_mut(..capacity)
                        .unwrap_or_else(|| unreachable!("capacity"))
                )
                .is_err()
        );
        assert_eq!(out, [0xA5; 256]);
    }
    let wrong_parts = [
        PathSegment::Fixed(FixedSegment::Crates),
        PathSegment::Crate(name),
    ];
    let mut out = [0xA5; 256];
    assert!(
        query
            .write_target(valid(ApiPath::new(&wrong_parts)), &mut out)
            .is_err()
    );
    assert_eq!(out, [0xA5; 256]);
    let [a, b, c] = params;
    let reversed = [c, b, a];
    assert_eq!(
        valid(valid(Query::new(QueryOperation::Crates, &reversed)).write_target(path, &mut out))
            .as_str(),
        expected
    );
}

#[test]
fn path_and_array_encoding_preserve_exact_identifiers() {
    let name = valid(CrateName::new("My_crate"));
    let version = valid(Version::new("1.2.3+build.01"));
    let parts = [
        PathSegment::Fixed(FixedSegment::Crates),
        PathSegment::Crate(name),
        PathSegment::Version(version),
    ];
    let mut out = [0; 512];
    assert_eq!(
        valid(valid(ApiPath::new(&parts)).write(&mut out)).as_str(),
        "/api/v1/crates/My_crate/1.2.3%2Bbuild.01"
    );
    let parts = [
        PathSegment::Fixed(FixedSegment::Categories),
        PathSegment::Category(valid(CategorySlug::new("a::b"))),
    ];
    assert_eq!(
        valid(valid(ApiPath::new(&parts)).write(&mut out)).as_str(),
        "/api/v1/categories/a%3A%3Ab"
    );
    let values = [name, valid(CrateName::new("serde"))];
    let parameters = [Parameter::Ids(&values)];
    assert_eq!(
        valid(valid(Query::new(QueryOperation::Crates, &parameters)).write(&mut out)),
        "ids%5B%5D=My_crate&ids%5B%5D=serde"
    );
    let keywords = [valid(Keyword::new("c++")), valid(Keyword::new("parser"))];
    let parameters = [Parameter::AllKeywords(&keywords)];
    assert_eq!(
        valid(valid(Query::new(QueryOperation::Crates, &parameters)).write(&mut out)),
        "all_keywords=c%2B%2B%20parser"
    );
    assert!(ApiPath::new(&[]).is_err());
    assert!(ApiPath::new(&[PathSegment::Crate(name)]).is_err());
}

#[test]
fn query_ascii_properties_cannot_inject_additional_pairs() {
    for byte in b' '..=b'~' {
        if matches!(byte, b'#' | b'\\') {
            continue;
        }
        let text = format!("x{}y", char::from(byte));
        let parameter = [Parameter::Search(valid(SearchQuery::new(&text)))];
        let mut out = [0; 128];
        let encoded = valid(valid(Query::new(QueryOperation::Crates, &parameter)).write(&mut out));
        assert_eq!(encoded.split('&').count(), 1);
        assert_eq!(encoded.matches('=').count(), 1);
        assert!(!encoded.contains('+'));
        let mut target = [0; 256];
        let parts = [PathSegment::Fixed(FixedSegment::Crates)];
        assert!(
            valid(Query::new(QueryOperation::Crates, &parameter))
                .write_target(valid(ApiPath::new(&parts)), &mut target)
                .is_ok()
        );
    }
}
