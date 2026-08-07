//! Source-bound OVHcloud cursor and schema-header conformance fixtures.

use cloud_sdk::operation::OperationId;
use cloud_sdk::pagination::{
    CursorDigest, CursorHistory, HeaderCursorNext, HeaderCursorPolicy, PaginationError,
    PaginationLimits,
};
use cloud_sdk::schema::{ReviewedSchemaMajor, SchemaVersion, ValidationSchemaHeader};
use cloud_sdk::transport::{HeaderSensitivity, ResponseHeaders};

const PRINCIPLES_SHA256: [u8; 32] = [
    0xaf, 0xd1, 0x62, 0x53, 0xec, 0x3d, 0x12, 0x6c, 0x8c, 0xa1, 0xc6, 0x6b, 0x22, 0x9b, 0x8b, 0xe8,
    0x79, 0x76, 0xb3, 0x5a, 0xf3, 0x84, 0xff, 0xc3, 0x8e, 0x6f, 0x2d, 0x7f, 0x02, 0x90, 0x98, 0xae,
];
const SCHEMA_SHA256: [u8; 32] = [
    0x27, 0xa1, 0xc1, 0x72, 0xc0, 0x55, 0x61, 0x5e, 0x25, 0x67, 0xd4, 0x6f, 0x58, 0x3e, 0xd0, 0xf0,
    0xd4, 0xba, 0x2d, 0x77, 0xdf, 0x5a, 0x70, 0x32, 0x4b, 0x7a, 0x44, 0x6e, 0xea, 0xcc, 0x58, 0x5e,
];

fn policy() -> HeaderCursorPolicy<'static> {
    HeaderCursorPolicy::new(
        OperationId::new("ovhcloud_iam_policy_list").unwrap_or_else(|_| unreachable!()),
        "X-Pagination-Cursor",
        "X-Pagination-Size",
        "X-Pagination-Cursor-Next",
        5,
    )
    .unwrap_or_else(|_| unreachable!())
}

fn limits() -> PaginationLimits {
    PaginationLimits::new(8, 1_000, 64).unwrap_or_else(|_| unreachable!())
}

#[test]
fn source_locked_header_cursor_round_trip_and_terminal_signal_are_exact() {
    let mut response_storage = [0_u8; 128];
    let mut headers = ResponseHeaders::new(&mut response_storage);
    assert!(
        headers
            .try_push(
                "X-Pagination-Cursor-Next",
                b"source-locked-cursor",
                HeaderSensitivity::Sensitive,
            )
            .is_ok()
    );
    let mut transfer = [0xa5_u8; 64];
    let mut destination = [0xa5_u8; 64];
    {
        let HeaderCursorNext::Continue(continuation) = policy()
            .decode_next(&headers, &mut transfer, &mut destination, limits())
            .unwrap_or_else(|_| unreachable!())
        else {
            unreachable!("source-locked continuation became terminal");
        };
        let mut history_storage = [0_u8; 256];
        let mut history =
            CursorHistory::new(&mut history_storage, 4).unwrap_or_else(|_| unreachable!());
        let digest = CursorDigest::new([0x42; 32]);
        assert_eq!(continuation.observe_history(&mut history, digest), Ok(()));
        assert_eq!(
            continuation.observe_history(&mut history, digest),
            Err(PaginationError::CursorCycle)
        );
        let mut decimal = [0xa5_u8; 20];
        assert_eq!(
            continuation.with_request_headers(&mut decimal, |request| {
                request.get("X-Pagination-Cursor").map(|header| {
                    (
                        header.value().as_str() == "source-locked-cursor",
                        header.sensitivity(),
                    )
                })
            }),
            Ok(Some((true, HeaderSensitivity::Sensitive)))
        );
        assert_eq!(decimal, [0; 20]);
    }
    assert_eq!(transfer, [0; 64]);
    assert_eq!(destination, [0; 64]);

    drop(headers);
    let empty = ResponseHeaders::new(&mut response_storage);
    let terminal = policy().decode_next(&empty, &mut transfer, &mut destination, limits());
    assert!(terminal.is_ok_and(|next| next.is_complete()));
}

#[test]
fn raw_metadata_rejects_duplicate_next_headers_before_decoding() {
    let mut storage = [0_u8; 128];
    let mut headers = ResponseHeaders::new(&mut storage);
    assert!(
        headers
            .try_push(
                "X-Pagination-Cursor-Next",
                b"first",
                HeaderSensitivity::Sensitive,
            )
            .is_ok()
    );
    assert!(
        headers
            .try_push(
                "x-pagination-cursor-next",
                b"second",
                HeaderSensitivity::Sensitive,
            )
            .is_err()
    );
}

#[test]
fn schema_override_is_validation_only_and_bound_to_reviewed_major() {
    let evidence = ReviewedSchemaMajor::new(1, SCHEMA_SHA256).unwrap_or_else(|_| unreachable!());
    let version = SchemaVersion::parse(b"1.0").unwrap_or_else(|_| unreachable!());
    let validation = ValidationSchemaHeader::new("X-Schemas-Version", version, evidence)
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(validation.evidence().source_sha256(), SCHEMA_SHA256);
    assert_ne!(validation.evidence().source_sha256(), PRINCIPLES_SHA256);
    let mut scratch = [0xa5_u8; 16];
    assert_eq!(
        validation.with_validation_header(&mut scratch, |header| {
            (
                header.name().as_str() == "X-Schemas-Version",
                header.value().as_str() == "1.0",
            )
        }),
        Ok((true, true))
    );
    assert_eq!(scratch, [0; 16]);
}
