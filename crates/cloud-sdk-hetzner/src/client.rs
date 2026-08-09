//! Service-typed Hetzner client construction over provider-neutral transports.

mod construction;
#[cfg(feature = "serde")]
mod execution;

pub use construction::{
    CloudClient, CustomEndpointTrust, DnsClient, EndpointTrust, HetznerClient,
    HetznerClientConstructionError, OfficialEndpointTrust, SecurityClient, StorageClient,
};

#[cfg(test)]
mod tests;
