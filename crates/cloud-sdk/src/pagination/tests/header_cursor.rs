use super::super::header_cursor::DecodedHeaderCursor;
use super::super::{HeaderCursorPolicy, PaginationError, PaginationLimits};
use crate::operation::OperationId;
use crate::transport::{HeaderSensitivity, ResponseHeaders};

fn policy() -> HeaderCursorPolicy<'static> {
    HeaderCursorPolicy::new(
        OperationId::new("list_resources").unwrap_or_else(|_| unreachable!()),
        "X-Pagination-Cursor",
        "X-Pagination-Size",
        "X-Pagination-Cursor-Next",
        50,
    )
    .unwrap_or_else(|_| unreachable!())
}

fn limits(max_state_bytes: usize) -> PaginationLimits {
    PaginationLimits::new(8, 1_000, max_state_bytes).unwrap_or_else(|_| unreachable!())
}

#[test]
fn absent_next_header_is_terminal_and_clears_outputs() {
    let mut header_storage = [0_u8; 64];
    let headers = ResponseHeaders::new(&mut header_storage);
    let mut scratch = [0xa5_u8; 32];
    let mut destination = [0xa5_u8; 32];
    {
        let result = policy().decode_next(&headers, &mut scratch, &mut destination, limits(32));
        assert!(matches!(result, Ok(DecodedHeaderCursor::Complete)));
    }
    assert_eq!(scratch, [0; 32]);
    assert_eq!(destination, [0; 32]);
}

#[test]
fn next_cursor_round_trips_only_as_a_sensitive_request_header() {
    let mut header_storage = [0_u8; 128];
    let mut headers = ResponseHeaders::new(&mut header_storage);
    assert_eq!(
        headers.try_push(
            "X-Pagination-Cursor-Next",
            b"opaque-next-7",
            HeaderSensitivity::Sensitive,
        ),
        Ok(())
    );
    let mut transfer = [0xa5_u8; 32];
    let mut destination = [0xa5_u8; 32];
    {
        let next = policy()
            .decode_next(&headers, &mut transfer, &mut destination, limits(32))
            .unwrap_or_else(|_| unreachable!());
        let DecodedHeaderCursor::Continue(ref cursor) = next else {
            unreachable!("continuation fixture became terminal");
        };
        let mut decimal = [0xa5_u8; 20];
        let observed = policy().with_request_headers(Some(cursor), &mut decimal, |request| {
            let cursor = request.get("x-pagination-cursor");
            let size = request.get("x-pagination-size");
            (
                request.as_slice().len(),
                cursor.map(|value| value.value().as_str() == "opaque-next-7"),
                cursor.map(|value| value.sensitivity()),
                size.map(|value| value.value().as_str() == "50"),
            )
        });
        assert_eq!(
            observed,
            Ok((
                2,
                Some(true),
                Some(HeaderSensitivity::Sensitive),
                Some(true)
            ))
        );
        assert_eq!(decimal, [0; 20]);
    }
    assert_eq!(transfer, [0; 32]);
    assert_eq!(destination, [0; 32]);
}

#[test]
fn initial_request_contains_only_the_public_page_size() {
    let mut scratch = [0xa5_u8; 20];
    let result = policy().with_request_headers(None, &mut scratch, |headers| {
        (
            headers.as_slice().len(),
            headers.get("x-pagination-cursor").is_none(),
            headers
                .get("x-pagination-size")
                .map(|header| header.value().as_str() == "50"),
        )
    });
    assert_eq!(result, Ok((1, true, Some(true))));
    assert_eq!(scratch, [0; 20]);
}

#[test]
fn cursor_controls_oversize_empty_and_public_metadata_fail_closed() {
    for (value, sensitivity, expected) in [
        (
            b"".as_slice(),
            HeaderSensitivity::Sensitive,
            PaginationError::MissingState,
        ),
        (
            b"bad\tvalue".as_slice(),
            HeaderSensitivity::Sensitive,
            PaginationError::InvalidHeaderState,
        ),
        (
            b"non-ascii-\xff".as_slice(),
            HeaderSensitivity::Sensitive,
            PaginationError::InvalidHeaderState,
        ),
        (
            b"cursor".as_slice(),
            HeaderSensitivity::Public,
            PaginationError::InsecureHeaderState,
        ),
    ] {
        let mut header_storage = [0_u8; 128];
        let mut headers = ResponseHeaders::new(&mut header_storage);
        let pushed = headers.try_push("X-Pagination-Cursor-Next", value, sensitivity);
        if pushed.is_err() {
            assert_eq!(expected, PaginationError::InvalidHeaderState);
            continue;
        }
        let mut scratch = [0xa5_u8; 32];
        let mut destination = [0xa5_u8; 32];
        assert!(matches!(
            policy().decode_next(&headers, &mut scratch, &mut destination, limits(32)),
            Err(error) if error == expected
        ));
        assert_eq!(scratch, [0; 32]);
        assert_eq!(destination, [0; 32]);
    }

    let mut header_storage = [0_u8; 128];
    let mut headers = ResponseHeaders::new(&mut header_storage);
    assert!(
        headers
            .try_push(
                "X-Pagination-Cursor-Next",
                b"cursor-too-long",
                HeaderSensitivity::Sensitive,
            )
            .is_ok()
    );
    let mut scratch = [0xa5_u8; 32];
    let mut destination = [0xa5_u8; 32];
    assert!(matches!(
        policy().decode_next(&headers, &mut scratch, &mut destination, limits(4)),
        Err(PaginationError::StateTooLong)
    ));
}

#[test]
fn policy_rejects_zero_size_duplicate_roles_and_duplicate_response_headers() {
    let operation = OperationId::new("list_resources").unwrap_or_else(|_| unreachable!());
    assert!(matches!(
        HeaderCursorPolicy::new(operation, "x-cursor", "x-size", "x-next", 0),
        Err(PaginationError::PageSizeZero)
    ));
    assert!(matches!(
        HeaderCursorPolicy::new(operation, "x-cursor", "X-CURSOR", "x-next", 1),
        Err(PaginationError::InvalidHeaderPolicy)
    ));
    assert!(matches!(
        HeaderCursorPolicy::new(operation, "authorization", "x-size", "x-next", 1),
        Err(PaginationError::InvalidHeaderPolicy)
    ));
    assert!(matches!(
        HeaderCursorPolicy::new(operation, "x-cursor", "content-length", "x-next", 1),
        Err(PaginationError::InvalidHeaderPolicy)
    ));
    let mut storage = [0_u8; 128];
    let mut headers = ResponseHeaders::new(&mut storage);
    assert!(
        headers
            .try_push("x-next", b"first", HeaderSensitivity::Sensitive)
            .is_ok()
    );
    assert!(
        headers
            .try_push("X-NEXT", b"second", HeaderSensitivity::Sensitive)
            .is_err()
    );
}
