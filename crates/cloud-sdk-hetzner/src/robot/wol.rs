//! Source-locked Robot Wake-on-LAN discovery and execution.

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

pub use prepare::MAX_ROBOT_WOL_RESPONSE_BYTES;
pub use request::{RobotWolGetRequest, RobotWolIntent, RobotWolRequestError};

#[cfg(feature = "serde")]
pub use decode::{RobotWolDecodeError, decode_robot_wol};
#[cfg(feature = "serde")]
pub use evidence::{AuthorizedRobotWol, MAX_ROBOT_WOL_EVIDENCE_AGE_SECONDS, RobotWolEvidenceError};
#[cfg(feature = "serde")]
pub use exchange::{CheckedRobotWol, PreparedRobotWol};
#[cfg(feature = "serde")]
pub use failure::RobotWolFailureCode;
#[cfg(feature = "serde")]
pub use model::RobotWol;
#[cfg(feature = "serde")]
pub use permit::{
    RobotWolCanonicalPlanFingerprint, RobotWolMutationPermit, RobotWolPermitAttempt,
    RobotWolPermitRequest, RobotWolPlanConfirmation, RobotWolPlanFingerprintDigest,
    RobotWolPlanSubject, RobotWolSharedMutationPermit, build_robot_wol_canonical_plan,
    build_robot_wol_plan_digest,
};
#[cfg(feature = "serde")]
pub use preflight::RobotWolPreflightError;
#[cfg(feature = "serde")]
pub use request::RobotWolSendRequest;

#[cfg(all(test, feature = "serde"))]
mod failure_tests;
#[cfg(all(test, feature = "serde"))]
mod tests;
