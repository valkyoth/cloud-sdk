use crate::robot::RobotIpAddress;

/// Failure while validating or preparing a Robot failover operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotFailoverRequestError {
    /// The failover route and destination server use different IP families.
    AddressFamilyMismatch,
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

impl core::fmt::Display for RobotFailoverRequestError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::AddressFamilyMismatch => "Robot failover route and destination families differ",
            Self::Path => "Robot failover path preparation failed",
            Self::Form(_) => "Robot failover form preparation failed",
            Self::InvalidTarget(_) => "Robot failover target is invalid",
            Self::InvalidHeaders(_) => "Robot failover headers are invalid",
            Self::InvalidEndpoint(_) => "official Robot endpoint is invalid",
            Self::InvalidOperationId(_) => "Robot failover operation identifier is invalid",
            Self::InvalidMetadata(_) => "Robot failover metadata is invalid",
            Self::InvalidResponsePolicy(_) => "Robot failover response policy is invalid",
            Self::InvalidRawPolicy(_) => "Robot failover raw response policy is invalid",
            Self::InvalidPreparedPolicy(_) => "Robot failover prepared policy is invalid",
        })
    }
}

impl core::error::Error for RobotFailoverRequestError {}

/// Lists every failover route assigned to the authenticated account.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RobotFailoverListRequest;

impl RobotFailoverListRequest {
    /// Creates a failover-list request.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

macro_rules! route_request {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        pub struct $name {
            pub(super) route: RobotIpAddress,
        }

        impl $name {
            /// Creates a request for one canonical failover route address.
            #[must_use]
            pub const fn new(route: RobotIpAddress) -> Self {
                Self { route }
            }

            /// Returns the exact failover route address.
            #[must_use]
            pub const fn route(&self) -> &RobotIpAddress {
                &self.route
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(concat!(stringify!($name), "([redacted])"))
            }
        }
    };
}

route_request!(RobotFailoverGetRequest, "Gets one failover route.");
route_request!(
    RobotFailoverDeleteRouteRequest,
    "Deletes the active destination from one failover route."
);

/// Explicit intent to reroute one failover address to a canonical server address.
pub struct RobotFailoverRerouteRequest {
    pub(super) route: RobotIpAddress,
    pub(super) active_server: RobotIpAddress,
}

impl RobotFailoverRerouteRequest {
    /// Creates a reroute only when route and destination use the same IP family.
    pub fn new(
        route: RobotIpAddress,
        active_server: RobotIpAddress,
    ) -> Result<Self, RobotFailoverRequestError> {
        let same_family = route.with_addr(|route| {
            active_server.with_addr(|server| route.is_ipv4() == server.is_ipv4())
        });
        if !same_family {
            return Err(RobotFailoverRequestError::AddressFamilyMismatch);
        }
        Ok(Self {
            route,
            active_server,
        })
    }

    /// Returns the exact failover route address.
    #[must_use]
    pub const fn route(&self) -> &RobotIpAddress {
        &self.route
    }

    /// Returns the exact requested active server address.
    #[must_use]
    pub const fn active_server(&self) -> &RobotIpAddress {
        &self.active_server
    }
}

impl core::fmt::Debug for RobotFailoverRerouteRequest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotFailoverRerouteRequest([redacted])")
    }
}
