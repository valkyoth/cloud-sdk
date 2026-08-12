//! Source-locked Robot single-IP and separate-MAC operations.

mod prepare;
mod request;
mod value;

#[cfg(feature = "serde")]
mod decode;
#[cfg(feature = "serde")]
mod exchange;
#[cfg(feature = "serde")]
mod model;
#[cfg(feature = "serde")]
mod permit;

pub use request::{
    RobotIpGetRequest, RobotIpListRequest, RobotIpMacDeleteRequest, RobotIpMacGetRequest,
    RobotIpMacSetRequest, RobotIpRequestError, RobotIpTrafficUpdate, RobotIpUpdateRequest,
};
pub use value::{RobotMacAddress, RobotMacAddressError};

#[cfg(feature = "serde")]
pub use decode::{RobotIpDecodeError, decode_robot_ip, decode_robot_ip_list, decode_robot_ip_mac};
#[cfg(feature = "serde")]
pub use exchange::{CheckedRobotIp, PreparedRobotIp};
#[cfg(feature = "serde")]
pub use model::{
    MAX_ROBOT_IP_LIST_ITEMS, RobotIp, RobotIpList, RobotIpMac, RobotIpSummary, RobotIpTrafficPolicy,
};
#[cfg(feature = "serde")]
pub use permit::{
    RobotIpCanonicalPlanFingerprint, RobotIpDestructivePermit, RobotIpMutationPermit,
    RobotIpPermitAttempt, RobotIpPermitRequest, RobotIpPlanConfirmation,
    RobotIpPlanFingerprintDigest, RobotIpPlanSubject, RobotIpSharedDestructivePermit,
    RobotIpSharedMutationPermit, build_robot_ip_canonical_plan, build_robot_ip_plan_digest,
};

#[cfg(all(test, feature = "serde"))]
mod permit_tests;
#[cfg(all(test, feature = "serde"))]
mod tests;
