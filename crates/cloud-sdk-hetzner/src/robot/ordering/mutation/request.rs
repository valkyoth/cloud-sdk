use core::net::Ipv4Addr;

use super::super::{RobotAddonOrderPlan, RobotMarketOrderPlan, RobotStandardOrderPlan};
use super::cost::{addon_cost, market_cost, standard_cost};
use cloud_sdk::operation::PlanCost;

/// Source-locked Robot order-mutation request quota.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RobotOrderMutationQuota {
    /// Maximum requests in one provider window.
    pub requests: u32,
    /// Provider window in seconds.
    pub interval_seconds: u32,
}

/// All three ordering mutations share Robot's 20-request daily quota.
pub const ROBOT_ORDER_MUTATION_QUOTA: RobotOrderMutationQuota = RobotOrderMutationQuota {
    requests: 20,
    interval_seconds: 86_400,
};

/// Failure while preparing a billable Robot order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotOrderMutationRequestError {
    /// Catalog-derived cost validation failed.
    Cost(super::RobotOrderCostError),
    /// Caller-owned target storage was too small or encoding failed.
    Target,
    /// Form validation or atomic encoding failed.
    Form(crate::robot::RobotFormError),
    /// Temporary bounded form-field storage could not be allocated.
    Allocation,
    /// The constructed target was invalid.
    InvalidTarget(cloud_sdk::transport::RequestTargetError),
    /// Source-locked headers were invalid.
    InvalidHeaders(cloud_sdk::transport::HeaderError),
    /// The official endpoint policy was invalid.
    InvalidEndpoint(crate::endpoint::OfficialEndpointError),
    /// The operation identifier was invalid.
    InvalidOperationId(cloud_sdk::operation::OperationIdError),
    /// Operation metadata was inconsistent.
    InvalidMetadata(cloud_sdk::operation::OperationMetadataError),
    /// Success-response policy was inconsistent.
    InvalidResponsePolicy(cloud_sdk::operation::ResponsePolicyValidationError),
    /// Raw response policy was inconsistent.
    InvalidRawPolicy(cloud_sdk::transport::RawResponsePolicyError),
    /// Cross-policy validation failed.
    InvalidPreparedPolicy(cloud_sdk::operation::PreparedRequestPolicyError),
}

impl_static_error!(RobotOrderMutationRequestError,
    Self::Cost(_) => "Robot order cost validation failed",
    Self::Target => "Robot order target preparation failed",
    Self::Form(_) => "Robot order form preparation failed",
    Self::Allocation => "Robot order form allocation failed",
    Self::InvalidTarget(_) => "Robot order target is invalid",
    Self::InvalidHeaders(_) => "Robot order headers are invalid",
    Self::InvalidEndpoint(_) => "official Robot endpoint is invalid",
    Self::InvalidOperationId(_) => "Robot order operation identifier is invalid",
    Self::InvalidMetadata(_) => "Robot order metadata is invalid",
    Self::InvalidResponsePolicy(_) => "Robot order response policy is invalid",
    Self::InvalidRawPolicy(_) => "Robot order raw response policy is invalid",
    Self::InvalidPreparedPolicy(_) => "Robot order prepared policy is invalid",
);

/// Failure while decoding or binding a successful billable order response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotOrderMutationDecodeError {
    /// Transaction response decoding failed.
    Transaction(super::super::RobotOrderTransactionDecodeError),
    /// Returned transaction does not match the exact confirmed order intent.
    ResponseIntentMismatch,
}

impl_static_error!(RobotOrderMutationDecodeError,
    Self::Transaction(_) => "Robot order transaction response is invalid",
    Self::ResponseIntentMismatch => "Robot order response does not match the confirmed intent",
);

/// Maximum bytes accepted for one RIPE justification.
pub const MAX_ROBOT_RIPE_REASON_BYTES: usize = 1_024;

/// Invalid parameters for a billable per-server addon order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotAddonOrderCreateError {
    /// Catalog-derived cost validation failed.
    Cost(super::RobotOrderCostError),
    /// The RIPE justification was empty, oversized, or unsafe text.
    InvalidReason,
    /// Parameters do not match the exact catalog product type.
    ParameterMismatch,
}

impl_static_error!(RobotAddonOrderCreateError,
    Self::Cost(_) => "Robot addon order cost validation failed",
    Self::InvalidReason => "Robot addon order RIPE reason is invalid",
    Self::ParameterMismatch => "Robot addon order parameters do not match the catalog product type",
);

/// Bounded borrowed RIPE justification. Diagnostics never expose its text.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RobotRipeReason<'a>(&'a str);

