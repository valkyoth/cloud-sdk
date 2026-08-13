use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};

use crate::robot::protocol::decode_robot_failure_with;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

use super::{
    RobotRdnsDeleteRequest, RobotRdnsGetRequest, RobotRdnsListRequest, RobotRdnsSetRequest,
    RobotRdnsUpdateRequest,
};

/// Source-locked Robot reverse-DNS provider failure code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotRdnsFailureCode {
    /// No reverse-DNS resources matched a list request.
    NotFound,
    /// The requested address is not assigned to the account.
    IpNotFound,
    /// The requested address has no reverse-DNS entry.
    RdnsNotFound,
    /// A create request conflicts with an existing entry.
    AlreadyExists,
    /// Robot could not create the entry.
    CreateFailed,
    /// Robot could not update the entry.
    UpdateFailed,
    /// Robot could not delete the entry.
    DeleteFailed,
}

#[derive(Clone, Copy)]
enum Operation {
    List,
    Get,
    Set,
    Update,
    Delete,
}

macro_rules! decode_failure {
    ($type:ty, $operation:ident) => {
        impl $type {
            /// Decodes only failures source-locked for this reverse-DNS operation.
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

decode_failure!(RobotRdnsListRequest, List);
decode_failure!(RobotRdnsGetRequest, Get);
decode_failure!(RobotRdnsSetRequest, Set);
decode_failure!(RobotRdnsUpdateRequest, Update);
decode_failure!(RobotRdnsDeleteRequest, Delete);

fn decode(
    operation: Operation,
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    let (allow_invalid_input, statuses): (bool, &[u16]) = match operation {
        Operation::List | Operation::Get => (false, &[404]),
        Operation::Set => (true, &[404, 409, 500]),
        Operation::Update => (true, &[404, 500]),
        Operation::Delete => (false, &[404, 500]),
    };
    decode_robot_failure_with(
        response,
        workspace,
        allow_invalid_input,
        statuses,
        |status, code| classify(operation, status, code).map(Into::into),
    )
}

fn classify(operation: Operation, status: u16, code: &str) -> Option<RobotRdnsFailureCode> {
    match (operation, status, code) {
        (Operation::List, 404, "NOT_FOUND") => Some(RobotRdnsFailureCode::NotFound),
        (
            Operation::Get | Operation::Set | Operation::Update | Operation::Delete,
            404,
            "IP_NOT_FOUND",
        ) => Some(RobotRdnsFailureCode::IpNotFound),
        (Operation::Get, 404, "RDNS_NOT_FOUND") => Some(RobotRdnsFailureCode::RdnsNotFound),
        (Operation::Set, 409, "RDNS_ALREADY_EXISTS") => Some(RobotRdnsFailureCode::AlreadyExists),
        (Operation::Set | Operation::Update, 500, "RDNS_CREATE_FAILED") => {
            Some(RobotRdnsFailureCode::CreateFailed)
        }
        (Operation::Update | Operation::Delete, 500, "RDNS_UPDATE_FAILED") => {
            Some(RobotRdnsFailureCode::UpdateFailed)
        }
        (Operation::Delete, 500, "RDNS_DELETE_FAILED") => Some(RobotRdnsFailureCode::DeleteFailed),
        _ => None,
    }
}

impl From<RobotRdnsFailureCode> for RobotProviderErrorCode {
    fn from(code: RobotRdnsFailureCode) -> Self {
        match code {
            RobotRdnsFailureCode::NotFound => Self::NotFound,
            RobotRdnsFailureCode::IpNotFound => Self::IpNotFound,
            RobotRdnsFailureCode::RdnsNotFound => Self::RdnsNotFound,
            RobotRdnsFailureCode::AlreadyExists => Self::RdnsAlreadyExists,
            RobotRdnsFailureCode::CreateFailed => Self::RdnsCreateFailed,
            RobotRdnsFailureCode::UpdateFailed => Self::RdnsUpdateFailed,
            RobotRdnsFailureCode::DeleteFailed => Self::RdnsDeleteFailed,
        }
    }
}
