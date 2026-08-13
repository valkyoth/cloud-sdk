use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};

use crate::robot::protocol::decode_robot_failure_with;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

use super::{
    RobotSubnetGetRequest, RobotSubnetListRequest, RobotSubnetMacDeleteRequest,
    RobotSubnetMacGetRequest, RobotSubnetMacSetRequest, RobotSubnetUpdateRequest,
};

/// Source-locked Robot subnet provider failure code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotSubnetFailureCode {
    /// A subnet list had no matching entries.
    NotFound,
    /// The requested subnet was not found.
    SubnetNotFound,
    /// Separate MAC assignment is unavailable for the subnet.
    MacNotAvailable,
    /// Updating traffic-warning options failed internally.
    TrafficWarningUpdateFailed,
    /// Applying or restoring the subnet MAC failed internally.
    MacFailed,
}

impl RobotSubnetFailureCode {
    /// Narrows one shared Robot provider code to the subnet domain.
    #[must_use]
    pub const fn from_provider(code: RobotProviderErrorCode) -> Option<Self> {
        match code {
            RobotProviderErrorCode::NotFound => Some(Self::NotFound),
            RobotProviderErrorCode::SubnetNotFound => Some(Self::SubnetNotFound),
            RobotProviderErrorCode::MacNotAvailable => Some(Self::MacNotAvailable),
            RobotProviderErrorCode::TrafficWarningUpdateFailed => {
                Some(Self::TrafficWarningUpdateFailed)
            }
            RobotProviderErrorCode::MacFailed => Some(Self::MacFailed),
            RobotProviderErrorCode::ServerNotFound
            | RobotProviderErrorCode::ResetNotAvailable
            | RobotProviderErrorCode::ResetManualActive
            | RobotProviderErrorCode::ResetFailed
            | RobotProviderErrorCode::FailoverNewServerNotFound
            | RobotProviderErrorCode::FailoverAlreadyRouted
            | RobotProviderErrorCode::FailoverLocked
            | RobotProviderErrorCode::FailoverFailed
            | RobotProviderErrorCode::FailoverNotComplete
            | RobotProviderErrorCode::WolNotAvailable
            | RobotProviderErrorCode::WolFailed => None,
        }
    }
}

#[derive(Clone, Copy)]
enum Operation {
    List,
    Get,
    Update,
    MacGet,
    MacSet,
    MacDelete,
}

macro_rules! decode_failure {
    ($type:ty, $operation:ident) => {
        impl $type {
            /// Decodes only failures source-locked for this exact operation.
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

decode_failure!(RobotSubnetListRequest, List);
decode_failure!(RobotSubnetGetRequest, Get);
decode_failure!(RobotSubnetUpdateRequest, Update);
decode_failure!(RobotSubnetMacGetRequest, MacGet);
decode_failure!(RobotSubnetMacSetRequest, MacSet);
decode_failure!(RobotSubnetMacDeleteRequest, MacDelete);

fn decode(
    operation: Operation,
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    let (allow_invalid_input, provider_statuses): (bool, &[u16]) = match operation {
        Operation::Update => (true, &[404, 500]),
        Operation::MacSet | Operation::MacDelete => (false, &[404, 500]),
        Operation::List | Operation::Get | Operation::MacGet => (false, &[404]),
    };
    decode_robot_failure_with(
        response,
        workspace,
        allow_invalid_input,
        provider_statuses,
        |status, code| classify(operation, status, code).map(Into::into),
    )
}

fn classify(operation: Operation, status: u16, code: &str) -> Option<RobotSubnetFailureCode> {
    match (operation, status, code) {
        (Operation::List, 404, "NOT_FOUND") => Some(RobotSubnetFailureCode::NotFound),
        (Operation::Get | Operation::Update, 404, "SUBNET_NOT_FOUND") => {
            Some(RobotSubnetFailureCode::SubnetNotFound)
        }
        (Operation::MacGet | Operation::MacSet | Operation::MacDelete, 404, "SUBNET_NOT_FOUND") => {
            Some(RobotSubnetFailureCode::SubnetNotFound)
        }
        (
            Operation::MacGet | Operation::MacSet | Operation::MacDelete,
            404,
            "MAC_NOT_AVAILABLE",
        ) => Some(RobotSubnetFailureCode::MacNotAvailable),
        (Operation::Update, 500, "TRAFFIC_WARNING_UPDATE_FAILED") => {
            Some(RobotSubnetFailureCode::TrafficWarningUpdateFailed)
        }
        (Operation::MacSet | Operation::MacDelete, 500, "MAC_FAILED") => {
            Some(RobotSubnetFailureCode::MacFailed)
        }
        _ => None,
    }
}

impl From<RobotSubnetFailureCode> for RobotProviderErrorCode {
    fn from(code: RobotSubnetFailureCode) -> Self {
        match code {
            RobotSubnetFailureCode::NotFound => Self::NotFound,
            RobotSubnetFailureCode::SubnetNotFound => Self::SubnetNotFound,
            RobotSubnetFailureCode::MacNotAvailable => Self::MacNotAvailable,
            RobotSubnetFailureCode::TrafficWarningUpdateFailed => Self::TrafficWarningUpdateFailed,
            RobotSubnetFailureCode::MacFailed => Self::MacFailed,
        }
    }
}
