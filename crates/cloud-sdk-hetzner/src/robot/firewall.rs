//! Source-locked Robot firewall and firewall-template operations.

mod form;
mod prepare;
mod request;
mod types;
mod value;

#[cfg(feature = "serde")]
mod decode;
#[cfg(feature = "serde")]
mod decode_support;
#[cfg(feature = "serde")]
mod exchange;
#[cfg(feature = "serde")]
mod failure;
#[cfg(feature = "serde")]
mod model;
#[cfg(feature = "serde")]
mod permit;
#[cfg(feature = "serde")]
mod reconcile;

pub use prepare::{
    MAX_ROBOT_FIREWALL_ITEM_RESPONSE_BYTES, MAX_ROBOT_FIREWALL_TEMPLATE_LIST_RESPONSE_BYTES,
};
pub use request::{
    RobotFirewallDeleteRequest, RobotFirewallGetRequest, RobotFirewallReplaceIntent,
    RobotFirewallReplaceRequest, RobotFirewallRequestError, RobotFirewallTemplateCreateRequest,
    RobotFirewallTemplateDeleteRequest, RobotFirewallTemplateGetRequest,
    RobotFirewallTemplateListRequest, RobotFirewallTemplateUpdateRequest,
};
pub use types::{
    MAX_ROBOT_FIREWALL_RULE_NAME_BYTES, MAX_ROBOT_FIREWALL_RULES_PER_DIRECTION,
    MAX_ROBOT_FIREWALL_TEMPLATE_NAME_BYTES, RobotFirewallAction, RobotFirewallCidr,
    RobotFirewallIpVersion, RobotFirewallPortRange, RobotFirewallProtocol, RobotFirewallRuleError,
    RobotFirewallStatus, RobotFirewallTcpFlags, RobotFirewallTemplateId, RobotFirewallTemplateName,
};
pub use value::{RobotFirewallRule, RobotFirewallRules, RobotFirewallTemplateConfig};

#[cfg(feature = "serde")]
pub use decode::RobotFirewallDecodeError;
#[cfg(feature = "serde")]
pub use exchange::{CheckedRobotFirewall, PreparedRobotFirewall};
#[cfg(feature = "serde")]
pub use failure::RobotFirewallFailureCode;
#[cfg(feature = "serde")]
pub use model::{
    MAX_ROBOT_FIREWALL_TEMPLATE_LIST_ITEMS, RobotFirewall, RobotFirewallPort,
    RobotFirewallRuleModel, RobotFirewallRuleSet, RobotFirewallRuntimeStatus,
    RobotFirewallTemplate, RobotFirewallTemplateList, RobotFirewallTemplateSummary,
};
#[cfg(feature = "serde")]
pub use permit::{
    RobotFirewallCanonicalPlanFingerprint, RobotFirewallDestructivePermit,
    RobotFirewallMutationPermit, RobotFirewallPermitAttempt, RobotFirewallPermitRequest,
    RobotFirewallPlanConfirmation, RobotFirewallPlanFingerprintDigest, RobotFirewallPlanSubject,
    RobotFirewallSharedDestructivePermit, RobotFirewallSharedMutationPermit,
    build_robot_firewall_canonical_plan, build_robot_firewall_plan_digest,
};
#[cfg(feature = "serde")]
pub use reconcile::{
    PendingRobotFirewallTemplate, RobotFirewallTemplateMutationOutcome,
    RobotFirewallTemplateReconciliation,
};

#[cfg(all(test, feature = "serde"))]
mod tests;

#[cfg(doctest)]
mod compile_fail {
    /// Raw firewall decoders remain internal to preserve request association.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::decode_robot_firewall;
    /// ```
    fn raw_decoders_are_internal() {}
}
