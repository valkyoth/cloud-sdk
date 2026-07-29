//! Shared Cloud request helpers.

use cloud_sdk::buffer::{self, SnapshotEncoder};
use cloud_sdk::transport::MAX_REQUEST_TARGET_BYTES;

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
}

impl_static_error!(CloudRequestError,
    Self::InvalidPath(_) => "cloud endpoint path is invalid",
    Self::InvalidLabel(_) => "cloud label is invalid",
    Self::PathBufferTooSmall => "cloud path buffer is too small",
    Self::QueryBufferTooSmall => "cloud query buffer is too small",
    Self::PathEncodingFailed => "cloud path encoding failed",
    Self::InvalidName => "cloud resource name is invalid",
    Self::InvalidText => "cloud request text is invalid",
    Self::InvalidType => "cloud request type is invalid",
);

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

/// Atomically encodes one immutable Cloud query snapshot.
pub(crate) fn encode_query<F>(output: &mut [u8], encode: F) -> Result<usize, CloudRequestError>
where
    F: Copy
        + for<'encoder> Fn(
            &mut SnapshotEncoder<'encoder, CloudRequestError>,
            &mut bool,
        ) -> Result<(), CloudRequestError>,
{
    buffer::encode_snapshot_bounded(
        encode,
        output,
        MAX_REQUEST_TARGET_BYTES,
        CloudRequestError::QueryBufferTooSmall,
        |encode, encoder| {
            let mut first = true;
            encode(encoder, &mut first)
        },
    )
}

/// Writes a static endpoint path.
pub fn static_path(value: &'static str) -> Result<EndpointPath<'static>, CloudRequestError> {
    EndpointPath::new(value).map_err(CloudRequestError::InvalidPath)
}

/// Writes a static endpoint path into a caller-owned buffer.
pub fn write_static_path(output: &mut [u8], value: &str) -> Result<usize, CloudRequestError> {
    EndpointPath::new(value).map_err(CloudRequestError::InvalidPath)?;
    buffer::encode_snapshot_bounded(
        value,
        output,
        crate::request::MAX_ENDPOINT_PATH_BYTES,
        CloudRequestError::PathBufferTooSmall,
        |value, encoder| encoder.string(value),
    )
}

/// Writes `{prefix}{id}{suffix}` into a caller-owned path buffer.
pub fn write_id_path(
    output: &mut [u8],
    prefix: &str,
    id: CloudResourceId,
    suffix: &str,
) -> Result<usize, CloudRequestError> {
    let len = buffer::encode_snapshot_bounded(
        (prefix, id, suffix),
        output,
        crate::request::MAX_ENDPOINT_PATH_BYTES,
        CloudRequestError::PathBufferTooSmall,
        |(prefix, id, suffix), encoder| {
            encoder.string(prefix)?;
            encoder.u64(id.get())?;
            encoder.string(suffix)
        },
    )?;
    if let Err(error) = validate_written_path(output, len) {
        if let Some(target) = output.get_mut(..len) {
            cloud_sdk_sanitization::sanitize_bytes(target);
        }
        return Err(error);
    }
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
