//! Shared server request helpers.

use core::fmt;

use cloud_sdk::buffer;

use crate::request::{EndpointPath, EndpointPathError};

/// Maximum server name bytes.
pub const MAX_SERVER_NAME_BYTES: usize = 63;
/// Maximum ID-or-name reference bytes.
pub const MAX_REFERENCE_BYTES: usize = 128;
/// Maximum text value bytes.
pub const MAX_TEXT_BYTES: usize = 1024;
/// Maximum cloud-init user data bytes from the source-locked API.
pub const MAX_USER_DATA_BYTES: usize = 32 * 1024;

/// Server request error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerRequestError {
    /// Endpoint paths failed validation.
    InvalidPath(EndpointPathError),
    /// A change-alias-IPs request supplied no replacement addresses.
    EmptyAliasIps,
    /// A server action requiring a request body was requested as bodyless.
    ActionBodyRequired,
    /// Caller-provided path buffer is too small.
    PathBufferTooSmall,
    /// Caller-provided query buffer is too small.
    QueryBufferTooSmall,
    /// Caller-provided body buffer is too small.
    BodyBufferTooSmall,
    /// Decimal conversion failed.
    NumberEncodingFailed,
    /// Path bytes failed UTF-8 conversion after construction.
    PathEncodingFailed,
    /// Name failed hostname validation.
    InvalidName,
    /// Reference failed conservative validation.
    InvalidReference,
    /// Text value failed conservative validation.
    InvalidText,
    /// User data failed size or byte validation.
    InvalidUserData,
    /// Timestamp failed conservative RFC3339 validation.
    InvalidTimestamp,
    /// End time must be later than start time.
    InvalidTimeRange,
    /// Fields cannot be combined safely.
    MutuallyExclusiveFields,
}

/// Nonzero server-adjacent resource identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ResourceId(u64);

impl ResourceId {
    /// Creates a nonzero identifier.
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            return None;
        }
        Some(Self(value))
    }

    /// Returns the raw identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Server hostname.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerName<'a> {
    value: &'a str,
}

impl<'a> ServerName<'a> {
    /// Creates a conservative RFC 1123 hostname.
    pub fn new(value: &'a str) -> Result<Self, ServerRequestError> {
        if value.is_empty()
            || value.len() > MAX_SERVER_NAME_BYTES
            || value.starts_with('-')
            || value.ends_with('-')
            || value
                .bytes()
                .any(|byte| !(byte.is_ascii_alphanumeric() || byte == b'-'))
        {
            return Err(ServerRequestError::InvalidName);
        }
        Ok(Self { value })
    }

    /// Returns the name.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

/// ID-or-name reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerReference<'a> {
    value: &'a str,
}

impl<'a> ServerReference<'a> {
    /// Creates a bounded printable reference.
    pub fn new(value: &'a str) -> Result<Self, ServerRequestError> {
        if value.is_empty()
            || value.len() > MAX_REFERENCE_BYTES
            || has_control_byte(value)
            || has_json_significant_byte(value)
        {
            return Err(ServerRequestError::InvalidReference);
        }
        Ok(Self { value })
    }

    /// Returns the reference.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

/// Bounded text value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextValue<'a> {
    value: &'a str,
}

impl<'a> TextValue<'a> {
    /// Creates a bounded non-control text value.
    pub fn new(value: &'a str) -> Result<Self, ServerRequestError> {
        if value.is_empty()
            || value.len() > MAX_TEXT_BYTES
            || has_control_byte(value)
            || has_json_significant_byte(value)
        {
            return Err(ServerRequestError::InvalidText);
        }
        Ok(Self { value })
    }

    /// Returns the value.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

/// Cloud-init user data.
///
/// User data may contain quotes, backslashes, and newlines. Body writers must
/// use [`UserData::write_json_string`]. The raw value is intentionally not
/// exposed, so future SDK body serialization cannot accidentally interpolate it
/// into JSON without escaping.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct UserData<'a> {
    value: &'a str,
}

impl<'a> UserData<'a> {
    /// Creates a source-limited user data value.
    pub fn new(value: &'a str) -> Result<Self, ServerRequestError> {
        if value.len() > MAX_USER_DATA_BYTES || value.bytes().any(|byte| byte == 0) {
            return Err(ServerRequestError::InvalidUserData);
        }
        Ok(Self { value })
    }

    /// Writes this value as a complete JSON string into a caller-owned buffer.
    /// An undersized buffer is not modified.
    pub fn write_json_string(self, output: &mut [u8]) -> Result<usize, ServerRequestError> {
        let mut len = 0;
        buffer::write_json_string(
            output,
            &mut len,
            self.value,
            ServerRequestError::BodyBufferTooSmall,
        )?;
        Ok(len)
    }
}

impl fmt::Debug for UserData<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("UserData([redacted])")
    }
}

/// RFC3339 timestamp string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimestampValue<'a> {
    value: &'a str,
}

