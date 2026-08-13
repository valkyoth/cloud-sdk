use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};

use crate::robot::protocol::decode_robot_failure_with;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

use super::{RobotWolGetRequest, RobotWolSendRequest};

/// Source-locked Robot Wake-on-LAN provider failure code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotWolFailureCode {
    /// The canonical server number does not exist.
    ServerNotFound,
    /// Robot does not advertise Wake-on-LAN for the server.
    WolNotAvailable,
    /// Robot failed to send the packet.
    WolFailed,
}

impl RobotWolGetRequest {
    /// Decodes only failures source-locked for WOL discovery.
    pub fn decode_failure(
        &self,
        response: TransportResponse<'_, '_>,
        workspace: &mut ResponseDecodeWorkspace,
    ) -> Result<RobotFailure, RobotDecodeError> {
        decode(false, response, workspace)
    }
}

impl RobotWolSendRequest<'_> {
    /// Decodes only failures source-locked for sending WOL.
    pub fn decode_failure(
        &self,
        response: TransportResponse<'_, '_>,
        workspace: &mut ResponseDecodeWorkspace,
    ) -> Result<RobotFailure, RobotDecodeError> {
        decode(true, response, workspace)
    }
}

fn decode(
    send: bool,
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    decode_robot_failure_with(response, workspace, false, &[404, 500], |status, code| {
        classify(send, status, code).map(Into::into)
    })
}

pub(super) fn classify(send: bool, status: u16, code: &str) -> Option<RobotWolFailureCode> {
    match (send, status, code) {
        (_, 404, "SERVER_NOT_FOUND") => Some(RobotWolFailureCode::ServerNotFound),
        (_, 404, "WOL_NOT_AVAILABLE") => Some(RobotWolFailureCode::WolNotAvailable),
        (true, 500, "WOL_FAILED") => Some(RobotWolFailureCode::WolFailed),
        _ => None,
    }
}

impl From<RobotWolFailureCode> for RobotProviderErrorCode {
    fn from(code: RobotWolFailureCode) -> Self {
        match code {
            RobotWolFailureCode::ServerNotFound => Self::ServerNotFound,
            RobotWolFailureCode::WolNotAvailable => Self::WolNotAvailable,
            RobotWolFailureCode::WolFailed => Self::WolFailed,
        }
    }
}
