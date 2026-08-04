//! Canonical endpoint identity components.

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::net::Ipv6Addr;
use core::str::FromStr;

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
    /// The host is not canonical lowercase ASCII DNS, IPv4, or bracketed IPv6.
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
    Self::InvalidHost => "endpoint identity host is not canonical",
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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum CanonicalHost<'a> {
    Dns(&'a str),
    Ipv4([u8; 4]),
    Ipv6([u8; 16]),
}

/// Borrowed normalized identity of a credential-bound transport endpoint.
///
/// DNS input must already be canonical lowercase ASCII. Internationalized
/// names must use their lowercase ASCII A-label form. IPv6 input must be
/// bracketed; comparison uses its parsed 128-bit address, so equivalent text
/// forms have one identity. Zone identifiers are never admitted.
#[derive(Clone, Copy)]
pub struct EndpointIdentity<'a> {
    scheme: EndpointScheme,
    host: &'a str,
    canonical_host: CanonicalHost<'a>,
    effective_port: u16,
    base_path: &'a str,
}

impl<'a> EndpointIdentity<'a> {
    /// Creates an identity from normalized endpoint components.
    pub fn new(
        scheme: EndpointScheme,
        host: &'a str,
        effective_port: u16,
        base_path: &'a str,
    ) -> Result<Self, EndpointIdentityError> {
        let canonical_host = validate_host(host)?;
        if effective_port == 0 {
            return Err(EndpointIdentityError::InvalidPort);
        }
        validate_base_path(base_path)?;
        Ok(Self {
            scheme,
            host,
            canonical_host,
            effective_port,
            base_path,
        })
    }

    /// Returns the normalized network scheme.
    #[must_use]
    pub const fn scheme(self) -> EndpointScheme {
        self.scheme
    }

    /// Returns the admitted host text without credentials or a port.
    #[must_use]
    pub const fn host(self) -> &'a str {
        self.host
    }

    /// Returns the parsed identity used by equality, hashing, and signing.
    #[must_use]
    pub(crate) const fn canonical_host(self) -> CanonicalHost<'a> {
        self.canonical_host
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

    fn comparison_key(self) -> (EndpointScheme, CanonicalHost<'a>, u16, &'a str) {
        (
            self.scheme,
            self.canonical_host,
            self.effective_port,
            self.base_path,
        )
    }
}

impl PartialEq for EndpointIdentity<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.comparison_key() == other.comparison_key()
    }
}

impl Eq for EndpointIdentity<'_> {}

impl PartialOrd for EndpointIdentity<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EndpointIdentity<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.comparison_key().cmp(&other.comparison_key())
    }
}

impl Hash for EndpointIdentity<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.comparison_key().hash(state);
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

fn validate_host(host: &str) -> Result<CanonicalHost<'_>, EndpointIdentityError> {
    if host.is_empty() {
        return Err(EndpointIdentityError::EmptyHost);
    }
    if host.len() > MAX_ENDPOINT_HOST_BYTES {
        return Err(EndpointIdentityError::HostTooLong);
    }
    if !host.is_ascii() || host.contains(['%', '@', '/', '\\', '?', '#']) {
        return Err(EndpointIdentityError::InvalidHost);
    }
    if host.starts_with('[') || host.ends_with(']') {
        return parse_ipv6(host);
    }
    if host.contains(':') {
        return Err(EndpointIdentityError::InvalidHost);
    }
    if host
        .bytes()
        .all(|byte| byte.is_ascii_digit() || byte == b'.')
    {
        return parse_ipv4(host);
    }
    validate_dns(host)?;
    Ok(CanonicalHost::Dns(host))
}

fn parse_ipv6(host: &str) -> Result<CanonicalHost<'_>, EndpointIdentityError> {
    let address = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .ok_or(EndpointIdentityError::InvalidHost)?;
    if address.is_empty() || address.contains('%') {
        return Err(EndpointIdentityError::InvalidHost);
    }
    Ipv6Addr::from_str(address)
        .map(|value| CanonicalHost::Ipv6(value.octets()))
        .map_err(|_| EndpointIdentityError::InvalidHost)
}

