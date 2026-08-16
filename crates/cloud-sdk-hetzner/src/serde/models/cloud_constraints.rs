//! Allocation-free validation for source-locked Cloud schema constraints.

use crate::serde::strict_json::Value;

use super::ResponseModelError;

pub(super) fn validate_format(value: &Value, format: &str) -> Result<(), ResponseModelError> {
    if format == "-" {
        return Ok(());
    }
    match format {
        "int32" => value
            .as_i64()
            .filter(|value| i32::try_from(*value).is_ok())
            .map(|_| ())
            .ok_or(ResponseModelError::InvalidNumber),
        "int64" => value
            .as_i64()
            .map(|_| ())
            .ok_or(ResponseModelError::InvalidNumber),
        "double" => value
            .as_f64()
            .filter(|value| value.is_finite())
            .map(|_| ())
            .ok_or(ResponseModelError::InvalidNumber),
        "date-time" => validate_text(value, valid_rfc3339),
        "decimal" => validate_text(value, valid_decimal),
        _ => Err(ResponseModelError::SchemaMismatch),
    }
}

pub(super) fn validate_pattern(value: &Value, pattern: &str) -> Result<(), ResponseModelError> {
    if pattern == "-" {
        return Ok(());
    }
    let validator: fn(&str) -> bool = match pattern {
        "^[a-z0-9]+(-?[a-z0-9]*)*$" => valid_lowercase_identifier,
        r"^\S(.*\S)?$" => valid_trimmed_text,
        _ => return Err(ResponseModelError::SchemaMismatch),
    };
    validate_text(value, validator)
}

fn validate_text(value: &Value, validator: fn(&str) -> bool) -> Result<(), ResponseModelError> {
    value
        .try_with_str(validator)
        .map_err(|_| ResponseModelError::InvalidText)?
        .ok_or(ResponseModelError::WrongType)
        .and_then(|valid| {
            if valid {
                Ok(())
            } else {
                Err(ResponseModelError::InvalidText)
            }
        })
}

fn valid_decimal(value: &str) -> bool {
    let value = value.strip_prefix('-').unwrap_or(value);
    if value.is_empty() {
        return false;
    }
    let mut parts = value.split('.');
    let integer = parts.next().unwrap_or_default();
    let fraction = parts.next();
    if parts.next().is_some() || integer.is_empty() || !integer.bytes().all(is_digit) {
        return false;
    }
    fraction.is_none_or(|value| !value.is_empty() && value.bytes().all(is_digit))
}

pub(crate) fn valid_rfc3339(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < 20
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || !matches!(bytes.get(10), Some(b'T' | b't'))
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
    {
        return false;
    }
    let Some(year) = decimal_component(bytes, 0, 4) else {
        return false;
    };
    let Some(month) = decimal_component(bytes, 5, 2) else {
        return false;
    };
    let Some(day) = decimal_component(bytes, 8, 2) else {
        return false;
    };
    let Some(hour) = decimal_component(bytes, 11, 2) else {
        return false;
    };
    let Some(minute) = decimal_component(bytes, 14, 2) else {
        return false;
    };
    let Some(second) = decimal_component(bytes, 17, 2) else {
        return false;
    };
    if month == 0
        || month > 12
        || day == 0
        || day > days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return false;
    }

    let mut index = 19;
    if bytes.get(index) == Some(&b'.') {
        let Some(next) = index.checked_add(1) else {
            return false;
        };
        index = next;
        let start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            let Some(next) = index.checked_add(1) else {
                return false;
            };
            index = next;
        }
        if index == start {
            return false;
        }
    }
    match bytes.get(index) {
        Some(b'Z' | b'z') => index.checked_add(1) == Some(bytes.len()),
        Some(b'+' | b'-') => {
            let Some(hour_start) = index.checked_add(1) else {
                return false;
            };
            let Some(separator) = index.checked_add(3) else {
                return false;
            };
            let Some(minute_start) = index.checked_add(4) else {
                return false;
            };
            index.checked_add(6) == Some(bytes.len())
                && bytes.get(separator) == Some(&b':')
                && decimal_component(bytes, hour_start, 2).is_some_and(|value| value <= 23)
                && decimal_component(bytes, minute_start, 2).is_some_and(|value| value <= 59)
        }
        _ => false,
    }
}

