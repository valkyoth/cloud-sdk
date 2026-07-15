//! Immutable endpoint identity for credential-bound transports.

use core::fmt;

/// Maximum normalized endpoint host length admitted by the core contract.
pub const MAX_ENDPOINT_HOST_BYTES: usize = 253;

/// Maximum normalized endpoint base-path length admitted by the core contract.
pub const MAX_ENDPOINT_BASE_PATH_BYTES: usize = 1024;

/// Endpoint identity validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointIdentityError {
    /// The transport has not been bound to an endpoint identity.
    UnboundTransport,
    /// Endpoint hosts must not be empty.
    EmptyHost,
    /// Endpoint hosts exceed [`MAX_ENDPOINT_HOST_BYTES`].
    HostTooLong,
    /// Endpoint hosts must already be normalized, visible ASCII without URL
    /// authority delimiters.
    InvalidHost,
    /// Effective endpoint ports must be nonzero.
    InvalidPort,
    /// Endpoint base paths must use normalized absolute-path form.
    InvalidBasePath,
    /// Endpoint base paths exceed [`MAX_ENDPOINT_BASE_PATH_BYTES`].
    BasePathTooLong,
}

impl_static_error!(EndpointIdentityError,
    Self::UnboundTransport => "transport endpoint identity is unbound",
    Self::EmptyHost => "endpoint identity host is empty",
    Self::HostTooLong => "endpoint identity host exceeds the length limit",
    Self::InvalidHost => "endpoint identity host is not normalized",
    Self::InvalidPort => "endpoint identity port is invalid",
    Self::InvalidBasePath => "endpoint identity base path is not normalized",
    Self::BasePathTooLong => "endpoint identity base path exceeds the length limit",
);

/// Network scheme in an immutable endpoint identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EndpointScheme {
    /// Plain HTTP, intended only for explicitly admitted local test transports.
    Http,
    /// HTTPS.
    Https,
}

/// Borrowed normalized identity of a credential-bound transport endpoint.
///
/// The identity contains no credential material. Provider facades can compare
/// all four fields with their official endpoint constants before execution.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EndpointIdentity<'a> {
    scheme: EndpointScheme,
    host: &'a str,
    effective_port: u16,
    base_path: &'a str,
}

impl<'a> EndpointIdentity<'a> {
    /// Creates an identity from already normalized endpoint components.
    pub fn new(
        scheme: EndpointScheme,
        host: &'a str,
        effective_port: u16,
        base_path: &'a str,
    ) -> Result<Self, EndpointIdentityError> {
        validate_host(host)?;
        if effective_port == 0 {
            return Err(EndpointIdentityError::InvalidPort);
        }
        validate_base_path(base_path)?;
        Ok(Self {
            scheme,
            host,
            effective_port,
            base_path,
        })
    }

    /// Returns the normalized network scheme.
    #[must_use]
    pub const fn scheme(self) -> EndpointScheme {
        self.scheme
    }

    /// Returns the normalized host without credentials or a port.
    #[must_use]
    pub const fn host(self) -> &'a str {
        self.host
    }

    /// Returns the effective port, including the scheme default when omitted.
    #[must_use]
    pub const fn effective_port(self) -> u16 {
        self.effective_port
    }

    /// Returns the normalized absolute base path.
    #[must_use]
    pub const fn base_path(self) -> &'a str {
        self.base_path
    }
}

impl fmt::Debug for EndpointIdentity<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EndpointIdentity")
            .field("scheme", &self.scheme)
            .field("host", &"[redacted]")
            .field("effective_port", &self.effective_port)
            .field("base_path", &"[redacted]")
            .finish()
    }
}

/// Transport whose credentials are permanently bound to one endpoint.
pub trait BoundTransport {
    /// Returns the immutable normalized endpoint identity.
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError>;
}

fn validate_host(host: &str) -> Result<(), EndpointIdentityError> {
    if host.is_empty() {
        return Err(EndpointIdentityError::EmptyHost);
    }
    if host.len() > MAX_ENDPOINT_HOST_BYTES {
        return Err(EndpointIdentityError::HostTooLong);
    }
    if !host.bytes().all(|byte| {
        byte.is_ascii_graphic()
            && !byte.is_ascii_uppercase()
            && !matches!(byte, b'/' | b'\\' | b'@' | b'?' | b'#' | b'%')
    }) {
        return Err(EndpointIdentityError::InvalidHost);
    }
    Ok(())
}

fn validate_base_path(base_path: &str) -> Result<(), EndpointIdentityError> {
    if base_path.len() > MAX_ENDPOINT_BASE_PATH_BYTES {
        return Err(EndpointIdentityError::BasePathTooLong);
    }
    if !base_path.starts_with('/')
        || (base_path != "/" && base_path.ends_with('/'))
        || base_path.contains("//")
        || base_path.split('/').any(|part| matches!(part, "." | ".."))
        || !base_path
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && !matches!(byte, b'\\' | b'?' | b'#' | b'%'))
    {
        return Err(EndpointIdentityError::InvalidBasePath);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{EndpointIdentity, EndpointIdentityError, EndpointScheme};

    #[test]
    fn endpoint_identity_requires_normalized_complete_components() {
        let official =
            EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1");
        assert!(official.is_ok());
        if let Ok(official) = official {
            assert_eq!(official.scheme(), EndpointScheme::Https);
            assert_eq!(official.host(), "api.hetzner.cloud");
            assert_eq!(official.effective_port(), 443);
            assert_eq!(official.base_path(), "/v1");
        }
        assert_eq!(
            EndpointIdentity::new(EndpointScheme::Https, "", 443, "/v1"),
            Err(EndpointIdentityError::EmptyHost)
        );
        assert_eq!(
            EndpointIdentity::new(EndpointScheme::Https, "API.Hetzner.Cloud", 443, "/v1"),
            Err(EndpointIdentityError::InvalidHost)
        );
        assert_eq!(
            EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 0, "/v1"),
            Err(EndpointIdentityError::InvalidPort)
        );
        for path in ["v1", "/v1/", "/v1//admin", "/v1/../admin", "/v1?x=1"] {
            assert_eq!(
                EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, path),
                Err(EndpointIdentityError::InvalidBasePath)
            );
        }
    }

    #[test]
    fn endpoint_identity_comparison_detects_every_destination_change() {
        let official =
            EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1");
        let Ok(official) = official else { return };
        for candidate in [
            EndpointIdentity::new(EndpointScheme::Http, "api.hetzner.cloud", 443, "/v1"),
            EndpointIdentity::new(EndpointScheme::Https, "evil.example", 443, "/v1"),
            EndpointIdentity::new(EndpointScheme::Https, "sub.api.hetzner.cloud", 443, "/v1"),
            EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 8443, "/v1"),
            EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v2"),
        ] {
            assert!(candidate.is_ok());
            if let Ok(candidate) = candidate {
                assert_ne!(candidate, official);
            }
        }
    }
}
