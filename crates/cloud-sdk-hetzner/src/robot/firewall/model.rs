use alloc::vec::Vec;

use crate::robot::{
    RobotFirewallAction, RobotFirewallIpVersion, RobotFirewallProtocol, RobotFirewallTemplateId,
    RobotServerNumber,
};
use crate::serde::SensitiveText;

/// Maximum firewall templates admitted from one list response.
pub const MAX_ROBOT_FIREWALL_TEMPLATE_LIST_ITEMS: usize = 4_096;

/// Provider-reported firewall transition state.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotFirewallRuntimeStatus {
    /// Filtering is active.
    Active,
    /// Filtering is disabled.
    Disabled,
    /// A replacement or clear operation is still in progress.
    InProcess,
}

/// Provider-reported physical firewall switch port.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotFirewallPort {
    /// Main server switch port.
    Main,
    /// KVM switch port.
    Kvm,
}

/// One source-validated response rule. Rule order remains significant.
pub struct RobotFirewallRuleModel {
    pub(super) ip_version: Option<RobotFirewallIpVersion>,
    pub(super) name: Option<SensitiveText>,
    pub(super) destination_ip: Option<SensitiveText>,
    pub(super) source_ip: Option<SensitiveText>,
    pub(super) destination_port: Option<SensitiveText>,
    pub(super) source_port: Option<SensitiveText>,
    pub(super) protocol: Option<RobotFirewallProtocol>,
    pub(super) tcp_flags: Option<SensitiveText>,
    pub(super) action: RobotFirewallAction,
}

impl RobotFirewallRuleModel {
    /// Returns the optional IP version.
    #[must_use]
    pub const fn ip_version(&self) -> Option<RobotFirewallIpVersion> {
        self.ip_version
    }

    /// Returns the optional protocol.
    #[must_use]
    pub const fn protocol(&self) -> Option<RobotFirewallProtocol> {
        self.protocol
    }

    /// Returns the required action.
    #[must_use]
    pub const fn action(&self) -> RobotFirewallAction {
        self.action
    }

    /// Runs a closure with the protected optional rule name.
    pub fn try_with_name<R>(
        &self,
        inspect: impl FnOnce(Option<&str>) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        with_optional_text(self.name.as_ref(), inspect)
    }

    /// Runs a closure with the protected optional destination selector.
    pub fn try_with_destination_ip<R>(
        &self,
        inspect: impl FnOnce(Option<&str>) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        with_optional_text(self.destination_ip.as_ref(), inspect)
    }

    /// Runs a closure with the protected optional source selector.
    pub fn try_with_source_ip<R>(
        &self,
        inspect: impl FnOnce(Option<&str>) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        with_optional_text(self.source_ip.as_ref(), inspect)
    }
}

impl core::fmt::Debug for RobotFirewallRuleModel {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotFirewallRuleModel")
            .field("ip_version", &self.ip_version)
            .field("values", &"[redacted]")
            .field("protocol", &self.protocol)
            .field("action", &self.action)
            .finish()
    }
}

/// Bounded ordered incoming and outgoing response rules.
pub struct RobotFirewallRuleSet {
    pub(super) input: Vec<RobotFirewallRuleModel>,
    pub(super) output: Vec<RobotFirewallRuleModel>,
}

impl RobotFirewallRuleSet {
    /// Returns ordered incoming rules.
    #[must_use]
    pub fn input(&self) -> &[RobotFirewallRuleModel] {
        &self.input
    }

    /// Returns ordered outgoing rules.
    #[must_use]
    pub fn output(&self) -> &[RobotFirewallRuleModel] {
        &self.output
    }
}

impl core::fmt::Debug for RobotFirewallRuleSet {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotFirewallRuleSet")
            .field("input", &self.input.len())
            .field("output", &self.output.len())
            .finish()
    }
}

/// One server firewall bound to a canonical server identity.
pub struct RobotFirewall {
    pub(super) server_ip: SensitiveText,
    pub(super) server_number: RobotServerNumber,
    pub(super) status: RobotFirewallRuntimeStatus,
    pub(super) filter_ipv6: bool,
    pub(super) whitelist_hos: bool,
    pub(super) port: RobotFirewallPort,
    pub(super) rules: RobotFirewallRuleSet,
}