fn decimal_component(value: &[u8], start: usize, length: usize) -> Option<u32> {
    let bytes = value.get(start..start.checked_add(length)?)?;
    if !bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }
    bytes.iter().try_fold(0_u32, |result, value| {
        result
            .checked_mul(10)?
            .checked_add(u32::from(value.checked_sub(b'0')?))
    })
}

const fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        2 if year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100)) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn valid_lowercase_identifier(value: &str) -> bool {
    value.as_bytes().split_first().is_some_and(|(first, rest)| {
        is_lowercase_alphanumeric(*first)
            && rest
                .iter()
                .all(|value| is_lowercase_alphanumeric(*value) || *value == b'-')
    })
}

fn valid_trimmed_text(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    !first.is_whitespace() && !characters.next_back().is_some_and(char::is_whitespace)
}

const fn is_lowercase_alphanumeric(value: u8) -> bool {
    value.is_ascii_lowercase() || value.is_ascii_digit()
}

const fn is_digit(value: u8) -> bool {
    value.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::{validate_format, validate_pattern};
    use crate::serde::models::ResponseModelError;
    use crate::serde::strict_json::parse;

    fn parsed(input: &[u8]) -> crate::serde::strict_json::Value {
        let value = parse(input);
        let Ok(value) = value else {
            unreachable!("constraint fixture failed to parse")
        };
        value
    }

    #[test]
    fn rfc3339_validation_checks_calendar_fraction_and_offset() {
        for input in [
            &br#""2024-02-29T23:59:60Z""#[..],
            &br#""2026-08-08t12:34:56.123+02:30""#[..],
        ] {
            assert_eq!(validate_format(&parsed(input), "date-time"), Ok(()));
        }
        for input in [
            &br#""2025-02-29T00:00:00Z""#[..],
            &br#""2026-01-01T24:00:00Z""#[..],
            &br#""2026-01-01T00:00:00""#[..],
            &br#""2026-01-01T00:00:00.Z""#[..],
        ] {
            assert_eq!(
                validate_format(&parsed(input), "date-time"),
                Err(ResponseModelError::InvalidText)
            );
        }
    }

    #[test]
    fn decimal_and_integer_formats_reject_ambiguous_or_wide_values() {
        for input in [&br#""1""#[..], &br#""-0.125""#[..]] {
            assert_eq!(validate_format(&parsed(input), "decimal"), Ok(()));
        }
        for input in [&br#""+1""#[..], &br#""1.""#[..], &br#""1e3""#[..]] {
            assert_eq!(
                validate_format(&parsed(input), "decimal"),
                Err(ResponseModelError::InvalidText)
            );
        }
        assert_eq!(validate_format(&parsed(b"2147483647"), "int32"), Ok(()));
        assert_eq!(
            validate_format(&parsed(b"2147483648"), "int32"),
            Err(ResponseModelError::InvalidNumber)
        );
        assert_eq!(
            validate_format(&parsed(b"9223372036854775808"), "int64"),
            Err(ResponseModelError::InvalidNumber)
        );
    }

    #[test]
    fn source_patterns_are_exact_and_unknown_patterns_fail_closed() {
        let identifier = "^[a-z0-9]+(-?[a-z0-9]*)*$";
        assert_eq!(
            validate_pattern(&parsed(br#""fsn1-dc14""#), identifier),
            Ok(())
        );
        assert_eq!(
            validate_pattern(&parsed(br#""FSN1""#), identifier),
            Err(ResponseModelError::InvalidText)
        );
        assert_eq!(
            validate_pattern(&parsed(br#"" value ""#), r"^\S(.*\S)?$"),
            Err(ResponseModelError::InvalidText)
        );
        assert_eq!(
            validate_pattern(&parsed(br#""value""#), "future-pattern"),
            Err(ResponseModelError::SchemaMismatch)
        );
    }
}
