//! Bounded crate search and metadata, not an index mirror or dependency resolver.
//! Use the sparse index for resolution and database dumps for bulk analysis.

mod request;
pub use request::{CatalogOperation, CatalogRequest};
#[cfg(feature = "alloc")]
mod decode;
#[cfg(feature = "alloc")]
mod models;
#[cfg(feature = "alloc")]
mod pagination;
#[cfg(feature = "alloc")]
mod schema;
#[cfg(feature = "alloc")]
mod schema_table;
#[cfg(feature = "alloc")]
pub use models::*;
#[cfg(feature = "alloc")]
pub use pagination::{CatalogContinuation, CatalogLink};
#[cfg(any(feature = "blocking", feature = "async"))]
mod client;
pub use crate::discovery::DiscoveryError as CatalogError;
#[cfg(any(feature = "blocking", feature = "async"))]
pub use crate::discovery::DiscoveryExecutionError as CatalogExecutionError;
#[cfg(any(feature = "blocking", feature = "async"))]
pub use client::CatalogClient;

#[cfg(all(test, feature = "alloc"))]
mod tests;
