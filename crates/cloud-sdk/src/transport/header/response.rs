use core::fmt;

use cloud_sdk_sanitization::{sanitize_bytes, sanitize_value};

use super::{
    HeaderError, HeaderSensitivity, MAX_RESPONSE_HEADER_BYTES, MAX_RESPONSE_HEADERS,
    encoded_line_len, validate_name, validate_response_value,
};
use crate::transport::retained::{ProtectedRequestId, RetainedMetadataError};

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

    /// Creates a deliberate second cleanup-owning copy for retained test or
    /// diagnostic state.
    ///
    /// Both the source and returned collection independently clear their
    /// complete fixed storage on drop.
    #[must_use]
    pub fn retain_copy(&self) -> Self {
        let mut retained = Self::new();
        retained.bytes.copy_from_slice(&self.bytes);
        retained.ranges.copy_from_slice(&self.ranges);
        retained.bytes_len = self.bytes_len;
        retained.count = self.count;
        retained.encoded_len = self.encoded_len;
        retained
    }

    pub(crate) fn take_request_id(
        &mut self,
    ) -> Result<Option<ProtectedRequestId>, RetainedMetadataError> {
        let found = self
            .ranges
            .get(..self.count)
            .unwrap_or_default()
            .iter()
            .copied()
            .enumerate()
            .find(|(_, range)| {
                self.view(*range)
                    .is_some_and(|header| header.name.eq_ignore_ascii_case("x-request-id"))
            });
        let Some((index, range)) = found else {
            return Ok(None);
        };
        let start = usize::from(range.value_start);
        let end = start
            .checked_add(usize::from(range.value_len))
            .ok_or(RetainedMetadataError::RequestIdTooLong)?;
        let source = self
            .bytes
            .get_mut(start..end)
            .ok_or(RetainedMetadataError::RequestIdTooLong)?;
        let protected = ProtectedRequestId::take_from(source)?;
        self.remove(index, range)?;
        Ok(Some(protected))
    }

    fn remove(&mut self, index: usize, range: HeaderRange) -> Result<(), RetainedMetadataError> {
        let name_start = usize::from(range.name_start);
        let name_len = usize::from(range.name_len);
        let value_start = usize::from(range.value_start);
        let value_len = usize::from(range.value_len);
        if name_start.checked_add(name_len) != Some(value_start) {
            return Err(RetainedMetadataError::RequestIdTooLong);
        }
        let value_end = value_start
            .checked_add(value_len)
            .ok_or(RetainedMetadataError::RequestIdTooLong)?;
        let removed_len = value_end
            .checked_sub(name_start)
            .ok_or(RetainedMetadataError::RequestIdTooLong)?;
        let new_bytes_len = self
            .bytes_len
            .checked_sub(removed_len)
            .ok_or(RetainedMetadataError::RequestIdTooLong)?;
        let removed_encoded_len = name_len
            .checked_add(value_len)
            .and_then(|length| length.checked_add(4))
            .ok_or(RetainedMetadataError::RequestIdTooLong)?;
        let new_encoded_len = self
            .encoded_len
            .checked_sub(removed_encoded_len)
            .ok_or(RetainedMetadataError::RequestIdTooLong)?;
        let new_count = self
            .count
            .checked_sub(1)
            .ok_or(RetainedMetadataError::RequestIdTooLong)?;
        let shift =
            u16::try_from(removed_len).map_err(|_| RetainedMetadataError::RequestIdTooLong)?;
        let tail_start = index
            .checked_add(1)
            .ok_or(RetainedMetadataError::RequestIdTooLong)?;
        let tail = self
            .ranges
            .get(tail_start..self.count)
            .ok_or(RetainedMetadataError::RequestIdTooLong)?;
        if value_end > self.bytes_len
            || tail.iter().any(|entry| {
                entry.name_start.checked_sub(shift).is_none()
                    || entry.value_start.checked_sub(shift).is_none()
            })
        {
            return Err(RetainedMetadataError::RequestIdTooLong);
        }

        self.bytes
            .copy_within(value_end..self.bytes_len, name_start);
        sanitize_bytes(
            self.bytes
                .get_mut(new_bytes_len..self.bytes_len)
                .unwrap_or_default(),
        );
        for entry in self
            .ranges
            .get_mut(tail_start..self.count)
            .unwrap_or_default()
        {
            entry.name_start = entry.name_start.saturating_sub(shift);
            entry.value_start = entry.value_start.saturating_sub(shift);
        }
        self.ranges.copy_within(tail_start..self.count, index);
        if let Some(last) = self.ranges.get_mut(new_count) {
            clear_range(last);
        }
        self.bytes_len = new_bytes_len;
        self.count = new_count;
        self.encoded_len = new_encoded_len;
        Ok(())
    }

    fn clear(&mut self) {
        sanitize_bytes(&mut self.bytes);
        for range in &mut self.ranges {
            clear_range(range);
        }
        sanitize_value(&mut self.bytes_len);
        sanitize_value(&mut self.count);
        sanitize_value(&mut self.encoded_len);
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

fn clear_range(range: &mut HeaderRange) {
    sanitize_value(&mut range.name_start);
    sanitize_value(&mut range.name_len);
    sanitize_value(&mut range.value_start);
    sanitize_value(&mut range.value_len);
    range.sensitivity = HeaderSensitivity::Public;
}

impl Default for ResponseHeaders {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ResponseHeaders {
    fn drop(&mut self) {
        self.clear();
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

#[cfg(test)]
mod cleanup_tests {
    use super::{HeaderSensitivity, ResponseHeaders};

    #[test]
    fn complete_header_storage_and_ranges_clear() {
        let mut headers = ResponseHeaders::new();
        assert!(
            headers
                .try_push(
                    "x-request-id",
                    b"sensitive-id",
                    HeaderSensitivity::Sensitive
                )
                .is_ok()
        );
        headers.clear();
        assert!(headers.bytes.iter().all(|byte| *byte == 0));
        assert!(headers.ranges.iter().all(|range| {
            range.name_start == 0
                && range.name_len == 0
                && range.value_start == 0
                && range.value_len == 0
                && range.sensitivity == HeaderSensitivity::Public
        }));
        assert_eq!(
            (headers.bytes_len, headers.count, headers.encoded_len),
            (0, 0, 0)
        );
    }

    #[test]
    fn taking_request_id_compacts_the_complete_header_table() {
        let mut headers = ResponseHeaders::new();
        assert!(
            headers
                .try_push("date", b"1", HeaderSensitivity::Public)
                .is_ok()
        );
        assert!(
            headers
                .try_push(
                    "x-request-id",
                    b"sensitive-id",
                    HeaderSensitivity::Sensitive
                )
                .is_ok()
        );
        assert!(
            headers
                .try_push("x-public", b"ok", HeaderSensitivity::Public)
                .is_ok()
        );

        let protected = headers.take_request_id();
        assert!(matches!(protected, Ok(Some(_))));
        assert_eq!(headers.len(), 2);
        assert_eq!(headers.encoded_len(), 23);
        assert!(headers.get("x-request-id").is_none());
        let mut retained = headers.iter();
        assert!(
            retained
                .next()
                .is_some_and(|header| { header.name() == "date" && header.value() == b"1" })
        );
        assert!(
            retained
                .next()
                .is_some_and(|header| { header.name() == "x-public" && header.value() == b"ok" })
        );
        assert!(retained.next().is_none());
        assert_eq!(headers.bytes_len, 15);
        assert!(
            headers
                .bytes
                .get(headers.bytes_len..)
                .unwrap_or_default()
                .iter()
                .all(|byte| *byte == 0)
        );
        assert!(
            headers
                .bytes
                .windows(b"sensitive-id".len())
                .all(|window| window != b"sensitive-id")
        );
    }
}
