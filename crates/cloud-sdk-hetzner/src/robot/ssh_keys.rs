//! Source-locked Robot SSH-key operations.

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

pub use prepare::{MAX_ROBOT_SSH_KEY_ITEM_RESPONSE_BYTES, MAX_ROBOT_SSH_KEY_LIST_RESPONSE_BYTES};
pub use request::{
    RobotSshKeyCreateRequest, RobotSshKeyDeleteRequest, RobotSshKeyGetRequest,
    RobotSshKeyListRequest, RobotSshKeyRequestError, RobotSshKeyUpdateRequest,
};
pub use value::{
    MAX_ROBOT_SSH_KEY_DATA_BYTES, MAX_ROBOT_SSH_KEY_NAME_BYTES, RobotSshKeyData,
    RobotSshKeyFingerprint, RobotSshKeyName, RobotSshKeyValueError,
};

#[cfg(feature = "serde")]
pub use decode::RobotSshKeyDecodeError;
#[cfg(feature = "serde")]
pub use exchange::{CheckedRobotSshKey, PreparedRobotSshKey};
#[cfg(feature = "serde")]
pub use failure::RobotSshKeyFailureCode;
#[cfg(feature = "serde")]
pub use model::{
    MAX_ROBOT_SSH_KEY_LIST_ITEMS, RobotSshKey, RobotSshKeyAlgorithm, RobotSshKeyCreatedAt,
    RobotSshKeyList,
};
#[cfg(feature = "serde")]
pub use permit::{
    RobotSshKeyCanonicalPlanFingerprint, RobotSshKeyDestructivePermit, RobotSshKeyMutationPermit,
    RobotSshKeyPermitAttempt, RobotSshKeyPermitRequest, RobotSshKeyPlanConfirmation,
    RobotSshKeyPlanFingerprintDigest, RobotSshKeyPlanSubject, RobotSshKeySharedDestructivePermit,
    RobotSshKeySharedMutationPermit, build_robot_ssh_key_canonical_plan,
    build_robot_ssh_key_plan_digest,
};

#[cfg(all(test, feature = "serde"))]
mod failure_tests;
#[cfg(all(test, feature = "serde"))]
mod permit_tests;
#[cfg(all(test, feature = "serde"))]
mod tests;

#[cfg(doctest)]
mod compile_fail {
    /// Raw decoders stay internal so request provenance cannot be bypassed.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::decode_robot_ssh_key;
    /// ```
    fn raw_decoders_are_internal() {}
}
