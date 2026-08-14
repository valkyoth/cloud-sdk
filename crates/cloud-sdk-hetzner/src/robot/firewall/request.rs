use crate::robot::{RobotFirewallRuleError, RobotServerNumber};

use super::{
    RobotFirewallRules, RobotFirewallStatus, RobotFirewallTemplateConfig, RobotFirewallTemplateId,
};

/// Failure while validating or preparing a Robot firewall operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotFirewallRequestError {
    /// Caller-owned path storage was too small or encoding failed.
    Path,
    /// A firewall value or field combination was invalid.
    Rule(RobotFirewallRuleError),
    /// Robot form validation, allocation, or encoding failed.
    Form(crate::robot::RobotFormError),
    /// Temporary form construction storage could not be allocated.
    Allocation,
    /// The constructed request target was rejected.
    InvalidTarget(cloud_sdk::transport::RequestTargetError),
    /// Source-locked request headers were rejected.
    InvalidHeaders(cloud_sdk::transport::HeaderError),
    /// The official Robot endpoint policy was invalid.
    InvalidEndpoint(crate::endpoint::OfficialEndpointError),
    /// A source-locked operation identifier was invalid.
    InvalidOperationId(cloud_sdk::operation::OperationIdError),
    /// Operation safety metadata was internally inconsistent.
    InvalidMetadata(cloud_sdk::operation::OperationMetadataError),
    /// The success-response policy was internally inconsistent.
    InvalidResponsePolicy(cloud_sdk::operation::ResponsePolicyValidationError),
    /// The raw response-wire policy was internally inconsistent.
    InvalidRawPolicy(cloud_sdk::transport::RawResponsePolicyError),
    /// Cross-policy prepared-request validation failed.
    InvalidPreparedPolicy(cloud_sdk::operation::PreparedRequestPolicyError),
}

impl_static_error!(RobotFirewallRequestError,
    Self::Path => "Robot firewall path preparation failed",
    Self::Rule(_) => "Robot firewall value is invalid",
    Self::Form(_) => "Robot firewall form preparation failed",
    Self::Allocation => "Robot firewall preparation allocation failed",
    Self::InvalidTarget(_) => "Robot firewall target is invalid",
    Self::InvalidHeaders(_) => "Robot firewall headers are invalid",
    Self::InvalidEndpoint(_) => "official Robot endpoint is invalid",
    Self::InvalidOperationId(_) => "Robot firewall operation identifier is invalid",
    Self::InvalidMetadata(_) => "Robot firewall metadata is invalid",
    Self::InvalidResponsePolicy(_) => "Robot firewall response policy is invalid",
    Self::InvalidRawPolicy(_) => "Robot firewall raw response policy is invalid",
    Self::InvalidPreparedPolicy(_) => "Robot firewall prepared policy is invalid",
);

/// Gets one server firewall by canonical server number.
#[derive(Debug)]
pub struct RobotFirewallGetRequest {
    pub(super) server: RobotServerNumber,
}

impl RobotFirewallGetRequest {
    /// Creates a firewall read request.
    #[must_use]
    pub const fn new(server: RobotServerNumber) -> Self {
        Self { server }
    }
}

/// Mutually exclusive source-locked firewall replacement intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotFirewallReplaceIntent<'a> {
    /// Replaces status and ordered inline rules.
    Inline {
        /// Requested lifecycle status.
        status: RobotFirewallStatus,
        /// Optional explicit IPv6 filter state.
        filter_ipv6: Option<bool>,
        /// Whether Hetzner services remain whitelisted.
        whitelist_hos: bool,
        /// Complete replacement rule set.
        rules: RobotFirewallRules<'a>,
    },
    /// Applies an existing template without inline rules or whitelist fields.
    Template {
        /// Requested lifecycle status.
        status: RobotFirewallStatus,
        /// Optional explicit IPv6 filter state.
        filter_ipv6: Option<bool>,
        /// Existing template identity.
        template_id: RobotFirewallTemplateId,
    },
}

/// Replaces one complete server firewall configuration.
#[derive(Debug)]
pub struct RobotFirewallReplaceRequest<'a> {
    pub(super) server: RobotServerNumber,
    pub(super) intent: RobotFirewallReplaceIntent<'a>,
}

impl<'a> RobotFirewallReplaceRequest<'a> {
    /// Creates an exact complete replacement request.
    #[must_use]
    pub const fn new(server: RobotServerNumber, intent: RobotFirewallReplaceIntent<'a>) -> Self {
        Self { server, intent }
    }
}

/// Clears one server firewall configuration.
#[derive(Debug)]
pub struct RobotFirewallDeleteRequest {
    pub(super) server: RobotServerNumber,
}

impl RobotFirewallDeleteRequest {
    /// Creates a destructive clear request.
    #[must_use]
    pub const fn new(server: RobotServerNumber) -> Self {
        Self { server }
    }
}

/// Lists available firewall templates.
#[derive(Clone, Copy, Debug, Default)]
pub struct RobotFirewallTemplateListRequest;

impl RobotFirewallTemplateListRequest {
    /// Creates an account-wide template list request.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// Creates one complete firewall template.
#[derive(Clone, Copy, Debug)]
pub struct RobotFirewallTemplateCreateRequest<'a> {
    pub(super) config: RobotFirewallTemplateConfig<'a>,
}

impl<'a> RobotFirewallTemplateCreateRequest<'a> {
    /// Creates a template request from a complete configuration.
    #[must_use]
    pub const fn new(config: RobotFirewallTemplateConfig<'a>) -> Self {
        Self { config }
    }
}

macro_rules! template_id_request {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug)]
        pub struct $name {
            pub(super) template_id: RobotFirewallTemplateId,
        }

        impl $name {
            /// Creates a request for one template identity.
            #[must_use]
            pub const fn new(template_id: RobotFirewallTemplateId) -> Self {
                Self { template_id }
            }
        }
    };
}

template_id_request!(
    RobotFirewallTemplateGetRequest,
    "Gets one firewall template."
);
template_id_request!(
    RobotFirewallTemplateDeleteRequest,
    "Deletes one firewall template."
);

/// Replaces one complete firewall template.
#[derive(Clone, Copy, Debug)]
pub struct RobotFirewallTemplateUpdateRequest<'a> {
    pub(super) template_id: RobotFirewallTemplateId,
    pub(super) config: RobotFirewallTemplateConfig<'a>,
}

impl<'a> RobotFirewallTemplateUpdateRequest<'a> {
    /// Creates a complete template replacement.
    #[must_use]
    pub const fn new(
        template_id: RobotFirewallTemplateId,
        config: RobotFirewallTemplateConfig<'a>,
    ) -> Self {
        Self {
            template_id,
            config,
        }
    }
}
