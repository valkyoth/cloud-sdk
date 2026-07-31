use super::super::{
    CursorDigest, CursorHistory, MAX_OPAQUE_STATE_BYTES, PaginationCursor, PaginationError,
    PaginationLimits, PaginationMarker,
};
use super::assert_redacted;

fn limits(state_bytes: usize) -> PaginationLimits {
    PaginationLimits::new(8, 100, state_bytes).unwrap_or_else(|_| unreachable!())
}

fn cursor<'a>(
    source: &mut [u8],
    destination: &'a mut [u8],
) -> Result<PaginationCursor<'a>, PaginationError> {
    PaginationCursor::transfer_from(source, destination, limits(64))
}

#[test]
fn transfers_cursor_and_marker_atomically_and_clears_complete_storage() {
    let mut cursor_source = *b"opaque-cursor";
    let mut cursor_storage = [0xa5_u8; 32];
    {
        let cursor =
            cursor(&mut cursor_source, &mut cursor_storage).unwrap_or_else(|_| unreachable!());
        assert!(cursor.with_cursor(|value| value == b"opaque-cursor"));
        assert_eq!(cursor_source, [0; 13]);
        assert_redacted(&cursor);
    }
    assert_eq!(cursor_storage, [0; 32]);

    let mut marker_source = *b"marker-7";
    let mut marker_storage = [0xa5_u8; 24];
    {
        let marker =
            PaginationMarker::transfer_from(&mut marker_source, &mut marker_storage, limits(16))
                .unwrap_or_else(|_| unreachable!());
        assert!(marker.with_marker(|value| value == b"marker-7"));
    }
    assert_eq!(marker_source, [0; 8]);
    assert_eq!(marker_storage, [0; 24]);
}

#[test]
fn failed_transfer_clears_source_and_destination_without_partial_state() {
    let mut source = *b"too-long";
    let mut destination = [0xa5_u8; 4];
    assert!(matches!(
        PaginationCursor::transfer_from(&mut source, &mut destination, limits(32)),
        Err(PaginationError::OutputTooSmall)
    ));
    assert_eq!(source, [0; 8]);
    assert_eq!(destination, [0; 4]);

    let mut source = *b"oversized";
    let mut destination = [0xa5_u8; 16];
    assert!(matches!(
        PaginationCursor::transfer_from(&mut source, &mut destination, limits(4)),
        Err(PaginationError::StateTooLong)
    ));
    assert_eq!(source, [0; 9]);
    assert_eq!(destination, [0; 16]);

    let mut history_storage = [0xa5_u8; 16];
    assert!(matches!(
        CursorHistory::new(&mut history_storage, 0),
        Err(PaginationError::ZeroLimit)
    ));
    assert_eq!(history_storage, [0; 16]);
}

#[test]
fn exact_opaque_hard_bound_is_admitted_and_plus_one_limit_is_rejected() {
    let mut source = [b'x'; MAX_OPAQUE_STATE_BYTES];
    let mut destination = [0_u8; MAX_OPAQUE_STATE_BYTES];
    {
        let cursor = PaginationCursor::transfer_from(
            &mut source,
            &mut destination,
            limits(MAX_OPAQUE_STATE_BYTES),
        )
        .unwrap_or_else(|_| unreachable!());
        assert!(cursor.with_cursor(|value| value.len() == MAX_OPAQUE_STATE_BYTES));
    }
    assert!(source.iter().all(|byte| *byte == 0));
    assert!(destination.iter().all(|byte| *byte == 0));
    assert_eq!(
        PaginationLimits::new(1, 1, MAX_OPAQUE_STATE_BYTES + 1),
        Err(PaginationError::StateLimitTooLarge)
    );
}

#[test]
fn history_detects_exact_cycles_digest_collisions_and_digest_changes() {
    let mut history_storage = [0xa5_u8; 256];
    {
        let mut history =
            CursorHistory::new(&mut history_storage, 4).unwrap_or_else(|_| unreachable!());
        let mut first_source = *b"cursor-a";
        let mut first_storage = [0_u8; 16];
        let first =
            cursor(&mut first_source, &mut first_storage).unwrap_or_else(|_| unreachable!());
        let first_digest = CursorDigest::new([1; 32]);
        assert_eq!(history.observe(&first, first_digest), Ok(()));
        assert_eq!(history.entries(), 1);

        let mut repeated_source = *b"cursor-a";
        let mut repeated_storage = [0_u8; 16];
        let repeated =
            cursor(&mut repeated_source, &mut repeated_storage).unwrap_or_else(|_| unreachable!());
        assert_eq!(
            history.observe(&repeated, first_digest),
            Err(PaginationError::CursorCycle)
        );
        assert_eq!(
            history.observe(&repeated, CursorDigest::new([2; 32])),
            Err(PaginationError::CursorDigestChanged)
        );

        let mut collision_source = *b"cursor-b";
        let mut collision_storage = [0_u8; 16];
        let collision = cursor(&mut collision_source, &mut collision_storage)
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(
            history.observe(&collision, first_digest),
            Err(PaginationError::CursorDigestCollision)
        );
        assert_eq!(history.entries(), 1);
    }
    assert_eq!(history_storage, [0; 256]);
}

#[test]
fn history_entry_and_byte_budgets_fail_without_appending() {
    let mut history_storage = [0_u8; 48];
    let mut history =
        CursorHistory::new(&mut history_storage, 1).unwrap_or_else(|_| unreachable!());
    let mut first_source = *b"first";
    let mut first_storage = [0_u8; 8];
    let first = cursor(&mut first_source, &mut first_storage).unwrap_or_else(|_| unreachable!());
    assert_eq!(history.observe(&first, CursorDigest::new([1; 32])), Ok(()));
    let mut second_source = *b"second";
    let mut second_storage = [0_u8; 8];
    let second = cursor(&mut second_source, &mut second_storage).unwrap_or_else(|_| unreachable!());
    assert_eq!(
        history.observe(&second, CursorDigest::new([2; 32])),
        Err(PaginationError::HistoryBudgetExceeded)
    );
    assert_eq!(history.entries(), 1);
}
