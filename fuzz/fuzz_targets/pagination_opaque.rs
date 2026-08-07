#![no_main]

use cloud_sdk::operation::OperationId;
use cloud_sdk::pagination::{
    CursorDigest, CursorHistory, HeaderCursorPolicy, PaginationCursor, PaginationError,
    PaginationLimits, PaginationMarker,
};
use cloud_sdk::schema::SchemaVersion;
use cloud_sdk::transport::{HeaderSensitivity, ResponseHeaders};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let state_limit = usize::from(read_u16(data, 0) % 8_192).saturating_add(1);
    let output_len = usize::from(read_u16(data, 2) % 8_193);
    let Ok(limits) = PaginationLimits::new(8, 1_000, state_limit) else {
        return;
    };
    let state = data.get(4..).unwrap_or_default();
    let mut source = state.get(..8_192).unwrap_or(state).to_vec();
    let mut collision_source = source.clone();
    if let Some(first) = collision_source.first_mut() {
        *first ^= 0xff;
    }
    let mut marker_source = source.clone();
    let mut destination = vec![0xa5_u8; output_len];

    {
        let result = PaginationCursor::transfer_from(&mut source, &mut destination, limits);
        match result {
            Ok(cursor) => {
                assert!(source.iter().all(|byte| *byte == 0));
                exercise_history(&cursor, &mut collision_source, limits, data);
                drop(cursor);
            }
            Err(_) => assert!(source.iter().all(|byte| *byte == 0)),
        }
    }
    assert!(destination.iter().all(|byte| *byte == 0));

    let mut marker_destination = vec![0xa5_u8; output_len];
    let marker =
        PaginationMarker::transfer_from(&mut marker_source, &mut marker_destination, limits);
    assert!(marker_source.iter().all(|byte| *byte == 0));
    drop(marker);
    assert!(marker_destination.iter().all(|byte| *byte == 0));

    exercise_header_cursor(data, limits);
    let _ = SchemaVersion::parse(data);
});

fn exercise_header_cursor(data: &[u8], limits: PaginationLimits) {
    let Ok(operation) = OperationId::new("fuzz_header_cursor") else {
        return;
    };
    let Ok(policy) = HeaderCursorPolicy::new(operation, "x-cursor", "x-size", "x-next", 50) else {
        return;
    };
    let mut header_storage = vec![0_u8; 2_048];
    let mut headers = ResponseHeaders::new(&mut header_storage);
    let value = data.get(..data.len().min(1_024)).unwrap_or_default();
    let sensitivity = if data.first().is_some_and(|byte| byte & 1 == 0) {
        HeaderSensitivity::Sensitive
    } else {
        HeaderSensitivity::Public
    };
    if headers.try_push("x-next", value, sensitivity).is_err() {
        return;
    }
    let mut scratch = vec![0xa5_u8; 8_192];
    let mut destination = vec![0xa5_u8; 8_192];
    {
        let next = policy.decode_next(&headers, &mut scratch, &mut destination, limits);
        if let Ok(next) = next {
            if let cloud_sdk::pagination::HeaderCursorNext::Continue(continuation) = next {
                let mut decimal = [0xa5_u8; 20];
                let result = continuation.with_request_headers(&mut decimal, |_| ());
                assert!(result.is_ok());
                assert_eq!(decimal, [0; 20]);
            }
        }
    }
    assert!(scratch.iter().all(|byte| *byte == 0));
    assert!(destination.iter().all(|byte| *byte == 0));
}

fn exercise_history(
    cursor: &PaginationCursor<'_>,
    collision_source: &mut [u8],
    limits: PaginationLimits,
    data: &[u8],
) {
    let mut history_storage = vec![0xa5_u8; 16_384];
    let Ok(mut history) = CursorHistory::new(&mut history_storage, 4) else {
        return;
    };
    let digest = CursorDigest::new(digest_bytes(data));
    assert_eq!(history.observe(cursor, digest), Ok(()));
    assert_eq!(
        history.observe(cursor, digest),
        Err(PaginationError::CursorCycle)
    );

    let mut collision_destination = vec![0xa5_u8; 8_192];
    if let Ok(collision) =
        PaginationCursor::transfer_from(collision_source, &mut collision_destination, limits)
    {
        if !collision.with_cursor(|value| cursor.with_cursor(|stored| value == stored)) {
            assert_eq!(
                history.observe(&collision, digest),
                Err(PaginationError::CursorDigestCollision)
            );
        }
    }
    drop(history);
    assert!(history_storage.iter().all(|byte| *byte == 0));
}

fn digest_bytes(data: &[u8]) -> [u8; 32] {
    let mut digest = [0_u8; 32];
    for (output, input) in digest.iter_mut().zip(data.iter().copied()) {
        *output = input;
    }
    digest
}

fn read_u16(data: &[u8], start: usize) -> u16 {
    let Some(end) = start.checked_add(2) else {
        return 0;
    };
    let Some(bytes) = data.get(start..end) else {
        return 0;
    };
    let Ok(bytes) = <[u8; 2]>::try_from(bytes) else {
        return 0;
    };
    u16::from_be_bytes(bytes)
}
