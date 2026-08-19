//! Canonical Hetzner Server Metadata requests and strict response decoding.
//!
//! The service is available only from inside a Hetzner Cloud server at the
//! fixed link-local HTTP destination. It uses no credentials, redirects,
//! proxies, retries, request bodies, or caller-selected endpoint.

mod execution;
mod network;
mod request;
mod text;

pub use execution::{
    MetadataExecutionError, execute_metadata_async, execute_metadata_blocking,
    execute_metadata_local_async,
};
pub use network::{
    AliasIpv4Addresses, MetadataPrivateNetwork, MetadataPrivateNetworks,
    MetadataPrivateNetworksIter,
};
pub use request::{
    METADATA_BASE_URL, METADATA_MAX_ERROR_BYTES, MetadataEndpointError, MetadataRequest,
    MetadataRoute, MetadataWireError, metadata_endpoint_identity, metadata_endpoint_policy,
    verify_metadata_endpoint,
};
pub use text::{
    MetadataDecodeError, MetadataResponse, MetadataSummary, decode_metadata_body,
    decode_metadata_response,
};

/// Maximum canonical Server Metadata response accepted by the SDK.
pub const MAX_METADATA_RESPONSE_BYTES: usize = 65_536;
/// Maximum private-network records accepted in one response.
pub const MAX_METADATA_PRIVATE_NETWORKS: usize = 64;
/// Maximum alias addresses accepted across one private-network response.
pub const MAX_METADATA_ALIAS_IPS: usize = 512;

#[cfg(test)]
mod tests;
