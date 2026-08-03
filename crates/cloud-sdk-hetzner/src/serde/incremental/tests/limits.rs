use alloc::format;

use super::super::{IncrementalJsonError, IncrementalJsonLimits};
use super::support::decode_with_limits;

fn valid_limits(
    result: Result<IncrementalJsonLimits, super::super::IncrementalJsonLimitsError>,
) -> IncrementalJsonLimits {
    assert!(result.is_ok());
    result.unwrap_or_default()
}

#[test]
fn input_limit_is_exact_and_charges_whitespace_amplification() {
    let limits = valid_limits(IncrementalJsonLimits::DEFAULT.with_input_bytes(4));
    assert!(decode_with_limits(b"null", limits).is_ok());
    assert!(matches!(
        decode_with_limits(b"null ", limits),
        Err(IncrementalJsonError::InputLimit)
    ));
}

#[test]
fn depth_limit_is_exact() {
    let limits = valid_limits(IncrementalJsonLimits::DEFAULT.with_depth(1));
    assert!(decode_with_limits(b"[0]", limits).is_ok());
    assert!(matches!(
        decode_with_limits(b"[[0]]", limits),
        Err(IncrementalJsonError::DepthLimit)
    ));
    let scalars = valid_limits(IncrementalJsonLimits::DEFAULT.with_depth(0));
    assert!(decode_with_limits(b"0", scalars).is_ok());
    assert!(matches!(
        decode_with_limits(b"[]", scalars),
        Err(IncrementalJsonError::DepthLimit)
    ));
}

#[test]
fn token_limit_counts_values_and_keys() {
    let exact = valid_limits(IncrementalJsonLimits::DEFAULT.with_tokens(3));
    assert!(decode_with_limits(br#"{"a":0}"#, exact).is_ok());
    let too_low = valid_limits(IncrementalJsonLimits::DEFAULT.with_tokens(2));
    assert!(matches!(
        decode_with_limits(br#"{"a":0}"#, too_low),
        Err(IncrementalJsonError::TokenLimit)
    ));
}

#[test]
fn aggregate_and_per_object_field_limits_are_independent() {
    let aggregate = valid_limits(IncrementalJsonLimits::DEFAULT.with_fields(1));
    assert!(decode_with_limits(br#"{"a":0}"#, aggregate).is_ok());
    assert!(matches!(
        decode_with_limits(br#"{"a":{"b":0}}"#, aggregate),
        Err(IncrementalJsonError::FieldLimit)
    ));

    let per_object = valid_limits(IncrementalJsonLimits::DEFAULT.with_object_fields(1));
    assert!(decode_with_limits(br#"{"a":{"b":0}}"#, per_object).is_ok());
    assert!(matches!(
        decode_with_limits(br#"{"a":0,"b":1}"#, per_object),
        Err(IncrementalJsonError::ObjectFieldLimit)
    ));
}

#[test]
fn decoded_string_limit_handles_raw_utf8_and_escapes() {
    let two = valid_limits(IncrementalJsonLimits::DEFAULT.with_string_bytes(2));
    assert!(decode_with_limits("\"é\"".as_bytes(), two).is_ok());
    assert!(decode_with_limits(br#""\u00e9""#, two).is_ok());
    let one = valid_limits(IncrementalJsonLimits::DEFAULT.with_string_bytes(1));
    assert!(matches!(
        decode_with_limits("\"é\"".as_bytes(), one),
        Err(IncrementalJsonError::StringLimit)
    ));
    assert!(matches!(
        decode_with_limits(br#""\u00e9""#, one),
        Err(IncrementalJsonError::StringLimit)
    ));
}

#[test]
fn number_and_exponent_limits_are_exact() {
    let number = valid_limits(IncrementalJsonLimits::DEFAULT.with_number_bytes(3));
    assert!(decode_with_limits(b"123", number).is_ok());
    assert!(matches!(
        decode_with_limits(b"1234", number),
        Err(IncrementalJsonError::NumberLimit)
    ));

    let exponent = valid_limits(IncrementalJsonLimits::DEFAULT.with_exponent_digits(2));
    assert!(decode_with_limits(b"1e12", exponent).is_ok());
    assert!(matches!(
        decode_with_limits(b"1e123", exponent),
        Err(IncrementalJsonError::ExponentLimit)
    ));
}

#[test]
fn limit_builders_reject_zero_or_above_hard_maxima() {
    assert!(IncrementalJsonLimits::DEFAULT.with_tokens(0).is_err());
    assert!(
        IncrementalJsonLimits::DEFAULT
            .with_number_bytes(129)
            .is_err()
    );
    assert!(
        IncrementalJsonLimits::DEFAULT
            .with_exponent_digits(7)
            .is_err()
    );
    assert!(
        IncrementalJsonLimits::DEFAULT
            .with_input_bytes(8_388_609)
            .is_err()
    );
}

#[test]
fn aggregate_token_amplification_fails_at_configured_boundary() {
    let limits = valid_limits(IncrementalJsonLimits::DEFAULT.with_tokens(8));
    let exact = format!("[{}]", ["0"; 7].join(","));
    let over = format!("[{}]", ["0"; 8].join(","));
    assert!(decode_with_limits(exact.as_bytes(), limits).is_ok());
    assert!(matches!(
        decode_with_limits(over.as_bytes(), limits),
        Err(IncrementalJsonError::TokenLimit)
    ));
}
