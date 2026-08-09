//! Service-typed Hetzner client construction over provider-neutral transports.

#[cfg(feature = "serde")]
mod cloud;
mod construction;
#[cfg(feature = "serde")]
mod execution;

#[cfg(feature = "serde")]
pub use cloud::{CLOUD_CLIENT_METHODS, CloudClientMethodDescriptor, CloudReadResult};
pub use construction::{
    CloudClient, CustomEndpointTrust, DnsClient, EndpointTrust, HetznerClient,
    HetznerClientConstructionError, OfficialEndpointTrust, SecurityClient, StorageClient,
};

#[cfg(test)]
mod tests;
