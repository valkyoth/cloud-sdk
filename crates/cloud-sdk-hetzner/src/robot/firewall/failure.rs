use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};

use crate::robot::protocol::decode_robot_failure_with;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

use super::request::*;

/// Source-locked Robot firewall provider failure code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotFirewallFailureCode {
    /// The selected server does not exist.
    ServerNotFound,
    /// The selected physical firewall port does not exist.
    PortNotFound,
    /// Firewall configuration is unavailable for the server.
    NotAvailable,
    /// The selected template does not exist.
    TemplateNotFound,
    /// A replacement is already in progress.
    InProcess,
    /// The provider firewall rule limit was exceeded.
    RuleLimitExceeded,
    /// Internal provider rules prevent disabling the firewall.
    CannotBeDisabled,
    /// No templates were returned or a template identity was not found.
    NotFound,
}

#[derive(Clone, Copy)]
enum Operation {
    Get,
    Replace,
    Delete,
    TemplateList,
    TemplateCreate,
    TemplateGet,
    TemplateUpdate,
    TemplateDelete,
}

macro_rules! decode_failure {
    ($type:ty, $operation:ident) => {
        impl $type {
            /// Decodes only failures source-locked for this firewall operation.
            pub fn decode_failure(
                &self,
                response: TransportResponse<'_, '_>,
                workspace: &mut ResponseDecodeWorkspace,
            ) -> Result<RobotFailure, RobotDecodeError> {
                decode(Operation::$operation, response, workspace)
            }
        }
    };
}

decode_failure!(RobotFirewallGetRequest, Get);
decode_failure!(RobotFirewallReplaceRequest<'_>, Replace);
decode_failure!(RobotFirewallDeleteRequest, Delete);
decode_failure!(RobotFirewallTemplateListRequest, TemplateList);
decode_failure!(RobotFirewallTemplateCreateRequest<'_>, TemplateCreate);
decode_failure!(RobotFirewallTemplateGetRequest, TemplateGet);
decode_failure!(RobotFirewallTemplateUpdateRequest<'_>, TemplateUpdate);
decode_failure!(RobotFirewallTemplateDeleteRequest, TemplateDelete);

fn decode(
    operation: Operation,
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    let (allow_invalid_input, statuses): (bool, &[u16]) = match operation {
        Operation::Get | Operation::TemplateList | Operation::TemplateGet => (false, &[404]),
        Operation::Replace => (true, &[404, 409]),
        Operation::Delete => (false, &[404, 409]),
        Operation::TemplateCreate => (true, &[]),
        Operation::TemplateUpdate => (true, &[404]),
        Operation::TemplateDelete => (false, &[404]),
    };
    decode_robot_failure_with(
        response,
        workspace,
        allow_invalid_input,
        statuses,
        |status, code| classify(operation, status, code).map(Into::into),
    )
}

fn classify(operation: Operation, status: u16, code: &str) -> Option<RobotFirewallFailureCode> {
    match (operation, status, code) {
        (Operation::Get | Operation::Replace | Operation::Delete, 404, "SERVER_NOT_FOUND") => {
            Some(RobotFirewallFailureCode::ServerNotFound)
        }
        (
            Operation::Get | Operation::Replace | Operation::Delete,
            404,
            "FIREWALL_PORT_NOT_FOUND",
        ) => Some(RobotFirewallFailureCode::PortNotFound),
        (
            Operation::Get | Operation::Replace | Operation::Delete,
            404,
            "FIREWALL_NOT_AVAILABLE",
        ) => Some(RobotFirewallFailureCode::NotAvailable),
        (Operation::Replace, 404, "FIREWALL_TEMPLATE_NOT_FOUND") => {
            Some(RobotFirewallFailureCode::TemplateNotFound)
        }
        (Operation::Replace | Operation::Delete, 409, "FIREWALL_IN_PROCESS") => {
            Some(RobotFirewallFailureCode::InProcess)
        }
        (Operation::Replace | Operation::Delete, 409, "FIREWALL_RULE_LIMIT_EXCEEDED") => {
            Some(RobotFirewallFailureCode::RuleLimitExceeded)
        }
        (Operation::Replace | Operation::Delete, 409, "FIREWALL_CANNOT_BE_DISABLED") => {
            Some(RobotFirewallFailureCode::CannotBeDisabled)
        }
        (
            Operation::TemplateList
            | Operation::TemplateGet
            | Operation::TemplateUpdate
            | Operation::TemplateDelete,
            404,
            "NOT_FOUND",
        ) => Some(RobotFirewallFailureCode::NotFound),
        _ => None,
    }
}

impl From<RobotFirewallFailureCode> for RobotProviderErrorCode {
    fn from(code: RobotFirewallFailureCode) -> Self {
        match code {
            RobotFirewallFailureCode::ServerNotFound => Self::ServerNotFound,
            RobotFirewallFailureCode::PortNotFound => Self::FirewallPortNotFound,
            RobotFirewallFailureCode::NotAvailable => Self::FirewallNotAvailable,
            RobotFirewallFailureCode::TemplateNotFound => Self::FirewallTemplateNotFound,
            RobotFirewallFailureCode::InProcess => Self::FirewallInProcess,
            RobotFirewallFailureCode::RuleLimitExceeded => Self::FirewallRuleLimitExceeded,
            RobotFirewallFailureCode::CannotBeDisabled => Self::FirewallCannotBeDisabled,
            RobotFirewallFailureCode::NotFound => Self::NotFound,
        }
    }
}
