use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};

use crate::robot::protocol::decode_robot_failure_with;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

use super::{RobotResetExecuteRequest, RobotResetGetRequest, RobotResetListRequest};

/// Source-locked Robot reset provider failure code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotResetFailureCode {
    /// A list contained no reset-capable servers.
    NotFound,
    /// The canonical server number does not exist.
    ServerNotFound,
    /// The server does not provide reset control.
    ResetNotAvailable,
    /// A manual reset is already active.
    ResetManualActive,
    /// Robot could not execute the selected reset.
    ResetFailed,
}

#[derive(Clone, Copy)]
enum Operation {
    List,
    Get,
    Execute,
}

impl RobotResetListRequest {
    /// Decodes only failures source-locked for reset listing.
    pub fn decode_failure(
        &self,
        response: TransportResponse<'_, '_>,
        workspace: &mut ResponseDecodeWorkspace,
    ) -> Result<RobotFailure, RobotDecodeError> {
        decode(Operation::List, response, workspace)
    }
}

impl RobotResetGetRequest {
    /// Decodes only failures source-locked for reset discovery.
    pub fn decode_failure(
        &self,
        response: TransportResponse<'_, '_>,
        workspace: &mut ResponseDecodeWorkspace,
    ) -> Result<RobotFailure, RobotDecodeError> {
        decode(Operation::Get, response, workspace)
    }
}

impl RobotResetExecuteRequest<'_> {
    /// Decodes only failures source-locked for reset execution.
    pub fn decode_failure(
        &self,
        response: TransportResponse<'_, '_>,
        workspace: &mut ResponseDecodeWorkspace,
    ) -> Result<RobotFailure, RobotDecodeError> {
        decode(Operation::Execute, response, workspace)
    }
}

fn decode(
    operation: Operation,
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    let (allow_invalid_input, statuses): (bool, &[u16]) = match operation {
        Operation::List | Operation::Get => (false, &[404]),
        Operation::Execute => (true, &[404, 409, 500]),
    };
    decode_robot_failure_with(
        response,
        workspace,
        allow_invalid_input,
        statuses,
        |status, code| classify(operation, status, code).map(Into::into),
    )
}

fn classify(operation: Operation, status: u16, code: &str) -> Option<RobotResetFailureCode> {
    match (operation, status, code) {
        (Operation::List, 404, "NOT_FOUND") => Some(RobotResetFailureCode::NotFound),
        (Operation::Get | Operation::Execute, 404, "SERVER_NOT_FOUND") => {
            Some(RobotResetFailureCode::ServerNotFound)
        }
        (Operation::Get | Operation::Execute, 404, "RESET_NOT_AVAILABLE") => {
            Some(RobotResetFailureCode::ResetNotAvailable)
        }
        (Operation::Execute, 409, "RESET_MANUAL_ACTIVE") => {
            Some(RobotResetFailureCode::ResetManualActive)
        }
        (Operation::Execute, 500, "RESET_FAILED") => Some(RobotResetFailureCode::ResetFailed),
        _ => None,
    }
}

impl From<RobotResetFailureCode> for RobotProviderErrorCode {
    fn from(code: RobotResetFailureCode) -> Self {
        match code {
            RobotResetFailureCode::NotFound => Self::NotFound,
            RobotResetFailureCode::ServerNotFound => Self::ServerNotFound,
            RobotResetFailureCode::ResetNotAvailable => Self::ResetNotAvailable,
            RobotResetFailureCode::ResetManualActive => Self::ResetManualActive,
            RobotResetFailureCode::ResetFailed => Self::ResetFailed,
        }
    }
}
