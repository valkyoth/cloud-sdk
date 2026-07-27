use core::fmt;

use super::{
    ContentType, HeaderError, HeaderName, HeaderSensitivity, HeaderValue, MAX_REQUEST_HEADER_BYTES,
    MAX_REQUEST_HEADERS, MediaType, encoded_line_len, is_reserved_request_name, typed_accept,
    typed_content_type,
};

/// One validated borrowed request header.
///
/// Ordinary equality is intentionally unavailable because the value may be
/// sensitive.
///
/// ```compile_fail
/// use cloud_sdk::transport::RequestHeader;
///
/// let left = RequestHeader::sensitive("x-secret", "secret").unwrap();
/// let right = RequestHeader::sensitive("x-secret", "secret").unwrap();
/// let _ = left == right;
/// ```
#[derive(Clone, Copy)]
pub struct RequestHeader<'a> {
    name: HeaderName<'a>,
    value: HeaderValue<'a>,
    sensitivity: HeaderSensitivity,
}

impl<'a> RequestHeader<'a> {
    /// Creates a public request header.
    pub fn new(name: &'a str, value: &'a str) -> Result<Self, HeaderError> {
        Self::from_parts(
            HeaderName::new(name)?,
            HeaderValue::new(value)?,
            HeaderSensitivity::Public,
        )
    }

    /// Creates a sensitive request header.
    pub fn sensitive(name: &'a str, value: &'a str) -> Result<Self, HeaderError> {
        Self::from_parts(
            HeaderName::new(name)?,
            HeaderValue::new(value)?,
            HeaderSensitivity::Sensitive,
        )
    }

    /// Creates a typed `Accept` header.
    #[must_use]
    pub const fn accept(media_type: MediaType<'a>) -> Self {
        Self {
            name: HeaderName("accept"),
            value: typed_accept(media_type),
            sensitivity: HeaderSensitivity::Public,
        }
    }

    /// Creates a typed `Content-Type` header.
    #[must_use]
    pub const fn content_type(content_type: ContentType<'a>) -> Self {
        Self {
            name: HeaderName("content-type"),
            value: typed_content_type(content_type),
            sensitivity: HeaderSensitivity::Public,
        }
    }

    /// Returns the exact header name.
    #[must_use]
    pub const fn name(self) -> HeaderName<'a> {
        self.name
    }

    /// Returns the exact header value.
    #[must_use]
    pub const fn value(self) -> HeaderValue<'a> {
        self.value
    }

    /// Returns the value sensitivity.
    #[must_use]
    pub const fn sensitivity(self) -> HeaderSensitivity {
        self.sensitivity
    }

    fn from_parts(
        name: HeaderName<'a>,
        value: HeaderValue<'a>,
        sensitivity: HeaderSensitivity,
    ) -> Result<Self, HeaderError> {
        if is_reserved_request_name(name) {
            return Err(HeaderError::ReservedRequestHeader);
        }
        if name.eq_ignore_ascii_case("content-type") && ContentType::new(value.as_str()).is_err() {
            return Err(HeaderError::InvalidContentType);
        }
        Ok(Self {
            name,
            value,
            sensitivity,
        })
    }
}

impl fmt::Debug for RequestHeader<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestHeader")
            .field("name", &self.name)
            .field("value", &"[redacted]")
            .field("sensitivity", &self.sensitivity)
            .finish()
    }
}

/// Validated ordered request-header block.
#[derive(Clone, Copy)]
pub struct RequestHeaders<'a> {
    entries: &'a [RequestHeader<'a>],
    encoded_len: usize,
}

impl RequestHeaders<'static> {
    /// Empty request-header block.
    pub const EMPTY: Self = Self {
        entries: &[],
        encoded_len: 0,
    };
}

impl<'a> RequestHeaders<'a> {
    /// Validates count, aggregate size, reserved ownership, and duplicates.
    pub fn new(entries: &'a [RequestHeader<'a>]) -> Result<Self, HeaderError> {
        if entries.len() > MAX_REQUEST_HEADERS {
            return Err(HeaderError::TooManyHeaders);
        }
        let mut encoded_len = 0_usize;
        for (index, entry) in entries.iter().enumerate() {
            if is_reserved_request_name(entry.name) {
                return Err(HeaderError::ReservedRequestHeader);
            }
            if entries.get(..index).is_some_and(|seen| {
                seen.iter()
                    .any(|candidate| candidate.name.eq_ignore_ascii_case(entry.name.as_str()))
            }) {
                return Err(HeaderError::DuplicateName);
            }
            let line_len = encoded_line_len(entry.name.as_str().len(), entry.value.as_str().len())?;
            encoded_len = encoded_len
                .checked_add(line_len)
                .ok_or(HeaderError::AggregateTooLarge)?;
            if encoded_len > MAX_REQUEST_HEADER_BYTES {
                return Err(HeaderError::AggregateTooLarge);
            }
        }
        Ok(Self {
            entries,
            encoded_len,
        })
    }

    /// Returns the ordered entries.
    #[must_use]
    pub const fn as_slice(self) -> &'a [RequestHeader<'a>] {
        self.entries
    }

    /// Returns the encoded HTTP/1 field-line length without a final empty line.
    #[must_use]
    pub const fn encoded_len(self) -> usize {
        self.encoded_len
    }

    /// Finds a header using ASCII case-insensitive name comparison.
    #[must_use]
    pub fn get(self, name: &str) -> Option<RequestHeader<'a>> {
        self.entries
            .iter()
            .copied()
            .find(|entry| entry.name.eq_ignore_ascii_case(name))
    }

    /// Atomically writes all field lines as `name: value\r\n`.
    ///
    /// The output is unchanged when it is too small.
    pub fn encode_http1(self, output: &mut [u8]) -> Result<usize, HeaderError> {
        if output.len() < self.encoded_len {
            return Err(HeaderError::OutputTooSmall);
        }
        let target = output
            .get_mut(..self.encoded_len)
            .ok_or(HeaderError::OutputTooSmall)?;
        let mut offset = 0_usize;
        for entry in self.entries {
            offset = write_line(target, offset, *entry)?;
        }
        Ok(offset)
    }
}

impl fmt::Debug for RequestHeaders<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RequestHeaders")
            .field("count", &self.entries.len())
            .field("encoded_len", &self.encoded_len)
            .field("values", &"[redacted]")
            .finish()
    }
}

fn write_line(
    output: &mut [u8],
    offset: usize,
    header: RequestHeader<'_>,
) -> Result<usize, HeaderError> {
    let name = header.name.as_str().as_bytes();
    let value = header.value.as_str().as_bytes();
    let line_len = encoded_line_len(name.len(), value.len())?;
    let end = offset
        .checked_add(line_len)
        .ok_or(HeaderError::OutputTooSmall)?;
    let line = output
        .get_mut(offset..end)
        .ok_or(HeaderError::OutputTooSmall)?;
    let (name_out, rest) = line.split_at_mut(name.len());
    let (separator, rest) = rest.split_at_mut(2);
    let (value_out, ending) = rest.split_at_mut(value.len());
    name_out.copy_from_slice(name);
    separator.copy_from_slice(b": ");
    value_out.copy_from_slice(value);
    ending.copy_from_slice(b"\r\n");
    Ok(end)
}
