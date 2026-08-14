use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PrepareOperation, PreparedRequest,
    ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;

use super::decode::{
    RobotFirewallDecodeError, decode_robot_firewall, decode_robot_firewall_template,
    decode_robot_firewall_template_list,
};
use super::model::*;
use super::request::*;
use super::types::RobotFirewallTemplateId;
use super::value::{RobotFirewallRule, RobotFirewallRules, RobotFirewallTemplateConfig};

/// Prepared Robot firewall request retaining its exact typed association.
pub struct PreparedRobotFirewall<'storage, 'request, R> {
    request: &'request R,
    inner: PreparedRequest<'storage>,
}

impl<'storage, 'request, R> PreparedRobotFirewall<'storage, 'request, R> {
    /// Borrows the provider-neutral prepared request for inspection.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }

    /// Applies the exact response policy and retains request provenance.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedRobotFirewall<'buffer, 'request, R>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedRobotFirewall {
                request: self.request,
                inner,
            })
    }

    pub(super) fn into_plan_parts(self) -> (PreparedRequest<'storage>, &'request R) {
        (self.inner, self.request)
    }
}

impl<R> core::fmt::Debug for PreparedRobotFirewall<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PreparedRobotFirewall")
            .field("request", &"[bound]")
            .field("prepared", &self.inner)
            .finish()
    }
}

/// Checked Robot firewall response retaining its admitting request.
pub struct CheckedRobotFirewall<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
}

impl<'buffer, 'request, R> CheckedRobotFirewall<'buffer, 'request, R> {
    pub(super) const fn from_executed(
        request: &'request R,
        inner: CheckedResponseGuard<'buffer>,
    ) -> Self {
        Self { request, inner }
    }
}

impl<R> core::fmt::Debug for CheckedRobotFirewall<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CheckedRobotFirewall")
            .field("request", &"[bound]")
            .field("response", &"[checked]")
            .finish()
    }
}

macro_rules! prepare_bound {
    ($($type:ty),+ $(,)?) => {$ (
        impl $type {
            /// Prepares this operation while retaining exact response association.
            pub fn prepare_bound<'storage, 'request>(
                &'request self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRobotFirewall<'storage, 'request, Self>, RobotFirewallRequestError> {
                let inner = self.prepare(storage)?;
                Ok(PreparedRobotFirewall { request: self, inner })
            }
        }
    )+ };
}

prepare_bound!(
    RobotFirewallGetRequest,
    RobotFirewallReplaceRequest<'_>,
    RobotFirewallDeleteRequest,
    RobotFirewallTemplateListRequest,
    RobotFirewallTemplateCreateRequest<'_>,
    RobotFirewallTemplateGetRequest,
    RobotFirewallTemplateUpdateRequest<'_>,
    RobotFirewallTemplateDeleteRequest,
);

impl CheckedRobotFirewall<'_, '_, RobotFirewallGetRequest> {
    /// Decodes one firewall and binds it to the requested server.
    pub fn decode_response(self) -> Result<RobotFirewall, RobotFirewallDecodeError> {
        let result = decode_firewall(self.inner)?;
        require_server(&result, &self.request.server)?;
        Ok(result)
    }
}

impl CheckedRobotFirewall<'_, '_, RobotFirewallReplaceRequest<'_>> {
    /// Requires an in-progress response matching the exact replacement request.
    pub fn decode_response(self) -> Result<RobotFirewall, RobotFirewallDecodeError> {
        let result = decode_firewall(self.inner)?;
        require_server(&result, &self.request.server)?;
        if result.status != RobotFirewallRuntimeStatus::InProcess {
            return Err(RobotFirewallDecodeError::MutationOutcomeMismatch);
        }
        match self.request.intent {
            RobotFirewallReplaceIntent::Inline {
                filter_ipv6,
                whitelist_hos,
                rules,
                ..
            } => {
                if filter_ipv6.is_some_and(|expected| expected != result.filter_ipv6)
                    || whitelist_hos != result.whitelist_hos
                    || !rules_match(rules, &result.rules)
                {
                    return Err(RobotFirewallDecodeError::MutationOutcomeMismatch);
                }
            }
            RobotFirewallReplaceIntent::Template { filter_ipv6, .. } => {
                if filter_ipv6.is_some_and(|expected| expected != result.filter_ipv6) {
                    return Err(RobotFirewallDecodeError::MutationOutcomeMismatch);
                }
            }
        }
        Ok(result)
    }
}

impl CheckedRobotFirewall<'_, '_, RobotFirewallDeleteRequest> {
    /// Requires an in-progress empty-rule clear response for the requested server.
    pub fn decode_response(self) -> Result<RobotFirewall, RobotFirewallDecodeError> {
        let result = decode_firewall(self.inner)?;
        require_server(&result, &self.request.server)?;
        if result.status != RobotFirewallRuntimeStatus::InProcess
            || !result.rules.input.is_empty()
            || !result.rules.output.is_empty()
        {
            return Err(RobotFirewallDecodeError::MutationOutcomeMismatch);
        }
        Ok(result)
    }
}

impl CheckedRobotFirewall<'_, '_, RobotFirewallTemplateListRequest> {
    /// Decodes a bounded template inventory with unique IDs.
    pub fn decode_response(self) -> Result<RobotFirewallTemplateList, RobotFirewallDecodeError> {
        self.inner
            .decode_owned_with_workspace(decode_robot_firewall_template_list)
    }
}

impl CheckedRobotFirewall<'_, '_, RobotFirewallTemplateGetRequest> {
    /// Decodes one template and binds it to the requested identity.
    pub fn decode_response(self) -> Result<RobotFirewallTemplate, RobotFirewallDecodeError> {
        let result = decode_template(self.inner)?;
        require_template(&result, self.request.template_id)?;
        Ok(result)
    }
}

