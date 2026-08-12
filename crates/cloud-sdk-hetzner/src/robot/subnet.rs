//! Source-locked Robot subnet and subnet-MAC operations.

mod prepare;
mod request;

#[cfg(feature = "serde")]
mod decode;
#[cfg(feature = "serde")]
mod exchange;
#[cfg(feature = "serde")]
mod model;
#[cfg(feature = "serde")]
mod permit;

pub use request::{
    RobotSubnetGetRequest, RobotSubnetListRequest, RobotSubnetMacDeleteRequest,
    RobotSubnetMacGetRequest, RobotSubnetMacSetRequest, RobotSubnetRequestError,
    RobotSubnetTrafficUpdate, RobotSubnetUpdateRequest,
};

#[cfg(feature = "serde")]
pub use decode::{
    RobotSubnetDecodeError, decode_robot_subnet, decode_robot_subnet_list, decode_robot_subnet_mac,
};
#[cfg(feature = "serde")]
pub use exchange::{CheckedRobotSubnet, PreparedRobotSubnet};
#[cfg(feature = "serde")]
pub use model::{
    MAX_ROBOT_SUBNET_LIST_ITEMS, MAX_ROBOT_SUBNET_MAC_OPTIONS, RobotSubnet, RobotSubnetList,
    RobotSubnetMac, RobotSubnetMacOption, RobotSubnetTrafficPolicy,
};
#[cfg(feature = "serde")]
pub use permit::{
    RobotSubnetCanonicalPlanFingerprint, RobotSubnetDestructivePermit, RobotSubnetMutationPermit,
    RobotSubnetPermitAttempt, RobotSubnetPermitRequest, RobotSubnetPlanConfirmation,
    RobotSubnetPlanFingerprintDigest, RobotSubnetPlanSubject, RobotSubnetSharedDestructivePermit,
    RobotSubnetSharedMutationPermit, build_robot_subnet_canonical_plan,
    build_robot_subnet_plan_digest,
};

#[cfg(all(test, feature = "serde"))]
mod permit_tests;
#[cfg(all(test, feature = "serde"))]
mod tests;
