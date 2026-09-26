//! GitHub and GitLab trusted-publishing model ownership.
#[cfg(feature = "alloc")]
mod config;
#[cfg(feature = "alloc")]
mod decode;
#[cfg(feature = "alloc")]
mod oidc;
#[cfg(feature = "alloc")]
mod request;
#[cfg(feature = "alloc")]
mod schema_table;
#[cfg(feature = "alloc")]
mod temporary;
#[cfg(feature = "alloc")]
pub use crate::discovery::DiscoveryError as TrustedPublishingError;
#[cfg(feature = "alloc")]
pub use config::{Publisher, PublisherConfig};
#[cfg(feature = "alloc")]
pub use decode::{Configuration, ConfigurationPage, TrustedPublishingResponse};
#[cfg(feature = "alloc")]
pub use oidc::ExchangePolicy;
#[cfg(feature = "alloc")]
pub use request::{TrustedPublishingOperation, TrustedPublishingPermit};
#[cfg(feature = "alloc")]
pub use temporary::TemporaryToken;
#[cfg(feature = "blocking")]
mod client;
#[cfg(feature = "blocking")]
mod empty;
#[cfg(feature = "blocking")]
pub use client::{TrustedPublishingBuffers, TrustedPublishingClient};
#[cfg(all(test, feature = "alloc"))]
mod tests;
