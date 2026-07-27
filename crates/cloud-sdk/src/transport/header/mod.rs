//! Bounded HTTP request and response header contracts.

mod request;
mod response;

pub use request::{RequestHeader, RequestHeaders};
pub use response::{ResponseHeader, ResponseHeaders};

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};

use super::{ContentType, MediaType};

/// Maximum header-name length admitted by the transport contract.
pub const MAX_HEADER_NAME_BYTES: usize = 64;
/// Maximum single header-value length admitted by the transport contract.
pub const MAX_HEADER_VALUE_BYTES: usize = 1024;
/// Maximum number of request headers.
pub const MAX_REQUEST_HEADERS: usize = 32;
/// Maximum encoded request-header bytes, excluding the final empty line.
pub const MAX_REQUEST_HEADER_BYTES: usize = 8192;
/// Maximum number of retained response headers.
pub const MAX_RESPONSE_HEADERS: usize = 32;
/// Maximum encoded response-header bytes, excluding the final empty line.
pub const MAX_RESPONSE_HEADER_BYTES: usize = 8192;

const HEADER_LINE_OVERHEAD: usize = 4;

/// Whether a header value must be treated as sensitive.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum HeaderSensitivity {
    /// Ordinary metadata. Debug output remains redacted.
    Public,
    /// Credential, token, cookie, or other secret-bearing metadata.
    Sensitive,
}

/// Bounded HTTP header validation or capacity failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeaderError {
    /// Header names must not be empty.
    EmptyName,
    /// The header name exceeds [`MAX_HEADER_NAME_BYTES`].
    NameTooLong,
    /// The header name is not an HTTP token.
    InvalidName,
    /// The header value exceeds [`MAX_HEADER_VALUE_BYTES`].
    ValueTooLong,
    /// The header value contains an unsafe control or non-ASCII request byte.
    InvalidValue,
    /// A caller attempted to own an adapter- or protocol-owned request header.
    ReservedRequestHeader,
    /// Header names must be unique under ASCII case-insensitive comparison.
    DuplicateName,
    /// The header count exceeds the applicable request or response limit.
    TooManyHeaders,
    /// The encoded header block exceeds its aggregate byte limit.
    AggregateTooLarge,
    /// A generic `Content-Type` value failed typed media validation.
    InvalidContentType,
    /// Caller output cannot hold the complete encoded header block.
    OutputTooSmall,
}

impl_static_error!(HeaderError,
    Self::EmptyName => "HTTP header name is empty",
    Self::NameTooLong => "HTTP header name exceeds the length limit",
    Self::InvalidName => "HTTP header name is invalid",
    Self::ValueTooLong => "HTTP header value exceeds the length limit",
    Self::InvalidValue => "HTTP header value is invalid",
    Self::ReservedRequestHeader => "HTTP request header ownership is reserved",
    Self::DuplicateName => "HTTP header name is duplicated",
    Self::TooManyHeaders => "HTTP header count exceeds the limit",
    Self::AggregateTooLarge => "HTTP header block exceeds the byte limit",
    Self::InvalidContentType => "HTTP content type is invalid",
    Self::OutputTooSmall => "HTTP header output is too small",
);

/// Borrowed, validated HTTP header name.
#[derive(Clone, Copy)]
pub struct HeaderName<'a>(&'a str);

impl<'a> HeaderName<'a> {
    /// Validates an HTTP field name without normalizing its bytes.
    pub fn new(value: &'a str) -> Result<Self, HeaderError> {
        validate_name(value)?;
        Ok(Self(value))
    }

    /// Returns the exact validated name.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }

    /// Compares two names using HTTP's ASCII case-insensitive semantics.
    #[must_use]
    pub fn eq_ignore_ascii_case(self, other: &str) -> bool {
        self.0.eq_ignore_ascii_case(other)
    }
}

impl PartialEq for HeaderName<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(other.0)
    }
}

impl Eq for HeaderName<'_> {}

impl PartialOrd for HeaderName<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeaderName<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .bytes()
            .map(|byte| byte.to_ascii_lowercase())
            .cmp(other.0.bytes().map(|byte| byte.to_ascii_lowercase()))
    }
}

impl Hash for HeaderName<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for byte in self.0.bytes() {
            state.write_u8(byte.to_ascii_lowercase());
        }
    }
}

impl fmt::Debug for HeaderName<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("HeaderName").field(&self.0).finish()
    }
}

/// Borrowed, validated request header value.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct HeaderValue<'a>(&'a str);

impl<'a> HeaderValue<'a> {
    /// Validates one canonical visible-ASCII request header value.
    pub fn new(value: &'a str) -> Result<Self, HeaderError> {
        validate_request_value(value)?;
        Ok(Self(value))
    }

    /// Returns the exact validated value.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }

    const fn validated(value: &'a str) -> Self {
        Self(value)
    }
}

impl fmt::Debug for HeaderValue<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("HeaderValue([redacted])")
    }
}

fn validate_name(value: &str) -> Result<(), HeaderError> {
    if value.is_empty() {
        return Err(HeaderError::EmptyName);
    }
    if value.len() > MAX_HEADER_NAME_BYTES {
        return Err(HeaderError::NameTooLong);
    }
    if !value.bytes().all(is_token_byte) {
        return Err(HeaderError::InvalidName);
    }
    Ok(())
}

fn validate_request_value(value: &str) -> Result<(), HeaderError> {
    if value.len() > MAX_HEADER_VALUE_BYTES {
        return Err(HeaderError::ValueTooLong);
    }
    if !value.bytes().all(|byte| (b' '..=b'~').contains(&byte))
        || value.starts_with(' ')
        || value.ends_with(' ')
    {
        return Err(HeaderError::InvalidValue);
    }
    Ok(())
}

fn validate_response_value(value: &[u8]) -> Result<(), HeaderError> {
    if value.len() > MAX_HEADER_VALUE_BYTES {
        return Err(HeaderError::ValueTooLong);
    }
    if value.iter().any(|byte| *byte < b' ' || *byte == 0x7f) {
        return Err(HeaderError::InvalidValue);
    }
    Ok(())
}

fn encoded_line_len(name_len: usize, value_len: usize) -> Result<usize, HeaderError> {
    name_len
        .checked_add(value_len)
        .and_then(|len| len.checked_add(HEADER_LINE_OVERHEAD))
        .ok_or(HeaderError::AggregateTooLarge)
}

fn is_reserved_request_name(name: HeaderName<'_>) -> bool {
    const RESERVED: &[&str] = &[
        "authorization",
        "connection",
        "content-length",
        "host",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "proxy-connection",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
    ];
    RESERVED
        .iter()
        .any(|reserved| name.eq_ignore_ascii_case(reserved))
}

fn is_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'!' | b'#'
                | b'$'
                | b'%'
                | b'&'
                | b'\''
                | b'*'
                | b'+'
                | b'-'
                | b'.'
                | b'^'
                | b'_'
                | b'`'
                | b'|'
                | b'~'
        )
}

const fn typed_accept<'a>(media_type: MediaType<'a>) -> HeaderValue<'a> {
    HeaderValue::validated(media_type.as_str())
}

const fn typed_content_type<'a>(content_type: ContentType<'a>) -> HeaderValue<'a> {
    HeaderValue::validated(content_type.as_str())
}

#[cfg(test)]
mod tests;
