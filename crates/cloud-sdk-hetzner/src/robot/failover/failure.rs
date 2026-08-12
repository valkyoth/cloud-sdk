use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};

use crate::robot::protocol::decode_robot_failure_with;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

use super::{
    RobotFailoverDeleteRouteRequest, RobotFailoverGetRequest, RobotFailoverListRequest,
    RobotFailoverRerouteRequest,
};

/// Source-locked Robot failover provider failure code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotFailoverFailureCode {
    /// The failover route does not exist.
    NotFound,
    /// The requested destination server could not be found.
    NewServerNotFound,
    /// The failover route already points to the requested destination.
    AlreadyRouted,
    /// Another failover transition currently holds the provider lock.
    Locked,
    /// Robot could not change the failover route.
    Failed,
    /// Robot could not confirm completion of the route transition.
    NotComplete,
}

#[derive(Clone, Copy)]
enum Operation {
    List,
    Get,
    Reroute,
    Delete,
}

macro_rules! decode_failure {
    ($type:ty, $operation:ident) => {
        impl $type {
            /// Decodes only failures source-locked for this failover operation.
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

decode_failure!(RobotFailoverListRequest, List);
decode_failure!(RobotFailoverGetRequest, Get);
decode_failure!(RobotFailoverRerouteRequest, Reroute);
decode_failure!(RobotFailoverDeleteRouteRequest, Delete);

fn decode(
    operation: Operation,
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    let (allow_invalid_input, statuses): (bool, &[u16]) = match operation {
        Operation::List | Operation::Get => (false, &[404]),
        Operation::Reroute => (true, &[404, 409, 500]),
        Operation::Delete => (false, &[404, 409, 500]),
    };
    decode_robot_failure_with(
        response,
        workspace,
        allow_invalid_input,
        statuses,
        |status, code| classify(operation, status, code).map(Into::into),
    )
}

fn classify(operation: Operation, status: u16, code: &str) -> Option<RobotFailoverFailureCode> {
    match (operation, status, code) {
        (
            Operation::List | Operation::Get | Operation::Reroute | Operation::Delete,
            404,
            "NOT_FOUND",
        ) => Some(RobotFailoverFailureCode::NotFound),
        (Operation::Reroute, 404, "FAILOVER_NEW_SERVER_NOT_FOUND") => {
            Some(RobotFailoverFailureCode::NewServerNotFound)
        }
        (Operation::Reroute, 409, "FAILOVER_ALREADY_ROUTED") => {
            Some(RobotFailoverFailureCode::AlreadyRouted)
        }
        (Operation::Reroute | Operation::Delete, 409, "FAILOVER_LOCKED") => {
            Some(RobotFailoverFailureCode::Locked)
        }
        (Operation::Reroute | Operation::Delete, 500, "FAILOVER_FAILED") => {
            Some(RobotFailoverFailureCode::Failed)
        }
        (Operation::Reroute | Operation::Delete, 500, "FAILOVER_NOT_COMPLETE") => {
            Some(RobotFailoverFailureCode::NotComplete)
        }
        _ => None,
    }
}

impl From<RobotFailoverFailureCode> for RobotProviderErrorCode {
    fn from(code: RobotFailoverFailureCode) -> Self {
        match code {
            RobotFailoverFailureCode::NotFound => Self::NotFound,
            RobotFailoverFailureCode::NewServerNotFound => Self::FailoverNewServerNotFound,
            RobotFailoverFailureCode::AlreadyRouted => Self::FailoverAlreadyRouted,
            RobotFailoverFailureCode::Locked => Self::FailoverLocked,
            RobotFailoverFailureCode::Failed => Self::FailoverFailed,
            RobotFailoverFailureCode::NotComplete => Self::FailoverNotComplete,
        }
    }
}
