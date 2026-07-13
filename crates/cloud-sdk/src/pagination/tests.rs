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
        PageLimit::new(3).unwrap_or_else(|_| unreachable!()),
    );
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
        PageLimit::new(1).unwrap_or_else(|_| unreachable!()),
    );
    let boundary = cursor.observe(metadata(1, None), 0, None);
    assert!(boundary.is_ok_and(|value| value.is_terminal() && value.entries() == 0));
}

#[test]
fn rejects_empty_and_repeated_nonterminal_pages() {
    let mut cursor = PaginationCursor::new(
        page(1),
        PageLimit::new(3).unwrap_or_else(|_| unreachable!()),
    );
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
        PageLimit::new(1).unwrap_or_else(|_| unreachable!()),
    );
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
        PageMetadata::new(page(2), 25, None, Some(page(2)), None, None),
        Err(PaginationError::InvalidNextPage)
    );
    assert_eq!(
        PageMetadata::new(page(2), 25, Some(page(2)), None, None, None),
        Err(PaginationError::InvalidPreviousPage)
    );
    assert_eq!(
        PageMetadata::new(page(2), 25, None, Some(page(4)), Some(page(3)), None),
        Err(PaginationError::InvalidLastPage)
    );
}
