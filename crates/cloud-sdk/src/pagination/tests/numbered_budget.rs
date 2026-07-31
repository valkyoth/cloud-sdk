use super::super::{
    NumberedPageMetadata, NumberedPagination, PageNumber, PaginationBudget, PaginationError,
    PaginationLimits, SnapshotId, SnapshotPolicy,
};
use crate::rate_limit::RateLimit;

fn page(value: u64) -> PageNumber {
    PageNumber::new(value).unwrap_or_else(|_| unreachable!())
}

fn limits(requests: u32, items: u64) -> PaginationLimits {
    PaginationLimits::new(requests, items, 128).unwrap_or_else(|_| unreachable!())
}

fn traversal(requests: u32, items: u64) -> NumberedPagination {
    NumberedPagination::new(
        page(1),
        25,
        PaginationBudget::new(limits(requests, items), SnapshotPolicy::Forbidden),
    )
    .unwrap_or_else(|_| unreachable!())
}

fn metadata(current: u64, next: Option<u64>) -> NumberedPageMetadata {
    NumberedPageMetadata::new(
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
fn walks_numbered_pages_with_hard_request_item_and_rate_limit_state() {
    let mut traversal = traversal(3, 30);
    let rate_limit = RateLimit::new(3600, 3599, 42).ok();
    let first = traversal.observe(metadata(1, Some(2)), 25, rate_limit, None);
    assert_eq!(first.map(|value| value.rate_limit()), Ok(rate_limit));
    assert_eq!(traversal.next_page(), Ok(page(2)));

    let last = traversal.observe(metadata(2, None), 3, None, None);
    assert!(last.is_ok_and(|value| {
        value.is_terminal() && value.progress().requests() == 2 && value.progress().items() == 28
    }));
    assert_eq!(traversal.next_page(), Err(PaginationError::Complete));
}

#[test]
fn request_and_item_budgets_fail_before_state_advances() {
    let mut request_limited = traversal(1, 100);
    assert_eq!(
        request_limited.observe(metadata(1, Some(2)), 25, None, None),
        Err(PaginationError::RequestBudgetExceeded)
    );
    assert_eq!(request_limited.next_page(), Ok(page(1)));
    assert_eq!(request_limited.progress().requests(), 0);

    let mut item_limited = traversal(2, 24);
    assert_eq!(
        item_limited.observe(metadata(1, Some(2)), 25, None, None),
        Err(PaginationError::ItemBudgetExceeded)
    );
    assert_eq!(item_limited.next_page(), Ok(page(1)));
    assert_eq!(item_limited.progress().items(), 0);
}

#[test]
fn snapshot_omission_presence_and_drift_are_explicit_and_transactional() {
    let snapshot = SnapshotId::new(b"snapshot-7").ok();
    let mut required = PaginationBudget::new(limits(3, 100), SnapshotPolicy::Required);
    assert_eq!(
        required.admit(1, true, None),
        Err(PaginationError::SnapshotRequired)
    );
    assert_eq!(required.progress().requests(), 0);
    assert!(required.admit(1, true, snapshot).is_ok());
    assert_eq!(
        required.admit(1, false, SnapshotId::new(b"snapshot-8").ok()),
        Err(PaginationError::SnapshotChanged)
    );
    assert_eq!(required.progress().requests(), 1);

    let mut forbidden = PaginationBudget::new(limits(2, 10), SnapshotPolicy::Forbidden);
    assert_eq!(
        forbidden.admit(1, false, snapshot),
        Err(PaginationError::SnapshotForbidden)
    );

    let mut optional = PaginationBudget::new(limits(3, 10), SnapshotPolicy::Optional);
    assert!(optional.admit(1, true, None).is_ok());
    assert_eq!(
        optional.admit(1, false, snapshot),
        Err(PaginationError::SnapshotChanged)
    );
}

#[test]
fn snapshot_identity_uses_exact_bounded_bytes_without_hash_collisions() {
    use super::super::MAX_SNAPSHOT_ID_BYTES;

    assert_eq!(SnapshotId::new(b""), Err(PaginationError::SnapshotIdEmpty));
    let exact = [b'x'; MAX_SNAPSHOT_ID_BYTES];
    assert!(SnapshotId::new(&exact).is_ok());
    let oversized = [b'x'; MAX_SNAPSHOT_ID_BYTES + 1];
    assert_eq!(
        SnapshotId::new(&oversized),
        Err(PaginationError::SnapshotIdTooLong)
    );

    let first = b"same-prefix-snapshot-a";
    let changed = b"same-prefix-snapshot-b";
    let mut budget = PaginationBudget::new(limits(3, 10), SnapshotPolicy::Required);
    assert!(budget.admit(1, true, SnapshotId::new(first).ok()).is_ok());
    assert_eq!(
        budget.admit(1, false, SnapshotId::new(changed).ok()),
        Err(PaginationError::SnapshotChanged)
    );
    assert_eq!(budget.progress().requests(), 1);

    let mut transactional = PaginationBudget::new(limits(1, 10), SnapshotPolicy::Required);
    assert_eq!(
        transactional.admit(1, true, SnapshotId::new(first).ok()),
        Err(PaginationError::RequestBudgetExceeded)
    );
    assert!(
        transactional
            .admit(1, false, SnapshotId::new(changed).ok())
            .is_ok()
    );
}

#[test]
fn validates_navigation_counts_and_locked_metadata() {
    assert_eq!(PageNumber::new(0), Err(PaginationError::PageZero));
    assert_eq!(
        NumberedPageMetadata::new(page(2), 25, None, Some(page(2)), None, None),
        Err(PaginationError::InvalidNextPage)
    );
    assert_eq!(
        NumberedPageMetadata::new(page(2), 25, Some(page(2)), None, None, None),
        Err(PaginationError::InvalidPreviousPage)
    );
    assert_eq!(
        NumberedPageMetadata::new(page(1), 25, None, None, Some(page(4)), Some(100)),
        Err(PaginationError::InvalidLastPage)
    );

    let mut traversal = traversal(4, 200);
    let first =
        NumberedPageMetadata::new(page(1), 25, None, Some(page(2)), Some(page(4)), Some(100))
            .unwrap_or_else(|_| unreachable!());
    assert!(traversal.observe(first, 25, None, None).is_ok());

    let changed = NumberedPageMetadata::new(
        page(2),
        25,
        Some(page(1)),
        Some(page(3)),
        Some(page(4)),
        Some(101),
    )
    .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        traversal.observe(changed, 25, None, None),
        Err(PaginationError::TraversalChanged)
    );
    assert_eq!(traversal.next_page(), Ok(page(2)));
    assert_eq!(traversal.progress().requests(), 1);
}

#[test]
fn accepts_empty_terminal_and_rejects_empty_continuation() {
    let mut terminal = traversal(1, 1);
    assert!(
        terminal
            .observe(metadata(1, None), 0, None, None)
            .is_ok_and(|value| value.is_terminal())
    );

    let mut nonterminal = traversal(2, 10);
    assert_eq!(
        nonterminal.observe(metadata(1, Some(2)), 0, None, None),
        Err(PaginationError::EmptyPageWithContinuation)
    );
}
