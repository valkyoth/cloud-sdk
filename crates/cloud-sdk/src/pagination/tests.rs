use super::{PageLimit, PageMetadata, PageNumber, PaginationCursor, PaginationError};
use crate::rate_limit::RateLimit;

fn page(value: u64) -> PageNumber {
    PageNumber::new(value).unwrap_or_else(|_| unreachable!())
}

fn metadata(current: u64, next: Option<u64>) -> PageMetadata {
    PageMetadata::new(
        page(current),
        25,
        current.checked_sub(1).filter(|value| *value != 0).map(page),
        next.map(page),
        Some(page(next.unwrap_or(current))),
        None,
    )
    .unwrap_or_else(|_| unreachable!())
}

#[test]
fn walks_explicit_pages_and_propagates_rate_limits() {
    let mut cursor = PaginationCursor::new(
        page(1),
        25,
        PageLimit::new(3).unwrap_or_else(|_| unreachable!()),
    )
    .unwrap_or_else(|_| unreachable!());
    let rate_limit = RateLimit::new(3600, 3599, 42).ok();
    let first = cursor.observe(metadata(1, Some(2)), 25, rate_limit);
    assert!(first.is_ok());
    assert_eq!(first.map(|value| value.rate_limit()), Ok(rate_limit));
    assert_eq!(cursor.next_page(), Ok(page(2)));

    let last = cursor.observe(metadata(2, None), 3, None);
    assert!(last.is_ok());
    assert!(last.is_ok_and(|value| value.is_terminal()));
    assert_eq!(cursor.pages_seen(), 2);
    assert_eq!(cursor.next_page(), Err(PaginationError::Complete));
}

#[test]
fn accepts_an_empty_terminal_page() {
    let mut cursor = PaginationCursor::new(
        page(1),
        25,
        PageLimit::new(1).unwrap_or_else(|_| unreachable!()),
    )
    .unwrap_or_else(|_| unreachable!());
    let boundary = cursor.observe(metadata(1, None), 0, None);
    assert!(boundary.is_ok_and(|value| value.is_terminal() && value.entries() == 0));
}

#[test]
fn rejects_empty_and_repeated_nonterminal_pages() {
    let mut cursor = PaginationCursor::new(
        page(1),
        25,
        PageLimit::new(3).unwrap_or_else(|_| unreachable!()),
    )
    .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        cursor.observe(metadata(1, Some(2)), 0, None),
        Err(PaginationError::EmptyPageWithNextPage)
    );
    assert_eq!(cursor.pages_seen(), 0);

    assert!(cursor.observe(metadata(1, Some(2)), 1, None).is_ok());
    assert_eq!(
        cursor.observe(metadata(1, Some(2)), 1, None),
        Err(PaginationError::UnexpectedPage)
    );
}

#[test]
fn page_limit_fails_before_advancing_and_does_not_mutate() {
    let mut cursor = PaginationCursor::new(
        page(1),
        25,
        PageLimit::new(1).unwrap_or_else(|_| unreachable!()),
    )
    .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        cursor.observe(metadata(1, Some(2)), 25, None),
        Err(PaginationError::PageLimitExceeded)
    );
    assert_eq!(cursor.pages_seen(), 0);
    assert_eq!(cursor.next_page(), Ok(page(1)));
}

