//! Source-locked API token inspection and explicit, non-retrying revocation.
//! Metadata is not authorization. Provision replacement tokens out of band.
mod decode;
mod models;
mod request;
mod schema_table;
pub use models::{EndpointScope, TokenMetadata, TokenResponse};
pub use request::{TokenOperation, TokenPermit};
#[cfg(feature = "blocking")]
mod client;
#[cfg(feature = "blocking")]
pub use crate::discovery::DiscoveryExecutionError as TokenExecutionError;
#[cfg(feature = "blocking")]
pub use client::{TokenBuffers, TokenClient};
#[cfg(test)]
mod tests;

/// Local upper bound for each returned scope list; never silently truncated.
pub const MAX_TOKEN_SCOPES: usize = 128;
