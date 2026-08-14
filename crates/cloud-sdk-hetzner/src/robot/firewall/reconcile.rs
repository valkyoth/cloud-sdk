use super::model::{
    RobotFirewall, RobotFirewallRuleModel, RobotFirewallRuleSet, RobotFirewallTemplate,
    RobotFirewallTemplateSummary,
};
use super::value::{RobotFirewallRule, RobotFirewallRules, RobotFirewallTemplateConfig};
use crate::serde::SensitiveText;

/// Result of comparing a detailed template with one complete requested policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotFirewallTemplateReconciliation {
    /// Every returned field, including the protected name, matches.
    Confirmed,
    /// Every returned field matches, but Robot omitted the documented name.
    NameUnconfirmed,
    /// At least one returned field contradicts the requested policy.
    Mismatch,
}

/// Detailed template whose protected name still requires reconciliation.
#[must_use = "pending Robot firewall templates must be reconciled"]
pub struct PendingRobotFirewallTemplate {
    template: RobotFirewallTemplate,
}

impl PendingRobotFirewallTemplate {
    pub(super) const fn new(template: RobotFirewallTemplate) -> Self {
        Self { template }
    }

    /// Borrows the observed detailed state without marking it confirmed.
    #[must_use]
    pub const fn observed(&self) -> &RobotFirewallTemplate {
        &self.template
    }

    /// Confirms this pending mutation using its name-bearing list summary.
    ///
    /// Robot does not expose a revision binding the list and detail reads.
    /// Callers must prevent concurrent template mutation while collecting both
    /// observations or repeat reconciliation after any possible race.
    pub fn reconcile_with_summary(
        self,
        summary: &RobotFirewallTemplateSummary,
        expected: RobotFirewallTemplateConfig<'_>,
    ) -> Result<RobotFirewallTemplate, Self> {
        let (name, filter_ipv6, whitelist_hos, is_default, _) = expected.parts();
        let identity_matches = summary.id == self.template.summary.id;
        let name_matches = summary
            .name
            .as_ref()
            .is_some_and(|actual| actual.constant_time_eq(name.as_str()));
        let summary_matches_detail = (summary.filter_ipv6 == self.template.summary.filter_ipv6)
            & (summary.whitelist_hos == self.template.summary.whitelist_hos)
            & (summary.is_default == self.template.summary.is_default);
        let summary_matches_expected = filter_ipv6.is_none_or(|value| value == summary.filter_ipv6)
            & (whitelist_hos == summary.whitelist_hos)
            & (is_default == summary.is_default);
        let detail_matches = template_policy_matches_without_name(expected, &self.template);
        if identity_matches
            & name_matches
            & summary_matches_detail
            & summary_matches_expected
            & detail_matches
        {
            Ok(self.template)
        } else {
            Err(self)
        }
    }
}

impl core::fmt::Debug for PendingRobotFirewallTemplate {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_tuple("PendingRobotFirewallTemplate")
            .field(&"[redacted template]")
            .finish()
    }
}

/// Checked result of a successful template mutation.
#[must_use = "template mutation confirmation state must be handled"]
pub enum RobotFirewallTemplateMutationOutcome {
    /// Robot returned enough state to confirm the complete requested policy.
    Confirmed(RobotFirewallTemplate),
    /// Robot omitted the name, so list and detail state must be reconciled.
    ReconciliationRequired(PendingRobotFirewallTemplate),
}

impl RobotFirewallTemplateMutationOutcome {
    /// Borrows the template only when every requested field was confirmed.
    #[must_use]
    pub const fn confirmed(&self) -> Option<&RobotFirewallTemplate> {
        match self {
            Self::Confirmed(template) => Some(template),
            Self::ReconciliationRequired(_) => None,
        }
    }

    /// Borrows the explicit pending state when reconciliation is required.
    #[must_use]
    pub const fn pending(&self) -> Option<&PendingRobotFirewallTemplate> {
        match self {
            Self::Confirmed(_) => None,
            Self::ReconciliationRequired(pending) => Some(pending),
        }
    }

    /// Consumes the outcome without erasing unresolved confirmation state.
    #[must_use = "unresolved template state must not be discarded"]
    pub fn into_confirmed(self) -> Result<RobotFirewallTemplate, PendingRobotFirewallTemplate> {
        match self {
            Self::Confirmed(template) => Ok(template),
            Self::ReconciliationRequired(pending) => Err(pending),
        }
    }
}

impl core::fmt::Debug for RobotFirewallTemplateMutationOutcome {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_tuple(match self {
                Self::Confirmed(_) => "Confirmed",
                Self::ReconciliationRequired(_) => "ReconciliationRequired",
            })
            .field(&"[redacted template]")
            .finish()
    }
}

impl RobotFirewallRuleModel {
    /// Compares every rule field without exposing protected response text.
    #[must_use]
    pub fn matches(&self, expected: RobotFirewallRule<'_>) -> bool {
        rule_matches(expected, self)
    }
}

impl RobotFirewallRuleSet {
    /// Compares both ordered rule directions exactly.
    #[must_use]
    pub fn matches(&self, expected: RobotFirewallRules<'_>) -> bool {
        rules_match(expected, self)
    }
}

