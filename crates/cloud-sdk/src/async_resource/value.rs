use core::{cmp::Ordering, fmt};

/// Maximum opaque asynchronous-resource identifier length.
pub const MAX_ASYNC_ID_BYTES: usize = 256;
/// Maximum task, progress, error, or event text length.
pub const MAX_ASYNC_TEXT_BYTES: usize = 4096;
/// Maximum non-executable related-resource link length.
pub const MAX_ASYNC_LINK_BYTES: usize = 4096;
/// Maximum progress steps retained in one task snapshot.
pub const MAX_ASYNC_PROGRESS_STEPS: usize = 1024;
/// Maximum errors retained in one task snapshot.
pub const MAX_ASYNC_ERRORS: usize = 1024;
/// Maximum events admitted in one borrowed event batch.
pub const MAX_ASYNC_EVENTS: usize = 1024;

/// Invalid asynchronous task or event data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AsyncResourceValidationError {
    /// An identifier was empty.
    EmptyId,
    /// An identifier exceeded its hard byte bound.
    IdTooLong,
    /// An identifier was not visible ASCII.
    InvalidId,
    /// A text field was empty.
    EmptyText,
    /// A text field exceeded its hard byte bound.
    TextTooLong,
    /// A text field contained a control character.
    TextControl,
    /// A link field was empty.
    EmptyLink,
    /// A link exceeded its hard byte bound.
    LinkTooLong,
    /// A link contained whitespace or a control character.
    InvalidLink,
    /// A timestamp was outside the strict UTC nanosecond RFC 3339 subset.
    InvalidTimestamp,
    /// Task timestamps contradicted their lifecycle ordering.
    TimestampOrder,
    /// A terminal task omitted completion time or a running task supplied it.
    TerminalTimeMismatch,
    /// A successful task retained provider errors with ambiguous precedence.
    StatusErrorMismatch,
    /// A task contained too many progress steps.
    TooManyProgressSteps,
    /// A task contained too many errors.
    TooManyErrors,
    /// An event batch contained too many events.
    TooManyEvents,
}

impl_static_error!(AsyncResourceValidationError,
    Self::EmptyId => "asynchronous resource identifier is empty",
    Self::IdTooLong => "asynchronous resource identifier exceeds its hard limit",
    Self::InvalidId => "asynchronous resource identifier is invalid",
    Self::EmptyText => "asynchronous resource text is empty",
    Self::TextTooLong => "asynchronous resource text exceeds its hard limit",
    Self::TextControl => "asynchronous resource text contains a control character",
    Self::EmptyLink => "asynchronous resource link is empty",
    Self::LinkTooLong => "asynchronous resource link exceeds its hard limit",
    Self::InvalidLink => "asynchronous resource link is invalid",
    Self::InvalidTimestamp => "asynchronous resource timestamp is invalid",
    Self::TimestampOrder => "asynchronous resource timestamps are incoherent",
    Self::TerminalTimeMismatch => "asynchronous resource completion time contradicts its status",
    Self::StatusErrorMismatch => "asynchronous resource status contradicts its errors",
    Self::TooManyProgressSteps => "asynchronous task has too many progress steps",
    Self::TooManyErrors => "asynchronous task has too many errors",
    Self::TooManyEvents => "asynchronous resource batch has too many events",
);

macro_rules! sensitive_value {
    ($doc:literal, $name:ident, $maximum:ident, $empty:ident, $long:ident, $validate:expr) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Eq, PartialEq)]
        pub struct $name<'a>(&'a str);

        impl<'a> $name<'a> {
            /// Validates one borrowed sensitive value.
            pub fn new(value: &'a str) -> Result<Self, AsyncResourceValidationError> {
                if value.is_empty() {
                    return Err(AsyncResourceValidationError::$empty);
                }
                if value.len() > $maximum {
                    return Err(AsyncResourceValidationError::$long);
                }
                ($validate)(value)?;
                Ok(Self(value))
            }

            /// Runs a closure with the validated value without creating an owned copy.
            pub fn with_str<R>(self, inspect: impl FnOnce(&str) -> R) -> R {
                inspect(self.0)
            }
        }

        impl fmt::Debug for $name<'_> {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(concat!(stringify!($name), "([redacted])"))
            }
        }
    };
}

sensitive_value!(
    "Bounded opaque asynchronous-resource identifier with redacted diagnostics.",
    AsyncResourceId,
    MAX_ASYNC_ID_BYTES,
    EmptyId,
    IdTooLong,
    |value: &str| {
        if !value.bytes().all(|byte| byte.is_ascii_graphic()) {
            return Err(AsyncResourceValidationError::InvalidId);
        }
        Ok(())
    }
);

sensitive_value!(
    "Bounded sensitive asynchronous-resource text with redacted diagnostics.",
    AsyncResourceText,
    MAX_ASYNC_TEXT_BYTES,
    EmptyText,
    TextTooLong,
    |value: &str| {
        if value.chars().any(char::is_control) {
            return Err(AsyncResourceValidationError::TextControl);
        }
        Ok(())
    }
);

sensitive_value!(
    "Bounded non-executable related-resource link with redacted diagnostics.",
    AsyncResourceLink,
    MAX_ASYNC_LINK_BYTES,
    EmptyLink,
    LinkTooLong,
    |value: &str| {
        if value.chars().any(char::is_whitespace) || value.chars().any(char::is_control) {
            return Err(AsyncResourceValidationError::InvalidLink);
        }
        Ok(())
    }
);

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct TimestampParts {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
}

