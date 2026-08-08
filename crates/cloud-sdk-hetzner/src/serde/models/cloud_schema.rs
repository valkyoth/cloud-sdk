//! Source-derived validation for ordinary Hetzner Cloud resource models.

use crate::serde::strict_json::Value;

use super::ResponseModelError;
use super::cloud_constraints::{validate_format, validate_pattern};

const TABLE: &str = include_str!("../cloud_model_schema.tsv");

pub(super) fn validate_model(model: &str, value: &Value) -> Result<(), ResponseModelError> {
    value.as_object().ok_or(ResponseModelError::WrongType)?;
    let mut found = false;
    for line in TABLE.lines().skip(1) {
        let mut fields = line.split('\t');
        let Some(row_model) = fields.next() else {
            return Err(ResponseModelError::SchemaMismatch);
        };
        if row_model != model {
            continue;
        }
        found = true;
        let descriptor = Descriptor::parse(&mut fields)?;
        validate_path(value, descriptor.path, descriptor)?;
    }
    if !found {
        return Err(ResponseModelError::SchemaMismatch);
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct Descriptor<'a> {
    path: &'a str,
    required: bool,
    types: &'a str,
    minimum: &'a str,
    maximum: &'a str,
    min_length: &'a str,
    max_length: &'a str,
    min_items: &'a str,
    max_items: &'a str,
    format: &'a str,
    pattern: &'a str,
}

impl<'a> Descriptor<'a> {
    fn parse(fields: &mut impl Iterator<Item = &'a str>) -> Result<Self, ResponseModelError> {
        let path = fields.next().ok_or(ResponseModelError::SchemaMismatch)?;
        let required = match fields.next() {
            Some("yes") => true,
            Some("no") => false,
            _ => return Err(ResponseModelError::SchemaMismatch),
        };
        let descriptor = Self {
            path,
            required,
            types: fields.next().ok_or(ResponseModelError::SchemaMismatch)?,
            minimum: fields.next().ok_or(ResponseModelError::SchemaMismatch)?,
            maximum: fields.next().ok_or(ResponseModelError::SchemaMismatch)?,
            min_length: fields.next().ok_or(ResponseModelError::SchemaMismatch)?,
            max_length: fields.next().ok_or(ResponseModelError::SchemaMismatch)?,
            min_items: fields.next().ok_or(ResponseModelError::SchemaMismatch)?,
            max_items: fields.next().ok_or(ResponseModelError::SchemaMismatch)?,
            format: fields.next().ok_or(ResponseModelError::SchemaMismatch)?,
            pattern: fields.next().ok_or(ResponseModelError::SchemaMismatch)?,
        };
        if fields.next().is_none() || fields.next().is_some() {
            return Err(ResponseModelError::SchemaMismatch);
        }
        Ok(descriptor)
    }
}

fn validate_path(
    current: &Value,
    path: &str,
    descriptor: Descriptor<'_>,
) -> Result<(), ResponseModelError> {
    if current.is_null() {
        return Ok(());
    }
    let (segment, tail) = path
        .split_once('/')
        .map_or((path, None), |(head, tail)| (head, Some(tail)));
    let segment = Segment::parse(segment)?;
    let object = current.as_object().ok_or(ResponseModelError::WrongType)?;
    let Some(value) = object.get(segment.name) else {
        if tail.is_none() && descriptor.required {
            return Err(ResponseModelError::MissingField);
        }
        return Ok(());
    };

    match segment.collection {
        Collection::None => match tail {
            Some(tail) => validate_path(value, tail, descriptor),
            None => validate_value(value, descriptor),
        },
        Collection::All => {
            let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
            for item in values {
                match tail {
                    Some(tail) => validate_path(item, tail, descriptor)?,
                    None => validate_value(item, descriptor)?,
                }
            }
            Ok(())
        }
        Collection::Selected {
            field,
            value: expected,
        } => {
            let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
            for item in values {
                if selector_matches(item, field, expected)? {
                    match tail {
                        Some(tail) => validate_path(item, tail, descriptor)?,
                        None => validate_value(item, descriptor)?,
                    }
                }
            }
            Ok(())
        }
    }
}

fn selector_matches(
    value: &Value,
    field: &str,
    expected: &str,
) -> Result<bool, ResponseModelError> {
    let object = value.as_object().ok_or(ResponseModelError::WrongType)?;
    let Some(value) = object.get(field) else {
        return Ok(false);
    };
    value
        .try_with_str(|value| value == expected)
        .map_err(|_| ResponseModelError::InvalidText)?
        .ok_or(ResponseModelError::WrongType)
}

#[derive(Clone, Copy)]
struct Segment<'a> {
    name: &'a str,
    collection: Collection<'a>,
}

impl<'a> Segment<'a> {
    fn parse(value: &'a str) -> Result<Self, ResponseModelError> {
        if let Some(name) = value.strip_suffix("[]") {
            return nonempty(name).map(|name| Self {
                name,
                collection: Collection::All,
            });
        }
        if let Some((name, selector)) = value.split_once('[') {
            let selector = selector
                .strip_suffix(']')
                .ok_or(ResponseModelError::SchemaMismatch)?;
            let (field, expected) = selector
                .split_once('=')
                .ok_or(ResponseModelError::SchemaMismatch)?;
            return Ok(Self {
                name: nonempty(name)?,
                collection: Collection::Selected {
                    field: nonempty(field)?,
                    value: nonempty(expected)?,
                },
            });
        }
        Ok(Self {
            name: nonempty(value)?,
            collection: Collection::None,
        })
    }
}

