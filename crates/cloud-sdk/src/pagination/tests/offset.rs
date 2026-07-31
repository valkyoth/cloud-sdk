use super::super::{
    OffsetPageMetadata, OffsetPagination, PaginationBudget, PaginationError, PaginationLimits,
    SnapshotPolicy,
};

fn traversal(requests: u32, items: u64) -> OffsetPagination {
    let limits = PaginationLimits::new(requests, items, 64).unwrap_or_else(|_| unreachable!());
    OffsetPagination::new(
        0,
        20,
        PaginationBudget::new(limits, SnapshotPolicy::Forbidden),
    )
    .unwrap_or_else(|_| unreachable!())
}

#[test]
fn walks_offsets_and_binds_exact_progression() {
    let mut offset_traversal = traversal(3, 50);
    let first =
        OffsetPageMetadata::new(0, 20, Some(20), Some(23)).unwrap_or_else(|_| unreachable!());
    assert!(offset_traversal.observe(first, 20, None, None).is_ok());
    assert_eq!(offset_traversal.next_offset(), Ok(20));
    let last = OffsetPageMetadata::new(20, 20, None, Some(23)).unwrap_or_else(|_| unreachable!());
    assert!(
        offset_traversal
            .observe(last, 3, None, None)
            .is_ok_and(|value| { value.is_terminal() && value.progress().items() == 23 })
    );
}

#[test]
fn rejects_offset_skips_total_drift_and_budget_overrun_transactionally() {
    let mut offset_traversal = traversal(3, 50);
    let skipped = OffsetPageMetadata::new(0, 20, Some(21), None).unwrap_or_else(|_| unreachable!());
    assert_eq!(
        offset_traversal.observe(skipped, 20, None, None),
        Err(PaginationError::InvalidNextPage)
    );
    assert_eq!(offset_traversal.next_offset(), Ok(0));

    let first =
        OffsetPageMetadata::new(0, 20, Some(20), Some(40)).unwrap_or_else(|_| unreachable!());
    assert!(offset_traversal.observe(first, 20, None, None).is_ok());
    let changed =
        OffsetPageMetadata::new(20, 20, None, Some(21)).unwrap_or_else(|_| unreachable!());
    assert_eq!(
        offset_traversal.observe(changed, 1, None, None),
        Err(PaginationError::TraversalChanged)
    );
    assert_eq!(offset_traversal.next_offset(), Ok(20));

    let mut limited = traversal(2, 19);
    let response =
        OffsetPageMetadata::new(0, 20, None, Some(20)).unwrap_or_else(|_| unreachable!());
    assert_eq!(
        limited.observe(response, 20, None, None),
        Err(PaginationError::ItemBudgetExceeded)
    );
    assert_eq!(limited.progress().items(), 0);
}
