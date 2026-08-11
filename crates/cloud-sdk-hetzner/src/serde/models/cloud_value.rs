//! Public bounded field tree retained by source-complete Cloud models.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::serde::strict_json::{Map, Value};

use super::ResponseModelError;

/// One number retained without coercing integers through a floating-point type.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CloudNumber {
    /// Non-negative integer.
    Unsigned(u64),
    /// Negative integer.
    Signed(i64),
    /// Finite JSON number with a fractional or exponent component.
    Float(f64),
}

/// One fully retained field in an ordinary Hetzner Cloud resource model.
#[derive(PartialEq)]
pub enum CloudValue {
    /// Explicit JSON null.
    Null,
    /// Boolean value.
    Bool(bool),
    /// Integer or finite floating-point number.
    Number(CloudNumber),
    /// Bounded display-safe text, including unknown future enum values.
    Text(String),
    /// Bounded array.
    Array(Vec<Self>),
    /// Bounded object.
    Object(CloudObject),
}

impl CloudValue {
    /// Fallibly copies this value and all nested allocation-backed fields.
    pub fn try_clone(&self) -> Result<Self, ResponseModelError> {
        match self {
            Self::Null => Ok(Self::Null),
            Self::Bool(value) => Ok(Self::Bool(*value)),
            Self::Number(value) => Ok(Self::Number(*value)),
            Self::Text(value) => copy_string(value).map(Self::Text),
            Self::Array(values) => {
                let mut output = Vec::new();
                output
                    .try_reserve_exact(values.len())
                    .map_err(|_| ResponseModelError::Allocation)?;
                for value in values {
                    output.push(value.try_clone()?);
                }
                Ok(Self::Array(output))
            }
            Self::Object(value) => value.try_clone().map(Self::Object),
        }
    }

    /// Returns text without interpreting a future enum value.
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(value) => Some(value),
            _ => None,
        }
    }

    /// Returns a non-negative integer.
    #[must_use]
    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Number(CloudNumber::Unsigned(value)) => Some(*value),
            Self::Number(CloudNumber::Signed(value)) => u64::try_from(*value).ok(),
            _ => None,
        }
    }

    /// Returns a boolean.
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns an array.
    #[must_use]
    pub fn as_array(&self) -> Option<&[Self]> {
        match self {
            Self::Array(value) => Some(value),
            _ => None,
        }
    }

    /// Returns an object.
    #[must_use]
    pub const fn as_object(&self) -> Option<&CloudObject> {
        match self {
            Self::Object(value) => Some(value),
            _ => None,
        }
    }

    /// Reports whether this field was explicitly null.
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub(super) fn from_private(value: &Value) -> Result<Self, ResponseModelError> {
        match value {
            Value::Null => Ok(Self::Null),
            Value::Bool(_) => value
                .as_bool()
                .map(Self::Bool)
                .ok_or(ResponseModelError::WrongType),
            Value::Number(_) if value.is_integer() => {
                if let Some(value) = value.as_u64() {
                    Ok(Self::Number(CloudNumber::Unsigned(value)))
                } else {
                    value
                        .as_i64()
                        .map(CloudNumber::Signed)
                        .map(Self::Number)
                        .ok_or(ResponseModelError::InvalidNumber)
                }
            }
            Value::Number(_) => value
                .as_f64()
                .filter(|value| value.is_finite())
                .map(CloudNumber::Float)
                .map(Self::Number)
                .ok_or(ResponseModelError::InvalidNumber),
            Value::String(_) => value
                .try_with_str(copy_text)
                .map_err(|_| ResponseModelError::InvalidText)?
                .ok_or(ResponseModelError::WrongType)?,
            Value::Array(values) => {
                let mut output = Vec::new();
                output
                    .try_reserve_exact(values.len())
                    .map_err(|_| ResponseModelError::Allocation)?;
                for value in values {
                    output.push(Self::from_private(value)?);
                }
                Ok(Self::Array(output))
            }
            Value::Object(values) => CloudObject::from_private(values).map(Self::Object),
        }
    }
}

