//! Shared server request helpers.

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
    /// Required field was not supplied.
    MissingRequiredField,
    /// Caller-provided path buffer is too small.
    PathBufferTooSmall,
    /// Caller-provided query buffer is too small.
    QueryBufferTooSmall,
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
    /// DNS pointer action requires explicit set or reset.
    MissingDnsPtrIntent,
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
        if value.is_empty() || value.len() > MAX_REFERENCE_BYTES || has_control_byte(value) {
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
        if value.is_empty() || value.len() > MAX_TEXT_BYTES || has_control_byte(value) {
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

    /// Returns the value.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
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
        if bytes.len() < 20
            || bytes.get(4) != Some(&b'-')
            || bytes.get(7) != Some(&b'-')
            || bytes.get(10) != Some(&b'T')
            || bytes.get(13) != Some(&b':')
            || bytes.get(16) != Some(&b':')
            || !value.ends_with('Z')
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
    write_raw(
        output,
        &mut len,
        prefix,
        ServerRequestError::PathBufferTooSmall,
    )?;
    write_u64(
        output,
        &mut len,
        id.get(),
        ServerRequestError::PathBufferTooSmall,
    )?;
    write_raw(
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
    if *first {
        *first = false;
    } else {
        write_byte(output, len, b'&', ServerRequestError::QueryBufferTooSmall)?;
    }
    write_percent(output, len, key)?;
    write_byte(output, len, b'=', ServerRequestError::QueryBufferTooSmall)?;
    write_percent(output, len, value)
}

/// Writes a query key/u64 pair.
pub fn write_query_u64(
    output: &mut [u8],
    len: &mut usize,
    first: &mut bool,
    key: &str,
    value: u64,
) -> Result<(), ServerRequestError> {
    if *first {
        *first = false;
    } else {
        write_byte(output, len, b'&', ServerRequestError::QueryBufferTooSmall)?;
    }
    write_percent(output, len, key)?;
    write_byte(output, len, b'=', ServerRequestError::QueryBufferTooSmall)?;
    write_u64(output, len, value, ServerRequestError::QueryBufferTooSmall)
}

fn write_percent(
    output: &mut [u8],
    len: &mut usize,
    value: &str,
) -> Result<(), ServerRequestError> {
    for byte in value.bytes() {
        if matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~') {
            write_byte(output, len, byte, ServerRequestError::QueryBufferTooSmall)?;
        } else {
            write_byte(output, len, b'%', ServerRequestError::QueryBufferTooSmall)?;
            write_byte(
                output,
                len,
                hex_digit(byte >> 4),
                ServerRequestError::QueryBufferTooSmall,
            )?;
            write_byte(
                output,
                len,
                hex_digit(byte & 0x0f),
                ServerRequestError::QueryBufferTooSmall,
            )?;
        }
    }
    Ok(())
}

fn write_raw(
    output: &mut [u8],
    len: &mut usize,
    value: &str,
    error: ServerRequestError,
) -> Result<(), ServerRequestError> {
    for byte in value.bytes() {
        write_byte(output, len, byte, error)?;
    }
    Ok(())
}

fn write_u64(
    output: &mut [u8],
    len: &mut usize,
    mut value: u64,
    error: ServerRequestError,
) -> Result<(), ServerRequestError> {
    let mut digits = [0u8; 20];
    let mut cursor = digits.len();
    while value != 0 {
        cursor = cursor
            .checked_sub(1)
            .ok_or(ServerRequestError::NumberEncodingFailed)?;
        let digit =
            u8::try_from(value % 10).map_err(|_| ServerRequestError::NumberEncodingFailed)?;
        let slot = digits
            .get_mut(cursor)
            .ok_or(ServerRequestError::NumberEncodingFailed)?;
        *slot = b'0'
            .checked_add(digit)
            .ok_or(ServerRequestError::NumberEncodingFailed)?;
        value /= 10;
    }
    let encoded = digits
        .get(cursor..)
        .ok_or(ServerRequestError::NumberEncodingFailed)?;
    for byte in encoded {
        write_byte(output, len, *byte, error)?;
    }
    Ok(())
}

fn write_byte(
    output: &mut [u8],
    len: &mut usize,
    byte: u8,
    error: ServerRequestError,
) -> Result<(), ServerRequestError> {
    let slot = output.get_mut(*len).ok_or(error)?;
    *slot = byte;
    *len = len.checked_add(1).ok_or(error)?;
    Ok(())
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
}

const fn hex_digit(nibble: u8) -> u8 {
    match nibble {
        0 => b'0',
        1 => b'1',
        2 => b'2',
        3 => b'3',
        4 => b'4',
        5 => b'5',
        6 => b'6',
        7 => b'7',
        8 => b'8',
        9 => b'9',
        10 => b'A',
        11 => b'B',
        12 => b'C',
        13 => b'D',
        14 => b'E',
        _ => b'F',
    }
}
