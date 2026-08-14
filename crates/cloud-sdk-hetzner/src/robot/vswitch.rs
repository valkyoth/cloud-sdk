//! Source-locked Robot vSwitch inventory and lifecycle operations.

mod form;
mod prepare;
mod request;
mod types;

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

pub use prepare::{MAX_ROBOT_VSWITCH_ITEM_RESPONSE_BYTES, MAX_ROBOT_VSWITCH_LIST_RESPONSE_BYTES};
pub use request::{
    RobotVSwitchAddServersRequest, RobotVSwitchCancelRequest, RobotVSwitchCreateRequest,
    RobotVSwitchGetRequest, RobotVSwitchListRequest, RobotVSwitchRemoveServersRequest,
    RobotVSwitchRequestError, RobotVSwitchUpdateIntent, RobotVSwitchUpdateRequest,
};
pub use types::{
    MAX_ROBOT_VSWITCH_NAME_BYTES, MAX_ROBOT_VSWITCH_SERVERS_PER_REQUEST, RobotVSwitchId,
    RobotVSwitchName, RobotVSwitchServerIdentifier, RobotVSwitchServers, RobotVSwitchValueError,
    RobotVlanId,
};

#[cfg(feature = "serde")]
pub use decode::RobotVSwitchDecodeError;
#[cfg(feature = "serde")]
pub use exchange::{CheckedRobotVSwitch, PreparedRobotVSwitch};
#[cfg(feature = "serde")]
pub use failure::RobotVSwitchFailureCode;
#[cfg(feature = "serde")]
pub use model::{
    MAX_ROBOT_VSWITCH_CLOUD_NETWORKS, MAX_ROBOT_VSWITCH_LIST_ITEMS,
    MAX_ROBOT_VSWITCH_MEMBER_SERVERS, MAX_ROBOT_VSWITCH_SUBNETS, RobotVSwitch,
    RobotVSwitchCloudNetwork, RobotVSwitchList, RobotVSwitchServer, RobotVSwitchServerStatus,
    RobotVSwitchSubnet, RobotVSwitchSummary,
};
#[cfg(feature = "serde")]
pub use permit::{
    RobotVSwitchCanonicalPlanFingerprint, RobotVSwitchDestructivePermit,
    RobotVSwitchMutationPermit, RobotVSwitchPermitAttempt, RobotVSwitchPermitRequest,
    RobotVSwitchPlanConfirmation, RobotVSwitchPlanFingerprintDigest, RobotVSwitchPlanSubject,
    RobotVSwitchSharedDestructivePermit, RobotVSwitchSharedMutationPermit,
    build_robot_vswitch_canonical_plan, build_robot_vswitch_plan_digest,
};

#[cfg(all(test, feature = "serde"))]
mod failure_tests;
#[cfg(all(test, feature = "serde"))]
mod permit_tests;
#[cfg(all(test, feature = "serde"))]
mod tests;

#[cfg(doctest)]
mod compile_fail {
    /// Raw vSwitch decoders remain internal to preserve request association.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::decode_robot_vswitch;
    /// ```
    fn raw_decoders_are_internal() {}
}
