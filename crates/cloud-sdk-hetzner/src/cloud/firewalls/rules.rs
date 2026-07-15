//! Firewall rule validation.

use crate::cloud::ip::IpCidr;

/// Maximum rules accepted per Firewall.
pub const MAX_FIREWALL_RULES: usize = 50;
/// Maximum CIDR selectors accepted by one rule direction.
pub const MAX_RULE_CIDRS: usize = 100;
/// Maximum Firewall rule description bytes.
pub const MAX_RULE_DESCRIPTION_BYTES: usize = 255;

/// Firewall rule validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirewallRuleError {
    /// A rule has more than 100 CIDR selectors.
    TooManyCidrs,
    /// A rule contains a duplicate CIDR selector.
    DuplicateCidr,
    /// A ruleset has more than 50 rules.
    TooManyRules,
    /// A ruleset contains an exact duplicate rule.
    DuplicateRule,
    /// A description is empty, too long, or contains unsafe control text.
    InvalidDescription,
    /// A port or port range is malformed or outside `1..=65535`.
    InvalidPort,
    /// A port was supplied for a protocol other than TCP or UDP.
    PortProtocolConflict,
}

impl_static_error!(FirewallRuleError,
    Self::TooManyCidrs => "firewall rule exceeds the CIDR limit",
    Self::DuplicateCidr => "firewall rule contains a duplicate CIDR",
    Self::TooManyRules => "firewall ruleset exceeds the rule limit",
    Self::DuplicateRule => "firewall ruleset contains a duplicate rule",
    Self::InvalidDescription => "firewall rule description is invalid",
    Self::InvalidPort => "firewall rule port is invalid",
    Self::PortProtocolConflict => "firewall rule port conflicts with its protocol",
);

/// Firewall traffic protocol.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirewallProtocol {
    /// TCP.
    Tcp,
    /// UDP.
    Udp,
    /// ICMP.
    Icmp,
    /// Encapsulating Security Payload.
    Esp,
    /// Generic Routing Encapsulation.
    Gre,
}

/// Direction-specific IP selectors. This prevents source/destination conflicts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirewallSelectors<'a> {
    /// Incoming traffic selected by source CIDRs.
    Incoming(&'a [IpCidr<'a>]),
    /// Outgoing traffic selected by destination CIDRs.
    Outgoing(&'a [IpCidr<'a>]),
}

impl<'a> FirewallSelectors<'a> {
    /// Validates selector count and uniqueness.
    pub fn incoming(cidrs: &'a [IpCidr<'a>]) -> Result<Self, FirewallRuleError> {
        validate_cidrs(cidrs)?;
        Ok(Self::Incoming(cidrs))
    }

    /// Validates selector count and uniqueness.
    pub fn outgoing(cidrs: &'a [IpCidr<'a>]) -> Result<Self, FirewallRuleError> {
        validate_cidrs(cidrs)?;
        Ok(Self::Outgoing(cidrs))
    }
}

/// Validated Firewall port or inclusive port range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirewallPort<'a> {
    value: &'a str,
    first: u16,
    last: u16,
}

impl<'a> FirewallPort<'a> {
    /// Parses `port` or `first-last` syntax.
    pub fn new(value: &'a str) -> Result<Self, FirewallRuleError> {
        let (first, last) = match value.split_once('-') {
            Some((first, last)) if !first.is_empty() && !last.is_empty() => {
                if last.contains('-') {
                    return Err(FirewallRuleError::InvalidPort);
                }
                (parse_port(first)?, parse_port(last)?)
            }
            Some(_) => return Err(FirewallRuleError::InvalidPort),
            None => {
                let port = parse_port(value)?;
                (port, port)
            }
        };
        if first > last {
            return Err(FirewallRuleError::InvalidPort);
        }
        Ok(Self { value, first, last })
    }

    /// Returns the validated API string.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }

    /// Returns the inclusive bounds.
    #[must_use]
    pub const fn bounds(self) -> (u16, u16) {
        (self.first, self.last)
    }
}

