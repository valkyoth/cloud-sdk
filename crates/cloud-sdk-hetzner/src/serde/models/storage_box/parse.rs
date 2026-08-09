use alloc::string::String;

use super::super::{ResponseModelError, UtcTimestamp, WipeString, checked_text, required};
use crate::serde::strict_json::{Map, Value};

const MAX_PROVIDER_ID: u64 = 9_007_199_254_740_991;

pub(super) fn take_text(
    fields: &mut Map,
    key: &str,
    maximum: usize,
) -> Result<WipeString, ResponseModelError> {
    let value = required_mut(fields, key)?
        .take_string()
        .ok_or(ResponseModelError::WrongType)?;
    let output = value
        .try_with_secret(|text| checked_text(text, maximum))
        .map_err(|_| ResponseModelError::InvalidText)??;
    Ok(WipeString::new(output))
}

pub(super) fn take_text_allow_empty(
    fields: &mut Map,
    key: &str,
    maximum: usize,
) -> Result<WipeString, ResponseModelError> {
    let value = required_mut(fields, key)?
        .take_string()
        .ok_or(ResponseModelError::WrongType)?;
    let output = value
        .try_with_secret(|text| {
            if text.len() > maximum || text.chars().any(super::super::is_unsafe_display_character) {
                return Err(ResponseModelError::InvalidText);
            }
            let mut output = String::new();
            output
                .try_reserve_exact(text.len())
                .map_err(|_| ResponseModelError::Allocation)?;
            output.push_str(text);
            Ok(output)
        })
        .map_err(|_| ResponseModelError::InvalidText)??;
    Ok(WipeString::new(output))
}

pub(super) fn take_optional_text(
    fields: &mut Map,
    key: &str,
    maximum: usize,
) -> Result<Option<WipeString>, ResponseModelError> {
    if required(fields, key)?.is_null() {
        Ok(None)
    } else {
        take_text(fields, key, maximum).map(Some)
    }
}

pub(super) fn take_timestamp(
    fields: &mut Map,
    key: &str,
) -> Result<UtcTimestamp, ResponseModelError> {
    let value = required_mut(fields, key)?
        .take_string()
        .ok_or(ResponseModelError::WrongType)?;
    let output = value
        .try_with_secret(|text| checked_text(text, 64))
        .map_err(|_| ResponseModelError::InvalidText)??;
    UtcTimestamp::try_from_string(output)
}

pub(super) fn positive(fields: &Map, key: &str) -> Result<u64, ResponseModelError> {
    required(fields, key)?
        .as_u64()
        .filter(|value| (1..=MAX_PROVIDER_ID).contains(value))
        .ok_or(ResponseModelError::InvalidIdentifier)
}

pub(super) fn number(fields: &Map, key: &str) -> Result<u64, ResponseModelError> {
    required(fields, key)?
        .as_u64()
        .ok_or(ResponseModelError::InvalidNumber)
}

pub(super) fn nullable_u64(fields: &Map, key: &str) -> Result<Option<u64>, ResponseModelError> {
    let value = required(fields, key)?;
    if value.is_null() {
        Ok(None)
    } else {
        value
            .as_u64()
            .map(Some)
            .ok_or(ResponseModelError::InvalidNumber)
    }
}

pub(super) fn boolean(fields: &Map, key: &str) -> Result<bool, ResponseModelError> {
    required(fields, key)?
        .as_bool()
        .ok_or(ResponseModelError::WrongType)
}

pub(super) fn ranged_u8(
    fields: &Map,
    key: &str,
    min: u8,
    max: u8,
) -> Result<u8, ResponseModelError> {
    let value = number(fields, key)
        .and_then(|value| u8::try_from(value).map_err(|_| ResponseModelError::InvalidNumber))?;
    (min..=max)
        .contains(&value)
        .then_some(value)
        .ok_or(ResponseModelError::InvalidNumber)
}

pub(super) fn nullable_ranged_u8(
    fields: &Map,
    key: &str,
    min: u8,
    max: u8,
) -> Result<Option<u8>, ResponseModelError> {
    nullable_u64(fields, key)?
        .map(|value| {
            let value = u8::try_from(value).map_err(|_| ResponseModelError::InvalidNumber)?;
            (min..=max)
                .contains(&value)
                .then_some(value)
                .ok_or(ResponseModelError::InvalidNumber)
        })
        .transpose()
}

pub(super) fn object_mut(value: &mut Value) -> Result<&mut Map, ResponseModelError> {
    value.as_object_mut().ok_or(ResponseModelError::WrongType)
}

pub(super) fn required_mut<'a>(
    fields: &'a mut Map,
    key: &str,
) -> Result<&'a mut Value, ResponseModelError> {
    fields.get_mut(key).ok_or(ResponseModelError::MissingField)
}
