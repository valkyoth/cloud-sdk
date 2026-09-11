use super::*;

#[test]
fn numbered_links_counts_and_ceiling_do_not_silently_end() {
    for current in [1, 2, 10] {
        let params = [
            Parameter::Page(Page::new(current).fixture("page")),
            Parameter::PerPage(PerPage::new(1).fixture("size")),
        ];
        let request = CatalogRequest::list(&params).fixture("list");
        let mut v = value(0);
        put(&mut v, "/meta", "total", json!(100));
        let previous = if current == 1 {
            Value::Null
        } else {
            json!(format!(
                "?page={}&per_page=1",
                current.checked_sub(1).fixture("prior")
            ))
        };
        put(&mut v, "/meta", "prev_page", previous);
        if current == 10 {
            let CatalogResponse::Crates(page) = changed(request, &v).fixture("ceiling") else {
                unreachable!("list");
            };
            assert!(matches!(page.next, CatalogContinuation::LimitReached));
        } else {
            assert!(matches!(changed(request, &v), Err(CatalogError::Binding)));
        }
        let next = format!(
            "?page={}&per_page=1",
            current.checked_add(1).fixture("next")
        );
        put(&mut v, "/meta", "next_page", json!(next));
        let CatalogResponse::Crates(page) = changed(request, &v).fixture("numbered") else {
            unreachable!("list");
        };
        assert_eq!(
            matches!(page.next, CatalogContinuation::LimitReached),
            current == 10
        );
        for bad in [
            format!("https://evil.example/api/v1/crates{next}"),
            format!("{next}&sort=downloads"),
            "?page=11&per_page=2".into(),
        ] {
            put(&mut v, "/meta", "next_page", json!(bad));
            assert!(changed(request, &v).is_err());
        }
    }
}
#[test]
fn seek_links_preserve_query_and_relevance_truncation_is_visible() {
    let params = [
        Parameter::Search(SearchQuery::new("serde").fixture("search")),
        Parameter::PerPage(PerPage::new(100).fixture("size")),
    ];
    let request = CatalogRequest::list(&params).fixture("list");
    let mut v = value(0);
    put(&mut v, "/meta", "total", json!(1200));
    put(
        &mut v,
        "/meta",
        "next_page",
        json!("?q=serde&per_page=100&seek=abcd"),
    );
    assert!(changed(request, &v).is_ok());
    for bad in [
        "?q=other&per_page=100&seek=abcd",
        "?q=serde&per_page=100&seek=abcd&seek=efgh",
        "?q=serde&per_page=100&seek=abcd#x",
    ] {
        put(&mut v, "/meta", "next_page", json!(bad));
        assert!(changed(request, &v).is_err());
    }
    let seek = [
        Parameter::Search(SearchQuery::new("serde").fixture("search")),
        Parameter::Seek(Seek::new("abcd").fixture("seek")),
    ];
    let request = CatalogRequest::list(&seek).fixture("list");
    put(&mut v, "/meta", "next_page", Value::Null);
    let CatalogResponse::Crates(page) = changed(request, &v).fixture("truncated") else {
        unreachable!("list");
    };
    assert!(matches!(page.next, CatalogContinuation::LimitReached));
    put(&mut v, "/meta", "next_page", json!("?q=serde&seek=abcd"));
    assert!(changed(request, &v).is_err());
}
#[test]
fn sort_profiles_and_list_size_remain_bounded() {
    for sort in [
        Sort::Alphabetical,
        Sort::Relevance,
        Sort::Downloads,
        Sort::RecentDownloads,
        Sort::RecentUpdates,
        Sort::New,
    ] {
        let params = [Parameter::Sort(sort)];
        assert!(
            CatalogRequest::list(&params)
                .fixture("sort")
                .write_target(&mut [0; 4096])
                .is_ok()
        );
    }
    let mut v = value(0);
    let record = v.pointer("/crates/0").fixture("record").clone();
    put(&mut v, "", "crates", json!(vec![record; 2]));
    let params = [Parameter::PerPage(PerPage::new(1).fixture("size"))];
    assert!(matches!(
        changed(CatalogRequest::list(&params).fixture("list"), &v),
        Err(CatalogError::Limit)
    ));
}

#[test]
fn relevance_requires_search_and_combined_limits_are_explicit() {
    let params = [
        Parameter::Sort(Sort::Relevance),
        Parameter::Seek(Seek::new("abcd").fixture("seek")),
    ];
    let mut v = value(0);
    put(&mut v, "/meta", "total", json!(1200));
    let CatalogResponse::Crates(page) =
        changed(CatalogRequest::list(&params).fixture("list"), &v).fixture("alphabetical fallback")
    else {
        unreachable!("list");
    };
    assert!(matches!(page.next, CatalogContinuation::End));
    put(
        &mut v,
        "/meta",
        "prev_page",
        json!("?sort=relevance&seek=efgh"),
    );
    assert!(changed(CatalogRequest::list(&params).fixture("list"), &v).is_err());

    let params = [
        Parameter::Search(SearchQuery::new("serde").fixture("search")),
        Parameter::Page(Page::new(10).fixture("page")),
        Parameter::PerPage(PerPage::new(100).fixture("size")),
    ];
    let request = CatalogRequest::list(&params).fixture("request");
    put(
        &mut v,
        "/meta",
        "prev_page",
        json!("?q=serde&page=9&per_page=100"),
    );
    for next in [Value::Null, json!("?q=serde&page=11&per_page=100")] {
        put(&mut v, "/meta", "next_page", next);
        let CatalogResponse::Crates(page) = changed(request, &v).fixture("limit") else {
            unreachable!("list");
        };
        assert!(matches!(page.next, CatalogContinuation::LimitReached));
    }
    put(
        &mut v,
        "/meta",
        "next_page",
        json!("?q=other&page=11&per_page=100"),
    );
    assert!(changed(request, &v).is_err());
}
