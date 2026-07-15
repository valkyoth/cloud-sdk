//! Bounded HTTP media-type validation for requests and responses.

use core::fmt;

/// Maximum content-type header value length admitted by the core contract.
pub const MAX_CONTENT_TYPE_BYTES: usize = 128;

/// Content-type validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContentTypeError {
    /// Content types must not be empty.
    Empty,
    /// Content types exceed [`MAX_CONTENT_TYPE_BYTES`].
    TooLong,
    /// Content types must contain a valid ASCII media type and parameters.
    Invalid,
}

impl_static_error!(ContentTypeError,
    Self::Empty => "content type is empty",
    Self::TooLong => "content type exceeds the length limit",
    Self::Invalid => "content type is invalid",
);

/// Borrowed, validated HTTP content type.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ContentType<'a> {
    value: &'a str,
    essence_len: usize,
}

impl<'a> ContentType<'a> {
    /// `application/json`.
    pub const JSON: Self = Self {
        value: "application/json",
        essence_len: 16,
    };

    /// Validates a content-type header value conservatively.
    pub fn new(value: &'a str) -> Result<Self, ContentTypeError> {
        if value.is_empty() {
            return Err(ContentTypeError::Empty);
        }
        if value.len() > MAX_CONTENT_TYPE_BYTES {
            return Err(ContentTypeError::TooLong);
        }
        if !value.bytes().all(|byte| (b' '..=b'~').contains(&byte)) {
            return Err(ContentTypeError::Invalid);
        }

        let essence_len = value.find(';').unwrap_or(value.len());
        let essence = value.get(..essence_len).ok_or(ContentTypeError::Invalid)?;
        validate_essence(essence)?;
        validate_parameters(value.get(essence_len..).ok_or(ContentTypeError::Invalid)?)?;
        Ok(Self { value, essence_len })
    }

    /// Returns the complete validated header value.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }

    /// Returns the media-type essence without parameters.
    #[must_use]
    pub fn essence(self) -> &'a str {
        self.value.get(..self.essence_len).unwrap_or_default()
    }

    /// Reports whether this content type has the supplied media-type essence.
    #[must_use]
    pub fn matches(self, media_type: MediaType<'_>) -> bool {
        self.essence().eq_ignore_ascii_case(media_type.as_str())
    }
}

impl fmt::Debug for ContentType<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ContentType([redacted])")
    }
}

/// Borrowed media-type essence used by response policies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MediaType<'a> {
    value: &'a str,
}

impl<'a> MediaType<'a> {
    /// `application/json`.
    pub const JSON: Self = Self {
        value: "application/json",
    };

    /// Validates a media-type essence without parameters.
    pub fn new(value: &'a str) -> Result<Self, ContentTypeError> {
        if value.is_empty() {
            return Err(ContentTypeError::Empty);
        }
        if value.len() > MAX_CONTENT_TYPE_BYTES {
            return Err(ContentTypeError::TooLong);
        }
        validate_essence(value)?;
        Ok(Self { value })
    }

    /// Returns the validated media-type essence.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

/// Owned, bounded content type captured from an HTTP response.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ResponseContentType {
    bytes: [u8; MAX_CONTENT_TYPE_BYTES],
    len: usize,
    essence_len: usize,
}

impl ResponseContentType {
    /// Validates and copies one response header value into bounded storage.
    pub fn new(value: &str) -> Result<Self, ContentTypeError> {
        let validated = ContentType::new(value)?;
        let mut bytes = [0_u8; MAX_CONTENT_TYPE_BYTES];
        let target = bytes
            .get_mut(..value.len())
            .ok_or(ContentTypeError::TooLong)?;
        target.copy_from_slice(value.as_bytes());
        Ok(Self {
            bytes,
            len: value.len(),
            essence_len: validated.essence_len,
        })
    }

    /// Returns the complete validated header value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
    }

    /// Returns the validated borrowed view.
    #[must_use]
    pub fn as_content_type(&self) -> ContentType<'_> {
        ContentType {
            value: self.as_str(),
            essence_len: self.essence_len,
        }
    }

    /// Reports whether this response has the supplied media-type essence.
    #[must_use]
    pub fn matches(&self, media_type: MediaType<'_>) -> bool {
        self.as_content_type().matches(media_type)
    }
}

impl fmt::Debug for ResponseContentType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ResponseContentType([redacted])")
    }
}

