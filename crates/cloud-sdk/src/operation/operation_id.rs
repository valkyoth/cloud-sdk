//! Bounded provider operation identifiers.

use core::fmt;

/// Maximum bytes in a provider operation identifier.
pub const MAX_OPERATION_ID_BYTES: usize = 128;

/// Invalid provider operation identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperationIdError {
    /// The identifier is empty.
    Empty,
    /// The identifier exceeds [`MAX_OPERATION_ID_BYTES`].
    TooLong,
    /// The identifier contains a byte outside lowercase ASCII, digits, or `_`.
    InvalidByte,
}

impl fmt::Display for OperationIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "operation identifier is empty",
            Self::TooLong => "operation identifier is too long",
            Self::InvalidByte => "operation identifier contains an invalid byte",
        })
    }
}

impl core::error::Error for OperationIdError {}

/// Validated static identifier assigned by a provider specification.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationId(&'static str);

impl OperationId {
    /// Validates a provider operation identifier.
    pub fn new(value: &'static str) -> Result<Self, OperationIdError> {
        if value.is_empty() {
            return Err(OperationIdError::Empty);
        }
        if value.len() > MAX_OPERATION_ID_BYTES {
            return Err(OperationIdError::TooLong);
        }
        if value
            .bytes()
            .any(|byte| !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && byte != b'_')
        {
            return Err(OperationIdError::InvalidByte);
        }
        Ok(Self(value))
    }

    /// Returns the validated identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{OperationId, OperationIdError};

    #[test]
    fn accepts_source_style_identifiers_and_rejects_ambiguous_text() {
        assert_eq!(
            OperationId::new("get_server").map(OperationId::as_str),
            Ok("get_server")
        );
        assert_eq!(OperationId::new(""), Err(OperationIdError::Empty));
        assert_eq!(
            OperationId::new("Get-Server"),
            Err(OperationIdError::InvalidByte)
        );
    }
}
