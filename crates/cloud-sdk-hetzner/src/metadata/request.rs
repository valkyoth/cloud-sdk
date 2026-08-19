use cloud_sdk::Method;
use cloud_sdk::operation::{
    CostIntent, OperationImpact, OperationMetadata, OperationMetadataError, RequestIdPolicy,
    RequestSemantics, RetryEligibility,
};
use cloud_sdk::transport::{
    BoundTransport, EndpointIdentity, EndpointIdentityError, EndpointPolicy, EndpointPolicyError,
    EndpointScheme, HeaderName, MAX_INFORMATIONAL_RESPONSES, MediaType, RawResponsePolicy,
    RawResponsePolicyError, RequestTarget, RequestTargetError, ResponseMediaPolicy,
    TransportRequest,
};

use super::MAX_METADATA_RESPONSE_BYTES;

/// Canonical Hetzner Server Metadata link-local origin.
pub const METADATA_BASE_URL: &str = "http://169.254.169.254";
/// Maximum retained body for a non-success metadata response.
pub const METADATA_MAX_ERROR_BYTES: usize = 4_096;

const TEXT_MEDIA: &[MediaType<'static>] = &[MediaType::TEXT_PLAIN];
const YAML_MEDIA: &[MediaType<'static>] = &[
    MediaType::APPLICATION_YAML,
    MediaType::TEXT_YAML,
    MediaType::TEXT_PLAIN,
];

/// One of the seven canonical Server Metadata reads.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum MetadataRoute {
    /// YAML summary containing scalar server identity fields.
    Summary,
    /// Server name.
    Hostname,
    /// Numeric server identifier.
    InstanceId,
    /// Primary public IPv4 address.
    PublicIpv4,
    /// YAML list of attached private networks.
    PrivateNetworks,
    /// Availability-zone name.
    AvailabilityZone,
    /// Network-region name.
    Region,
}

impl MetadataRoute {
    /// Returns the exact request target for this canonical read.
    #[must_use]
    pub const fn target(self) -> &'static str {
        match self {
            Self::Summary => "/hetzner/v1/metadata",
            Self::Hostname => "/hetzner/v1/metadata/hostname",
            Self::InstanceId => "/hetzner/v1/metadata/instance-id",
            Self::PublicIpv4 => "/hetzner/v1/metadata/public-ipv4",
            Self::PrivateNetworks => "/hetzner/v1/metadata/private-networks",
            Self::AvailabilityZone => "/hetzner/v1/metadata/availability-zone",
            Self::Region => "/hetzner/v1/metadata/region",
        }
    }

    const fn media(self) -> &'static [MediaType<'static>] {
        match self {
            Self::Summary | Self::PrivateNetworks => YAML_MEDIA,
            _ => TEXT_MEDIA,
        }
    }
}

/// Prepared unauthenticated metadata request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetadataRequest {
    route: MetadataRoute,
}

impl MetadataRequest {
    /// Selects one canonical read. No arbitrary path can be supplied.
    #[must_use]
    pub const fn new(route: MetadataRoute) -> Self {
        Self { route }
    }

    /// Returns the selected route.
    #[must_use]
    pub const fn route(self) -> MetadataRoute {
        self.route
    }

    /// Returns immutable read-only operation metadata with retries disabled.
    pub const fn operation_metadata(self) -> Result<OperationMetadata, OperationMetadataError> {
        OperationMetadata::new(
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::Never,
            CostIntent::NoKnownCost,
            RequestIdPolicy::Discard,
        )
    }

    /// Builds the exact empty-body GET request.
    pub fn transport_request(self) -> Result<TransportRequest<'static>, MetadataWireError> {
        let target = RequestTarget::new(self.route.target()).map_err(MetadataWireError::Target)?;
        Ok(TransportRequest::new(Method::Get, target))
    }

    /// Builds the bounded response-wire policy for this route.
    pub fn response_policy(self) -> Result<RawResponsePolicy<'static>, MetadataWireError> {
        let content_type =
            HeaderName::new("content-type").map_err(|_| MetadataWireError::Headers)?;
        RawResponsePolicy::new(
            MAX_METADATA_RESPONSE_BYTES,
            METADATA_MAX_ERROR_BYTES,
            ResponseMediaPolicy::Optional(self.route.media()),
            ResponseMediaPolicy::Optional(TEXT_MEDIA),
            &[content_type],
            MAX_INFORMATIONAL_RESPONSES,
        )
        .map_err(MetadataWireError::ResponsePolicy)
    }
}

/// Metadata request construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataWireError {
    /// An SDK-owned target failed core validation.
    Target(RequestTargetError),
    /// Static headers failed validation.
    Headers,
    /// Static response policy failed validation.
    ResponsePolicy(RawResponsePolicyError),
}

impl_static_error!(MetadataWireError,
    Self::Target(_) => "canonical metadata request target is invalid",
    Self::Headers => "canonical metadata headers are invalid",
    Self::ResponsePolicy(_) => "canonical metadata response policy is invalid",
);

/// Returns the exact link-local metadata endpoint identity.
pub fn metadata_endpoint_identity() -> Result<EndpointIdentity<'static>, EndpointIdentityError> {
    EndpointIdentity::new(EndpointScheme::Http, "169.254.169.254", 80, "/")
}

/// Returns the provider-owned fixed endpoint policy.
pub fn metadata_endpoint_policy() -> Result<EndpointPolicy<'static>, EndpointIdentityError> {
    metadata_endpoint_identity().map(EndpointPolicy::fixed)
}

/// Verifies exact scheme, address, port, and base path before execution.
pub fn verify_metadata_endpoint(
    transport: &(impl BoundTransport + ?Sized),
) -> Result<(), MetadataEndpointError> {
    let actual = transport
        .endpoint_identity()
        .map_err(MetadataEndpointError::InvalidIdentity)?;
    metadata_endpoint_policy()
        .map_err(MetadataEndpointError::InvalidIdentity)?
        .verify(actual)
        .map_err(MetadataEndpointError::Mismatch)
}

/// Metadata transport destination failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataEndpointError {
    /// The executor returned an invalid identity.
    InvalidIdentity(EndpointIdentityError),
    /// The executor is not bound to the exact metadata destination.
    Mismatch(EndpointPolicyError),
}

impl core::fmt::Display for MetadataEndpointError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidIdentity(_) => "metadata endpoint identity is invalid",
            Self::Mismatch(_) => "transport is not confined to the Hetzner metadata endpoint",
        })
    }
}

impl core::error::Error for MetadataEndpointError {}
