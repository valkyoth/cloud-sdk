//! Shared Load Balancer request values.

use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use core::str::FromStr;

use crate::cloud::ip::IpValidationError;
use crate::cloud::shared::{CloudLabels, CloudRequestError, CloudResourceId};

/// Load Balancer identifier.
pub type LoadBalancerId = CloudResourceId;
/// Load Balancer action identifier.
pub type LoadBalancerActionId = CloudResourceId;
/// Load Balancer type identifier.
pub type LoadBalancerTypeId = CloudResourceId;
/// Certificate identifier used by HTTPS services.
pub type LoadBalancerCertificateId = CloudResourceId;
/// Network identifier used by Load Balancer actions.
pub type LoadBalancerNetworkId = CloudResourceId;
/// Server identifier used by Load Balancer targets.
pub type LoadBalancerServerId = CloudResourceId;
/// Load Balancer labels.
pub type LoadBalancerLabels<'a> = CloudLabels<'a>;

/// Error returned while building a Load Balancer request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadBalancerRequestError {
    /// A shared Cloud request value or buffer operation failed.
    Cloud(CloudRequestError),
    /// IP or CIDR validation failed.
    Ip(IpValidationError),
    /// A required request field was omitted.
    MissingRequiredField,
    /// A bounded string failed its endpoint-specific validation.
    InvalidText,
    /// A port was zero or outside the protocol's admitted range.
    InvalidPort,
    /// A health-check interval, timeout, or retry count is outside API bounds.
    InvalidHealthCheck,
    /// HTTP service configuration is incompatible with its protocol.
    InvalidServiceConfiguration,
    /// A target option is incompatible with its target type.
    InvalidTargetConfiguration,
    /// Metrics start must sort before metrics end.
    InvalidTimeRange,
    /// An array exceeds its source-locked API limit.
    TooManyItems,
    /// DNS pointer intent must explicitly set or reset the record.
    MissingDnsPtrIntent,
}

impl From<CloudRequestError> for LoadBalancerRequestError {
    fn from(value: CloudRequestError) -> Self {
        Self::Cloud(value)
    }
}

impl From<IpValidationError> for LoadBalancerRequestError {
    fn from(value: IpValidationError) -> Self {
        Self::Ip(value)
    }
}

macro_rules! bounded_text {
    ($name:ident, $max:expr, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name<'a>(&'a str);

        impl<'a> $name<'a> {
            /// Validates and creates the value.
            pub fn new(value: &'a str) -> Result<Self, LoadBalancerRequestError> {
                if invalid_text(value, $max, true) {
                    return Err(LoadBalancerRequestError::InvalidText);
                }
                Ok(Self(value))
            }

            /// Returns the validated value.
            #[must_use]
            pub const fn as_str(self) -> &'a str {
                self.0
            }
        }
    };
}

bounded_text!(LoadBalancerName, 128, "Validated Load Balancer name.");
bounded_text!(
    LoadBalancerType,
    128,
    "Load Balancer type ID-or-name reference."
);
bounded_text!(
    LoadBalancerLocation,
    128,
    "Load Balancer location ID-or-name reference."
);
bounded_text!(
    LoadBalancerNetworkZone,
    128,
    "Load Balancer network-zone name."
);
bounded_text!(
    HealthCheckResponse,
    256,
    "Expected HTTP health-check response fragment."
);
bounded_text!(
    HealthCheckStatusCode,
    16,
    "HTTP health-check status-code pattern."
);
bounded_text!(StickyCookieName, 100, "Sticky-session cookie name.");

/// HTTP health-check path without literal spaces.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HealthCheckPath<'a>(&'a str);

impl<'a> HealthCheckPath<'a> {
    /// Creates a bounded path. Spaces must be percent encoded before construction.
    pub fn new(value: &'a str) -> Result<Self, LoadBalancerRequestError> {
        if invalid_text(value, 256, true) || value.as_bytes().contains(&b' ') {
            return Err(LoadBalancerRequestError::InvalidText);
        }
        Ok(Self(value))
    }

