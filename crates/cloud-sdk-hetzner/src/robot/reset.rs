//! Source-locked Robot reset discovery and execution operations.

mod prepare;
mod request;

#[cfg(feature = "serde")]
mod decode;
#[cfg(feature = "serde")]
mod evidence;
#[cfg(feature = "serde")]
mod exchange;
#[cfg(feature = "serde")]
mod failure;
#[cfg(feature = "serde")]
mod model;
#[cfg(feature = "serde")]
mod permit;
#[cfg(feature = "serde")]
mod preflight;

pub use prepare::{
    MAX_ROBOT_RESET_ACTION_RESPONSE_BYTES, MAX_ROBOT_RESET_DETAIL_RESPONSE_BYTES,
    MAX_ROBOT_RESET_LIST_RESPONSE_BYTES,
};
#[cfg(feature = "serde")]
pub use request::RobotResetExecuteRequest;
pub use request::{
    RobotResetGetRequest, RobotResetIntent, RobotResetListRequest, RobotResetRequestError,
    RobotResetType,
};

#[cfg(feature = "serde")]
pub use decode::{
    RobotResetDecodeError, decode_robot_reset, decode_robot_reset_action, decode_robot_reset_list,
};
#[cfg(feature = "serde")]
pub use evidence::{
    AuthorizedRobotReset, MAX_ROBOT_RESET_EVIDENCE_AGE_SECONDS, RobotResetEvidenceError,
};
#[cfg(feature = "serde")]
pub use exchange::{CheckedRobotReset, PreparedRobotReset};
#[cfg(feature = "serde")]
pub use failure::RobotResetFailureCode;
#[cfg(feature = "serde")]
pub use model::{
    MAX_ROBOT_RESET_LIST_ITEMS, RobotReset, RobotResetAction, RobotResetList,
    RobotResetOperatingStatus, RobotResetSummary,
};
#[cfg(feature = "serde")]
pub use permit::{
    RobotResetCanonicalPlanFingerprint, RobotResetDestructivePermit, RobotResetPermitAttempt,
    RobotResetPermitRequest, RobotResetPlanConfirmation, RobotResetPlanFingerprintDigest,
    RobotResetPlanSubject, RobotResetSharedDestructivePermit, build_robot_reset_canonical_plan,
    build_robot_reset_plan_digest,
};
#[cfg(feature = "serde")]
pub use preflight::RobotResetPreflightError;

#[cfg(all(test, feature = "serde"))]
mod failure_tests;
#[cfg(all(test, feature = "serde"))]
mod tests;
