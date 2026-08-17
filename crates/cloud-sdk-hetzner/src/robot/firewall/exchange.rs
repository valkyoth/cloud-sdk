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
use super::reconcile::{
    PendingRobotFirewallTemplate, RobotFirewallTemplateMutationOutcome,
    RobotFirewallTemplateReconciliation, rules_match, template_reconciliation,
};
use super::request::*;
use super::types::RobotFirewallTemplateId;

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
    pub(crate) const fn from_executed(
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

impl<'a> CheckedRobotFirewall<'_, '_, RobotFirewallTemplateCreateRequest<'a>> {
    /// Reconciles the created template with the complete requested configuration.
    pub fn decode_response(
        self,
    ) -> Result<RobotFirewallTemplateMutationOutcome<'a>, RobotFirewallDecodeError> {
        let result = decode_template(self.inner)?;
        match template_reconciliation(self.request.config, &result) {
            RobotFirewallTemplateReconciliation::Confirmed => {
                Ok(RobotFirewallTemplateMutationOutcome::Confirmed(result))
            }
            RobotFirewallTemplateReconciliation::NameUnconfirmed => Ok(
                RobotFirewallTemplateMutationOutcome::ReconciliationRequired(
                    PendingRobotFirewallTemplate::new(result, self.request.config),
                ),
            ),
            RobotFirewallTemplateReconciliation::Mismatch => {
                Err(RobotFirewallDecodeError::MutationOutcomeMismatch)
            }
        }
    }
}

impl<'a> CheckedRobotFirewall<'_, '_, RobotFirewallTemplateUpdateRequest<'a>> {
    /// Requires identity preservation and reports whether all fields were confirmed.
    pub fn decode_response(
        self,
    ) -> Result<RobotFirewallTemplateMutationOutcome<'a>, RobotFirewallDecodeError> {
        let result = decode_template(self.inner)?;
        require_template(&result, self.request.template_id)?;
        match template_reconciliation(self.request.config, &result) {
            RobotFirewallTemplateReconciliation::Confirmed => {
                Ok(RobotFirewallTemplateMutationOutcome::Confirmed(result))
            }
            RobotFirewallTemplateReconciliation::NameUnconfirmed => Ok(
                RobotFirewallTemplateMutationOutcome::ReconciliationRequired(
                    PendingRobotFirewallTemplate::new(result, self.request.config),
                ),
            ),
            RobotFirewallTemplateReconciliation::Mismatch => {
                Err(RobotFirewallDecodeError::MutationOutcomeMismatch)
            }
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
