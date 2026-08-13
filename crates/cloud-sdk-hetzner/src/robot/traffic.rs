//! Source-locked Robot traffic queries.

mod decode;
mod exchange;
mod failure;
mod model;
mod prepare;
mod request;
mod value;

pub use decode::RobotTrafficDecodeError;
pub use exchange::{CheckedRobotTraffic, PreparedRobotTraffic};
pub use failure::RobotTrafficFailureCode;
pub use model::{
    RobotTrafficAmount, RobotTrafficData, RobotTrafficPoint, RobotTrafficReport,
    RobotTrafficResult, RobotTrafficResultTarget,
};
pub use prepare::MAX_ROBOT_TRAFFIC_RESPONSE_BYTES;
pub use request::{
    MAX_ROBOT_TRAFFIC_SINGLE_VALUE_TARGETS, MAX_ROBOT_TRAFFIC_TARGETS, ROBOT_TRAFFIC_QUOTA,
    RobotTrafficQuota, RobotTrafficRequest, RobotTrafficRequestError, RobotTrafficTarget,
};
pub use value::{RobotTrafficGranularity, RobotTrafficInterval, RobotTrafficIntervalError};

#[cfg(test)]
mod tests;

#[cfg(doctest)]
mod compile_fail {
    /// A checked traffic response remains associated with its exact request.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::{CheckedRobotTraffic, RobotTrafficRequest};
    /// fn erase<'a>(response: CheckedRobotTraffic<'_, 'a>) -> &'a RobotTrafficRequest {
    ///     response.request
    /// }
    /// ```
    fn association_is_not_exposed() {}
}