impl RobotFirewall {
    /// Compares all mutable inline-policy fields exactly.
    #[must_use]
    pub fn matches_inline_policy(
        &self,
        filter_ipv6: bool,
        whitelist_hos: bool,
        rules: RobotFirewallRules<'_>,
    ) -> bool {
        (self.filter_ipv6 == filter_ipv6)
            & (self.whitelist_hos == whitelist_hos)
            & rules_match(rules, &self.rules)
    }
}

impl RobotFirewallTemplate {
    /// Reconciles all template fields with one complete requested policy.
    #[must_use]
    pub fn reconcile(
        &self,
        expected: RobotFirewallTemplateConfig<'_>,
    ) -> RobotFirewallTemplateReconciliation {
        template_reconciliation(expected, self)
    }
}

pub(super) fn template_reconciliation(
    config: RobotFirewallTemplateConfig<'_>,
    result: &RobotFirewallTemplate,
) -> RobotFirewallTemplateReconciliation {
    let (name, ..) = config.parts();
    let name_matches = result
        .summary
        .name
        .as_ref()
        .is_some_and(|actual| actual.constant_time_eq(name.as_str()));
    let other_fields_match = template_policy_matches_without_name(config, result);
    if !other_fields_match || result.summary.name.is_some() && !name_matches {
        RobotFirewallTemplateReconciliation::Mismatch
    } else if name_matches {
        RobotFirewallTemplateReconciliation::Confirmed
    } else {
        RobotFirewallTemplateReconciliation::NameUnconfirmed
    }
}

fn template_policy_matches_without_name(
    config: RobotFirewallTemplateConfig<'_>,
    result: &RobotFirewallTemplate,
) -> bool {
    let (_, filter_ipv6, whitelist_hos, is_default, rules) = config.parts();
    filter_ipv6.is_none_or(|expected| expected == result.summary.filter_ipv6)
        & (whitelist_hos == result.summary.whitelist_hos)
        & (is_default == result.summary.is_default)
        & rules_match(rules, &result.rules)
}

pub(super) fn rules_match(expected: RobotFirewallRules<'_>, actual: &RobotFirewallRuleSet) -> bool {
    direction_matches(expected.input(), &actual.input)
        & direction_matches(expected.output(), &actual.output)
}

pub(super) fn rule_models_equal(
    left: &RobotFirewallRuleModel,
    right: &RobotFirewallRuleModel,
) -> bool {
    let name = optional_text_equal(left.name.as_ref(), right.name.as_ref());
    let destination_ip =
        optional_text_equal(left.destination_ip.as_ref(), right.destination_ip.as_ref());
    let source_ip = optional_text_equal(left.source_ip.as_ref(), right.source_ip.as_ref());
    let destination_port = optional_text_equal(
        left.destination_port.as_ref(),
        right.destination_port.as_ref(),
    );
    let source_port = optional_text_equal(left.source_port.as_ref(), right.source_port.as_ref());
    let tcp_flags = optional_text_equal(left.tcp_flags.as_ref(), right.tcp_flags.as_ref());
    (left.ip_version == right.ip_version)
        & (left.protocol == right.protocol)
        & (left.action == right.action)
        & name
        & destination_ip
        & source_ip
        & destination_port
        & source_port
        & tcp_flags
}

fn direction_matches(
    expected: &[RobotFirewallRule<'_>],
    actual: &[RobotFirewallRuleModel],
) -> bool {
    let values_match = expected
        .iter()
        .copied()
        .zip(actual)
        .fold(true, |matches, (expected, actual)| {
            rule_matches(expected, actual) & matches
        });
    (expected.len() == actual.len()) & values_match
}

fn rule_matches(expected: RobotFirewallRule<'_>, actual: &RobotFirewallRuleModel) -> bool {
    let Ok(expected) = expected.validate() else {
        return false;
    };
    let fields = expected.fields();
    let name = text_matches(actual.name.as_ref(), fields.name);
    let destination_ip = text_matches(
        actual.destination_ip.as_ref(),
        fields.destination_ip.map(|value| value.as_str()),
    );
    let source_ip = text_matches(
        actual.source_ip.as_ref(),
        fields.source_ip.map(|value| value.as_str()),
    );
    let destination_port = text_matches(
        actual.destination_port.as_ref(),
        fields.destination_port.map(|value| value.as_str()),
    );
    let source_port = text_matches(
        actual.source_port.as_ref(),
        fields.source_port.map(|value| value.as_str()),
    );
    let tcp_flags = text_matches(
        actual.tcp_flags.as_ref(),
        fields.tcp_flags.map(|value| value.as_str()),
    );
    (fields.ip_version == actual.ip_version)
        & (fields.protocol == actual.protocol)
        & (fields.action == actual.action)
        & name
        & destination_ip
        & source_ip
        & destination_port
        & source_port
        & tcp_flags
}

fn text_matches(actual: Option<&SensitiveText>, expected: Option<&str>) -> bool {
    match (actual, expected) {
        (None, None) => true,
        (Some(actual), Some(expected)) => actual.constant_time_eq(expected),
        _ => false,
    }
}

fn optional_text_equal(left: Option<&SensitiveText>, right: Option<&SensitiveText>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}
