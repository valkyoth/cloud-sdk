#[cfg(feature = "serde")]
use super::AuthorizedRobotReset;
use crate::robot::RobotServerNumber;

/// Finite reset capability values source-locked from Robot.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RobotResetType {
    /// Ask the operating system to reboot (`sw`).
    Software,
    /// Trigger the server's hardware reset (`hw`).
    Hardware,
    /// Press the remote power button (`power`).
    Power,
    /// Hold the remote power button (`power_long`).
    PowerLong,
    /// Request a manually performed reset (`man`).
    Manual,
}

#[cfg(feature = "serde")]
impl RobotResetType {
    pub(super) const fn wire(self) -> &'static str {
        match self {
            Self::Software => "sw",
            Self::Hardware => "hw",
            Self::Power => "power",
            Self::PowerLong => "power_long",
            Self::Manual => "man",
        }
    }

    pub(super) const fn parse(value: &str) -> Option<Self> {
        match value.as_bytes() {
            b"sw" => Some(Self::Software),
            b"hw" => Some(Self::Hardware),
            b"power" => Some(Self::Power),
            b"power_long" => Some(Self::PowerLong),
            b"man" => Some(Self::Manual),
            _ => None,
        }
    }
}

/// Explicit disruptive action selected by the caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotResetIntent {
    /// Execute an advertised reset capability.
    Execute(RobotResetType),
}

impl RobotResetIntent {
    /// Returns the exact selected provider capability.
    #[must_use]
    pub const fn reset_type(self) -> RobotResetType {
        match self {
            Self::Execute(value) => value,
        }
    }
}

/// Failure while validating or preparing a Robot reset operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotResetRequestError {
    /// The selected reset type was not present in checked provider state.
    UnsupportedCapability,
    /// Caller-owned path storage was too small or path encoding failed.
    Path,
    /// Robot form validation or encoding failed.
    Form(crate::robot::RobotFormError),
    /// The constructed request target was rejected.
    InvalidTarget(cloud_sdk::transport::RequestTargetError),
    /// Source-locked request headers were rejected.
    InvalidHeaders(cloud_sdk::transport::HeaderError),
    /// The official Robot endpoint policy was invalid.
    InvalidEndpoint(crate::endpoint::OfficialEndpointError),
    /// A source-locked operation identifier was invalid.
    InvalidOperationId(cloud_sdk::operation::OperationIdError),
    /// Operation safety metadata was internally inconsistent.
    InvalidMetadata(cloud_sdk::operation::OperationMetadataError),
    /// The success-response policy was internally inconsistent.
    InvalidResponsePolicy(cloud_sdk::operation::ResponsePolicyValidationError),
    /// The raw response-wire policy was internally inconsistent.
    InvalidRawPolicy(cloud_sdk::transport::RawResponsePolicyError),
    /// Cross-policy prepared-request validation failed.
    InvalidPreparedPolicy(cloud_sdk::operation::PreparedRequestPolicyError),
}

impl core::fmt::Display for RobotResetRequestError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedCapability => "Robot reset capability is not advertised",
            Self::Path => "Robot reset path preparation failed",
            Self::Form(_) => "Robot reset form preparation failed",
            Self::InvalidTarget(_) => "Robot reset target is invalid",
            Self::InvalidHeaders(_) => "Robot reset headers are invalid",
            Self::InvalidEndpoint(_) => "official Robot endpoint is invalid",
            Self::InvalidOperationId(_) => "Robot reset operation identifier is invalid",
            Self::InvalidMetadata(_) => "Robot reset metadata is invalid",
            Self::InvalidResponsePolicy(_) => "Robot reset response policy is invalid",
            Self::InvalidRawPolicy(_) => "Robot reset raw response policy is invalid",
            Self::InvalidPreparedPolicy(_) => "Robot reset prepared policy is invalid",
        })
    }
}

impl core::error::Error for RobotResetRequestError {}

/// Lists reset capabilities for all eligible servers.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct RobotResetListRequest;

impl RobotResetListRequest {
    /// Creates the bodyless list request.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl core::fmt::Debug for RobotResetListRequest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotResetListRequest")
    }
}

/// Gets checked reset capabilities for one canonical server number.
pub struct RobotResetGetRequest {
    pub(super) number: RobotServerNumber,
}

impl RobotResetGetRequest {
    /// Creates a canonical server-number request.
    #[must_use]
    pub const fn new(number: RobotServerNumber) -> Self {
        Self { number }
    }

    /// Returns the requested server number.
    #[must_use]
    pub const fn number(&self) -> &RobotServerNumber {
        &self.number
    }
}

impl core::fmt::Debug for RobotResetGetRequest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotResetGetRequest([redacted])")
    }
}

/// Executes one capability selected from checked reset state.
#[cfg(feature = "serde")]
pub struct RobotResetExecuteRequest<'state> {
    pub(super) reset: &'state AuthorizedRobotReset,
    pub(super) intent: RobotResetIntent,
}

#[cfg(feature = "serde")]
impl<'state> RobotResetExecuteRequest<'state> {
    /// Creates execution only from authenticated capability evidence.
    pub fn from_checked(
        reset: &'state AuthorizedRobotReset,
        intent: RobotResetIntent,
    ) -> Result<Self, RobotResetRequestError> {
        if !reset.reset().supports(intent.reset_type()) {
            return Err(RobotResetRequestError::UnsupportedCapability);
        }
        Ok(Self { reset, intent })
    }

    /// Returns the checked server number.
    #[must_use]
    pub const fn number(&self) -> &RobotServerNumber {
        self.reset.reset().summary().number()
    }

    /// Returns the exact disruptive intent.
    #[must_use]
    pub const fn intent(&self) -> RobotResetIntent {
        self.intent
    }
}

#[cfg(feature = "serde")]
impl core::fmt::Debug for RobotResetExecuteRequest<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotResetExecuteRequest([redacted])")
    }
}