fn validate_essence(value: &str) -> Result<(), ContentTypeError> {
    let Some((media_type, subtype)) = value.split_once('/') else {
        return Err(ContentTypeError::Invalid);
    };
    if media_type.is_empty()
        || subtype.is_empty()
        || subtype.contains('/')
        || !media_type.bytes().all(is_http_token_byte)
        || !subtype.bytes().all(is_http_token_byte)
    {
        return Err(ContentTypeError::Invalid);
    }
    Ok(())
}

fn validate_parameters(mut value: &str) -> Result<(), ContentTypeError> {
    while !value.is_empty() {
        let Some(rest) = value.strip_prefix(';') else {
            return Err(ContentTypeError::Invalid);
        };
        value = trim_ows_start(rest);
        let name_len = value
            .bytes()
            .take_while(|byte| is_http_token_byte(*byte))
            .count();
        if name_len == 0 {
            return Err(ContentTypeError::Invalid);
        }
        value = value.get(name_len..).ok_or(ContentTypeError::Invalid)?;
        let Some(rest) = value.strip_prefix('=') else {
            return Err(ContentTypeError::Invalid);
        };
        value = parse_parameter_value(rest)?;
        value = trim_ows_start(value);
        if !value.is_empty() && !value.starts_with(';') {
            return Err(ContentTypeError::Invalid);
        }
    }
    Ok(())
}

fn parse_parameter_value(value: &str) -> Result<&str, ContentTypeError> {
    if let Some(mut quoted) = value.strip_prefix('"') {
        loop {
            let Some(byte) = quoted.bytes().next() else {
                return Err(ContentTypeError::Invalid);
            };
            quoted = quoted.get(1..).ok_or(ContentTypeError::Invalid)?;
            match byte {
                b'"' => return Ok(quoted),
                b'\\' => {
                    let escaped = quoted.bytes().next().ok_or(ContentTypeError::Invalid)?;
                    if !(b' '..=b'~').contains(&escaped) {
                        return Err(ContentTypeError::Invalid);
                    }
                    quoted = quoted.get(1..).ok_or(ContentTypeError::Invalid)?;
                }
                b' '..=b'~' => {}
                _ => return Err(ContentTypeError::Invalid),
            }
        }
    }

    let token_len = value
        .bytes()
        .take_while(|byte| is_http_token_byte(*byte))
        .count();
    if token_len == 0 {
        return Err(ContentTypeError::Invalid);
    }
    value.get(token_len..).ok_or(ContentTypeError::Invalid)
}

fn trim_ows_start(value: &str) -> &str {
    value.trim_start_matches([' ', '\t'])
}

const fn is_http_token_byte(byte: u8) -> bool {
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

#[cfg(test)]
mod tests {
    use super::{ContentType, ContentTypeError, MediaType, ResponseContentType};

    #[test]
    fn validates_media_types_and_parameters_conservatively() {
        for value in [
            "application/json",
            "Application/JSON",
            "application/json;charset=utf-8",
            "application/json; charset=\"utf-8\"",
            "application/problem+json; note=\"escaped\\\"quote\"",
        ] {
            assert!(ContentType::new(value).is_ok(), "{value}");
        }
        for value in [
            "application",
            "application/",
            "/json",
            "application/json/extra",
            "application/json;",
            "application/json; charset",
            "application/json; =utf-8",
            "application/json; charset=",
            "application/json; charset=\"unterminated",
            "application/json ; charset=utf-8",
        ] {
            assert_eq!(ContentType::new(value), Err(ContentTypeError::Invalid));
        }
    }

    #[test]
    fn response_content_types_are_owned_bounded_and_match_by_essence() {
        let response = ResponseContentType::new("Application/JSON; charset=utf-8");
        assert!(response.is_ok());
        if let Ok(response) = response {
            assert_eq!(response.as_str(), "Application/JSON; charset=utf-8");
            assert!(response.matches(MediaType::JSON));
            assert!(!response.matches(MediaType::new("text/plain").unwrap_or(MediaType::JSON)));
        }
    }

    #[test]
    fn media_type_rejects_parameters() {
        assert!(MediaType::new("application/problem+json").is_ok());
        assert_eq!(
            MediaType::new("application/json; charset=utf-8"),
            Err(ContentTypeError::Invalid)
        );
    }
}
