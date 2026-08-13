//! Source-locked Robot reverse-DNS operations.

mod prepare;
mod request;
mod value;

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

pub use prepare::{MAX_ROBOT_RDNS_ITEM_RESPONSE_BYTES, MAX_ROBOT_RDNS_LIST_RESPONSE_BYTES};
pub use request::{
    RobotRdnsDeleteRequest, RobotRdnsGetRequest, RobotRdnsListRequest, RobotRdnsRequestError,
    RobotRdnsSetRequest, RobotRdnsUpdateRequest,
};
pub use value::{MAX_ROBOT_RDNS_NAME_BYTES, RobotRdnsName, RobotRdnsNameError};

#[cfg(feature = "serde")]
pub use decode::{RobotRdnsDecodeError, decode_robot_rdns, decode_robot_rdns_list};
#[cfg(feature = "serde")]
pub use exchange::{CheckedRobotRdns, PreparedRobotRdns};
#[cfg(feature = "serde")]
pub use failure::RobotRdnsFailureCode;
#[cfg(feature = "serde")]
pub use model::{MAX_ROBOT_RDNS_LIST_ITEMS, RobotRdns, RobotRdnsList};
#[cfg(feature = "serde")]
pub use permit::{
    RobotRdnsCanonicalPlanFingerprint, RobotRdnsDestructivePermit, RobotRdnsMutationPermit,
    RobotRdnsPermitAttempt, RobotRdnsPermitRequest, RobotRdnsPlanConfirmation,
    RobotRdnsPlanFingerprintDigest, RobotRdnsPlanSubject, RobotRdnsSharedDestructivePermit,
    RobotRdnsSharedMutationPermit, build_robot_rdns_canonical_plan, build_robot_rdns_plan_digest,
};

#[cfg(all(test, feature = "serde"))]
mod failure_tests;
#[cfg(all(test, feature = "serde"))]
mod permit_tests;
#[cfg(all(test, feature = "serde"))]
mod tests;
