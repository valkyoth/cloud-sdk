//! Immutable endpoint identities and provider-owned trust policies.

mod identity;
mod policy;

pub(crate) use identity::CanonicalHost;
pub use identity::{
    BoundTransport, EndpointIdentity, EndpointIdentityError, EndpointScheme,
    MAX_ENDPOINT_BASE_PATH_BYTES, MAX_ENDPOINT_HOST_BYTES,
};
pub use policy::{
    AcknowledgedCustomEndpoint, CustomEndpointAcknowledgement, EndpointPolicy, EndpointPolicyError,
    EndpointPolicyKind, MAX_ENDPOINT_REGION_BYTES, MAX_OFFICIAL_ENDPOINTS, RegionEndpoint,
};
