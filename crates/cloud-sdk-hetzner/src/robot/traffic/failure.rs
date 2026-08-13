use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};

use super::RobotTrafficRequest;
use crate::robot::protocol::decode_robot_failure_with;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

/// Source-locked Robot traffic provider failure code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotTrafficFailureCode {
    /// No traffic data matched the supplied targets and interval.
    NotFound,
    /// Robot failed while serving a valid traffic query.
    InternalError,
}

impl RobotTrafficRequest {
    /// Decodes only failures source-locked for `POST /traffic`.
    pub fn decode_failure(
        &self,
        response: TransportResponse<'_, '_>,
        workspace: &mut ResponseDecodeWorkspace,
    ) -> Result<RobotFailure, RobotDecodeError> {
        decode_robot_failure_with(
            response,
            workspace,
            true,
            &[404, 500],
            |status, code| match (status, code) {
                (404, "NOT_FOUND") => Some(RobotTrafficFailureCode::NotFound.into()),
                (500, "INTERNAL_ERROR") => Some(RobotTrafficFailureCode::InternalError.into()),
                _ => None,
            },
        )
    }
}

impl From<RobotTrafficFailureCode> for RobotProviderErrorCode {
    fn from(value: RobotTrafficFailureCode) -> Self {
        match value {
            RobotTrafficFailureCode::NotFound => Self::NotFound,
            RobotTrafficFailureCode::InternalError => Self::TrafficInternalError,
        }
    }
}
