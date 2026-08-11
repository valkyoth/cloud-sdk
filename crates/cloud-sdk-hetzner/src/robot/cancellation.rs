//! Source-locked Robot server, IP, and subnet cancellation operations.

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
    MAX_ROBOT_CANCELLATION_REASON_INPUT_BYTES, RobotCancellationReason,
    RobotCancellationRequestError, RobotCancellationSchedule, RobotIpCancellationCreateRequest,
    RobotIpCancellationDeleteRequest, RobotIpCancellationGetRequest,
    RobotLocationReservationIntent, RobotServerCancellationCreateRequest,
    RobotServerCancellationDeleteRequest, RobotServerCancellationGetRequest,
    RobotSubnetCancellationCreateRequest, RobotSubnetCancellationDeleteRequest,
    RobotSubnetCancellationGetRequest,
};
pub use value::{
    RobotCancellationDate, RobotCancellationValueError, RobotIpAddress, RobotSubnetAddress,
};

#[cfg(feature = "serde")]
pub use decode::{
    RobotCancellationDecodeError, decode_robot_ip_cancellation, decode_robot_server_cancellation,
    decode_robot_subnet_cancellation,
};
#[cfg(feature = "serde")]
pub use exchange::{CheckedCancellation, PreparedCancellation};
#[cfg(feature = "serde")]
pub use model::{
    MAX_ROBOT_CANCELLATION_REASON_BYTES, MAX_ROBOT_CANCELLATION_REASONS, RobotIpCancellation,
    RobotServerCancellation, RobotServerCancellationReason, RobotSubnetCancellation,
};
#[cfg(feature = "serde")]
pub use permit::{
    CancellationCanonicalPlanFingerprint, CancellationDestructivePermit, CancellationPermitAttempt,
    CancellationPlanConfirmation, CancellationPlanFingerprintDigest, CancellationPlanSubject,
    CancellationSharedDestructivePermit, build_cancellation_canonical_plan,
    build_cancellation_plan_digest,
};

#[cfg(all(test, feature = "serde"))]
mod exchange_tests;
#[cfg(all(test, feature = "serde"))]
mod permit_tests;
#[cfg(all(test, feature = "serde"))]
mod tests;
