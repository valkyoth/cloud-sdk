use core::fmt;

/// Maximum bytes in one provider-owned audience, account, or tenant binding.
pub const MAX_SCOPE_VALUE_BYTES: usize = 512;

/// Authentication-scope value validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeValueError {
    /// Scope values must not be empty.
    Empty,
    /// Scope values exceed [`MAX_SCOPE_VALUE_BYTES`].
    TooLong,
    /// Scope values must contain visible ASCII without whitespace or controls.
    InvalidByte,
}

impl_static_error!(ScopeValueError,
    Self::Empty => "authentication scope value is empty",
    Self::TooLong => "authentication scope value exceeds the length limit",
    Self::InvalidByte => "authentication scope value contains an invalid byte",
);

/// Borrowed, bounded provider-owned authentication-scope value.
///
/// Diagnostics never expose the value. Providers decide whether a value is an
/// audience, account, or tenant identifier by the field in which it is used.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScopeValue<'a>(&'a str);

impl<'a> ScopeValue<'a> {
    /// Validates a provider-owned authentication-scope value.
    pub fn new(value: &'a str) -> Result<Self, ScopeValueError> {
        if value.is_empty() {
            return Err(ScopeValueError::Empty);
        }
        if value.len() > MAX_SCOPE_VALUE_BYTES {
            return Err(ScopeValueError::TooLong);
        }
        if !value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b'\\')
        {
            return Err(ScopeValueError::InvalidByte);
        }
        Ok(Self(value))
    }

    /// Returns the validated value for exact provider-policy comparison.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

impl fmt::Debug for ScopeValue<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ScopeValue([redacted])")
    }
}
