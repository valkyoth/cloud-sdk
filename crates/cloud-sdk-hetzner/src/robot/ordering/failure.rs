use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};

use crate::robot::protocol::decode_robot_failure_with;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

use super::request::*;

/// Source-locked Robot ordering-catalog provider failure code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotOrderFailureCode {
    /// No catalog product or product list matched the request.
    NotFound,
    /// The server selected for addon discovery does not exist.
    ServerNotFound,
}

#[derive(Clone, Copy)]
enum Operation {
    StandardList,
    StandardGet,
    MarketList,
    MarketGet,
    AddonList,
    Currency,
}

macro_rules! decode_failure {
    ($type:ty, $operation:ident) => {
        impl $type {
            /// Decodes only failures source-locked for this catalog operation.
            pub fn decode_failure(
                &self,
                response: TransportResponse<'_, '_>,
                workspace: &mut ResponseDecodeWorkspace,
            ) -> Result<RobotFailure, RobotDecodeError> {
                decode(Operation::$operation, response, workspace)
            }
        }
    };
}

decode_failure!(RobotStandardProductListRequest, StandardList);
decode_failure!(RobotStandardProductGetRequest, StandardGet);
decode_failure!(RobotMarketProductListRequest, MarketList);
decode_failure!(RobotMarketProductGetRequest, MarketGet);
decode_failure!(RobotAddonProductListRequest, AddonList);
decode_failure!(RobotOrderCurrencyRequest, Currency);

fn decode(
    operation: Operation,
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    decode_robot_failure_with(response, workspace, false, &[404], |status, code| {
        classify(operation, status, code).map(Into::into)
    })
}

fn classify(operation: Operation, status: u16, code: &str) -> Option<RobotOrderFailureCode> {
    match (operation, status, code) {
        (_, 404, "NOT_FOUND") => Some(RobotOrderFailureCode::NotFound),
        (Operation::AddonList, 404, "SERVER_NOT_FOUND") => {
            Some(RobotOrderFailureCode::ServerNotFound)
        }
        _ => None,
    }
}

impl From<RobotOrderFailureCode> for RobotProviderErrorCode {
    fn from(code: RobotOrderFailureCode) -> Self {
        match code {
            RobotOrderFailureCode::NotFound => Self::NotFound,
            RobotOrderFailureCode::ServerNotFound => Self::ServerNotFound,
        }
    }
}
