use super::types::{
    MAX_ROBOT_FIREWALL_RULE_NAME_BYTES, MAX_ROBOT_FIREWALL_RULES_PER_DIRECTION,
    RobotFirewallAction, RobotFirewallCidr, RobotFirewallIpVersion, RobotFirewallPortRange,
    RobotFirewallProtocol, RobotFirewallRuleError, RobotFirewallTcpFlags,
    RobotFirewallTemplateName, validate_name,
};

/// One validated firewall rule; its position is part of policy semantics.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RobotFirewallRule<'a> {
    name: Option<&'a str>,
    ip_version: Option<RobotFirewallIpVersion>,
    destination_ip: Option<RobotFirewallCidr<'a>>,
    source_ip: Option<RobotFirewallCidr<'a>>,
    destination_port: Option<RobotFirewallPortRange<'a>>,
    source_port: Option<RobotFirewallPortRange<'a>>,
    protocol: Option<RobotFirewallProtocol>,
    tcp_flags: Option<RobotFirewallTcpFlags<'a>>,
    action: RobotFirewallAction,
}

impl<'a> RobotFirewallRule<'a> {
    /// Creates an action-only wildcard rule.
    #[must_use]
    pub const fn new(action: RobotFirewallAction) -> Self {
        Self {
            name: None,
            ip_version: None,
            destination_ip: None,
            source_ip: None,
            destination_port: None,
            source_port: None,
            protocol: None,
            tcp_flags: None,
            action,
        }
    }

    /// Adds a display-safe rule name.
    pub fn with_name(mut self, value: &'a str) -> Result<Self, RobotFirewallRuleError> {
        validate_name(value, MAX_ROBOT_FIREWALL_RULE_NAME_BYTES)?;
        self.name = Some(value);
        Ok(self)
    }

    /// Constrains the rule to one IP version.
    #[must_use]
    pub const fn with_ip_version(mut self, value: RobotFirewallIpVersion) -> Self {
        self.ip_version = Some(value);
        self
    }

    /// Adds a canonical destination IPv4 selector.
    #[must_use]
    pub const fn with_destination_ip(mut self, value: RobotFirewallCidr<'a>) -> Self {
        self.destination_ip = Some(value);
        self
    }

    /// Adds a canonical source IPv4 selector.
    #[must_use]
    pub const fn with_source_ip(mut self, value: RobotFirewallCidr<'a>) -> Self {
        self.source_ip = Some(value);
        self
    }

    /// Adds a destination TCP/UDP port constraint.
    #[must_use]
    pub const fn with_destination_port(mut self, value: RobotFirewallPortRange<'a>) -> Self {
        self.destination_port = Some(value);
        self
    }

    /// Adds a source TCP/UDP port constraint.
    #[must_use]
    pub const fn with_source_port(mut self, value: RobotFirewallPortRange<'a>) -> Self {
        self.source_port = Some(value);
        self
    }

    /// Adds a protocol constraint.
    #[must_use]
    pub const fn with_protocol(mut self, value: RobotFirewallProtocol) -> Self {
        self.protocol = Some(value);
        self
    }

    /// Adds a TCP-only flag expression.
    #[must_use]
    pub const fn with_tcp_flags(mut self, value: RobotFirewallTcpFlags<'a>) -> Self {
        self.tcp_flags = Some(value);
        self
    }

    /// Validates all source-locked cross-field restrictions.
    pub fn validate(self) -> Result<Self, RobotFirewallRuleError> {
        let has_ip = self.destination_ip.is_some() || self.source_ip.is_some();
        let has_port = self.destination_port.is_some() || self.source_port.is_some();
        let incompatible_port_protocol = has_port
            && self.protocol.is_some_and(|protocol| {
                !matches!(
                    protocol,
                    RobotFirewallProtocol::Tcp | RobotFirewallProtocol::Udp
                )
            });
        if has_ip && self.ip_version != Some(RobotFirewallIpVersion::Ipv4)
            || self.ip_version.is_none() && self.protocol.is_some()
            || incompatible_port_protocol
            || self.tcp_flags.is_some() && self.protocol != Some(RobotFirewallProtocol::Tcp)
            || self.ip_version == Some(RobotFirewallIpVersion::Ipv6) && has_ip
        {
            return Err(RobotFirewallRuleError::FieldConflict);
        }
        Ok(self)
    }

