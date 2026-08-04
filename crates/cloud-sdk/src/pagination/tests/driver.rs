use super::super::{
    NumberedPageMetadata, NumberedPageObservation, NumberedPagination, OffsetPageMetadata,
    OffsetPageObservation, OffsetPagination, PageNumber, PagerControl, PagerDriver,
    PagerDriverError, PagerStep, PaginationBudget, PaginationError, PaginationLimits,
    SnapshotPolicy,
};

fn page(value: u64) -> PageNumber {
    PageNumber::new(value).unwrap_or_else(|_| unreachable!())
}

#[test]
fn offset_strategy_uses_the_same_driver_contract() {
    let limits = PaginationLimits::new(2, 10, 64).unwrap_or_else(|_| unreachable!());
    let strategy = OffsetPagination::new(
        0,
        2,
        PaginationBudget::new(limits, SnapshotPolicy::Forbidden),
    )
    .unwrap_or_else(|_| unreachable!());
    let mut pager = PagerDriver::new(strategy);
    assert_eq!(
        pager.next_request(PagerControl::Continue),
        Ok(PagerStep::Request(0))
    );
    let metadata = OffsetPageMetadata::new(0, 2, None, Some(1)).unwrap_or_else(|_| unreachable!());
    let boundary = pager.observe(OffsetPageObservation::new(metadata, 1, None, None));
    assert!(boundary.is_ok_and(|value| value.is_terminal()));
    assert_eq!(
        pager.next_request(PagerControl::Continue),
        Ok(PagerStep::Complete)
    );
}

fn driver(max_requests: u32) -> PagerDriver<NumberedPagination> {
    let limits = PaginationLimits::new(max_requests, 100, 64).unwrap_or_else(|_| unreachable!());
    let strategy = NumberedPagination::new(
        page(1),
        2,
        PaginationBudget::new(limits, SnapshotPolicy::Forbidden),
    )
    .unwrap_or_else(|_| unreachable!());
    PagerDriver::new(strategy)
}

fn observation(
    current: u64,
    next: Option<u64>,
    entries: usize,
) -> NumberedPageObservation<'static> {
    let metadata = NumberedPageMetadata::new(
        page(current),
        2,
        current.checked_sub(1).filter(|value| *value != 0).map(page),
        next.map(page),
        Some(page(next.unwrap_or(current))),
        None,
    )
    .unwrap_or_else(|_| unreachable!());
    NumberedPageObservation::new(metadata, entries, None, None)
}

#[test]
fn pager_sequences_requests_responses_and_completion() {
    let mut pager = driver(2);
    assert_eq!(
        pager.next_request(PagerControl::Continue),
        Ok(PagerStep::Request(page(1)))
    );
    assert_eq!(
        pager.next_request(PagerControl::Continue),
        Err(PagerDriverError::ResponsePending)
    );
    assert!(pager.observe(observation(1, Some(2), 2)).is_ok());
    assert_eq!(
        pager.next_request(PagerControl::Continue),
        Ok(PagerStep::Request(page(2)))
    );
    assert!(pager.observe(observation(2, None, 1)).is_ok());
    assert_eq!(
        pager.next_request(PagerControl::Continue),
        Ok(PagerStep::Complete)
    );
    assert!(pager.is_terminal());
}

#[test]
fn pager_cancellation_and_strategy_failures_are_fail_closed() {
    let mut cancelled = driver(2);
    assert_eq!(
        cancelled.next_request(PagerControl::Cancel),
        Ok(PagerStep::Cancelled)
    );
    assert_eq!(
        cancelled.next_request(PagerControl::Continue),
        Err(PagerDriverError::Terminal)
    );

    let mut pager = driver(2);
    assert_eq!(
        pager.observe(observation(1, None, 1)),
        Err(PagerDriverError::UnexpectedObservation)
    );
    assert!(pager.next_request(PagerControl::Continue).is_ok());
    assert_eq!(
        pager.observe(observation(2, None, 1)),
        Err(PagerDriverError::Strategy(
            PaginationError::UnexpectedPosition
        ))
    );
    assert_eq!(
        pager.next_request(PagerControl::Continue),
        Err(PagerDriverError::ResponsePending)
    );
}
