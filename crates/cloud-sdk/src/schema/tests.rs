use super::{ReviewedSchemaMajor, SchemaVersion, SchemaVersionError, ValidationSchemaHeader};

const DIGEST: [u8; 32] = [0x5a; 32];

fn evidence() -> ReviewedSchemaMajor {
    ReviewedSchemaMajor::new(1, DIGEST).unwrap_or_else(|_| unreachable!())
}

#[test]
fn parses_only_canonical_versions_and_rejects_unreviewed_majors() {
    assert_eq!(SchemaVersion::parse(b"1.0"), SchemaVersion::new(1, 0));
    assert_eq!(SchemaVersion::parse(b"12.34"), SchemaVersion::new(12, 34));
    for invalid in [
        b"".as_slice(),
        b"1",
        b".1",
        b"1.",
        b"01.0",
        b"1.00",
        b"1.0.0",
        b"1.a",
    ] {
        assert_eq!(
            SchemaVersion::parse(invalid),
            Err(SchemaVersionError::InvalidVersion)
        );
    }
    let version = SchemaVersion::new(2, 0).unwrap_or_else(|_| unreachable!());
    assert_eq!(
        evidence().validate(version),
        Err(SchemaVersionError::UnreviewedMajor)
    );
    assert_eq!(evidence().source_sha256(), DIGEST);
}

#[test]
fn validation_header_is_explicit_bounded_and_scratch_clearing() {
    let version = SchemaVersion::new(1, 0).unwrap_or_else(|_| unreachable!());
    let header = ValidationSchemaHeader::new("X-Schemas-Version", version, evidence())
        .unwrap_or_else(|_| unreachable!());
    let mut scratch = [0xa5_u8; 16];
    let observed = header.with_validation_header(&mut scratch, |value| {
        (
            value.name().as_str() == "X-Schemas-Version",
            value.value().as_str() == "1.0",
        )
    });
    assert_eq!(observed, Ok((true, true)));
    assert_eq!(scratch, [0; 16]);

    let mut too_small = [0xa5_u8; 2];
    assert_eq!(
        header.with_validation_header(&mut too_small, |_| ()),
        Err(SchemaVersionError::OutputTooSmall)
    );
    assert_eq!(too_small, [0; 2]);
}

#[test]
fn validation_header_rejects_invalid_names_and_major_drift() {
    let reviewed = evidence();
    let version = SchemaVersion::new(1, 0).unwrap_or_else(|_| unreachable!());
    assert!(matches!(
        ValidationSchemaHeader::new("bad header", version, reviewed),
        Err(SchemaVersionError::InvalidHeader)
    ));
    assert!(matches!(
        ValidationSchemaHeader::new("authorization", version, reviewed),
        Err(SchemaVersionError::InvalidHeader)
    ));
    let drift = SchemaVersion::new(2, 0).unwrap_or_else(|_| unreachable!());
    assert!(matches!(
        ValidationSchemaHeader::new("X-Schemas-Version", drift, reviewed),
        Err(SchemaVersionError::UnreviewedMajor)
    ));
}
