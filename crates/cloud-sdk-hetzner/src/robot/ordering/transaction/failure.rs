use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};

use super::request::*;
use crate::robot::protocol::decode_robot_failure_with;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

/// Source-locked Robot transaction provider failure code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotOrderTransactionFailureCode {
    /// No transaction or transaction list matched the read request.
    NotFound,
}

macro_rules! decode_failure {
    ($($type:ty),+ $(,)?) => {$ (
        impl $type {
            /// Decodes only the source-locked transaction read failure.
            pub fn decode_failure(
                &self,
                response: TransportResponse<'_, '_>,
                workspace: &mut ResponseDecodeWorkspace,
            ) -> Result<RobotFailure, RobotDecodeError> {
                decode_robot_failure_with(response, workspace, false, &[404], |status, code| {
                    (status == 404 && code == "NOT_FOUND")
                        .then_some(RobotOrderTransactionFailureCode::NotFound.into())
                })
            }
        }
    )+ };
}

decode_failure!(
    RobotStandardTransactionListRequest,
    RobotStandardTransactionGetRequest,
    RobotMarketTransactionListRequest,
    RobotMarketTransactionGetRequest,
    RobotAddonTransactionListRequest,
    RobotAddonTransactionGetRequest,
);

impl From<RobotOrderTransactionFailureCode> for RobotProviderErrorCode {
    fn from(code: RobotOrderTransactionFailureCode) -> Self {
        match code {
            RobotOrderTransactionFailureCode::NotFound => Self::NotFound,
        }
    }
}
