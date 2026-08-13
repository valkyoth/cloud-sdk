#[cfg(feature = "serde")]
use super::AuthorizedRobotWol;
use crate::robot::RobotServerNumber;

/// Explicit Wake-on-LAN action selected by the caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotWolIntent {
    /// Send one Wake-on-LAN packet to the checked server.
    Send,
}

/// Failure while validating or preparing a Robot Wake-on-LAN operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotWolRequestError {
    /// Caller-owned path storage was too small or encoding failed.
    Path,
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

impl_static_error!(RobotWolRequestError,
    Self::Path => "Robot Wake-on-LAN path preparation failed",
    Self::InvalidTarget(_) => "Robot Wake-on-LAN target is invalid",
    Self::InvalidHeaders(_) => "Robot Wake-on-LAN headers are invalid",
    Self::InvalidEndpoint(_) => "official Robot endpoint is invalid",
    Self::InvalidOperationId(_) => "Robot Wake-on-LAN operation identifier is invalid",
    Self::InvalidMetadata(_) => "Robot Wake-on-LAN metadata is invalid",
    Self::InvalidResponsePolicy(_) => "Robot Wake-on-LAN response policy is invalid",
    Self::InvalidRawPolicy(_) => "Robot Wake-on-LAN raw response policy is invalid",
    Self::InvalidPreparedPolicy(_) => "Robot Wake-on-LAN prepared policy is invalid",
);

/// Discovers Wake-on-LAN availability for one canonical server number.
pub struct RobotWolGetRequest {
    pub(super) number: RobotServerNumber,
}

impl RobotWolGetRequest {
    /// Creates a server-number-only availability request.
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

impl core::fmt::Debug for RobotWolGetRequest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotWolGetRequest([redacted])")
    }
}

/// Sends one packet using authenticated, short-lived availability evidence.
#[cfg(feature = "serde")]
pub struct RobotWolSendRequest<'state> {
    pub(super) wol: &'state AuthorizedRobotWol,
    pub(super) intent: RobotWolIntent,
}

#[cfg(feature = "serde")]
impl<'state> RobotWolSendRequest<'state> {
    /// Binds explicit wake intent to checked provider capability state.
    #[must_use]
    pub const fn from_checked(wol: &'state AuthorizedRobotWol, intent: RobotWolIntent) -> Self {
        Self { wol, intent }
    }

    /// Returns the checked server number.
    #[must_use]
    pub const fn number(&self) -> &RobotServerNumber {
        self.wol.wol().server_number()
    }

    /// Returns the explicit action selected by the caller.
    #[must_use]
    pub const fn intent(&self) -> RobotWolIntent {
        self.intent
    }
}

#[cfg(feature = "serde")]
impl core::fmt::Debug for RobotWolSendRequest<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotWolSendRequest([redacted])")
    }
}
