use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};

use crate::robot::protocol::decode_robot_failure_with;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

use super::{
    RobotSshKeyCreateRequest, RobotSshKeyDeleteRequest, RobotSshKeyGetRequest,
    RobotSshKeyListRequest, RobotSshKeyUpdateRequest,
};

/// Source-locked Robot SSH-key provider failure code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotSshKeyFailureCode {
    /// No key or key list resource was found.
    NotFound,
    /// The supplied public key already exists.
    AlreadyExists,
    /// Robot could not create the key.
    CreateFailed,
    /// Robot could not update the key name.
    UpdateFailed,
    /// Robot could not delete the key.
    DeleteFailed,
}

#[derive(Clone, Copy)]
enum Operation {
    List,
    Create,
    Get,
    Update,
    Delete,
}

macro_rules! decode_failure {
    ($type:ty, $operation:ident) => {
        impl $type {
            /// Decodes only failures source-locked for this SSH-key operation.
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

decode_failure!(RobotSshKeyListRequest, List);
decode_failure!(RobotSshKeyCreateRequest<'_>, Create);
decode_failure!(RobotSshKeyGetRequest, Get);
decode_failure!(RobotSshKeyUpdateRequest, Update);
decode_failure!(RobotSshKeyDeleteRequest, Delete);

fn decode(
    operation: Operation,
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    let (allow_invalid_input, statuses): (bool, &[u16]) = match operation {
        Operation::List | Operation::Get => (false, &[404]),
        Operation::Create => (true, &[409, 500]),
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

fn classify(operation: Operation, status: u16, code: &str) -> Option<RobotSshKeyFailureCode> {
    match (operation, status, code) {
        (
            Operation::List | Operation::Get | Operation::Update | Operation::Delete,
            404,
            "NOT_FOUND",
        ) => Some(RobotSshKeyFailureCode::NotFound),
        (Operation::Create, 409, "KEY_ALREADY_EXISTS") => {
            Some(RobotSshKeyFailureCode::AlreadyExists)
        }
        (Operation::Create, 500, "KEY_CREATE_FAILED") => Some(RobotSshKeyFailureCode::CreateFailed),
        (Operation::Update, 500, "KEY_UPDATE_FAILED") => Some(RobotSshKeyFailureCode::UpdateFailed),
        (Operation::Delete, 500, "KEY_DELETE_FAILED") => Some(RobotSshKeyFailureCode::DeleteFailed),
        _ => None,
    }
}

impl From<RobotSshKeyFailureCode> for RobotProviderErrorCode {
    fn from(code: RobotSshKeyFailureCode) -> Self {
        match code {
            RobotSshKeyFailureCode::NotFound => Self::NotFound,
            RobotSshKeyFailureCode::AlreadyExists => Self::SshKeyAlreadyExists,
            RobotSshKeyFailureCode::CreateFailed => Self::SshKeyCreateFailed,
            RobotSshKeyFailureCode::UpdateFailed => Self::SshKeyUpdateFailed,
            RobotSshKeyFailureCode::DeleteFailed => Self::SshKeyDeleteFailed,
        }
    }
}