    /// Returns the validated path.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// Load balancing algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadBalancerAlgorithm {
    /// Distribute requests in rotation.
    RoundRobin,
    /// Prefer the target with fewer active connections.
    LeastConnections,
}

impl LoadBalancerAlgorithm {
    /// Returns the source-locked API value.
    #[must_use]
    pub const fn as_api_str(self) -> &'static str {
        match self {
            Self::RoundRobin => "round_robin",
            Self::LeastConnections => "least_connections",
        }
    }
}

/// Nonzero TCP/UDP port.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerPort(u16);

impl LoadBalancerPort {
    /// Creates a nonzero port.
    pub const fn new(value: u16) -> Result<Self, LoadBalancerRequestError> {
        if value == 0 {
            return Err(LoadBalancerRequestError::InvalidPort);
        }
        Ok(Self(value))
    }

    /// Returns the port.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// Syntactically valid IPv4 or IPv6 address.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerIp<'a> {
    value: &'a str,
    address: IpAddr,
}

impl<'a> LoadBalancerIp<'a> {
    /// Parses a single IP address.
    pub fn new(value: &'a str) -> Result<Self, LoadBalancerRequestError> {
        let address = IpAddr::from_str(value)
            .map_err(|_| LoadBalancerRequestError::Ip(IpValidationError::InvalidSyntax))?;
        Ok(Self { value, address })
    }

    /// Returns the original validated address.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }

    /// Returns the parsed address.
    #[must_use]
    pub const fn address(self) -> IpAddr {
        self.address
    }
}

/// Public server IP accepted by a server target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerPublicIp<'a>(LoadBalancerIp<'a>);

impl<'a> LoadBalancerPublicIp<'a> {
    /// Parses an address and rejects private, loopback, link-local, and multicast space.
    pub fn new(value: &'a str) -> Result<Self, LoadBalancerRequestError> {
        let ip = LoadBalancerIp::new(value)?;
        let invalid = match ip.address() {
            IpAddr::V4(address) => invalid_public_v4(address),
            IpAddr::V6(address) => invalid_public_v6(address),
        };
        if invalid {
            return Err(LoadBalancerRequestError::InvalidTargetConfiguration);
        }
        Ok(Self(ip))
    }

    /// Returns the validated public address.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0.as_str()
    }
}

fn invalid_public_v4(address: Ipv4Addr) -> bool {
    let [first, second, third, _] = address.octets();
    address.is_private()
        || address.is_loopback()
        || address.is_link_local()
        || address.is_multicast()
        || address.is_unspecified()
        || address.is_broadcast()
        || first == 0
        || (first == 100 && (64..=127).contains(&second))
        || (first == 192 && second == 0 && (third == 0 || third == 2))
        || (first == 192 && second == 88 && third == 99)
        || (first == 198 && (second == 18 || second == 19))
        || (first == 198 && second == 51 && third == 100)
        || (first == 203 && second == 0 && third == 113)
        || first >= 240
}

fn invalid_public_v6(address: Ipv6Addr) -> bool {
    let [first, second, third, fourth, _, _, _, _] = address.segments();
    // IANA marks 64:ff9b::/96 (NAT64), 2001::/32 (Teredo), and 2002::/16
    // (6to4) globally reachable. They are intentionally accepted here; Hetzner
    // still enforces that a selected target address belongs to the project.
    address.is_loopback()
        || address.is_unspecified()
        || address.is_multicast()
        || address.is_unique_local()
        || address.is_unicast_link_local()
        || address.to_ipv4_mapped().is_some_and(invalid_public_v4)
        || (first == 0x100 && second == 0 && third == 0 && fourth == 0)
        || (first == 0x2001 && second == 0x0db8)
}

fn invalid_text(value: &str, max: usize, reject_empty: bool) -> bool {
    (reject_empty && value.is_empty())
        || value.len() > max
        || value.starts_with(char::is_whitespace)
        || value.ends_with(char::is_whitespace)
        || value
            .chars()
            .any(|ch| ch.is_control() || is_bidi_control(ch))
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
