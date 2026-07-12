//! Shared Cloud request helpers.

use cloud_sdk::buffer;

use crate::labels::{LabelError, LabelKey, LabelValue};
use crate::request::{EndpointPath, EndpointPathError};

/// Maximum Cloud resource name bytes admitted by this SDK layer.
pub const MAX_CLOUD_NAME_BYTES: usize = 128;
/// Maximum Cloud resource text bytes admitted by this SDK layer.
pub const MAX_CLOUD_TEXT_BYTES: usize = 1024;

/// Error returned while building Cloud resource request components.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloudRequestError {
    /// Endpoint paths failed validation.
    InvalidPath(EndpointPathError),
    /// Labels failed validation.
    InvalidLabel(LabelError),
    /// Required field was not supplied.
    MissingRequiredField,
    /// Caller-provided path buffer is too small.
    PathBufferTooSmall,
    /// Caller-provided query buffer is too small.
    QueryBufferTooSmall,
    /// Path bytes failed UTF-8 conversion after construction.
    PathEncodingFailed,
    /// Name failed conservative validation.
    InvalidName,
    /// Text value failed conservative validation.
    InvalidText,
    /// Enum-like API value failed validation.
    InvalidType,
    /// DNS pointer action requires explicit set or reset.
    MissingDnsPtrIntent,
}

/// Nonzero Cloud resource identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CloudResourceId(u64);

impl CloudResourceId {
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

/// Borrowed, bounded Cloud resource name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloudName<'a> {
    value: &'a str,
}

impl<'a> CloudName<'a> {
    /// Creates a bounded JSON-safe name value.
    pub fn new(value: &'a str) -> Result<Self, CloudRequestError> {
        if invalid_text(value, MAX_CLOUD_NAME_BYTES, true) {
            return Err(CloudRequestError::InvalidName);
        }
        Ok(Self { value })
    }

    /// Returns the name.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

/// Borrowed, bounded Cloud text value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloudText<'a> {
    value: &'a str,
}

impl<'a> CloudText<'a> {
    /// Creates a bounded JSON-safe text value.
    pub fn new(value: &'a str) -> Result<Self, CloudRequestError> {
        if invalid_text(value, MAX_CLOUD_TEXT_BYTES, true) {
            return Err(CloudRequestError::InvalidText);
        }
        Ok(Self { value })
    }

    /// Returns the text value.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

/// Borrowed label entries for Cloud resource bodies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CloudLabels<'a> {
    entries: &'a [(LabelKey<'a>, LabelValue<'a>)],
}

impl<'a> CloudLabels<'a> {
    /// Creates a borrowed label slice after validating deterministic key order.
    pub fn new(entries: &'a [(LabelKey<'a>, LabelValue<'a>)]) -> Result<Self, CloudRequestError> {
        let mut previous: Option<&str> = None;
        for (key, _) in entries {
            if let Some(previous) = previous
                && previous >= key.as_str()
            {
                return Err(CloudRequestError::InvalidLabel(
                    LabelError::InvalidSelectorSyntax,
                ));
            }
            previous = Some(key.as_str());
        }
        Ok(Self { entries })
    }

    /// Returns the borrowed label entries.
    #[must_use]
    pub const fn entries(self) -> &'a [(LabelKey<'a>, LabelValue<'a>)] {
        self.entries
    }
}

#[cfg(feature = "serde")]
impl ::serde::Serialize for CloudLabels<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        use ::serde::ser::SerializeMap;

        let mut map = serializer.serialize_map(Some(self.entries.len()))?;
        for (key, value) in self.entries {
            map.serialize_entry(key.as_str(), value.as_str())?;
        }
        map.end()
    }
}

/// Small deterministic query writer.
pub struct CloudQueryWriter<'a> {
    output: &'a mut [u8],
    len: usize,
    first: bool,
}

impl<'a> CloudQueryWriter<'a> {
    /// Creates a query writer.
    pub fn new(output: &'a mut [u8]) -> Self {
        Self {
            output,
            len: 0,
            first: true,
        }
    }

    /// Writes a key/value pair.
    pub fn pair(&mut self, key: &str, value: &str) -> Result<(), CloudRequestError> {
        buffer::write_query_pair(
            self.output,
            &mut self.len,
            &mut self.first,
            key,
            value,
            CloudRequestError::QueryBufferTooSmall,
        )
    }

    /// Writes a key/u64 pair.
    pub fn u64_pair(&mut self, key: &str, value: u64) -> Result<(), CloudRequestError> {
        buffer::write_query_u64(
            self.output,
            &mut self.len,
            &mut self.first,
            key,
            value,
            CloudRequestError::QueryBufferTooSmall,
        )
    }

    /// Returns the bytes written.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns true when no bytes were written.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Writes a static endpoint path.
pub fn static_path(value: &'static str) -> Result<EndpointPath<'static>, CloudRequestError> {
    EndpointPath::new(value).map_err(CloudRequestError::InvalidPath)
}

/// Writes a static endpoint path into a caller-owned buffer.
pub fn write_static_path(output: &mut [u8], value: &str) -> Result<usize, CloudRequestError> {
    let mut len = 0;
    buffer::write_str(
        output,
        &mut len,
        value,
        CloudRequestError::PathBufferTooSmall,
    )?;
    validate_written_path(output, len)?;
    Ok(len)
}

/// Writes `{prefix}{id}{suffix}` into a caller-owned path buffer.
pub fn write_id_path(
    output: &mut [u8],
    prefix: &str,
    id: CloudResourceId,
    suffix: &str,
) -> Result<usize, CloudRequestError> {
    let mut len = 0;
    buffer::write_str(
        output,
        &mut len,
        prefix,
        CloudRequestError::PathBufferTooSmall,
    )?;
    buffer::write_u64(
        output,
        &mut len,
        id.get(),
        CloudRequestError::PathBufferTooSmall,
    )?;
    buffer::write_str(
        output,
        &mut len,
        suffix,
        CloudRequestError::PathBufferTooSmall,
    )?;
    validate_written_path(output, len)?;
    Ok(len)
}

fn validate_written_path(output: &[u8], len: usize) -> Result<(), CloudRequestError> {
    let bytes = output
        .get(..len)
        .ok_or(CloudRequestError::PathBufferTooSmall)?;
    let path = core::str::from_utf8(bytes).map_err(|_| CloudRequestError::PathEncodingFailed)?;
    EndpointPath::new(path).map_err(CloudRequestError::InvalidPath)?;
    Ok(())
}

fn invalid_text(value: &str, max: usize, reject_empty: bool) -> bool {
    (reject_empty && value.is_empty())
        || value.len() > max
        || value
            .bytes()
            .any(|byte| byte < 0x20 || byte == 0x7f || byte == b'"' || byte == b'\\')
        || value.chars().any(is_bidi_control)
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