/// Strict UTC nanosecond RFC 3339 timestamp retained as borrowed sensitive text.
#[derive(Clone, Copy)]
pub struct AsyncResourceTimestamp<'a> {
    value: &'a str,
    parts: TimestampParts,
}

impl PartialEq for AsyncResourceTimestamp<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.parts == other.parts
    }
}

impl Eq for AsyncResourceTimestamp<'_> {}

impl<'a> AsyncResourceTimestamp<'a> {
    /// Parses `YYYY-MM-DDTHH:MM:SS[.1-9 digits]Z` with calendar validation.
    ///
    /// Leap seconds are rejected because this type has no leap-second table.
    pub fn parse(value: &'a str) -> Result<Self, AsyncResourceValidationError> {
        let bytes = value.as_bytes();
        if !(20..=30).contains(&bytes.len())
            || bytes.get(4) != Some(&b'-')
            || bytes.get(7) != Some(&b'-')
            || bytes.get(10) != Some(&b'T')
            || bytes.get(13) != Some(&b':')
            || bytes.get(16) != Some(&b':')
            || bytes.last() != Some(&b'Z')
        {
            return Err(AsyncResourceValidationError::InvalidTimestamp);
        }
        let year = u16::try_from(decimal(bytes, 0, 4)?)
            .map_err(|_| AsyncResourceValidationError::InvalidTimestamp)?;
        let month = u8::try_from(decimal(bytes, 5, 2)?)
            .map_err(|_| AsyncResourceValidationError::InvalidTimestamp)?;
        let day = u8::try_from(decimal(bytes, 8, 2)?)
            .map_err(|_| AsyncResourceValidationError::InvalidTimestamp)?;
        let hour = u8::try_from(decimal(bytes, 11, 2)?)
            .map_err(|_| AsyncResourceValidationError::InvalidTimestamp)?;
        let minute = u8::try_from(decimal(bytes, 14, 2)?)
            .map_err(|_| AsyncResourceValidationError::InvalidTimestamp)?;
        let second = u8::try_from(decimal(bytes, 17, 2)?)
            .map_err(|_| AsyncResourceValidationError::InvalidTimestamp)?;
        let nanosecond = fraction(bytes)?;
        if year == 0
            || !(1..=12).contains(&month)
            || day == 0
            || day > days_in_month(year, month)
            || hour > 23
            || minute > 59
            || second > 59
        {
            return Err(AsyncResourceValidationError::InvalidTimestamp);
        }
        Ok(Self {
            value,
            parts: TimestampParts {
                year,
                month,
                day,
                hour,
                minute,
                second,
                nanosecond,
            },
        })
    }

    /// Runs a closure with the exact canonical timestamp.
    pub fn with_str<R>(self, inspect: impl FnOnce(&str) -> R) -> R {
        inspect(self.value)
    }

    pub(super) fn compare(self, other: Self) -> Ordering {
        self.parts.cmp(&other.parts)
    }
}

impl fmt::Debug for AsyncResourceTimestamp<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AsyncResourceTimestamp([redacted])")
    }
}

fn decimal(bytes: &[u8], start: usize, len: usize) -> Result<u32, AsyncResourceValidationError> {
    let mut value = 0_u32;
    let end = start
        .checked_add(len)
        .ok_or(AsyncResourceValidationError::InvalidTimestamp)?;
    for byte in bytes
        .get(start..end)
        .ok_or(AsyncResourceValidationError::InvalidTimestamp)?
    {
        if !byte.is_ascii_digit() {
            return Err(AsyncResourceValidationError::InvalidTimestamp);
        }
        let digit = byte
            .checked_sub(b'0')
            .ok_or(AsyncResourceValidationError::InvalidTimestamp)?;
        value = value
            .checked_mul(10)
            .and_then(|current| current.checked_add(u32::from(digit)))
            .ok_or(AsyncResourceValidationError::InvalidTimestamp)?;
    }
    Ok(value)
}

fn fraction(bytes: &[u8]) -> Result<u32, AsyncResourceValidationError> {
    if bytes.len() == 20 {
        return Ok(0);
    }
    if bytes.get(19) != Some(&b'.') {
        return Err(AsyncResourceValidationError::InvalidTimestamp);
    }
    let end = bytes
        .len()
        .checked_sub(1)
        .ok_or(AsyncResourceValidationError::InvalidTimestamp)?;
    let digits = bytes
        .get(20..end)
        .ok_or(AsyncResourceValidationError::InvalidTimestamp)?;
    if digits.is_empty() || digits.len() > 9 || !digits.iter().all(u8::is_ascii_digit) {
        return Err(AsyncResourceValidationError::InvalidTimestamp);
    }
    let mut value = 0_u32;
    for byte in digits {
        let digit = byte
            .checked_sub(b'0')
            .ok_or(AsyncResourceValidationError::InvalidTimestamp)?;
        value = value
            .checked_mul(10)
            .and_then(|current| current.checked_add(u32::from(digit)))
            .ok_or(AsyncResourceValidationError::InvalidTimestamp)?;
    }
    let exponent = 9_usize
        .checked_sub(digits.len())
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(AsyncResourceValidationError::InvalidTimestamp)?;
    let scale = 10_u32
        .checked_pow(exponent)
        .ok_or(AsyncResourceValidationError::InvalidTimestamp)?;
    value
        .checked_mul(scale)
        .ok_or(AsyncResourceValidationError::InvalidTimestamp)
}

const fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        2 if year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100)) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
