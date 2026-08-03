//! Limits for bounded incremental decoding.

use core::fmt;

use super::super::MAX_SERDE_RESPONSE_BYTES;

const HARD_MAX_DEPTH: usize = 64;
const HARD_MAX_TOKENS: usize = 65_536;
const HARD_MAX_FIELDS: usize = 65_536;
const HARD_MAX_OBJECT_FIELDS: usize = 4_096;
const HARD_MAX_STRING_BYTES: usize = 1_048_576;
const HARD_MAX_NUMBER_BYTES: usize = 128;
const HARD_MAX_EXPONENT_DIGITS: usize = 6;

/// Resource ceilings applied to one incremental JSON document.
///
/// Defaults match or strengthen the checked full-tree decoder. Builder methods
/// can only lower a ceiling, never exceed the reviewed hard maximum.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IncrementalJsonLimits {
    pub(crate) input_bytes: usize,
    pub(crate) depth: usize,
    pub(crate) tokens: usize,
    pub(crate) fields: usize,
    pub(crate) object_fields: usize,
    pub(crate) string_bytes: usize,
    pub(crate) number_bytes: usize,
    pub(crate) exponent_digits: usize,
}

impl IncrementalJsonLimits {
    /// Reviewed default limits.
    pub const DEFAULT: Self = Self {
        input_bytes: MAX_SERDE_RESPONSE_BYTES,
        depth: HARD_MAX_DEPTH,
        tokens: HARD_MAX_TOKENS,
        fields: HARD_MAX_FIELDS,
        object_fields: HARD_MAX_OBJECT_FIELDS,
        string_bytes: HARD_MAX_STRING_BYTES,
        number_bytes: HARD_MAX_NUMBER_BYTES,
        exponent_digits: HARD_MAX_EXPONENT_DIGITS,
    };

    /// Lowers the aggregate input-byte limit.
    pub fn with_input_bytes(mut self, limit: usize) -> Result<Self, IncrementalJsonLimitsError> {
        self.input_bytes = checked_nonzero(limit, MAX_SERDE_RESPONSE_BYTES)?;
        Ok(self)
    }

    /// Lowers the maximum number of open arrays and objects. Zero permits only scalars.
    pub fn with_depth(mut self, limit: usize) -> Result<Self, IncrementalJsonLimitsError> {
        self.depth = checked(limit, HARD_MAX_DEPTH)?;
        Ok(self)
    }

    /// Lowers the aggregate token limit. Values and object keys each charge one token.
    pub fn with_tokens(mut self, limit: usize) -> Result<Self, IncrementalJsonLimitsError> {
        self.tokens = checked_nonzero(limit, HARD_MAX_TOKENS)?;
        Ok(self)
    }

    /// Lowers the aggregate object-field limit.
    pub fn with_fields(mut self, limit: usize) -> Result<Self, IncrementalJsonLimitsError> {
        self.fields = checked_nonzero(limit, HARD_MAX_FIELDS)?;
        Ok(self)
    }

    /// Lowers the number of fields allowed in any one object.
    pub fn with_object_fields(mut self, limit: usize) -> Result<Self, IncrementalJsonLimitsError> {
        self.object_fields = checked_nonzero(limit, HARD_MAX_OBJECT_FIELDS)?;
        Ok(self)
    }

    /// Lowers the decoded-byte limit for each string or key.
    pub fn with_string_bytes(mut self, limit: usize) -> Result<Self, IncrementalJsonLimitsError> {
        self.string_bytes = checked_nonzero(limit, HARD_MAX_STRING_BYTES)?;
        Ok(self)
    }

    /// Lowers the byte limit for each complete JSON number token.
    pub fn with_number_bytes(mut self, limit: usize) -> Result<Self, IncrementalJsonLimitsError> {
        self.number_bytes = checked_nonzero(limit, HARD_MAX_NUMBER_BYTES)?;
        Ok(self)
    }

    /// Lowers the digit limit for each JSON number exponent.
    pub fn with_exponent_digits(
        mut self,
        limit: usize,
    ) -> Result<Self, IncrementalJsonLimitsError> {
        self.exponent_digits = checked_nonzero(limit, HARD_MAX_EXPONENT_DIGITS)?;
        Ok(self)
    }
}

impl Default for IncrementalJsonLimits {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Invalid incremental limit configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IncrementalJsonLimitsError;

impl fmt::Display for IncrementalJsonLimitsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("incremental JSON limit is zero or exceeds its reviewed maximum")
    }
}

impl core::error::Error for IncrementalJsonLimitsError {}

fn checked(limit: usize, maximum: usize) -> Result<usize, IncrementalJsonLimitsError> {
    (limit <= maximum)
        .then_some(limit)
        .ok_or(IncrementalJsonLimitsError)
}

fn checked_nonzero(limit: usize, maximum: usize) -> Result<usize, IncrementalJsonLimitsError> {
    (limit != 0 && limit <= maximum)
        .then_some(limit)
        .ok_or(IncrementalJsonLimitsError)
}
