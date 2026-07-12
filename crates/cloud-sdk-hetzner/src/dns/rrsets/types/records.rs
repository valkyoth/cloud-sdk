//! Validated DNS record values and mutation lists.

use core::fmt;

use cloud_sdk::buffer;

use super::RrsetRequestError;

/// Maximum record value bytes admitted by the SDK.
pub const MAX_RECORD_VALUE_BYTES: usize = 65_535;
/// Maximum record comment bytes admitted by the SDK.
pub const MAX_RECORD_COMMENT_BYTES: usize = 1_024;
/// Maximum records admitted in one RRSet mutation.
pub const MAX_RECORDS_PER_REQUEST: usize = 50;

/// Bounded DNS record value.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RecordValue<'a>(&'a str);

impl<'a> RecordValue<'a> {
    /// Creates a nonempty value without control or bidi-control characters.
    pub fn new(value: &'a str) -> Result<Self, RrsetRequestError> {
        if invalid_record_text(value, MAX_RECORD_VALUE_BYTES, true) {
            return Err(RrsetRequestError::InvalidRecordValue);
        }
        Ok(Self(value))
    }

    /// Writes a complete escaped JSON string. Failure leaves output unchanged.
    pub fn write_json_string(self, output: &mut [u8]) -> Result<usize, RrsetRequestError> {
        write_json(self.0, output)
    }
}

impl fmt::Debug for RecordValue<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RecordValue([redacted])")
    }
}

/// Bounded DNS record comment.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RecordComment<'a>(&'a str);

impl<'a> RecordComment<'a> {
    /// Creates a comment without control or bidi-control characters.
    pub fn new(value: &'a str) -> Result<Self, RrsetRequestError> {
        if invalid_record_text(value, MAX_RECORD_COMMENT_BYTES, false) {
            return Err(RrsetRequestError::InvalidRecordComment);
        }
        Ok(Self(value))
    }

    /// Writes a complete escaped JSON string. Failure leaves output unchanged.
    pub fn write_json_string(self, output: &mut [u8]) -> Result<usize, RrsetRequestError> {
        write_json(self.0, output)
    }
}

impl fmt::Debug for RecordComment<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RecordComment([redacted])")
    }
}

/// Record used by create, set, add, and remove operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Record<'a> {
    value: RecordValue<'a>,
    comment: Option<RecordComment<'a>>,
}

impl<'a> Record<'a> {
    /// Creates a record identified by its value.
    #[must_use]
    pub const fn new(value: RecordValue<'a>) -> Self {
        Self {
            value,
            comment: None,
        }
    }

    /// Sets an optional record comment.
    #[must_use]
    pub const fn with_comment(mut self, comment: RecordComment<'a>) -> Self {
        self.comment = Some(comment);
        self
    }

    /// Returns the value marker.
    #[must_use]
    pub const fn value(self) -> RecordValue<'a> {
        self.value
    }

    /// Returns the comment marker when supplied.
    #[must_use]
    pub const fn comment(self) -> Option<RecordComment<'a>> {
        self.comment
    }
}

/// Nonempty record list with unique values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Records<'a>(&'a [Record<'a>]);

impl<'a> Records<'a> {
    /// Validates `1..=50` records and value uniqueness.
    pub fn new(values: &'a [Record<'a>]) -> Result<Self, RrsetRequestError> {
        validate_record_count(values.len())?;
        for (index, value) in values.iter().enumerate() {
            if values
                .get(..index)
                .is_none_or(|previous| previous.iter().any(|item| item.value == value.value))
            {
                return Err(RrsetRequestError::DuplicateRecord);
            }
        }
        Ok(Self(values))
    }

    /// Returns validated records.
    #[must_use]
    pub const fn entries(self) -> &'a [Record<'a>] {
        self.0
    }
}

/// Record comment update, identified by value with a required new comment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordUpdate<'a> {
    value: RecordValue<'a>,
    comment: RecordComment<'a>,
}

impl<'a> RecordUpdate<'a> {
    /// Creates an explicit comment update.
    #[must_use]
    pub const fn new(value: RecordValue<'a>, comment: RecordComment<'a>) -> Self {
        Self { value, comment }
    }

    /// Returns the value marker.
    #[must_use]
    pub const fn value(self) -> RecordValue<'a> {
        self.value
    }

    /// Returns the required new comment.
    #[must_use]
    pub const fn comment(self) -> RecordComment<'a> {
        self.comment
    }
}

/// Nonempty update list with unique record values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordUpdates<'a>(&'a [RecordUpdate<'a>]);

impl<'a> RecordUpdates<'a> {
    /// Validates `1..=50` updates and value uniqueness.
    pub fn new(values: &'a [RecordUpdate<'a>]) -> Result<Self, RrsetRequestError> {
        validate_record_count(values.len())?;
        for (index, value) in values.iter().enumerate() {
            if values
                .get(..index)
                .is_none_or(|previous| previous.iter().any(|item| item.value == value.value))
            {
                return Err(RrsetRequestError::DuplicateRecord);
            }
        }
        Ok(Self(values))
    }

    /// Returns validated updates.
    #[must_use]
    pub const fn entries(self) -> &'a [RecordUpdate<'a>] {
        self.0
    }
}

fn validate_record_count(count: usize) -> Result<(), RrsetRequestError> {
    if count == 0 {
        return Err(RrsetRequestError::EmptyRecords);
    }
    if count > MAX_RECORDS_PER_REQUEST {
        return Err(RrsetRequestError::TooManyRecords);
    }
    Ok(())
}

fn invalid_record_text(value: &str, max: usize, reject_empty: bool) -> bool {
    (reject_empty && value.is_empty())
        || value.len() > max
        || value
            .chars()
            .any(|ch| ch.is_control() || is_bidi_control(ch))
}

const fn is_bidi_control(ch: char) -> bool {
    matches!(
        ch,
        '\u{202A}'
            | '\u{202B}'
            | '\u{202C}'
            | '\u{202D}'
            | '\u{202E}'
            | '\u{2066}'
            | '\u{2067}'
            | '\u{2068}'
            | '\u{2069}'
    )
}

fn write_json(value: &str, output: &mut [u8]) -> Result<usize, RrsetRequestError> {
    let mut len = 0;
    buffer::write_json_string(
        output,
        &mut len,
        value,
        RrsetRequestError::BodyBufferTooSmall,
    )?;
    Ok(len)
}