#[test]
fn validates_provider_navigation_metadata() {
    assert_eq!(PageNumber::new(0), Err(PaginationError::PageZero));
    assert_eq!(PageLimit::new(0), Err(PaginationError::PageLimitZero));
    assert_eq!(
        PaginationCursor::new(
            page(1),
            0,
            PageLimit::new(1).unwrap_or_else(|_| unreachable!())
        ),
        Err(PaginationError::PerPageZero)
    );
    assert_eq!(
        PageMetadata::new(page(2), 25, None, Some(page(2)), None, None),
        Err(PaginationError::InvalidNextPage)
    );
    assert_eq!(
        PageMetadata::new(page(2), 25, None, Some(page(4)), Some(page(4)), None),
        Err(PaginationError::InvalidNextPage)
    );
    assert_eq!(
        PageMetadata::new(page(2), 25, Some(page(2)), None, None, None),
        Err(PaginationError::InvalidPreviousPage)
    );
    assert_eq!(
        PageMetadata::new(page(3), 25, Some(page(1)), None, None, None),
        Err(PaginationError::InvalidPreviousPage)
    );
    assert_eq!(
        PageMetadata::new(page(u64::MAX), 25, None, Some(page(1)), None, None),
        Err(PaginationError::InvalidNextPage)
    );
    assert_eq!(
        PageMetadata::new(page(2), 25, None, Some(page(3)), Some(page(2)), None),
        Err(PaginationError::InvalidLastPage)
    );
    assert_eq!(
        PageMetadata::new(page(1), 25, None, None, Some(page(4)), Some(100)),
        Err(PaginationError::InvalidLastPage)
    );
}

#[test]
fn rejects_entry_counts_and_totals_without_mutating_the_cursor() {
    let limit = PageLimit::new(4).unwrap_or_else(|_| unreachable!());
    let mut cursor = PaginationCursor::new(page(1), 25, limit).unwrap_or_else(|_| unreachable!());
    assert_eq!(
        cursor.observe(metadata(1, None), 26, None),
        Err(PaginationError::InvalidEntryCount)
    );

    let short = PageMetadata::new(page(1), 25, None, Some(page(2)), Some(page(4)), Some(100))
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        cursor.observe(short, 24, None),
        Err(PaginationError::InvalidEntryCount)
    );

    let premature = PageMetadata::new(page(1), 25, None, None, None, Some(100))
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        cursor.observe(premature, 25, None),
        Err(PaginationError::InvalidEntryCount)
    );
    assert_eq!(cursor.pages_seen(), 0);
    assert_eq!(cursor.next_page(), Ok(page(1)));
}

#[test]
fn binds_page_size_total_and_last_page_across_the_traversal() {
    let limit = PageLimit::new(4).unwrap_or_else(|_| unreachable!());
    let mut cursor = PaginationCursor::new(page(1), 25, limit).unwrap_or_else(|_| unreachable!());
    let wrong_first = PageMetadata::new(page(1), 50, None, None, Some(page(1)), Some(0))
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        cursor.observe(wrong_first, 0, None),
        Err(PaginationError::PageSizeChanged)
    );
    assert_eq!(cursor.pages_seen(), 0);

    let first = PageMetadata::new(page(1), 25, None, Some(page(2)), Some(page(4)), Some(100))
        .unwrap_or_else(|_| unreachable!());
    assert!(cursor.observe(first, 25, None).is_ok());

    let changed_size =
        PageMetadata::new(page(2), 50, Some(page(1)), None, Some(page(2)), Some(100))
            .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        cursor.observe(changed_size, 50, None),
        Err(PaginationError::PageSizeChanged)
    );

    let changed_total = PageMetadata::new(
        page(2),
        25,
        Some(page(1)),
        Some(page(3)),
        Some(page(4)),
        Some(101),
    )
    .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        cursor.observe(changed_total, 25, None),
        Err(PaginationError::TraversalChanged)
    );

    let changed_last = PageMetadata::new(
        page(2),
        25,
        Some(page(1)),
        Some(page(3)),
        Some(page(5)),
        Some(100),
    )
    .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        cursor.observe(changed_last, 25, None),
        Err(PaginationError::TraversalChanged)
    );
    assert_eq!(cursor.pages_seen(), 1);
    assert_eq!(cursor.next_page(), Ok(page(2)));

    let second = PageMetadata::new(
        page(2),
        25,
        Some(page(1)),
        Some(page(3)),
        Some(page(4)),
        Some(100),
    )
    .unwrap_or_else(|_| unreachable!());
    assert!(cursor.observe(second, 25, None).is_ok());
    assert_eq!(cursor.pages_seen(), 2);
    assert_eq!(cursor.next_page(), Ok(page(3)));
}
