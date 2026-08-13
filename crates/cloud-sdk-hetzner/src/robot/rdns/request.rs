use crate::robot::{RobotIpAddress, RobotRdnsName};

/// Failure while validating or preparing a Robot reverse-DNS operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotRdnsRequestError {
    /// A list filter was not an IPv4 main server address.
    InvalidServerAddress,
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

impl core::fmt::Display for RobotRdnsRequestError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidServerAddress => "Robot reverse-DNS server filter is not IPv4",
            Self::Path => "Robot reverse-DNS path preparation failed",
            Self::Form(_) => "Robot reverse-DNS form preparation failed",
            Self::InvalidTarget(_) => "Robot reverse-DNS target is invalid",
            Self::InvalidHeaders(_) => "Robot reverse-DNS headers are invalid",
            Self::InvalidEndpoint(_) => "official Robot endpoint is invalid",
            Self::InvalidOperationId(_) => "Robot reverse-DNS operation identifier is invalid",
            Self::InvalidMetadata(_) => "Robot reverse-DNS metadata is invalid",
            Self::InvalidResponsePolicy(_) => "Robot reverse-DNS response policy is invalid",
            Self::InvalidRawPolicy(_) => "Robot reverse-DNS raw response policy is invalid",
            Self::InvalidPreparedPolicy(_) => "Robot reverse-DNS prepared policy is invalid",
        })
    }
}

impl core::error::Error for RobotRdnsRequestError {}

/// Optional canonical main-server filter for `GET /rdns`.
pub struct RobotRdnsListRequest {
    pub(super) server_address: Option<RobotIpAddress>,
}

impl RobotRdnsListRequest {
    /// Lists every reverse-DNS entry.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            server_address: None,
        }
    }

    /// Restricts the list to one canonical IPv4 main server address.
    pub fn for_server(server_address: RobotIpAddress) -> Result<Self, RobotRdnsRequestError> {
        if !server_address.with_addr(|address| address.is_ipv4()) {
            return Err(RobotRdnsRequestError::InvalidServerAddress);
        }
        Ok(Self {
            server_address: Some(server_address),
        })
    }

    /// Returns the optional server filter.
    #[must_use]
    pub const fn server_address(&self) -> Option<&RobotIpAddress> {
        self.server_address.as_ref()
    }
}

impl Default for RobotRdnsListRequest {
    fn default() -> Self {
        Self::all()
    }
}

impl core::fmt::Debug for RobotRdnsListRequest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotRdnsListRequest([redacted])")
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

address_request!(RobotRdnsGetRequest, "Gets one reverse-DNS entry.");
address_request!(RobotRdnsDeleteRequest, "Deletes one reverse-DNS entry.");

macro_rules! ptr_request {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        pub struct $name {
            pub(super) address: RobotIpAddress,
            pub(super) ptr: RobotRdnsName,
        }

        impl $name {
            /// Creates a request for one canonical address and PTR target.
            #[must_use]
            pub const fn new(address: RobotIpAddress, ptr: RobotRdnsName) -> Self {
                Self { address, ptr }
            }

            /// Returns the exact request address.
            #[must_use]
            pub const fn address(&self) -> &RobotIpAddress {
                &self.address
            }

            /// Returns the exact requested PTR target.
            #[must_use]
            pub const fn ptr(&self) -> &RobotRdnsName {
                &self.ptr
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(concat!(stringify!($name), "([redacted])"))
            }
        }
    };
}

ptr_request!(RobotRdnsSetRequest, "Creates a new reverse-DNS entry.");
ptr_request!(
    RobotRdnsUpdateRequest,
    "Creates or updates one reverse-DNS entry."
);
