use alloc::string::ToString;
use core::net::Ipv4Addr;
use core::str::FromStr;

/// Maximum source-locked rules admitted in either firewall direction.
pub const MAX_ROBOT_FIREWALL_RULES_PER_DIRECTION: usize = 100;
/// Maximum UTF-8 bytes admitted in a firewall rule name.
pub const MAX_ROBOT_FIREWALL_RULE_NAME_BYTES: usize = 128;
/// Maximum UTF-8 bytes admitted in a firewall template name.
pub const MAX_ROBOT_FIREWALL_TEMPLATE_NAME_BYTES: usize = 128;

/// Failure while validating a Robot firewall value or ordered rule set.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotFirewallRuleError {
    /// A name was empty, oversized, controlled, or directionally misleading.
    InvalidName,
    /// A template identifier was zero.
    InvalidTemplateId,
    /// An IP address or CIDR was malformed or noncanonical.
    InvalidCidr,
    /// A port or inclusive port range was malformed.
    InvalidPort,
    /// A TCP flag expression was malformed.
    InvalidTcpFlags,
    /// A field combination violates Robot's source constraints.
    FieldConflict,
    /// One direction exceeded the local rule bound.
    TooManyRules,
    /// One direction contains an exact duplicate rule.
    DuplicateRule,
}

impl_static_error!(RobotFirewallRuleError,
    Self::InvalidName => "Robot firewall name is invalid",
    Self::InvalidTemplateId => "Robot firewall template identifier is invalid",
    Self::InvalidCidr => "Robot firewall CIDR is invalid",
    Self::InvalidPort => "Robot firewall port is invalid",
    Self::InvalidTcpFlags => "Robot firewall TCP flags are invalid",
    Self::FieldConflict => "Robot firewall rule fields conflict",
    Self::TooManyRules => "Robot firewall direction exceeds the rule limit",
    Self::DuplicateRule => "Robot firewall direction contains a duplicate rule",
);

/// Requested firewall lifecycle status.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotFirewallStatus {
    /// Enable packet filtering.
    Active,
    /// Disable packet filtering.
    Disabled,
}

impl RobotFirewallStatus {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
        }
    }
}

/// Optional IP-version constraint for one rule.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotFirewallIpVersion {
    /// IPv4 traffic.
    Ipv4,
    /// IPv6 traffic.
    Ipv6,
}

impl RobotFirewallIpVersion {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Ipv4 => "ipv4",
            Self::Ipv6 => "ipv6",
        }
    }
}

/// Protocol above IP admitted by Robot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotFirewallProtocol {
    /// Transmission Control Protocol.
    Tcp,
    /// User Datagram Protocol.
    Udp,
    /// Generic Routing Encapsulation.
    Gre,
    /// Internet Control Message Protocol.
    Icmp,
    /// IP-in-IP encapsulation.
    Ipip,
    /// Authentication Header.
    Ah,
    /// Encapsulating Security Payload.
    Esp,
}

impl RobotFirewallProtocol {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Tcp => "tcp",
            Self::Udp => "udp",
            Self::Gre => "gre",
            Self::Icmp => "icmp",
            Self::Ipip => "ipip",
            Self::Ah => "ah",
            Self::Esp => "esp",
        }
    }
}

/// Required firewall rule action.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotFirewallAction {
    /// Admit matching traffic.
    Accept,
    /// Drop matching traffic.
    Discard,
}

impl RobotFirewallAction {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Discard => "discard",
        }
    }
}

/// Non-zero Robot firewall template identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RobotFirewallTemplateId(u64);

impl RobotFirewallTemplateId {
    /// Creates a non-zero template identifier.
    pub const fn new(value: u64) -> Result<Self, RobotFirewallRuleError> {
        if value == 0 {
            Err(RobotFirewallRuleError::InvalidTemplateId)
        } else {
            Ok(Self(value))
        }
    }

    /// Returns the provider identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Validated borrowed firewall template name.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RobotFirewallTemplateName<'a>(&'a str);

impl<'a> RobotFirewallTemplateName<'a> {
    /// Validates a display-safe non-empty template name.
    pub fn new(value: &'a str) -> Result<Self, RobotFirewallRuleError> {
        validate_name(value, MAX_ROBOT_FIREWALL_TEMPLATE_NAME_BYTES)?;
        Ok(Self(value))
    }

    pub(crate) const fn as_str(self) -> &'a str {
        self.0
    }
}

impl core::fmt::Debug for RobotFirewallTemplateName<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotFirewallTemplateName([redacted])")
    }
}

/// Canonical IPv4 host or network selector.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RobotFirewallCidr<'a>(&'a str);