impl CheckedRobotFirewall<'_, '_, RobotFirewallTemplateCreateRequest<'_>> {
    /// Requires the created template to match the complete requested configuration.
    pub fn decode_response(self) -> Result<RobotFirewallTemplate, RobotFirewallDecodeError> {
        let result = decode_template(self.inner)?;
        if template_matches(self.request.config, &result) {
            Ok(result)
        } else {
            Err(RobotFirewallDecodeError::MutationOutcomeMismatch)
        }
    }
}

impl CheckedRobotFirewall<'_, '_, RobotFirewallTemplateUpdateRequest<'_>> {
    /// Requires identity preservation and a complete replacement match.
    pub fn decode_response(self) -> Result<RobotFirewallTemplate, RobotFirewallDecodeError> {
        let result = decode_template(self.inner)?;
        require_template(&result, self.request.template_id)?;
        if template_matches(self.request.config, &result) {
            Ok(result)
        } else {
            Err(RobotFirewallDecodeError::MutationOutcomeMismatch)
        }
    }
}

impl CheckedRobotFirewall<'_, '_, RobotFirewallTemplateDeleteRequest> {
    /// Accepts and clears the exact empty delete acknowledgement.
    pub fn decode_response(self) -> Result<(), RobotFirewallDecodeError> {
        drop(self);
        Ok(())
    }
}

fn decode_firewall(
    checked: CheckedResponseGuard<'_>,
) -> Result<RobotFirewall, RobotFirewallDecodeError> {
    checked.decode_owned_with_workspace(decode_robot_firewall)
}

fn decode_template(
    checked: CheckedResponseGuard<'_>,
) -> Result<RobotFirewallTemplate, RobotFirewallDecodeError> {
    checked.decode_owned_with_workspace(decode_robot_firewall_template)
}

fn require_server(
    result: &RobotFirewall,
    expected: &crate::robot::RobotServerNumber,
) -> Result<(), RobotFirewallDecodeError> {
    if result.server_number == *expected {
        Ok(())
    } else {
        Err(RobotFirewallDecodeError::ResponseIdentityMismatch)
    }
}

fn require_template(
    result: &RobotFirewallTemplate,
    expected: RobotFirewallTemplateId,
) -> Result<(), RobotFirewallDecodeError> {
    if result.summary.id == expected {
        Ok(())
    } else {
        Err(RobotFirewallDecodeError::ResponseIdentityMismatch)
    }
}

fn template_matches(
    config: RobotFirewallTemplateConfig<'_>,
    result: &RobotFirewallTemplate,
) -> bool {
    let (name, filter_ipv6, whitelist_hos, is_default, rules) = config.parts();
    result
        .summary
        .name
        .try_with_secret(|actual| actual == name.as_str())
        .unwrap_or(false)
        && filter_ipv6.is_none_or(|expected| expected == result.summary.filter_ipv6)
        && whitelist_hos == result.summary.whitelist_hos
        && is_default == result.summary.is_default
        && rules_match(rules, &result.rules)
}

fn rules_match(expected: RobotFirewallRules<'_>, actual: &RobotFirewallRuleSet) -> bool {
    direction_matches(expected.input(), &actual.input)
        && direction_matches(expected.output(), &actual.output)
}

fn direction_matches(
    expected: &[RobotFirewallRule<'_>],
    actual: &[RobotFirewallRuleModel],
) -> bool {
    expected.len() == actual.len()
        && expected
            .iter()
            .copied()
            .zip(actual)
            .all(|(expected, actual)| rule_matches(expected, actual))
}

fn rule_matches(expected: RobotFirewallRule<'_>, actual: &RobotFirewallRuleModel) -> bool {
    let Ok(expected) = expected.validate() else {
        return false;
    };
    let fields = expected.fields();
    fields.ip_version == actual.ip_version
        && fields.protocol == actual.protocol
        && fields.action == actual.action
        && text_matches(actual.name.as_ref(), fields.name)
        && text_matches(
            actual.destination_ip.as_ref(),
            fields.destination_ip.map(|value| value.as_str()),
        )
        && text_matches(
            actual.source_ip.as_ref(),
            fields.source_ip.map(|value| value.as_str()),
        )
        && text_matches(
            actual.destination_port.as_ref(),
            fields.destination_port.map(|value| value.as_str()),
        )
        && text_matches(
            actual.source_port.as_ref(),
            fields.source_port.map(|value| value.as_str()),
        )
        && text_matches(
            actual.tcp_flags.as_ref(),
            fields.tcp_flags.map(|value| value.as_str()),
        )
}

fn text_matches(actual: Option<&crate::serde::SensitiveText>, expected: Option<&str>) -> bool {
    match (actual, expected) {
        (None, None) => true,
        (Some(actual), Some(expected)) => actual
            .try_with_secret(|actual| actual == expected)
            .unwrap_or(false),
        _ => false,
    }
}

#[cfg(doctest)]
mod compile_fail {
    /// Different operation types cannot consume each other's checked response.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::{
    ///     CheckedRobotFirewall, RobotFirewallGetRequest, RobotFirewallTemplateGetRequest,
    /// };
    /// fn consume(_: CheckedRobotFirewall<'_, '_, RobotFirewallGetRequest>) {}
    /// fn wrong(response: CheckedRobotFirewall<'_, '_, RobotFirewallTemplateGetRequest>) {
    ///     consume(response);
    /// }
    /// ```
    fn association() {}
}
