use core::fmt;

use super::{
    HeaderError, HeaderSensitivity, MAX_RESPONSE_HEADER_BYTES, MAX_RESPONSE_HEADERS,
    encoded_line_len, validate_name, validate_response_value,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct HeaderRange {
    name_start: u16,
    name_len: u8,
    value_start: u16,
    value_len: u16,
    sensitivity: HeaderSensitivity,
}

const EMPTY_RANGE: HeaderRange = HeaderRange {
    name_start: 0,
    name_len: 0,
    value_start: 0,
    value_len: 0,
    sensitivity: HeaderSensitivity::Public,
};

/// Borrowed view into one retained response header.
///
/// Ordinary equality is intentionally unavailable because the value may be
/// sensitive.
///
/// ```compile_fail
/// use cloud_sdk::transport::{HeaderSensitivity, ResponseHeaders};
///
/// let mut headers = ResponseHeaders::new();
/// headers.try_push("x-secret", b"secret", HeaderSensitivity::Sensitive).unwrap();
/// let header = headers.get("x-secret").unwrap();
/// let _ = header == header;
/// ```
#[derive(Clone, Copy)]
pub struct ResponseHeader<'a> {
    name: &'a str,
    value: &'a [u8],
    sensitivity: HeaderSensitivity,
}

impl<'a> ResponseHeader<'a> {
    /// Returns the exact retained name.
    #[must_use]
    pub const fn name(self) -> &'a str {
        self.name
    }

    /// Returns the exact retained value bytes.
    #[must_use]
    pub const fn value(self) -> &'a [u8] {
        self.value
    }

    /// Returns the value sensitivity.
    #[must_use]
    pub const fn sensitivity(self) -> HeaderSensitivity {
        self.sensitivity
    }
}

impl fmt::Debug for ResponseHeader<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResponseHeader")
            .field("name", &self.name)
            .field("value", &"[redacted]")
            .field("sensitivity", &self.sensitivity)
            .finish()
    }
}

/// Owned fixed-capacity response-header metadata.
///
/// Ordinary equality is intentionally unavailable because retained values may
/// be sensitive.
///
/// ```compile_fail
/// use cloud_sdk::transport::ResponseHeaders;
///
/// let headers = ResponseHeaders::new();
/// let _ = headers == headers;
/// ```
#[derive(Clone, Copy)]
pub struct ResponseHeaders {
    bytes: [u8; MAX_RESPONSE_HEADER_BYTES],
    ranges: [HeaderRange; MAX_RESPONSE_HEADERS],
    bytes_len: usize,
    count: usize,
    encoded_len: usize,
}

impl ResponseHeaders {
    /// Creates an empty response-header collection.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            bytes: [0; MAX_RESPONSE_HEADER_BYTES],
            ranges: [EMPTY_RANGE; MAX_RESPONSE_HEADERS],
            bytes_len: 0,
            count: 0,
            encoded_len: 0,
        }
    }

    /// Appends one header atomically after validating every bound.
    pub fn try_push(
        &mut self,
        name: &str,
        value: &[u8],
        sensitivity: HeaderSensitivity,
    ) -> Result<(), HeaderError> {
        validate_name(name)?;
        validate_response_value(value)?;
        if self.count >= MAX_RESPONSE_HEADERS {
            return Err(HeaderError::TooManyHeaders);
        }
        if self
            .iter()
            .any(|header| header.name.eq_ignore_ascii_case(name))
        {
            return Err(HeaderError::DuplicateName);
        }
        let line_len = encoded_line_len(name.len(), value.len())?;
        let encoded_len = self
            .encoded_len
            .checked_add(line_len)
            .ok_or(HeaderError::AggregateTooLarge)?;
        if encoded_len > MAX_RESPONSE_HEADER_BYTES {
            return Err(HeaderError::AggregateTooLarge);
        }
        let stored_len = name
            .len()
            .checked_add(value.len())
            .ok_or(HeaderError::AggregateTooLarge)?;
        let end = self
            .bytes_len
            .checked_add(stored_len)
            .ok_or(HeaderError::AggregateTooLarge)?;
        let name_start =
            u16::try_from(self.bytes_len).map_err(|_| HeaderError::AggregateTooLarge)?;
        let value_start = self
            .bytes_len
            .checked_add(name.len())
            .and_then(|offset| u16::try_from(offset).ok())
            .ok_or(HeaderError::AggregateTooLarge)?;
        let range = HeaderRange {
            name_start,
            name_len: u8::try_from(name.len()).map_err(|_| HeaderError::NameTooLong)?,
            value_start,
            value_len: u16::try_from(value.len()).map_err(|_| HeaderError::ValueTooLong)?,
            sensitivity,
        };
        let next_count = self
            .count
            .checked_add(1)
            .ok_or(HeaderError::TooManyHeaders)?;
        let slot = self
            .ranges
            .get_mut(self.count)
            .ok_or(HeaderError::TooManyHeaders)?;
        let region = self
            .bytes
            .get_mut(self.bytes_len..end)
            .ok_or(HeaderError::AggregateTooLarge)?;
        let (name_out, value_out) = region.split_at_mut(name.len());
        name_out.copy_from_slice(name.as_bytes());
        value_out.copy_from_slice(value);
        *slot = range;
        self.bytes_len = end;
        self.encoded_len = encoded_len;
        self.count = next_count;
        Ok(())
    }

    /// Returns the retained header count.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.count
    }

    /// Reports whether no headers are retained.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the aggregate encoded field-line length.
    #[must_use]
    pub const fn encoded_len(&self) -> usize {
        self.encoded_len
    }

    /// Iterates in retained order.
    pub fn iter(&self) -> impl Iterator<Item = ResponseHeader<'_>> {
        self.ranges
            .get(..self.count)
            .unwrap_or_default()
            .iter()
            .filter_map(|range| self.view(*range))
    }

    /// Finds a retained header by ASCII case-insensitive name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<ResponseHeader<'_>> {
        self.iter()
            .find(|header| header.name.eq_ignore_ascii_case(name))
    }

    fn view(&self, range: HeaderRange) -> Option<ResponseHeader<'_>> {
        let name_start = usize::from(range.name_start);
        let name_end = name_start.checked_add(usize::from(range.name_len))?;
        let value_start = usize::from(range.value_start);
        let value_end = value_start.checked_add(usize::from(range.value_len))?;
        let name = self
            .bytes
            .get(name_start..name_end)
            .and_then(|bytes| core::str::from_utf8(bytes).ok())?;
        let value = self.bytes.get(value_start..value_end)?;
        Some(ResponseHeader {
            name,
            value,
            sensitivity: range.sensitivity,
        })
    }
}

impl Default for ResponseHeaders {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ResponseHeaders {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResponseHeaders")
            .field("count", &self.count)
            .field("encoded_len", &self.encoded_len)
            .field("values", &"[redacted]")
            .finish()
    }
}
