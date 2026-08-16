//! Billable Robot ordering mutations and request-bound cost authority.

mod authorization;
mod cost;
mod exchange;
mod failure;
mod permit;
mod prepare;
mod reconcile;
mod request;

#[cfg(test)]
mod tests;

pub use authorization::RobotOrderAuthorizationEvidence;
pub use cost::{RobotOrderAccount, RobotOrderCostError};
pub use exchange::{CheckedRobotOrderMutation, PreparedRobotOrderMutation};
pub use failure::RobotOrderMutationFailureCode;
pub use permit::{
    RobotOrderCanonicalPlanFingerprint, RobotOrderCostPermit, RobotOrderPermitAttempt,
    RobotOrderPermitRequest, RobotOrderPlanConfirmation, RobotOrderPlanFingerprintDigest,
    RobotOrderPlanSubject, build_robot_order_canonical_plan, build_robot_order_plan_digest,
};
pub use prepare::MAX_ROBOT_ORDER_MUTATION_RESPONSE_BYTES;
pub use reconcile::{RobotOrderNotApplied, RobotOrderReconciliationError};
pub use request::{
    MAX_ROBOT_RIPE_REASON_BYTES, ROBOT_ORDER_MUTATION_QUOTA, RobotAddonOrderCreateError,
    RobotAddonOrderCreateRequest, RobotAddonOrderParameters, RobotMarketOrderCreateRequest,
    RobotOrderMutationDecodeError, RobotOrderMutationQuota, RobotOrderMutationRequestError,
    RobotRipeReason, RobotStandardOrderCreateRequest,
};
