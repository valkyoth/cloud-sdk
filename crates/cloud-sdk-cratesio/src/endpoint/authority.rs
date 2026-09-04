use core::fmt;

use cloud_sdk::transport::{
    AcknowledgedCustomEndpoint, BoundTransport, CustomEndpointAcknowledgement, EndpointIdentity,
    EndpointIdentityError, EndpointPolicy, EndpointScheme,
};

/// Canonical production crates.io API origin.
pub const CRATES_IO_API_BASE_URL: &str = "https://crates.io";

/// Canonical crates.io staging API origin.
pub const CRATES_IO_STAGING_API_BASE_URL: &str = "https://staging.crates.io";

/// Canonical authority for immutable crates.io package downloads.
pub const CRATES_IO_STATIC_DOWNLOAD_BASE_URL: &str = "https://static.crates.io";

/// Purpose attached to an official crates.io endpoint.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OfficialEndpointPurpose {
    /// Production registry API requests.
    ProductionApi,
    /// Staging registry API requests.
    StagingApi,
    /// Anonymous immutable package downloads.
    StaticDownloads,
}

/// One provider-owned official crates.io endpoint.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OfficialCratesIoEndpoint {
    purpose: OfficialEndpointPurpose,
}

impl OfficialCratesIoEndpoint {
    /// Selects the production registry API.
    #[must_use]
    pub const fn production_api() -> Self {
        Self {
            purpose: OfficialEndpointPurpose::ProductionApi,
        }
    }

    /// Selects the staging registry API.
    #[must_use]
    pub const fn staging_api() -> Self {
        Self {
            purpose: OfficialEndpointPurpose::StagingApi,
        }
    }

    /// Selects the anonymous static package-download authority.
    #[must_use]
    pub const fn static_downloads() -> Self {
        Self {
            purpose: OfficialEndpointPurpose::StaticDownloads,
        }
    }

    /// Returns the endpoint's fixed provider-owned purpose.
    #[must_use]
    pub const fn purpose(self) -> OfficialEndpointPurpose {
        self.purpose
    }

    /// Returns the exact official HTTPS base URL.
    #[must_use]
    pub const fn base_url(self) -> &'static str {
        match self.purpose {
            OfficialEndpointPurpose::ProductionApi => CRATES_IO_API_BASE_URL,
            OfficialEndpointPurpose::StagingApi => CRATES_IO_STAGING_API_BASE_URL,
            OfficialEndpointPurpose::StaticDownloads => CRATES_IO_STATIC_DOWNLOAD_BASE_URL,
        }
    }

    /// Returns the normalized fixed-origin identity.
    pub fn identity(self) -> Result<EndpointIdentity<'static>, CratesIoEndpointError> {
        let host = match self.purpose {
            OfficialEndpointPurpose::ProductionApi => "crates.io",
            OfficialEndpointPurpose::StagingApi => "staging.crates.io",
            OfficialEndpointPurpose::StaticDownloads => "static.crates.io",
        };
        EndpointIdentity::new(EndpointScheme::Https, host, 443, "/")
            .map_err(|_| CratesIoEndpointError::InvalidOfficialEndpoint)
    }

    /// Returns the fixed provider-owned endpoint policy.
    pub fn policy(self) -> Result<EndpointPolicy<'static>, CratesIoEndpointError> {
        self.identity().map(EndpointPolicy::fixed)
    }

    /// Verifies one credential-bound transport against this exact origin.
    pub fn verify_transport(
        self,
        transport: &(impl BoundTransport + ?Sized),
    ) -> Result<(), CratesIoEndpointError> {
        let candidate = transport
            .endpoint_identity()
            .map_err(CratesIoEndpointError::InvalidIdentity)?;
        self.policy()?
            .verify(candidate)
            .map_err(|_| CratesIoEndpointError::DestinationMismatch)
    }
}

/// An API endpoint selected by trusted operator configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AcknowledgedCustomApiEndpoint<'a> {
    endpoint: AcknowledgedCustomEndpoint<'a>,
}

impl<'a> AcknowledgedCustomApiEndpoint<'a> {
    /// Explicitly acknowledges a custom HTTPS API credential destination.
    ///
    /// The identity must come from trusted operator configuration. Tenant,
    /// request, webhook, or other attacker-controlled input must never select
    /// this destination.
    pub fn new(
        identity: EndpointIdentity<'a>,
        acknowledgement: CustomEndpointAcknowledgement,
    ) -> Result<Self, CratesIoEndpointError> {
        if identity.scheme() != EndpointScheme::Https {
            return Err(CratesIoEndpointError::HttpsRequired);
        }
        Ok(Self {
            endpoint: AcknowledgedCustomEndpoint::new(identity, acknowledgement),
        })
    }

    /// Returns the acknowledged endpoint identity.
    #[must_use]
    pub const fn identity(self) -> EndpointIdentity<'a> {
        self.endpoint.identity()
    }

    /// Returns the explicit custom-destination policy.
    #[must_use]
    pub const fn policy(self) -> EndpointPolicy<'a> {
        EndpointPolicy::acknowledged_custom(self.endpoint)
    }
}

/// crates.io endpoint construction or destination error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CratesIoEndpointError {
    /// An SDK-owned endpoint constant could not form its declared identity.
    InvalidOfficialEndpoint,
    /// A transport returned an invalid endpoint identity.
    InvalidIdentity(EndpointIdentityError),
    /// The candidate does not exactly match the selected endpoint.
    DestinationMismatch,
    /// Custom crates.io API endpoints must use HTTPS.
    HttpsRequired,
}

impl fmt::Display for CratesIoEndpointError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidOfficialEndpoint => "official crates.io endpoint is invalid",
            Self::InvalidIdentity(_) => "transport endpoint identity is invalid",
            Self::DestinationMismatch => "transport destination does not match crates.io policy",
            Self::HttpsRequired => "custom crates.io API endpoint must use HTTPS",
        })
    }
}

impl core::error::Error for CratesIoEndpointError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::InvalidIdentity(error) => Some(error),
            Self::InvalidOfficialEndpoint | Self::DestinationMismatch | Self::HttpsRequired => None,
        }
    }
}
