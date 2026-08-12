//! Source-locked Robot failover route operations.

mod prepare;
mod request;

#[cfg(feature = "serde")]
mod decode;
#[cfg(feature = "serde")]
mod exchange;
#[cfg(feature = "serde")]
mod failure;
#[cfg(feature = "serde")]
mod model;
#[cfg(feature = "serde")]
mod permit;

pub use prepare::{MAX_ROBOT_FAILOVER_ITEM_RESPONSE_BYTES, MAX_ROBOT_FAILOVER_LIST_RESPONSE_BYTES};
pub use request::{
    RobotFailoverDeleteRouteRequest, RobotFailoverGetRequest, RobotFailoverListRequest,
    RobotFailoverRequestError, RobotFailoverRerouteRequest,
};

#[cfg(feature = "serde")]
pub use decode::{RobotFailoverDecodeError, decode_robot_failover, decode_robot_failover_list};
#[cfg(feature = "serde")]
pub use exchange::{CheckedRobotFailover, PreparedRobotFailover};
#[cfg(feature = "serde")]
pub use failure::RobotFailoverFailureCode;
#[cfg(feature = "serde")]
pub use model::{MAX_ROBOT_FAILOVER_LIST_ITEMS, RobotFailover, RobotFailoverList};
#[cfg(feature = "serde")]
pub use permit::{
    RobotFailoverCanonicalPlanFingerprint, RobotFailoverDestructivePermit,
    RobotFailoverMutationPermit, RobotFailoverPermitAttempt, RobotFailoverPermitRequest,
    RobotFailoverPlanConfirmation, RobotFailoverPlanFingerprintDigest, RobotFailoverPlanSubject,
    RobotFailoverSharedDestructivePermit, RobotFailoverSharedMutationPermit,
    build_robot_failover_canonical_plan, build_robot_failover_plan_digest,
};

#[cfg(all(test, feature = "serde"))]
mod failure_tests;
#[cfg(all(test, feature = "serde"))]
mod permit_tests;
#[cfg(all(test, feature = "serde"))]
mod tests;
