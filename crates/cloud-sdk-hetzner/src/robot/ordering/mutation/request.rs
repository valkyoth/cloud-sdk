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
    pub(super) cost: PlanCost,
}

impl<'a, 'request> RobotAddonOrderCreateRequest<'a, 'request> {
    /// Binds one current addon plan to an explicit first-invoice ceiling.
    pub fn new(
        plan: &'a RobotAddonOrderPlan<'a, 'request>,
        ceiling_units_at_scale_4: u128,
    ) -> Result<Self, super::RobotOrderCostError> {
        Ok(Self {
            plan,
            cost: addon_cost(plan, ceiling_units_at_scale_4)?,
        })
    }

    pub(super) const fn cost(&self) -> PlanCost {
        self.cost
    }
}

impl core::fmt::Debug for RobotAddonOrderCreateRequest<'_, '_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotAddonOrderCreateRequest([redacted])")
    }
}