impl<'a> RobotRipeReason<'a> {
    /// Validates a non-empty reason without controls or directional formatting.
    pub fn new(value: &'a str) -> Result<Self, RobotAddonOrderCreateError> {
        if value.is_empty()
            || value.len() > MAX_ROBOT_RIPE_REASON_BYTES
            || value
                .chars()
                .any(crate::display::is_unsafe_display_character)
        {
            Err(RobotAddonOrderCreateError::InvalidReason)
        } else {
            Ok(Self(value))
        }
    }

    pub(super) const fn as_str(self) -> &'a str {
        self.0
    }
}

impl core::fmt::Debug for RobotRipeReason<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotRipeReason([redacted])")
    }
}

/// Catalog-type-specific parameters for a per-server addon purchase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotAddonOrderParameters<'a> {
    /// Parameters for an `ip_ipv4` or `failover_subnet_ipv4` addon.
    Ip {
        /// Mandatory RIPE justification.
        reason: RobotRipeReason<'a>,
    },
    /// Parameters for a `subnet_ipv4` addon.
    Subnet {
        /// Mandatory RIPE justification.
        reason: RobotRipeReason<'a>,
        /// Optional routing target; omission uses the server's primary IPv4.
        gateway: Option<Ipv4Addr>,
    },
    /// No extra parameters for a catalog type without RIPE requirements.
    Other,
}

macro_rules! order_request {
    ($name:ident, $plan:ty, $cost:ident, $description:literal) => {
        #[doc = $description]
        pub struct $name<'a> {
            pub(super) plan: &'a $plan,
            pub(super) cost: PlanCost,
        }

        impl<'a> $name<'a> {
            /// Binds one current catalog plan to an explicit first-invoice ceiling.
            pub fn new(
                plan: &'a $plan,
                ceiling_units_at_scale_4: u128,
            ) -> Result<Self, super::RobotOrderCostError> {
                Ok(Self {
                    plan,
                    cost: $cost(plan, ceiling_units_at_scale_4)?,
                })
            }

            pub(super) const fn cost(&self) -> PlanCost {
                self.cost
            }
        }

        impl core::fmt::Debug for $name<'_> {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(concat!(stringify!($name), "([redacted])"))
            }
        }
    };
}

order_request!(
    RobotStandardOrderCreateRequest,
    RobotStandardOrderPlan<'a>,
    standard_cost,
    "Billable standard-server order derived from one current catalog plan."
);
order_request!(
    RobotMarketOrderCreateRequest,
    RobotMarketOrderPlan<'a>,
    market_cost,
    "Billable Server Auction order derived from one current catalog plan."
);

/// Billable per-server addon order derived from a request-bound addon catalog.
pub struct RobotAddonOrderCreateRequest<'a, 'request> {
    pub(super) plan: &'a RobotAddonOrderPlan<'a, 'request>,
    pub(super) parameters: RobotAddonOrderParameters<'a>,
    pub(super) cost: PlanCost,
}

impl<'a, 'request> RobotAddonOrderCreateRequest<'a, 'request> {
    /// Binds one current addon plan to an explicit first-invoice ceiling.
    pub fn new(
        plan: &'a RobotAddonOrderPlan<'a, 'request>,
        parameters: RobotAddonOrderParameters<'a>,
        ceiling_units_at_scale_4: u128,
    ) -> Result<Self, RobotAddonOrderCreateError> {
        validate_addon_parameters(plan, parameters)?;
        Ok(Self {
            plan,
            parameters,
            cost: addon_cost(plan, ceiling_units_at_scale_4)
                .map_err(RobotAddonOrderCreateError::Cost)?,
        })
    }

    pub(super) const fn cost(&self) -> PlanCost {
        self.cost
    }
}

fn validate_addon_parameters(
    plan: &RobotAddonOrderPlan<'_, '_>,
    parameters: RobotAddonOrderParameters<'_>,
) -> Result<(), RobotAddonOrderCreateError> {
    let matches = plan
        .product()
        .kind()
        .try_with_text(|kind| {
            matches!(
                (kind, parameters),
                (
                    "ip_ipv4" | "failover_subnet_ipv4",
                    RobotAddonOrderParameters::Ip { .. }
                ) | ("subnet_ipv4", RobotAddonOrderParameters::Subnet { .. })
            ) || (!matches!(kind, "ip_ipv4" | "subnet_ipv4" | "failover_subnet_ipv4")
                && matches!(parameters, RobotAddonOrderParameters::Other))
        })
        .unwrap_or(false);
    matches
        .then_some(())
        .ok_or(RobotAddonOrderCreateError::ParameterMismatch)
}

impl core::fmt::Debug for RobotAddonOrderCreateRequest<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotAddonOrderCreateRequest([redacted])")
    }
}