/// Validated optional rule description.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirewallDescription<'a>(&'a str);

impl<'a> FirewallDescription<'a> {
    /// Validates a description.
    pub fn new(value: &'a str) -> Result<Self, FirewallRuleError> {
        if value.is_empty()
            || value.len() > MAX_RULE_DESCRIPTION_BYTES
            || value.bytes().any(|byte| byte < 0x20 || byte == 0x7f)
            || value.chars().any(is_bidi_control)
        {
            return Err(FirewallRuleError::InvalidDescription);
        }
        Ok(Self(value))
    }

    /// Returns the description.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// One validated Firewall rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirewallRule<'a> {
    selectors: FirewallSelectors<'a>,
    protocol: FirewallProtocol,
    port: Option<FirewallPort<'a>>,
    description: Option<FirewallDescription<'a>>,
}

impl<'a> FirewallRule<'a> {
    /// Creates a rule while enforcing protocol/port compatibility.
    pub fn try_new(
        selectors: FirewallSelectors<'a>,
        protocol: FirewallProtocol,
        port: Option<FirewallPort<'a>>,
    ) -> Result<Self, FirewallRuleError> {
        if port.is_some() && !matches!(protocol, FirewallProtocol::Tcp | FirewallProtocol::Udp) {
            return Err(FirewallRuleError::PortProtocolConflict);
        }
        Ok(Self {
            selectors,
            protocol,
            port,
            description: None,
        })
    }

    /// Sets the optional description.
    #[must_use]
    pub const fn with_description(mut self, description: FirewallDescription<'a>) -> Self {
        self.description = Some(description);
        self
    }

    /// Returns the direction-specific selectors.
    #[must_use]
    pub const fn selectors(self) -> FirewallSelectors<'a> {
        self.selectors
    }

    /// Returns the protocol.
    #[must_use]
    pub const fn protocol(self) -> FirewallProtocol {
        self.protocol
    }

    /// Returns the optional port constraint.
    #[must_use]
    pub const fn port(self) -> Option<FirewallPort<'a>> {
        self.port
    }

    /// Returns the optional description.
    #[must_use]
    pub const fn description(self) -> Option<FirewallDescription<'a>> {
        self.description
    }
}

/// Borrowed, validated Firewall ruleset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirewallRuleSet<'a>(&'a [FirewallRule<'a>]);

impl<'a> FirewallRuleSet<'a> {
    /// Validates rule count and rejects exact duplicate rules.
    pub fn new(rules: &'a [FirewallRule<'a>]) -> Result<Self, FirewallRuleError> {
        if rules.len() > MAX_FIREWALL_RULES {
            return Err(FirewallRuleError::TooManyRules);
        }
        let mut remaining = rules;
        while let Some((rule, tail)) = remaining.split_first() {
            if tail.contains(rule) {
                return Err(FirewallRuleError::DuplicateRule);
            }
            remaining = tail;
        }
        Ok(Self(rules))
    }

    /// Returns the validated rules.
    #[must_use]
    pub const fn rules(self) -> &'a [FirewallRule<'a>] {
        self.0
    }
}

fn parse_port(value: &str) -> Result<u16, FirewallRuleError> {
    if value.is_empty() || value.bytes().any(|byte| !byte.is_ascii_digit()) {
        return Err(FirewallRuleError::InvalidPort);
    }
    let port = value
        .parse::<u16>()
        .map_err(|_| FirewallRuleError::InvalidPort)?;
    if port == 0 {
        return Err(FirewallRuleError::InvalidPort);
    }
    Ok(port)
}

fn validate_cidrs(cidrs: &[IpCidr<'_>]) -> Result<(), FirewallRuleError> {
    if cidrs.len() > MAX_RULE_CIDRS {
        return Err(FirewallRuleError::TooManyCidrs);
    }
    let mut remaining = cidrs;
    while let Some((cidr, tail)) = remaining.split_first() {
        if tail.contains(cidr) {
            return Err(FirewallRuleError::DuplicateCidr);
        }
        remaining = tail;
    }
    Ok(())
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