    pub(crate) const fn fields(self) -> RobotFirewallRuleFields<'a> {
        RobotFirewallRuleFields {
            name: self.name,
            ip_version: self.ip_version,
            destination_ip: self.destination_ip,
            source_ip: self.source_ip,
            destination_port: self.destination_port,
            source_port: self.source_port,
            protocol: self.protocol,
            tcp_flags: self.tcp_flags,
            action: self.action,
        }
    }
}

impl core::fmt::Debug for RobotFirewallRule<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotFirewallRule([redacted])")
    }
}

pub(crate) struct RobotFirewallRuleFields<'a> {
    pub(crate) name: Option<&'a str>,
    pub(crate) ip_version: Option<RobotFirewallIpVersion>,
    pub(crate) destination_ip: Option<RobotFirewallCidr<'a>>,
    pub(crate) source_ip: Option<RobotFirewallCidr<'a>>,
    pub(crate) destination_port: Option<RobotFirewallPortRange<'a>>,
    pub(crate) source_port: Option<RobotFirewallPortRange<'a>>,
    pub(crate) protocol: Option<RobotFirewallProtocol>,
    pub(crate) tcp_flags: Option<RobotFirewallTcpFlags<'a>>,
    pub(crate) action: RobotFirewallAction,
}

/// Borrowed ordered input and output firewall rules.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RobotFirewallRules<'a> {
    input: &'a [RobotFirewallRule<'a>],
    output: &'a [RobotFirewallRule<'a>],
}

impl<'a> RobotFirewallRules<'a> {
    /// Validates direction bounds and exact duplicate rejection without reordering.
    pub fn new(
        input: &'a [RobotFirewallRule<'a>],
        output: &'a [RobotFirewallRule<'a>],
    ) -> Result<Self, RobotFirewallRuleError> {
        validate_direction(input)?;
        validate_direction(output)?;
        Ok(Self { input, output })
    }

    /// Returns ordered incoming rules.
    #[must_use]
    pub const fn input(self) -> &'a [RobotFirewallRule<'a>] {
        self.input
    }

    /// Returns ordered outgoing rules.
    #[must_use]
    pub const fn output(self) -> &'a [RobotFirewallRule<'a>] {
        self.output
    }
}

impl core::fmt::Debug for RobotFirewallRules<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotFirewallRules")
            .field("input", &self.input.len())
            .field("output", &self.output.len())
            .finish()
    }
}

/// Complete replacement configuration for one firewall template.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RobotFirewallTemplateConfig<'a> {
    name: RobotFirewallTemplateName<'a>,
    filter_ipv6: Option<bool>,
    whitelist_hos: bool,
    is_default: bool,
    rules: RobotFirewallRules<'a>,
}

impl<'a> RobotFirewallTemplateConfig<'a> {
    /// Creates a complete template replacement.
    #[must_use]
    pub const fn new(
        name: RobotFirewallTemplateName<'a>,
        whitelist_hos: bool,
        is_default: bool,
        rules: RobotFirewallRules<'a>,
    ) -> Self {
        Self {
            name,
            filter_ipv6: None,
            whitelist_hos,
            is_default,
            rules,
        }
    }

    /// Explicitly sets the optional IPv6 filtering flag.
    #[must_use]
    pub const fn with_filter_ipv6(mut self, value: bool) -> Self {
        self.filter_ipv6 = Some(value);
        self
    }

    pub(crate) const fn parts(
        self,
    ) -> (
        RobotFirewallTemplateName<'a>,
        Option<bool>,
        bool,
        bool,
        RobotFirewallRules<'a>,
    ) {
        (
            self.name,
            self.filter_ipv6,
            self.whitelist_hos,
            self.is_default,
            self.rules,
        )
    }
}

fn validate_direction(rules: &[RobotFirewallRule<'_>]) -> Result<(), RobotFirewallRuleError> {
    if rules.len() > MAX_ROBOT_FIREWALL_RULES_PER_DIRECTION {
        return Err(RobotFirewallRuleError::TooManyRules);
    }
    let mut remaining = rules;
    while let Some((rule, tail)) = remaining.split_first() {
        rule.validate()?;
        if tail.contains(rule) {
            return Err(RobotFirewallRuleError::DuplicateRule);
        }
        remaining = tail;
    }
    Ok(())
}
