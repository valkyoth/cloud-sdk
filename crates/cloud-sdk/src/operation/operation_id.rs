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
    pub const fn new(value: &'static str) -> Result<Self, OperationIdError> {
        if value.is_empty() {
            return Err(OperationIdError::Empty);
        }
        if value.len() > MAX_OPERATION_ID_BYTES {
            return Err(OperationIdError::TooLong);
        }
        let mut remaining = value.as_bytes();
        while let Some((byte, tail)) = remaining.split_first() {
            if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'_' {
                return Err(OperationIdError::InvalidByte);
            }
            remaining = tail;
        }
        Ok(Self(value))
    }

    /// Returns the validated identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// Creates a compile-time validated [`OperationId`].
///
/// Invalid literals fail during compilation:
///
/// ```compile_fail
/// use cloud_sdk::operation_id;
///
/// let _ = operation_id!("Get-Server");
/// ```
#[macro_export]
macro_rules! operation_id {
    ($value:literal) => {{
        const VALUE: $crate::operation::OperationId =
            match $crate::operation::OperationId::new($value) {
                Ok(value) => value,
                Err(_) => panic!("invalid operation identifier literal"),
            };
        VALUE
    }};
}

#[cfg(test)]
mod tests {
    use super::{OperationId, OperationIdError};

    const GET_SERVER: OperationId = operation_id!("get_server");

    #[test]
    fn accepts_source_style_identifiers_and_rejects_ambiguous_text() {
        assert_eq!(GET_SERVER.as_str(), "get_server");
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