#[derive(Clone, Copy)]
enum Collection<'a> {
    None,
    All,
    Selected { field: &'a str, value: &'a str },
}

fn nonempty(value: &str) -> Result<&str, ResponseModelError> {
    if value.is_empty() {
        Err(ResponseModelError::SchemaMismatch)
    } else {
        Ok(value)
    }
}

fn validate_value(value: &Value, descriptor: Descriptor<'_>) -> Result<(), ResponseModelError> {
    if value.is_null() {
        return if admits(descriptor.types, "null") {
            Ok(())
        } else {
            Err(ResponseModelError::WrongType)
        };
    }
    let type_matches = (admits(descriptor.types, "boolean") && value.as_bool().is_some())
        || (admits(descriptor.types, "integer") && value.is_integer())
        || (admits(descriptor.types, "number") && value.is_number())
        || (admits(descriptor.types, "string") && value.is_string())
        || (admits(descriptor.types, "array") && value.as_array().is_some())
        || (admits(descriptor.types, "object") && value.as_object().is_some());
    if !type_matches {
        return Err(ResponseModelError::WrongType);
    }
    validate_number(value, descriptor.minimum, descriptor.maximum)?;
    validate_string(value, descriptor.min_length, descriptor.max_length)?;
    validate_format(value, descriptor.format)?;
    validate_pattern(value, descriptor.pattern)?;
    validate_items(value, descriptor.min_items, descriptor.max_items)
}

fn admits(types: &str, expected: &str) -> bool {
    types.split('|').any(|value| value == expected)
}

fn validate_number(value: &Value, minimum: &str, maximum: &str) -> Result<(), ResponseModelError> {
    let Some(value) = value.as_f64() else {
        return Ok(());
    };
    if !value.is_finite() {
        return Err(ResponseModelError::InvalidNumber);
    }
    if let Some(minimum) = bound(minimum)?
        && value < minimum
    {
        return Err(ResponseModelError::InvalidNumber);
    }
    if let Some(maximum) = bound(maximum)?
        && value > maximum
    {
        return Err(ResponseModelError::InvalidNumber);
    }
    Ok(())
}

fn bound(value: &str) -> Result<Option<f64>, ResponseModelError> {
    if value == "-" {
        return Ok(None);
    }
    value
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
        .map(Some)
        .ok_or(ResponseModelError::SchemaMismatch)
}

fn validate_string(value: &Value, minimum: &str, maximum: &str) -> Result<(), ResponseModelError> {
    value
        .try_with_str(|text| {
            if text.len() > 1_048_576 || text.chars().any(unsafe_character) {
                return Err(ResponseModelError::InvalidText);
            }
            let character_count = text.chars().count();
            validate_usize_bound(
                character_count,
                minimum,
                maximum,
                ResponseModelError::InvalidText,
            )
        })
        .map_err(|_| ResponseModelError::InvalidText)?
        .unwrap_or(Ok(()))
}

fn validate_items(value: &Value, minimum: &str, maximum: &str) -> Result<(), ResponseModelError> {
    let Some(values) = value.as_array() else {
        return Ok(());
    };
    validate_usize_bound(
        values.len(),
        minimum,
        maximum,
        ResponseModelError::TooManyItems,
    )
}

fn validate_usize_bound(
    actual: usize,
    minimum: &str,
    maximum: &str,
    error: ResponseModelError,
) -> Result<(), ResponseModelError> {
    let minimum = usize_bound(minimum)?;
    let maximum = usize_bound(maximum)?;
    if minimum.is_some_and(|value| actual < value) || maximum.is_some_and(|value| actual > value) {
        return Err(error);
    }
    Ok(())
}

fn usize_bound(value: &str) -> Result<Option<usize>, ResponseModelError> {
    if value == "-" {
        Ok(None)
    } else {
        value
            .parse::<usize>()
            .map(Some)
            .map_err(|_| ResponseModelError::SchemaMismatch)
    }
}

fn unsafe_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{061c}'
                | '\u{200b}'..='\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2060}'..='\u{2069}'
                | '\u{feff}'
        )
}

#[cfg(test)]
mod tests {
    use alloc::format;

    use super::validate_string;
    use crate::serde::models::ResponseModelError;
    use crate::serde::strict_json::parse;

    #[test]
    fn source_string_limits_count_unicode_scalars_not_utf8_bytes() {
        let exact = format!("\"{}\"", "é".repeat(128));
        let exact = parse(exact.as_bytes());
        let Ok(exact) = exact else {
            unreachable!("Unicode boundary fixture failed")
        };
        assert_eq!(validate_string(&exact, "1", "128"), Ok(()));

        let over = format!("\"{}\"", "é".repeat(129));
        let over = parse(over.as_bytes());
        let Ok(over) = over else {
            unreachable!("Unicode over-bound fixture failed")
        };
        assert_eq!(
            validate_string(&over, "1", "128"),
            Err(ResponseModelError::InvalidText)
        );
    }
}
