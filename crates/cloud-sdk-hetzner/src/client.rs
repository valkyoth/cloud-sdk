//! Service-typed Hetzner client construction over provider-neutral transports.

#[cfg(feature = "serde")]
mod cloud;
mod construction;
#[cfg(feature = "serde")]
mod dns;
#[cfg(feature = "serde")]
mod execution;
#[cfg(feature = "serde")]
mod security;
#[cfg(feature = "serde")]
mod storage;

#[cfg(feature = "serde")]
pub use cloud::{CLOUD_CLIENT_METHODS, CloudClientMethodDescriptor, CloudReadResult};
pub use construction::{
    CloudClient, CustomEndpointTrust, DnsClient, EndpointTrust, HetznerClient,
    HetznerClientConstructionError, OfficialEndpointTrust, SecurityClient, StorageClient,
};
#[cfg(feature = "serde")]
pub use dns::{DNS_CLIENT_METHODS, DnsClientMethodDescriptor, DnsReadResult};
#[cfg(feature = "serde")]
pub use security::{SECURITY_CLIENT_METHODS, SecurityClientMethodDescriptor, SecurityReadResult};
#[cfg(feature = "serde")]
pub use storage::{STORAGE_CLIENT_METHODS, StorageClientMethodDescriptor, StorageReadResult};

#[cfg(test)]
mod tests;