impl<'a> RobotFirewallCidr<'a> {
    /// Accepts a canonical IPv4 address or CIDR with no host bits set.
    pub fn new(value: &'a str) -> Result<Self, RobotFirewallRuleError> {
        let (address, prefix) = match value.split_once('/') {
            Some((address, prefix)) => {
                if prefix.is_empty() || prefix.bytes().any(|byte| !byte.is_ascii_digit()) {
                    return Err(RobotFirewallRuleError::InvalidCidr);
                }
                let prefix = prefix
                    .parse::<u8>()
                    .map_err(|_| RobotFirewallRuleError::InvalidCidr)?;
                (address, Some(prefix))
            }
            None => (value, None),
        };
        let parsed =
            Ipv4Addr::from_str(address).map_err(|_| RobotFirewallRuleError::InvalidCidr)?;
        if parsed.to_string() != address || prefix.is_some_and(|prefix| prefix > 32) {
            return Err(RobotFirewallRuleError::InvalidCidr);
        }
        if let Some(prefix) = prefix {
            let mask = if prefix == 0 {
                0
            } else {
                let shift = 32_u32
                    .checked_sub(u32::from(prefix))
                    .ok_or(RobotFirewallRuleError::InvalidCidr)?;
                u32::MAX
                    .checked_shl(shift)
                    .ok_or(RobotFirewallRuleError::InvalidCidr)?
            };
            if u32::from(parsed) & !mask != 0 {
                return Err(RobotFirewallRuleError::InvalidCidr);
            }
        }
        Ok(Self(value))
    }

    pub(crate) const fn as_str(self) -> &'a str {
        self.0
    }
}

impl core::fmt::Debug for RobotFirewallCidr<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotFirewallCidr([redacted])")
    }
}

/// Validated single port or inclusive range.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RobotFirewallPortRange<'a> {
    text: &'a str,
    first: u16,
    last: u16,
}

impl<'a> RobotFirewallPortRange<'a> {
    /// Parses `port` or canonical `first-last` syntax.
    pub fn new(value: &'a str) -> Result<Self, RobotFirewallRuleError> {
        let (first, last) = match value.split_once('-') {
            Some((first, last)) if !last.contains('-') => (parse_port(first)?, parse_port(last)?),
            Some(_) => return Err(RobotFirewallRuleError::InvalidPort),
            None => {
                let port = parse_port(value)?;
                (port, port)
            }
        };
        if first > last || (first != last && value != alloc::format!("{first}-{last}")) {
            return Err(RobotFirewallRuleError::InvalidPort);
        }
        Ok(Self {
            text: value,
            first,
            last,
        })
    }

    /// Returns inclusive numeric bounds.
    #[must_use]
    pub const fn bounds(self) -> (u16, u16) {
        (self.first, self.last)
    }

    pub(crate) const fn as_str(self) -> &'a str {
        self.text
    }
}

/// Bounded logical TCP-flag expression.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RobotFirewallTcpFlags<'a>(&'a str);

impl<'a> RobotFirewallTcpFlags<'a> {
    /// Validates known lower-case flags joined by `|`, `&`, or `!`.
    pub fn new(value: &'a str) -> Result<Self, RobotFirewallRuleError> {
        if value.is_empty() || value.len() > 64 || !valid_tcp_flags(value) {
            return Err(RobotFirewallRuleError::InvalidTcpFlags);
        }
        Ok(Self(value))
    }

    pub(crate) const fn as_str(self) -> &'a str {
        self.0
    }
}

impl core::fmt::Debug for RobotFirewallTcpFlags<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotFirewallTcpFlags([redacted])")
    }
}

pub(crate) fn validate_name(value: &str, maximum: usize) -> Result<(), RobotFirewallRuleError> {
    if value.is_empty()
        || value.len() > maximum
        || value.starts_with(char::is_whitespace)
        || value.ends_with(char::is_whitespace)
        || value.chars().any(prohibited_name_character)
    {
        Err(RobotFirewallRuleError::InvalidName)
    } else {
        Ok(())
    }
}

fn prohibited_name_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{061c}'
                | '\u{200b}'..='\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2060}'..='\u{2069}'
                | '\u{feff}'
        )
}

fn parse_port(value: &str) -> Result<u16, RobotFirewallRuleError> {
    if value.is_empty()
        || value.len() > 5
        || value.bytes().any(|byte| !byte.is_ascii_digit())
        || value.len() > 1 && value.starts_with('0')
    {
        return Err(RobotFirewallRuleError::InvalidPort);
    }
    value
        .parse::<u16>()
        .ok()
        .filter(|port| *port != 0)
        .ok_or(RobotFirewallRuleError::InvalidPort)
}

fn valid_tcp_flags(value: &str) -> bool {
    let mut start = 0;
    let bytes = value.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if matches!(byte, b'|' | b'&' | b'!') {
            if !valid_flag(&value[start..index]) {
                return false;
            }
            let Some(next) = index.checked_add(1) else {
                return false;
            };
            start = next;
        } else if !byte.is_ascii_lowercase() {
            return false;
        }
    }
    valid_flag(&value[start..])
}

fn valid_flag(value: &str) -> bool {
    matches!(value, "fin" | "syn" | "rst" | "psh" | "ack" | "urg")
}
