use super::*;

fn pagination(response: DiscoveryResponse) -> (PageContinuation, Option<Page>) {
    match response {
        DiscoveryResponse::Categories(page) => (page.next, page.previous),
        DiscoveryResponse::Keywords(page) => (page.next, page.previous),
        _ => unreachable!("wrong discovery list model"),
    }
}

#[test]
fn supplied_next_links_must_agree_with_counts_in_both_list_operations() {
    // Cover continuation, exact/beyond completion, and both ceiling outcomes.
    for (current, total, expected) in [
        (1, 100, PageContinuation::Page(Page::new(2).fixture("page"))),
        (2, 100, PageContinuation::Page(Page::new(3).fixture("page"))),
        (1, 1, PageContinuation::End),
        (2, 1, PageContinuation::End),
        (10, 100, PageContinuation::LimitReached),
        (10, 10, PageContinuation::End),
    ] {
        let params = [
            Parameter::Page(Page::new(current).fixture("page")),
            Parameter::PerPage(PerPage::new(1).fixture("size")),
        ];
        for (request, index) in [
            (DiscoveryRequest::categories(&params).fixture("query"), 0),
            (DiscoveryRequest::keywords(&params).fixture("query"), 3),
        ] {
            let mut value: Value =
                serde_json::from_str(FIXTURES.get(index).fixture("source")).fixture("JSON");
            put(&mut value, "/meta", "total", json!(total));
            assert_eq!(
                pagination(changed(request, &value).fixture("no link")).0,
                expected
            );

            put(&mut value, "/meta", "next_page", Value::Null);
            if expected == PageContinuation::End {
                assert_eq!(
                    pagination(changed(request, &value).fixture("end")).0,
                    expected
                );
            } else {
                assert!(matches!(
                    changed(request, &value),
                    Err(DiscoveryError::Binding)
                ));
            }

            let following = current.checked_add(1).fixture("following page");
            put(
                &mut value,
                "/meta",
                "next_page",
                json!(format!("?page={following}&per_page=1")),
            );
            if matches!(expected, PageContinuation::Page(_)) {
                assert_eq!(
                    pagination(changed(request, &value).fixture("next link")).0,
                    expected
                );
            } else {
                assert!(matches!(
                    changed(request, &value),
                    Err(DiscoveryError::Binding)
                ));
            }
        }
    }
}

#[test]
fn supplied_previous_links_cannot_erase_history_in_both_list_operations() {
    for current in [1_u32, 2, 10] {
        let params = [
            Parameter::Page(Page::new(current).fixture("page")),
            Parameter::PerPage(PerPage::new(1).fixture("size")),
        ];
        let previous = current.checked_sub(1).fixture("previous");
        for (request, index) in [
            (DiscoveryRequest::categories(&params).fixture("query"), 0),
            (DiscoveryRequest::keywords(&params).fixture("query"), 3),
        ] {
            let mut value: Value =
                serde_json::from_str(FIXTURES.get(index).fixture("source")).fixture("JSON");
            put(&mut value, "/meta", "total", json!(100));
            let expected = Page::new(previous).ok();
            assert_eq!(
                pagination(changed(request, &value).fixture("no link")).1,
                expected
            );

            put(&mut value, "/meta", "prev_page", Value::Null);
            if current == 1 {
                assert_eq!(
                    pagination(changed(request, &value).fixture("first page")).1,
                    None
                );
            } else {
                assert!(matches!(
                    changed(request, &value),
                    Err(DiscoveryError::Binding)
                ));
            }

            put(
                &mut value,
                "/meta",
                "prev_page",
                json!(format!("?page={previous}&per_page=1")),
            );
            if current == 1 {
                assert!(matches!(
                    changed(request, &value),
                    Err(DiscoveryError::Binding)
                ));
            } else {
                assert_eq!(
                    pagination(changed(request, &value).fixture("previous link")).1,
                    expected
                );
            }
        }
    }
}
