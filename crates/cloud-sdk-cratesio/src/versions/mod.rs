//! Bounded version metadata, dependencies and README locations.
//! No dependency resolution, HTML rendering or automatic static-CDN fetches.
mod request;
pub use crate::discovery::DiscoveryError as VersionError;
pub use request::{VersionOperation, VersionRequest};
#[cfg(feature = "alloc")]
mod decode;
#[cfg(feature = "alloc")]
mod models;
#[cfg(feature = "alloc")]
mod schema_table;
#[cfg(feature = "alloc")]
pub use models::*;
#[cfg(any(feature = "blocking", feature = "async"))]
mod client;
#[cfg(any(feature = "blocking", feature = "async"))]
pub use crate::discovery::DiscoveryExecutionError as VersionExecutionError;
#[cfg(any(feature = "blocking", feature = "async"))]
pub use client::VersionClient;
#[cfg(all(test, feature = "alloc"))]
mod tests;