fn parse_ipv4(host: &str) -> Result<CanonicalHost<'_>, EndpointIdentityError> {
    let mut octets = [0_u8; 4];
    let mut count = 0_usize;
    for part in host.split('.') {
        if count >= octets.len() || part.is_empty() || part.len() > 1 && part.starts_with('0') {
            return Err(EndpointIdentityError::InvalidHost);
        }
        let value = part
            .parse::<u8>()
            .map_err(|_| EndpointIdentityError::InvalidHost)?;
        let slot = octets
            .get_mut(count)
            .ok_or(EndpointIdentityError::InvalidHost)?;
        *slot = value;
        count = count
            .checked_add(1)
            .ok_or(EndpointIdentityError::InvalidHost)?;
    }
    if count != octets.len() {
        return Err(EndpointIdentityError::InvalidHost);
    }
    Ok(CanonicalHost::Ipv4(octets))
}

fn validate_dns(host: &str) -> Result<(), EndpointIdentityError> {
    if host.ends_with('.') {
        return Err(EndpointIdentityError::InvalidHost);
    }
    for label in host.split('.') {
        let reserved_hyphens = label
            .as_bytes()
            .get(2..4)
            .is_some_and(|bytes| bytes == b"--");
        if label.is_empty()
            || label.len() > 63
            || label.starts_with('-')
            || label.ends_with('-')
            || reserved_hyphens && !label.starts_with("xn--")
            || label == "xn--"
            || !label
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            return Err(EndpointIdentityError::InvalidHost);
        }
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
    fn identity_requires_complete_canonical_components() {
        let official =
            EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1");
        assert!(official.is_ok());
        if let Ok(official) = official {
            assert_eq!(official.host(), "api.hetzner.cloud");
            assert_eq!(official.effective_port(), 443);
        } else {
            unreachable!("endpoint-identity fixture construction failed");
        }
        for host in [
            "API.Hetzner.Cloud",
            "api.hetzner.cloud.",
            "api..example",
            "-api.example",
            "api_.example",
            "t\u{00e4}st.example",
            "api%2eexample",
            "user@api.example",
            "127.1",
            "127.00.0.1",
            "::1",
            "[fe80::1%25eth0]",
        ] {
            assert_eq!(
                EndpointIdentity::new(EndpointScheme::Https, host, 443, "/v1"),
                Err(EndpointIdentityError::InvalidHost),
                "{host}"
            );
        }
        assert!(
            EndpointIdentity::new(EndpointScheme::Https, "xn--tst-qla.example", 443, "/v1").is_ok()
        );
        assert!(EndpointIdentity::new(EndpointScheme::Https, "127.0.0.1", 443, "/v1").is_ok());
        for host in ["127.0.0.1.2", "256.0.0.1"] {
            assert_eq!(
                EndpointIdentity::new(EndpointScheme::Https, host, 443, "/v1"),
                Err(EndpointIdentityError::InvalidHost),
                "{host}"
            );
        }
    }

    #[test]
    fn bracketed_ipv6_comparison_uses_canonical_address_bits() {
        let compressed = EndpointIdentity::new(EndpointScheme::Https, "[2001:db8::1]", 443, "/v1");
        let expanded =
            EndpointIdentity::new(EndpointScheme::Https, "[2001:0db8:0:0:0:0:0:1]", 443, "/v1");
        assert!(compressed.is_ok() && expanded.is_ok());
        if let (Ok(compressed), Ok(expanded)) = (compressed, expanded) {
            assert_eq!(compressed, expanded);
        } else {
            unreachable!("IPv6 endpoint fixture construction failed");
        }
    }

    #[test]
    fn comparison_detects_every_destination_change() {
        let official =
            EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1");
        let Ok(official) = official else {
            unreachable!("endpoint-identity fixture construction failed");
        };
        for candidate in [
            EndpointIdentity::new(EndpointScheme::Http, "api.hetzner.cloud", 443, "/v1"),
            EndpointIdentity::new(EndpointScheme::Https, "evil.example", 443, "/v1"),
            EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 8443, "/v1"),
            EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v2"),
        ] {
            assert!(candidate.is_ok());
            if let Ok(candidate) = candidate {
                assert_ne!(candidate, official);
            } else {
                unreachable!("endpoint candidate fixture construction failed");
            }
        }
    }
}
