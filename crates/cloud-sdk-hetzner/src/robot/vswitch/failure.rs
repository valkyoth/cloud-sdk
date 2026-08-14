use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};

use crate::robot::protocol::decode_robot_failure_with;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

use super::request::*;

/// Source-locked Robot vSwitch provider failure code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotVSwitchFailureCode {
    /// The selected vSwitch does not exist.
    NotFound,
    /// A submitted server does not exist.
    ServerNotFound,
    /// vSwitch connectivity is unavailable for a submitted server.
    NotAvailable,
    /// The account vSwitch count limit was reached.
    LimitReached,
    /// Another vSwitch update is already running.
    InProcess,
    /// The requested VLAN conflicts with another vSwitch.
    VlanNotUnique,
    /// The vSwitch server membership limit was reached.
    ServerLimitReached,
    /// A submitted server's per-server vSwitch limit was reached.
    PerServerLimitReached,
    /// The vSwitch is already cancelled.
    AlreadyCancelled,
}

#[derive(Clone, Copy)]
enum Operation {
    List,
    Create,
    Get,
    Update,
    Cancel,
    AddServers,
    RemoveServers,
}

macro_rules! decode_failure {
    ($type:ty, $operation:ident) => {
        impl $type {
            /// Decodes only failures source-locked for this vSwitch operation.
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

decode_failure!(RobotVSwitchListRequest, List);
decode_failure!(RobotVSwitchCreateRequest, Create);
decode_failure!(RobotVSwitchGetRequest, Get);
decode_failure!(RobotVSwitchUpdateRequest, Update);
decode_failure!(RobotVSwitchCancelRequest, Cancel);
decode_failure!(RobotVSwitchAddServersRequest<'_>, AddServers);
decode_failure!(RobotVSwitchRemoveServersRequest<'_>, RemoveServers);

fn decode(
    operation: Operation,
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    let (allow_invalid_input, statuses): (bool, &[u16]) = match operation {
        Operation::List => (false, &[]),
        Operation::Create => (true, &[409]),
        Operation::Get => (false, &[404]),
        Operation::Update | Operation::Cancel | Operation::AddServers => (true, &[404, 409]),
        Operation::RemoveServers => (true, &[404, 409]),
    };
    decode_robot_failure_with(
        response,
        workspace,
        allow_invalid_input,
        statuses,
        |status, code| classify(operation, status, code).map(Into::into),
    )
}

fn classify(operation: Operation, status: u16, code: &str) -> Option<RobotVSwitchFailureCode> {
    match (operation, status, code) {
        (
            Operation::Get
            | Operation::Update
            | Operation::Cancel
            | Operation::AddServers
            | Operation::RemoveServers,
            404,
            "NOT_FOUND",
        ) => Some(RobotVSwitchFailureCode::NotFound),
        (Operation::AddServers | Operation::RemoveServers, 404, "SERVER_NOT_FOUND") => {
            Some(RobotVSwitchFailureCode::ServerNotFound)
        }
        (Operation::AddServers, 404, "VSWITCH_NOT_AVAILABLE") => {
            Some(RobotVSwitchFailureCode::NotAvailable)
        }
        (Operation::Create, 409, "VSWITCH_LIMIT_REACHED") => {
            Some(RobotVSwitchFailureCode::LimitReached)
        }
        (
            Operation::Update | Operation::AddServers | Operation::RemoveServers,
            409,
            "VSWITCH_IN_PROCESS",
        ) => Some(RobotVSwitchFailureCode::InProcess),
        (Operation::Update | Operation::AddServers, 409, "VSWITCH_VLAN_NOT_UNIQUE") => {
            Some(RobotVSwitchFailureCode::VlanNotUnique)
        }
        (Operation::AddServers, 409, "VSWITCH_SERVER_LIMIT_REACHED") => {
            Some(RobotVSwitchFailureCode::ServerLimitReached)
        }
        (Operation::AddServers, 409, "VSWITCH_PER_SERVER_LIMIT_REACHED") => {
            Some(RobotVSwitchFailureCode::PerServerLimitReached)
        }
        (Operation::Cancel, 409, "CONFLICT") => Some(RobotVSwitchFailureCode::AlreadyCancelled),
        _ => None,
    }
}

impl From<RobotVSwitchFailureCode> for RobotProviderErrorCode {
    fn from(code: RobotVSwitchFailureCode) -> Self {
        match code {
            RobotVSwitchFailureCode::NotFound => Self::NotFound,
            RobotVSwitchFailureCode::ServerNotFound => Self::ServerNotFound,
            RobotVSwitchFailureCode::NotAvailable => Self::VSwitchNotAvailable,
            RobotVSwitchFailureCode::LimitReached => Self::VSwitchLimitReached,
            RobotVSwitchFailureCode::InProcess => Self::VSwitchInProcess,
            RobotVSwitchFailureCode::VlanNotUnique => Self::VSwitchVlanNotUnique,
            RobotVSwitchFailureCode::ServerLimitReached => Self::VSwitchServerLimitReached,
            RobotVSwitchFailureCode::PerServerLimitReached => Self::VSwitchPerServerLimitReached,
            RobotVSwitchFailureCode::AlreadyCancelled => Self::VSwitchAlreadyCancelled,
        }
    }
}
