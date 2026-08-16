use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};

use super::request::{
    RobotAddonOrderCreateRequest, RobotMarketOrderCreateRequest, RobotStandardOrderCreateRequest,
};
use crate::robot::protocol::decode_robot_failure_with;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

/// Source-locked provider failures for billable Robot orders.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotOrderMutationFailureCode {
    /// Form parameters were invalid.
    InvalidInput,
    /// Addon ordering conflicted with current provider state.
    Conflict,
    /// Robot requires the web frontend or another prerequisite.
    PreconditionFailed,
    /// Robot failed while creating the transaction.
    InternalError,
}

#[derive(Clone, Copy)]
enum Family {
    Server,
    Addon,
}

macro_rules! failure_decoder {
    ($type:ty, $family:ident) => {
        impl $type {
            /// Decodes only provider failures documented for this order family.
            pub fn decode_failure(
                &self,
                response: TransportResponse<'_, '_>,
                workspace: &mut ResponseDecodeWorkspace,
            ) -> Result<RobotFailure, RobotDecodeError> {
                decode(Family::$family, response, workspace)
            }
        }
    };
}

failure_decoder!(RobotStandardOrderCreateRequest<'_>, Server);
failure_decoder!(RobotMarketOrderCreateRequest<'_>, Server);
failure_decoder!(RobotAddonOrderCreateRequest<'_, '_>, Addon);

fn decode(
    family: Family,
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    let statuses: &[u16] = match family {
        Family::Server => &[400, 412, 500],
        Family::Addon => &[400, 409, 412, 500],
    };
    decode_robot_failure_with(response, workspace, false, statuses, |status, code| {
        match (family, status, code) {
            (_, 400, "INVALID_INPUT") => Some(RobotOrderMutationFailureCode::InvalidInput),
            (Family::Addon, 409, "CONFLICT") => Some(RobotOrderMutationFailureCode::Conflict),
            (_, 412, "PRECONDITION_FAILED") => {
                Some(RobotOrderMutationFailureCode::PreconditionFailed)
            }
            (_, 500, "INTERNAL_ERROR") => Some(RobotOrderMutationFailureCode::InternalError),
            _ => None,
        }
        .map(Into::into)
    })
}

impl From<RobotOrderMutationFailureCode> for RobotProviderErrorCode {
    fn from(code: RobotOrderMutationFailureCode) -> Self {
        match code {
            RobotOrderMutationFailureCode::InvalidInput => Self::OrderInvalidInput,
            RobotOrderMutationFailureCode::Conflict => Self::OrderConflict,
            RobotOrderMutationFailureCode::PreconditionFailed => Self::OrderPreconditionFailed,
            RobotOrderMutationFailureCode::InternalError => Self::OrderInternalError,
        }
    }
}
