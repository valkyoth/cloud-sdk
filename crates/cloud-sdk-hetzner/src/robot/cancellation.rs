//! Source-locked Robot server, IP, and subnet cancellation operations.

mod prepare;
mod request;
mod value;

#[cfg(feature = "serde")]
mod decode;
#[cfg(feature = "serde")]
mod model;

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
pub use model::{
    MAX_ROBOT_CANCELLATION_REASON_BYTES, MAX_ROBOT_CANCELLATION_REASONS, RobotIpCancellation,
    RobotServerCancellation, RobotServerCancellationReason, RobotSubnetCancellation,
};

#[cfg(all(test, feature = "serde"))]
mod tests;
