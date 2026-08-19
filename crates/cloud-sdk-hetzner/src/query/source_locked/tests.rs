use super::*;

fn text(value: &str) -> SourceQueryText<'_> {
    SourceQueryText::new(value).unwrap_or_else(|_| unreachable!("valid query fixture"))
}

#[test]
fn image_query_expresses_every_scalar_and_repeated_filter() {
    let arguments = [
        SourceQueryArgument::text(SourceQueryParameter::Type, text("system")),
        SourceQueryArgument::text(SourceQueryParameter::Type, text("snapshot")),
        SourceQueryArgument::text(SourceQueryParameter::Status, text("available")),
        SourceQueryArgument::text(SourceQueryParameter::Sort, text("created:desc")),
        SourceQueryArgument::text(SourceQueryParameter::LabelSelector, text("env=prod")),
        SourceQueryArgument::text(SourceQueryParameter::Name, text("debian")),
        SourceQueryArgument::boolean(SourceQueryParameter::IncludeDeprecated, false),
        SourceQueryArgument::text(SourceQueryParameter::BoundTo, text("42")),
        SourceQueryArgument::text(SourceQueryParameter::Architecture, text("arm")),
    ];
    let query = SourceLockedQuery::try_new(SourceQueryOperation::LIST_IMAGES, &arguments)
        .unwrap_or_else(|_| unreachable!("valid query fixture"));
    let mut output = [0xaa; 256];
    let written = query
        .write_query(&mut output)
        .unwrap_or_else(|_| unreachable!());
    let actual = output
        .get(..written)
        .and_then(|encoded| core::str::from_utf8(encoded).ok())
        .unwrap_or("");
    assert_eq!(
        actual,
        "architecture=arm&bound_to=42&include_deprecated=false&label_selector=env%3Dprod&name=debian&sort=created%3Adesc&status=available&type=system&type=snapshot"
    );
}

#[test]
fn validates_operation_ownership_required_fields_and_cardinality() {
    let cpu = SourceQueryArgument::text(SourceQueryParameter::Type, text("cpu"));
    assert_eq!(
        SourceLockedQuery::try_new(SourceQueryOperation::GET_SERVER_METRICS, &[cpu]),
        Err(SourceQueryError::MissingRequiredParameter)
    );
    assert_eq!(
        SourceLockedQuery::try_new(SourceQueryOperation::LIST_SERVERS, &[cpu]),
        Err(SourceQueryError::UnknownParameter)
    );
    let page = SourceQueryArgument::integer(SourceQueryParameter::Page, 1);
    assert_eq!(
        SourceLockedQuery::try_new(SourceQueryOperation::LIST_SERVERS, &[page, page]),
        Err(SourceQueryError::DuplicateScalar)
    );
    let image_type = SourceQueryArgument::text(SourceQueryParameter::Type, text("system"));
    assert_eq!(
        SourceLockedQuery::try_new(SourceQueryOperation::LIST_IMAGES, &[image_type, image_type]),
        Err(SourceQueryError::DuplicateArrayValue)
    );
}

#[test]
fn rejects_wrong_types_invalid_enums_steps_and_timestamps() {
    let wrong = SourceQueryArgument::boolean(SourceQueryParameter::Page, true);
    assert_eq!(
        SourceLockedQuery::try_new(SourceQueryOperation::LIST_SERVERS, &[wrong]),
        Err(SourceQueryError::WrongValueKind)
    );
    let invalid = SourceQueryArgument::text(SourceQueryParameter::Architecture, text("sparc"));
    assert_eq!(
        SourceLockedQuery::try_new(SourceQueryOperation::LIST_IMAGES, &[invalid]),
        Err(SourceQueryError::InvalidEnumValue)
    );
    let metrics = [
        SourceQueryArgument::text(SourceQueryParameter::End, text("2026-08-18T01:00:00Z")),
        SourceQueryArgument::text(SourceQueryParameter::Start, text("not-a-timestamp")),
        SourceQueryArgument::text(SourceQueryParameter::Step, text("0")),
        SourceQueryArgument::text(SourceQueryParameter::Type, text("cpu")),
    ];
    assert_eq!(
        SourceLockedQuery::try_new(SourceQueryOperation::GET_SERVER_METRICS, &metrics),
        Err(SourceQueryError::InvalidTimestamp)
    );

    let non_decimal_timestamp = [
        SourceQueryArgument::text(SourceQueryParameter::End, text("2026-09-01T00:00:00Z")),
        SourceQueryArgument::text(SourceQueryParameter::Start, text("2026-08-0:T00:00:00Z")),
        SourceQueryArgument::text(SourceQueryParameter::Type, text("cpu")),
    ];
    assert_eq!(
        SourceLockedQuery::try_new(
            SourceQueryOperation::GET_SERVER_METRICS,
            &non_decimal_timestamp,
        ),
        Err(SourceQueryError::InvalidTimestamp)
    );
}

#[test]
fn metrics_use_the_source_documented_comma_encoding() {
    let arguments = [
        SourceQueryArgument::text(SourceQueryParameter::End, text("2026-08-18T01:00:00Z")),
        SourceQueryArgument::text(SourceQueryParameter::Start, text("2026-08-18T00:00:00Z")),
        SourceQueryArgument::text(SourceQueryParameter::Step, text("60")),
        SourceQueryArgument::text(SourceQueryParameter::Type, text("network")),
        SourceQueryArgument::text(SourceQueryParameter::Type, text("cpu")),
    ];
    let query = SourceLockedQuery::try_new(SourceQueryOperation::GET_SERVER_METRICS, &arguments)
        .unwrap_or_else(|_| unreachable!());
    let mut output = [0_u8; 128];
    let written = query
        .write_query(&mut output)
        .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        output
            .get(..written)
            .and_then(|encoded| core::str::from_utf8(encoded).ok())
            .unwrap_or(""),
        "end=2026-08-18T01%3A00%3A00Z&start=2026-08-18T00%3A00%3A00Z&step=60&type=network%2Ccpu"
    );
}

#[test]
fn failed_encoding_preserves_the_output_snapshot() {
    let arguments = [SourceQueryArgument::text(
        SourceQueryParameter::Name,
        text("long name"),
    )];
    let query = SourceLockedQuery::try_new(SourceQueryOperation::LIST_SERVERS, &arguments)
        .unwrap_or_else(|_| unreachable!());
    let mut output = [0xaa; 3];
    assert_eq!(
        query.write_query(&mut output),
        Err(SourceQueryError::QueryBufferTooSmall)
    );
    assert_eq!(output, [0xaa, 0xaa, 0xaa]);
}
