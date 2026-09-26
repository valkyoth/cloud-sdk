//! Anonymous download locations, usage statistics and reverse dependencies.
//! Artifact streaming uses a separately bound anonymous static transport.
mod artifact;
mod request;
pub use crate::discovery::DiscoveryError as DownloadError;
pub use artifact::*;
#[cfg(feature = "artifact-sha256")]
mod sha256;
pub use request::{DownloadOperation, DownloadRequest};
#[cfg(feature = "artifact-sha256")]
pub use sha256::Sha256Checksum;
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
pub use crate::discovery::DiscoveryExecutionError as DownloadExecutionError;
#[cfg(all(test, feature = "alloc"))]
mod tests;
#[cfg(any(feature = "blocking", feature = "async"))]
pub use client::DownloadClient;
