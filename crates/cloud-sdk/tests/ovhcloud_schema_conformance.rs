//! Source-bound OVHcloud schema-header conformance fixture.

use cloud_sdk::schema::{ReviewedSchemaMajor, SchemaVersion, ValidationSchemaHeader};

const PRINCIPLES_SHA256: [u8; 32] = [
    0xaf, 0xd1, 0x62, 0x53, 0xec, 0x3d, 0x12, 0x6c, 0x8c, 0xa1, 0xc6, 0x6b, 0x22, 0x9b, 0x8b, 0xe8,
    0x79, 0x76, 0xb3, 0x5a, 0xf3, 0x84, 0xff, 0xc3, 0x8e, 0x6f, 0x2d, 0x7f, 0x02, 0x90, 0x98, 0xae,
];
const SCHEMA_SHA256: [u8; 32] = [
    0x27, 0xa1, 0xc1, 0x72, 0xc0, 0x55, 0x61, 0x5e, 0x25, 0x67, 0xd4, 0x6f, 0x58, 0x3e, 0xd0, 0xf0,
    0xd4, 0xba, 0x2d, 0x77, 0xdf, 0x5a, 0x70, 0x32, 0x4b, 0x7a, 0x44, 0x6e, 0xea, 0xcc, 0x58, 0x5e,
];

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