impl RobotFirewall {
    /// Runs a closure with the protected server IPv4 identity.
    pub fn try_with_server_ip<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.server_ip.try_with_secret(inspect)
    }

    /// Returns the protected server number.
    #[must_use]
    pub const fn server_number(&self) -> &RobotServerNumber {
        &self.server_number
    }

    /// Returns the provider transition state.
    #[must_use]
    pub const fn status(&self) -> RobotFirewallRuntimeStatus {
        self.status
    }

    /// Returns whether IPv6 filtering is enabled.
    #[must_use]
    pub const fn filter_ipv6(&self) -> bool {
        self.filter_ipv6
    }

    /// Returns whether Hetzner services are whitelisted.
    #[must_use]
    pub const fn whitelist_hos(&self) -> bool {
        self.whitelist_hos
    }

    /// Returns the selected switch port.
    #[must_use]
    pub const fn port(&self) -> RobotFirewallPort {
        self.port
    }

    /// Returns ordered source-validated rules.
    #[must_use]
    pub const fn rules(&self) -> &RobotFirewallRuleSet {
        &self.rules
    }
}

impl core::fmt::Debug for RobotFirewall {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotFirewall")
            .field("identity", &"[redacted]")
            .field("status", &self.status)
            .field("filter_ipv6", &self.filter_ipv6)
            .field("whitelist_hos", &self.whitelist_hos)
            .field("port", &self.port)
            .field("rules", &self.rules)
            .finish()
    }
}

/// Firewall-template inventory entry without rule payloads.
pub struct RobotFirewallTemplateSummary {
    pub(super) id: RobotFirewallTemplateId,
    pub(super) name: SensitiveText,
    pub(super) filter_ipv6: bool,
    pub(super) whitelist_hos: bool,
    pub(super) is_default: bool,
}

impl RobotFirewallTemplateSummary {
    /// Returns the template identity.
    #[must_use]
    pub const fn id(&self) -> RobotFirewallTemplateId {
        self.id
    }

    /// Returns whether this is Robot's default template.
    #[must_use]
    pub const fn is_default(&self) -> bool {
        self.is_default
    }

    /// Runs a closure with the protected name.
    pub fn try_with_name<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.name.try_with_secret(inspect)
    }
}

impl core::fmt::Debug for RobotFirewallTemplateSummary {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotFirewallTemplateSummary")
            .field("id", &self.id)
            .field("name", &"[redacted]")
            .field("filter_ipv6", &self.filter_ipv6)
            .field("whitelist_hos", &self.whitelist_hos)
            .field("is_default", &self.is_default)
            .finish()
    }
}

/// Detailed firewall template with ordered replacement rules.
pub struct RobotFirewallTemplate {
    pub(super) summary: RobotFirewallTemplateSummary,
    pub(super) rules: RobotFirewallRuleSet,
}

impl RobotFirewallTemplate {
    /// Returns summary fields and identity.
    #[must_use]
    pub const fn summary(&self) -> &RobotFirewallTemplateSummary {
        &self.summary
    }

    /// Returns ordered template rules.
    #[must_use]
    pub const fn rules(&self) -> &RobotFirewallRuleSet {
        &self.rules
    }
}

impl core::fmt::Debug for RobotFirewallTemplate {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotFirewallTemplate")
            .field("summary", &self.summary)
            .field("rules", &self.rules)
            .finish()
    }
}

/// Bounded template inventory with unique template identifiers.
pub struct RobotFirewallTemplateList(pub(super) Vec<RobotFirewallTemplateSummary>);

impl RobotFirewallTemplateList {
    /// Returns source-validated template summaries.
    #[must_use]
    pub fn as_slice(&self) -> &[RobotFirewallTemplateSummary] {
        &self.0
    }

    /// Returns the template count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Reports whether the inventory is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl core::fmt::Debug for RobotFirewallTemplateList {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_tuple("RobotFirewallTemplateList")
            .field(&self.0.len())
            .finish()
    }
}

fn with_optional_text<R>(
    value: Option<&SensitiveText>,
    inspect: impl FnOnce(Option<&str>) -> R,
) -> Result<R, core::str::Utf8Error> {
    match value {
        Some(value) => value.try_with_secret(|value| inspect(Some(value))),
        None => Ok(inspect(None)),
    }
}