impl<'a> TimestampValue<'a> {
    /// Creates a conservative RFC3339 UTC timestamp value.
    pub fn new(value: &'a str) -> Result<Self, ServerRequestError> {
        let bytes = value.as_bytes();
        if bytes.len() != 20
            || !ascii_digits(bytes, 0, 4)
            || bytes.get(4) != Some(&b'-')
            || !ascii_digits(bytes, 5, 7)
            || bytes.get(7) != Some(&b'-')
            || !ascii_digits(bytes, 8, 10)
            || bytes.get(10) != Some(&b'T')
            || !ascii_digits(bytes, 11, 13)
            || bytes.get(13) != Some(&b':')
            || !ascii_digits(bytes, 14, 16)
            || bytes.get(16) != Some(&b':')
            || !ascii_digits(bytes, 17, 19)
            || bytes.get(19) != Some(&b'Z')
            || has_control_byte(value)
        {
            return Err(ServerRequestError::InvalidTimestamp);
        }
        Ok(Self { value })
    }

    /// Returns the timestamp.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

/// Small deterministic query writer.
pub struct ServerQueryError<'a> {
    output: &'a mut [u8],
    len: usize,
    first: bool,
}

impl<'a> ServerQueryError<'a> {
    /// Creates a query writer.
    pub fn new(output: &'a mut [u8]) -> Self {
        Self {
            output,
            len: 0,
            first: true,
        }
    }

    /// Writes a key/value pair.
    pub fn pair(&mut self, key: &str, value: &str) -> Result<(), ServerRequestError> {
        write_query_pair(self.output, &mut self.len, &mut self.first, key, value)
    }

    /// Writes a key/u64 pair.
    pub fn u64_pair(&mut self, key: &str, value: u64) -> Result<(), ServerRequestError> {
        write_query_u64(self.output, &mut self.len, &mut self.first, key, value)
    }

    /// Returns written length.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }
}

/// Returns a static path.
pub fn static_path(value: &'static str) -> Result<EndpointPath<'static>, ServerRequestError> {
    EndpointPath::new(value).map_err(ServerRequestError::InvalidPath)
}

/// Writes a static path.
pub fn write_static_path(output: &mut [u8], value: &str) -> Result<usize, ServerRequestError> {
    let bytes = value.as_bytes();
    let target = output
        .get_mut(..bytes.len())
        .ok_or(ServerRequestError::PathBufferTooSmall)?;
    target.copy_from_slice(bytes);
    Ok(bytes.len())
}

/// Writes `{prefix}{id}{suffix}` into a caller-owned path buffer.
pub fn write_id_path(
    output: &mut [u8],
    prefix: &str,
    id: ResourceId,
    suffix: &str,
) -> Result<usize, ServerRequestError> {
    let mut len = 0;
    buffer::write_str(
        output,
        &mut len,
        prefix,
        ServerRequestError::PathBufferTooSmall,
    )?;
    buffer::write_u64(
        output,
        &mut len,
        id.get(),
        ServerRequestError::PathBufferTooSmall,
    )?;
    buffer::write_str(
        output,
        &mut len,
        suffix,
        ServerRequestError::PathBufferTooSmall,
    )?;
    validate_written_path(output, len)?;
    Ok(len)
}

/// Writes a query key/value pair.
pub fn write_query_pair(
    output: &mut [u8],
    len: &mut usize,
    first: &mut bool,
    key: &str,
    value: &str,
) -> Result<(), ServerRequestError> {
    buffer::write_query_pair(
        output,
        len,
        first,
        key,
        value,
        ServerRequestError::QueryBufferTooSmall,
    )
}

/// Writes a query key/u64 pair.
pub fn write_query_u64(
    output: &mut [u8],
    len: &mut usize,
    first: &mut bool,
    key: &str,
    value: u64,
) -> Result<(), ServerRequestError> {
    buffer::write_query_u64(
        output,
        len,
        first,
        key,
        value,
        ServerRequestError::QueryBufferTooSmall,
    )
}

fn validate_written_path(output: &[u8], len: usize) -> Result<(), ServerRequestError> {
    let bytes = output
        .get(..len)
        .ok_or(ServerRequestError::PathBufferTooSmall)?;
    let path = core::str::from_utf8(bytes).map_err(|_| ServerRequestError::PathEncodingFailed)?;
    EndpointPath::new(path).map_err(ServerRequestError::InvalidPath)?;
    Ok(())
}

fn has_control_byte(value: &str) -> bool {
    value.bytes().any(|byte| byte < 0x20 || byte == 0x7f)
        || value.chars().any(|ch| {
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
        })
}

fn has_json_significant_byte(value: &str) -> bool {
    value.bytes().any(|byte| matches!(byte, b'"' | b'\\'))
}

fn ascii_digits(bytes: &[u8], start: usize, end: usize) -> bool {
    bytes
        .get(start..end)
        .is_some_and(|slice| slice.iter().all(u8::is_ascii_digit))
}
