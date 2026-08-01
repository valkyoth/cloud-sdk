use super::http_date::parse_http_date;

/// A relative delay measured in whole seconds.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct DelaySeconds(u64);

impl DelaySeconds {
    /// Creates a relative delay.
    #[must_use]
    pub const fn new(seconds: u64) -> Self {
        Self(seconds)
    }

    /// Returns the delay in whole seconds.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// An absolute Unix wall-clock timestamp in whole seconds.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WallClockTimestamp(u64);

impl WallClockTimestamp {
    /// Creates an absolute Unix timestamp.
    #[must_use]
    pub const fn new(epoch_seconds: u64) -> Self {
        Self(epoch_seconds)
    }

    /// Returns whole seconds since the Unix epoch.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A parsed HTTP date represented as signed Unix seconds.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HttpDate(i64);

impl HttpDate {
    pub(crate) const fn new(epoch_seconds: i64) -> Self {
        Self(epoch_seconds)
    }

    /// Returns signed whole seconds relative to the Unix epoch.
    #[must_use]
    pub const fn epoch_seconds(self) -> i64 {
        self.0
    }
}

/// Standard HTTP `Retry-After` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryAfter {
    /// Relative delay-seconds form.
    Delay(DelaySeconds),
    /// Absolute HTTP-date form.
    HttpDate(HttpDate),
}

impl RetryAfter {
    /// Parses delay-seconds or any HTTP-date form required by RFC 9110.
    ///
    /// `now` is used only to resolve the two-digit year in the obsolete
    /// RFC 850 form. The function acquires no clock itself.
    pub fn parse(value: &[u8], now: WallClockTimestamp) -> Result<Self, RetryAfterError> {
        if value.is_empty() {
            return Err(RetryAfterError::Empty);
        }
        if value.iter().all(u8::is_ascii_digit) {
            return parse_decimal(value).map(DelaySeconds::new).map(Self::Delay);
        }
        parse_http_date(value, now).map(Self::HttpDate)
    }
}

fn parse_decimal(value: &[u8]) -> Result<u64, RetryAfterError> {
    let mut parsed = 0_u64;
    for byte in value {
        parsed = parsed
            .checked_mul(10)
            .and_then(|current| current.checked_add(u64::from(*byte & 0x0f)))
            .ok_or(RetryAfterError::Overflow)?;
    }
    Ok(parsed)
}

/// Invalid `Retry-After` syntax or value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryAfterError {
    /// The value was empty.
    Empty,
    /// The delay or date cannot be represented.
    Overflow,
    /// The value was neither decimal delay-seconds nor a supported HTTP date.
    InvalidSyntax,
    /// A date component was outside its valid range.
    InvalidDate,
    /// The advertised weekday contradicted the calendar date.
    WeekdayMismatch,
}

impl_static_error!(RetryAfterError,
    Self::Empty => "Retry-After value is empty",
    Self::Overflow => "Retry-After value exceeds its numeric range",
    Self::InvalidSyntax => "Retry-After syntax is invalid",
    Self::InvalidDate => "Retry-After date is invalid",
    Self::WeekdayMismatch => "Retry-After weekday does not match its date",
);
