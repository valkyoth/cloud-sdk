use core::fmt;
use core::marker::PhantomData;

use cloud_sdk::ServiceMarker;
use cloud_sdk::client::ClientKernel;
use cloud_sdk::transport::{
    AcknowledgedCustomEndpoint, BoundTransport, CustomEndpointAcknowledgement,
    EndpointIdentityError, EndpointPolicy, EndpointPolicyError, EndpointScheme,
};

use crate::endpoint::{OfficialEndpointError, verify_official_endpoint};
use crate::identity::{CloudService, DnsService, SecurityService, StorageService};
use crate::request::ApiBaseUrl;

/// Marker for a client bound to one SDK-owned official endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OfficialEndpointTrust {}

/// Marker for explicitly acknowledged operator-controlled endpoint trust.
///
/// Custom endpoint trust cannot enter the official execution path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CustomEndpointTrust {}

/// Inspectable endpoint provenance retained by a client type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointTrust {
    /// The transport was verified against one exact SDK-owned endpoint.
    Official,
    /// The transport uses an HTTPS endpoint explicitly trusted by its operator.
    Custom,
}

/// Failure while binding a transport to one Hetzner service and trust class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HetznerClientConstructionError {
    /// The transport did not expose a valid normalized endpoint identity.
    InvalidIdentity(EndpointIdentityError),
    /// The transport did not match the selected official service endpoint.
    OfficialEndpoint(OfficialEndpointError),
    /// Custom credential destinations must use HTTPS.
    InsecureCustomEndpoint,
    /// The explicit custom endpoint policy rejected the destination.
    CustomEndpointPolicy(EndpointPolicyError),
}

impl fmt::Display for HetznerClientConstructionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidIdentity(_) => "Hetzner client transport identity is invalid",
            Self::OfficialEndpoint(_) => "Hetzner client official endpoint verification failed",
            Self::InsecureCustomEndpoint => "Hetzner custom endpoint must use HTTPS",
            Self::CustomEndpointPolicy(_) => "Hetzner custom endpoint policy rejected transport",
        })
    }
}

impl core::error::Error for HetznerClientConstructionError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::InvalidIdentity(error) => Some(error),
            Self::OfficialEndpoint(error) => Some(error),
            Self::CustomEndpointPolicy(error) => Some(error),
            Self::InsecureCustomEndpoint => None,
        }
    }
}

/// Provider facade bound to one service, endpoint trust class, and transport.
///
/// The client owns no executor, queue, clock, retry policy, or request storage.
/// Official execution methods borrow it through `&self`, permitting bounded
/// concurrency controlled entirely by caller-owned workspace leases.
pub struct HetznerClient<T, S, E = OfficialEndpointTrust> {
    pub(crate) kernel: ClientKernel<T>,
    marker: PhantomData<fn() -> (S, E)>,
}

/// Client bound to the official Hetzner Cloud API.
pub type CloudClient<T> = HetznerClient<T, CloudService>;
/// Client bound to the official Hetzner DNS API.
pub type DnsClient<T> = HetznerClient<T, DnsService>;
/// Client bound to official Hetzner security-resource endpoints.
pub type SecurityClient<T> = HetznerClient<T, SecurityService>;
/// Client bound to the official Hetzner Console Storage API.
pub type StorageClient<T> = HetznerClient<T, StorageService>;

macro_rules! service_constructors {
    ($service:ty, $official:ident, $custom:ident, $base:expr) => {
        impl<T: BoundTransport> HetznerClient<T, $service, OfficialEndpointTrust> {
            #[doc = "Constructs a client after exact official endpoint verification."]
            pub fn $official(transport: T) -> Result<Self, HetznerClientConstructionError> {
                verify_official_endpoint(&transport, $base)
                    .map_err(HetznerClientConstructionError::OfficialEndpoint)?;
                Ok(Self::new(transport))
            }

            #[doc = "Constructs an HTTPS custom-endpoint client from trusted operator configuration."]
            #[doc = " Tenant-controlled input must never select this credential destination."]
            pub fn $custom(
                transport: T,
                acknowledgement: CustomEndpointAcknowledgement,
            ) -> Result<HetznerClient<T, $service, CustomEndpointTrust>, HetznerClientConstructionError>
            {
                HetznerClient::new_custom(transport, acknowledgement)
            }
        }
    };
}

service_constructors!(
    CloudService,
    cloud,
    cloud_with_custom_endpoint,
    ApiBaseUrl::CloudV1
);
service_constructors!(
    DnsService,
    dns,
    dns_with_custom_endpoint,
    ApiBaseUrl::CloudV1
);
service_constructors!(
    SecurityService,
    security,
    security_with_custom_endpoint,
    ApiBaseUrl::CloudV1
);
service_constructors!(
    StorageService,
    storage,
    storage_with_custom_endpoint,
    ApiBaseUrl::HetznerV1
);

impl<T, S> HetznerClient<T, S, OfficialEndpointTrust> {
    const fn new(transport: T) -> Self {
        Self {
            kernel: ClientKernel::new(transport),
            marker: PhantomData,
        }
    }
}

impl<T: BoundTransport, S> HetznerClient<T, S, CustomEndpointTrust> {
    fn new_custom(
        transport: T,
        acknowledgement: CustomEndpointAcknowledgement,
    ) -> Result<Self, HetznerClientConstructionError> {
        let identity = transport
            .endpoint_identity()
            .map_err(HetznerClientConstructionError::InvalidIdentity)?;
        if identity.scheme() != EndpointScheme::Https {
            return Err(HetznerClientConstructionError::InsecureCustomEndpoint);
        }
        EndpointPolicy::acknowledged_custom(AcknowledgedCustomEndpoint::new(
            identity,
            acknowledgement,
        ))
        .verify(identity)
        .map_err(HetznerClientConstructionError::CustomEndpointPolicy)?;
        Ok(Self {
            kernel: ClientKernel::new(transport),
            marker: PhantomData,
        })
    }
}

impl<T, S: ServiceMarker> HetznerClient<T, S, OfficialEndpointTrust> {
    /// Returns official endpoint provenance.
    #[must_use]
    pub const fn endpoint_trust(&self) -> EndpointTrust {
        EndpointTrust::Official
    }
}

impl<T, S: ServiceMarker> HetznerClient<T, S, CustomEndpointTrust> {
    /// Returns explicitly acknowledged custom endpoint provenance.
    #[must_use]
    pub const fn endpoint_trust(&self) -> EndpointTrust {
        EndpointTrust::Custom
    }
}

impl<T, S: ServiceMarker, E> HetznerClient<T, S, E> {
    /// Returns the compile-time selected Hetzner service identifier.
    #[must_use]
    pub const fn service_id(&self) -> cloud_sdk::ServiceId {
        S::ID
    }

    /// Returns the immutable endpoint-bound authenticated transport.
    #[must_use]
    pub const fn transport(&self) -> &T {
        self.kernel.transport()
    }

    /// Consumes the client and returns its transport.
    #[must_use]
    pub fn into_transport(self) -> T {
        self.kernel.into_transport()
    }
}

impl<T, S: ServiceMarker, E> fmt::Debug for HetznerClient<T, S, E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HetznerClient")
            .field("service", &S::ID)
            .field("transport", &"[bound]")
            .finish_non_exhaustive()
    }
}
