use cloud_sdk::transport::{ResponseDecodeWorkspace, TransportResponse};

use super::request::*;
use crate::robot::protocol::decode_robot_failure_with;
use crate::robot::{RobotDecodeError, RobotFailure, RobotProviderErrorCode};

/// Source-locked Robot boot provider failure code.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotBootFailureCode {
    /// The canonical server number does not exist.
    ServerNotFound,
    /// Boot configuration is unavailable for the server.
    BootNotAvailable,
    /// Robot could not activate the selected configuration.
    BootActivationFailed,
    /// Robot could not deactivate the selected configuration.
    BootDeactivationFailed,
    /// The server has no Windows addon.
    WindowsMissingAddon,
    /// The selected or active Windows version is unsupported.
    WindowsOutdatedVersion,
}

#[derive(Clone, Copy)]
enum FailureKind {
    Read,
    Activate,
    Deactivate,
    WindowsRead,
    WindowsActivate,
    WindowsDeactivate,
}

macro_rules! failure_decoder {
    ($type:ty, $kind:ident) => {
        impl $type {
            /// Decodes only failures source-locked for this boot operation.
            pub fn decode_failure(
                &self,
                response: TransportResponse<'_, '_>,
                workspace: &mut ResponseDecodeWorkspace,
            ) -> Result<RobotFailure, RobotDecodeError> {
                decode(FailureKind::$kind, response, workspace)
            }
        }
    };
}

failure_decoder!(RobotBootGetRequest, Read);
failure_decoder!(RobotRescueGetRequest, Read);
failure_decoder!(RobotRescueLastRequest, Read);
failure_decoder!(RobotLinuxGetRequest, Read);
failure_decoder!(RobotLinuxLastRequest, Read);
failure_decoder!(RobotVncGetRequest, Read);
failure_decoder!(RobotRescueActivateRequest<'_>, Activate);
failure_decoder!(RobotLinuxActivateRequest<'_>, Activate);
failure_decoder!(RobotVncActivateRequest<'_>, Activate);
failure_decoder!(RobotRescueDeactivateRequest, Deactivate);
failure_decoder!(RobotLinuxDeactivateRequest, Deactivate);
failure_decoder!(RobotVncDeactivateRequest, Deactivate);
failure_decoder!(RobotWindowsGetRequest, WindowsRead);
failure_decoder!(RobotWindowsActivateRequest<'_>, WindowsActivate);
failure_decoder!(RobotWindowsDeactivateRequest, WindowsDeactivate);

fn decode(
    kind: FailureKind,
    response: TransportResponse<'_, '_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailure, RobotDecodeError> {
    let invalid_input = matches!(kind, FailureKind::Activate | FailureKind::WindowsActivate);
    decode_robot_failure_with(
        response,
        workspace,
        invalid_input,
        &[404, 500],
        |status, code| classify(kind, status, code).map(Into::into),
    )
}

fn classify(kind: FailureKind, status: u16, code: &str) -> Option<RobotBootFailureCode> {
    match (kind, status, code) {
        (_, 404, "SERVER_NOT_FOUND") => Some(RobotBootFailureCode::ServerNotFound),
        (_, 404, "BOOT_NOT_AVAILABLE") => Some(RobotBootFailureCode::BootNotAvailable),
        (FailureKind::Activate | FailureKind::WindowsActivate, 500, "BOOT_ACTIVATION_FAILED") => {
            Some(RobotBootFailureCode::BootActivationFailed)
        }
        (
            FailureKind::Deactivate | FailureKind::WindowsDeactivate,
            500,
            "BOOT_DEACTIVATION_FAILED",
        ) => Some(RobotBootFailureCode::BootDeactivationFailed),
        (FailureKind::WindowsActivate, 404, "WINDOWS_MISSING_ADDON") => {
            Some(RobotBootFailureCode::WindowsMissingAddon)
        }
        (
            FailureKind::WindowsRead
            | FailureKind::WindowsActivate
            | FailureKind::WindowsDeactivate,
            404,
            "WINDOWS_OUTDATED_VERSION",
        ) => Some(RobotBootFailureCode::WindowsOutdatedVersion),
        _ => None,
    }
}

impl From<RobotBootFailureCode> for RobotProviderErrorCode {
    fn from(code: RobotBootFailureCode) -> Self {
        match code {
            RobotBootFailureCode::ServerNotFound => Self::ServerNotFound,
            RobotBootFailureCode::BootNotAvailable => Self::BootNotAvailable,
            RobotBootFailureCode::BootActivationFailed => Self::BootActivationFailed,
            RobotBootFailureCode::BootDeactivationFailed => Self::BootDeactivationFailed,
            RobotBootFailureCode::WindowsMissingAddon => Self::WindowsMissingAddon,
            RobotBootFailureCode::WindowsOutdatedVersion => Self::WindowsOutdatedVersion,
        }
    }
}
