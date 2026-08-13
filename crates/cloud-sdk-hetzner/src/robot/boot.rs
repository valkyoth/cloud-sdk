//! Source-locked Robot boot configuration operations.

#[cfg(feature = "serde")]
mod decode;
#[cfg(feature = "serde")]
mod exchange;
#[cfg(feature = "serde")]
mod failure;
#[cfg(feature = "serde")]
mod model;
mod prepare;
mod request;
mod value;

#[cfg(feature = "serde")]
pub use decode::RobotBootDecodeError;
#[cfg(feature = "serde")]
pub use exchange::{CheckedRobotBoot, PreparedRobotBoot};
#[cfg(feature = "serde")]
pub use failure::RobotBootFailureCode;
#[cfg(feature = "serde")]
pub use model::{RobotBoot, RobotBootChoice, RobotBootEntry, RobotBootFamily, RobotBootSecret};
pub use prepare::MAX_ROBOT_BOOT_RESPONSE_BYTES;
pub use request::{
    ROBOT_BOOT_QUOTA, RobotBootGetRequest, RobotBootQuota, RobotBootRequestError,
    RobotLinuxActivateRequest, RobotLinuxDeactivateRequest, RobotLinuxGetRequest,
    RobotLinuxLastRequest, RobotRescueActivateRequest, RobotRescueDeactivateRequest,
    RobotRescueGetRequest, RobotRescueLastRequest, RobotVncActivateRequest,
    RobotVncDeactivateRequest, RobotVncGetRequest, RobotWindowsActivateRequest,
    RobotWindowsDeactivateRequest, RobotWindowsGetRequest,
};
pub use value::{
    MAX_ROBOT_BOOT_AUTHORIZED_KEYS, MAX_ROBOT_BOOT_KEY_BYTES, MAX_ROBOT_BOOT_VALUE_BYTES,
    RobotBootKey, RobotBootValue, RobotKeyboardLayout,
};

#[cfg(all(test, feature = "serde"))]
mod state_tests;
#[cfg(all(test, feature = "serde"))]
mod tests;
