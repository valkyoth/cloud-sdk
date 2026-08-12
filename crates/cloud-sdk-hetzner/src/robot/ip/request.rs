use crate::robot::RobotIpAddress;

/// Failure while validating or preparing a Robot IP operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotIpRequestError {
    /// Caller-owned path storage was too small or encoding failed.
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

impl core::fmt::Display for RobotIpRequestError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::Path => "Robot IP path preparation failed",
            Self::Form(_) => "Robot IP form preparation failed",
            Self::InvalidTarget(_) => "Robot IP target is invalid",
            Self::InvalidHeaders(_) => "Robot IP headers are invalid",
            Self::InvalidEndpoint(_) => "official Robot endpoint is invalid",
            Self::InvalidOperationId(_) => "Robot IP operation identifier is invalid",
            Self::InvalidMetadata(_) => "Robot IP metadata is invalid",
            Self::InvalidResponsePolicy(_) => "Robot IP response policy is invalid",
            Self::InvalidRawPolicy(_) => "Robot IP raw response policy is invalid",
            Self::InvalidPreparedPolicy(_) => "Robot IP prepared policy is invalid",
        })
    }
}

impl core::error::Error for RobotIpRequestError {}

/// Optional canonical server filter for `GET /ip`.
pub struct RobotIpListRequest {
    pub(super) server_address: Option<RobotIpAddress>,
}

impl RobotIpListRequest {
    /// Lists every assigned single IP address.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            server_address: None,
        }
    }

    /// Lists only addresses assigned to one server main address.
    #[must_use]
    pub const fn for_server(server_address: RobotIpAddress) -> Self {
        Self {
            server_address: Some(server_address),
        }
    }

    /// Returns the optional canonical server filter.
    #[must_use]
    pub const fn server_address(&self) -> Option<&RobotIpAddress> {
        self.server_address.as_ref()
    }
}

impl Default for RobotIpListRequest {
    fn default() -> Self {
        Self::all()
    }
}

impl core::fmt::Debug for RobotIpListRequest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotIpListRequest([redacted])")
    }
}

macro_rules! address_request {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        pub struct $name {
            pub(super) address: RobotIpAddress,
        }

        impl $name {
            /// Creates a request for one canonical address.
            #[must_use]
            pub const fn new(address: RobotIpAddress) -> Self {
                Self { address }
            }

            /// Returns the exact request address.
            #[must_use]
            pub const fn address(&self) -> &RobotIpAddress {
                &self.address
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(concat!(stringify!($name), "([redacted])"))
            }
        }
    };
}

address_request!(RobotIpGetRequest, "Gets one Robot single-IP resource.");
address_request!(RobotIpMacGetRequest, "Gets one generated separate MAC.");
address_request!(RobotIpMacSetRequest, "Generates one separate MAC.");
address_request!(RobotIpMacDeleteRequest, "Removes one separate MAC.");

/// Non-empty partial traffic-warning update.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RobotIpTrafficUpdate {
    pub(super) warnings: Option<bool>,
    pub(super) hourly: Option<u64>,
    pub(super) daily: Option<u64>,
    pub(super) monthly: Option<u64>,
}

impl RobotIpTrafficUpdate {
    /// Starts an update by enabling or disabling warning notifications.
    #[must_use]
    pub const fn warnings(enabled: bool) -> Self {
        Self {
            warnings: Some(enabled),
            hourly: None,
            daily: None,
            monthly: None,
        }
    }

    /// Starts an update with an hourly threshold in megabytes.
    #[must_use]
    pub const fn hourly(megabytes: u64) -> Self {
        Self {
            warnings: None,
            hourly: Some(megabytes),
            daily: None,
            monthly: None,
        }
    }

    /// Starts an update with a daily threshold in megabytes.
    #[must_use]
    pub const fn daily(megabytes: u64) -> Self {
        Self {
            warnings: None,
            hourly: None,
            daily: Some(megabytes),
            monthly: None,
        }
    }

    /// Starts an update with a monthly threshold in gigabytes.
    #[must_use]
    pub const fn monthly(gigabytes: u64) -> Self {
        Self {
            warnings: None,
            hourly: None,
            daily: None,
            monthly: Some(gigabytes),
        }
    }

    /// Includes the warning-notification state.
    #[must_use]
    pub const fn with_warnings(mut self, enabled: bool) -> Self {
        self.warnings = Some(enabled);
        self
    }
    /// Includes the hourly threshold in megabytes.
    #[must_use]
    pub const fn with_hourly(mut self, megabytes: u64) -> Self {
        self.hourly = Some(megabytes);
        self
    }
    /// Includes the daily threshold in megabytes.
    #[must_use]
    pub const fn with_daily(mut self, megabytes: u64) -> Self {
        self.daily = Some(megabytes);
        self
    }
    /// Includes the monthly threshold in gigabytes.
    #[must_use]
    pub const fn with_monthly(mut self, gigabytes: u64) -> Self {
        self.monthly = Some(gigabytes);
        self
    }
}

/// Updates traffic-warning settings for one canonical address.
pub struct RobotIpUpdateRequest {
    pub(super) address: RobotIpAddress,
    pub(super) update: RobotIpTrafficUpdate,
}

impl RobotIpUpdateRequest {
    /// Creates an explicit non-empty traffic update.
    #[must_use]
    pub const fn new(address: RobotIpAddress, update: RobotIpTrafficUpdate) -> Self {
        Self { address, update }
    }
    /// Returns the exact request address.
    #[must_use]
    pub const fn address(&self) -> &RobotIpAddress {
        &self.address
    }
    /// Returns the requested partial traffic policy.
    #[must_use]
    pub const fn update(&self) -> RobotIpTrafficUpdate {
        self.update
    }
}

impl core::fmt::Debug for RobotIpUpdateRequest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotIpUpdateRequest([redacted])")
    }
}
