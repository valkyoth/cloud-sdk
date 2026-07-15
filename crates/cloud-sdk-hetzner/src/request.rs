//! Request-domain primitives shared by endpoint modules.

pub use cloud_sdk::Method;

/// Hetzner Cloud API base URL for the public v1 REST API.
pub const CLOUD_API_BASE_URL: &str = "https://api.hetzner.cloud/v1";

/// Hetzner API base URL for Storage Box operations in the public v1 REST API.
pub const HETZNER_API_BASE_URL: &str = "https://api.hetzner.com/v1";

/// Hetzner Cloud API major version currently targeted by this SDK.
pub const CLOUD_API_VERSION: u8 = 1;

/// Maximum endpoint path length admitted by the SDK policy.
///
/// This covers independently maximum-sized validated Zone and RRSet names,
/// percent encoding, RR types, and action suffixes.
pub const MAX_ENDPOINT_PATH_BYTES: usize = 1_024;

/// API base URL selected for an endpoint family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ApiBaseUrl {
    /// `https://api.hetzner.cloud/v1`.
    CloudV1,
    /// `https://api.hetzner.com/v1`.
    HetznerV1,
}

impl ApiBaseUrl {
    /// Returns the base URL string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CloudV1 => CLOUD_API_BASE_URL,
            Self::HetznerV1 => HETZNER_API_BASE_URL,
        }
    }
}

/// Endpoint path validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointPathError {
    /// Endpoint paths must start with `/`.
    MissingLeadingSlash,
    /// Endpoint paths must not be empty.
    Empty,
    /// Endpoint paths are capped by [`MAX_ENDPOINT_PATH_BYTES`].
    TooLong,
    /// Endpoint paths must not contain control bytes or spaces.
    InvalidByte,
    /// Endpoint paths must be relative to the selected base URL.
    AbsoluteUrl,
    /// Endpoint paths must not contain parent directory segments.
    ParentDirectorySegment,
}

impl_static_error!(EndpointPathError,
    Self::MissingLeadingSlash => "endpoint path lacks a leading slash",
    Self::Empty => "endpoint path is empty",
    Self::TooLong => "endpoint path exceeds the length limit",
    Self::InvalidByte => "endpoint path contains an invalid byte",
    Self::AbsoluteUrl => "endpoint path must not be an absolute URL",
    Self::ParentDirectorySegment => "endpoint path contains a parent-directory segment",
);

/// Borrowed, validated endpoint path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EndpointPath<'a> {
    value: &'a str,
}

impl<'a> EndpointPath<'a> {
    /// Creates a validated endpoint path.
    pub fn new(value: &'a str) -> Result<Self, EndpointPathError> {
        if value.is_empty() {
            return Err(EndpointPathError::Empty);
        }
        if value.len() > MAX_ENDPOINT_PATH_BYTES {
            return Err(EndpointPathError::TooLong);
        }
        if !value.starts_with('/') || value.starts_with("//") {
            return Err(EndpointPathError::MissingLeadingSlash);
        }
        if value.contains("://") || value.contains('\\') {
            return Err(EndpointPathError::AbsoluteUrl);
        }
        if has_invalid_path_byte(value) {
            return Err(EndpointPathError::InvalidByte);
        }
        if has_parent_directory_segment(value) {
            return Err(EndpointPathError::ParentDirectorySegment);
        }
        Ok(Self { value })
    }

    /// Returns the validated path.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

fn has_invalid_path_byte(value: &str) -> bool {
    for byte in value.bytes() {
        if byte <= b' ' || byte == 0x7f || matches!(byte, b'?' | b'#') {
            return true;
        }
    }
    false
}

fn has_parent_directory_segment(value: &str) -> bool {
    value.split('/').any(|segment| segment == "..")
}

#[cfg(test)]
mod tests {
    use super::{
        ApiBaseUrl, CLOUD_API_BASE_URL, CLOUD_API_VERSION, EndpointPath, EndpointPathError,
        HETZNER_API_BASE_URL, MAX_ENDPOINT_PATH_BYTES,
    };

    #[test]
    fn exposes_cloud_v1_base_url() {
        assert_eq!(CLOUD_API_BASE_URL, "https://api.hetzner.cloud/v1");
        assert_eq!(HETZNER_API_BASE_URL, "https://api.hetzner.com/v1");
        assert_eq!(CLOUD_API_VERSION, 1);
        assert_eq!(ApiBaseUrl::CloudV1.as_str(), CLOUD_API_BASE_URL);
        assert_eq!(ApiBaseUrl::HetznerV1.as_str(), HETZNER_API_BASE_URL);
    }

    #[test]
    fn validates_relative_endpoint_paths() {
        assert_eq!(
            EndpointPath::new("servers"),
            Err(EndpointPathError::MissingLeadingSlash)
        );
        assert_eq!(
            EndpointPath::new("/servers bad"),
            Err(EndpointPathError::InvalidByte)
        );
        assert_eq!(
            EndpointPath::new("//evil.example/steal"),
            Err(EndpointPathError::MissingLeadingSlash)
        );
        assert_eq!(
            EndpointPath::new("/\\evil.example/steal"),
            Err(EndpointPathError::AbsoluteUrl)
        );
        assert_eq!(
            EndpointPath::new("/servers?admin=true"),
            Err(EndpointPathError::InvalidByte)
        );
        assert_eq!(
            EndpointPath::new("/servers#fragment"),
            Err(EndpointPathError::InvalidByte)
        );
        assert_eq!(
            EndpointPath::new("/servers/../../v2/admin"),
            Err(EndpointPathError::ParentDirectorySegment)
        );
        assert_eq!(
            EndpointPath::new("/servers").map(EndpointPath::as_str),
            Ok("/servers")
        );

        let mut maximum = [b'a'; MAX_ENDPOINT_PATH_BYTES];
        maximum[0] = b'/';
        let maximum = core::str::from_utf8(&maximum);
        assert!(maximum.is_ok());
        let Ok(maximum) = maximum else {
            return;
        };
        assert!(EndpointPath::new(maximum).is_ok());

        let mut too_long = [b'a'; MAX_ENDPOINT_PATH_BYTES + 1];
        too_long[0] = b'/';
        let too_long = core::str::from_utf8(&too_long);
        assert!(too_long.is_ok());
        let Ok(too_long) = too_long else {
            return;
        };
        assert_eq!(EndpointPath::new(too_long), Err(EndpointPathError::TooLong));
    }
}