impl fmt::Debug for CloudValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => formatter.write_str("Null"),
            Self::Bool(_) => formatter.write_str("Bool([redacted])"),
            Self::Number(_) => formatter.write_str("Number([redacted])"),
            Self::Text(_) => formatter.write_str("Text([redacted])"),
            Self::Array(values) => formatter
                .debug_struct("Array")
                .field("item_count", &values.len())
                .field("items", &"[redacted]")
                .finish(),
            Self::Object(value) => formatter.debug_tuple("Object").field(value).finish(),
        }
    }
}

/// Sorted source field collection for one resource or nested object.
///
/// This allocation-heavy type deliberately does not implement [`Clone`]. Use
/// [`Self::try_clone`] when an owned copy is required.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::serde::CloudObject;
///
/// fn requires_clone<T: Clone>() {}
/// requires_clone::<CloudObject>();
/// ```
#[derive(PartialEq)]
pub struct CloudObject(Vec<(String, CloudValue)>);

impl CloudObject {
    /// Fallibly copies every retained field and nested value.
    pub fn try_clone(&self) -> Result<Self, ResponseModelError> {
        let mut output = Vec::new();
        output
            .try_reserve_exact(self.0.len())
            .map_err(|_| ResponseModelError::Allocation)?;
        for (name, value) in &self.0 {
            output.push((copy_string(name)?, value.try_clone()?));
        }
        Ok(Self(output))
    }

    /// Returns a field by its exact provider name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&CloudValue> {
        self.0
            .binary_search_by(|(candidate, _)| candidate.as_str().cmp(name))
            .ok()
            .and_then(|index| self.0.get(index))
            .map(|(_, value)| value)
    }

    /// Returns a text field. Explicit null and absence both return `None`;
    /// use [`Self::get`] when that distinction matters.
    #[must_use]
    pub fn text(&self, name: &str) -> Option<&str> {
        self.get(name).and_then(CloudValue::as_text)
    }

    /// Returns a non-negative integer field.
    #[must_use]
    pub fn u64(&self, name: &str) -> Option<u64> {
        self.get(name).and_then(CloudValue::as_u64)
    }

    /// Returns a boolean field.
    #[must_use]
    pub fn boolean(&self, name: &str) -> Option<bool> {
        self.get(name).and_then(CloudValue::as_bool)
    }

    /// Returns the number of retained fields, including unknown future fields.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Reports whether the object has no fields.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates over source fields in stable lexical order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &CloudValue)> {
        self.0.iter().map(|(name, value)| (name.as_str(), value))
    }

    pub(super) fn from_value(value: &Value) -> Result<Self, ResponseModelError> {
        value
            .as_object()
            .ok_or(ResponseModelError::WrongType)
            .and_then(Self::from_private)
    }

    fn from_private(value: &Map) -> Result<Self, ResponseModelError> {
        let mut output = Vec::new();
        output
            .try_reserve_exact(value.len())
            .map_err(|_| ResponseModelError::Allocation)?;
        for (name, value) in value.iter() {
            let name = copy_key(name.as_str())?;
            output.push((name, CloudValue::from_private(value)?));
        }
        Ok(Self(output))
    }
}

impl fmt::Debug for CloudObject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CloudObject")
            .field("field_count", &self.0.len())
            .field("fields", &"[redacted]")
            .finish()
    }
}

fn copy_key(value: &str) -> Result<String, ResponseModelError> {
    if value.is_empty() || value.len() > 256 || value.chars().any(unsafe_character) {
        return Err(ResponseModelError::InvalidText);
    }
    copy_string(value)
}

fn copy_text(value: &str) -> Result<CloudValue, ResponseModelError> {
    if value.len() > 1_048_576 || value.chars().any(unsafe_character) {
        return Err(ResponseModelError::InvalidText);
    }
    copy_string(value).map(CloudValue::Text)
}

fn copy_string(value: &str) -> Result<String, ResponseModelError> {
    let mut output = String::new();
    output
        .try_reserve_exact(value.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    output.push_str(value);
    Ok(output)
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
