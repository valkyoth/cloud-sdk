//! Anonymous taxonomy and site discovery, with bounded checked responses.
//!
//! Request construction is allocation-free. Models require `alloc`; execution
//! requires `blocking` or `async`. Neither execution path accepts credentials.

mod request;
pub use request::{DiscoveryOperation, DiscoveryRequest};
#[cfg(feature = "alloc")]
mod crate_model;
#[cfg(feature = "alloc")]
mod decode;
#[cfg(feature = "alloc")]
mod models;
#[cfg(feature = "alloc")]
mod value;
#[cfg(feature = "alloc")]
pub use crate_model::{CrateLinks, SummaryCrate};
#[cfg(feature = "alloc")]
pub use models::*;
#[cfg(feature = "alloc")]
pub use value::DiscoveryValue;
#[cfg(any(feature = "blocking", feature = "async"))]
mod client;
#[cfg(any(feature = "blocking", feature = "async"))]
pub use client::{DiscoveryClient, DiscoveryExecutionError};

/// An invalid discovery response, policy or allocation request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryError {
    /// Required data is missing or has the wrong JSON type.
    Schema,
    /// A declared or SDK-owned byte, depth, node or item limit was exceeded.
    Limit,
    /// A timestamp, count, or other known value is invalid.
    Value,
    /// Bounded storage could not be allocated.
    Allocation,
    /// The destination or continuation does not match the operation.
    Binding,
    /// JSON events did not form a complete checked response.
    Json,
}
impl core::fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Schema => "crates.io discovery schema mismatch",
            Self::Limit => "crates.io discovery bound exceeded",
            Self::Value => "crates.io discovery value is invalid",
            Self::Allocation => "crates.io discovery allocation failed",
            Self::Binding => "crates.io discovery binding mismatch",
            Self::Json => "crates.io discovery JSON is invalid",
        })
    }
}
impl core::error::Error for DiscoveryError {}

/// SDK response ceiling. Unknown fields remain subject to the same limits.
pub const MAX_DISCOVERY_BYTES: usize = 8_388_608;
/// SDK maximum for non-paginated arrays; list pages are additionally capped at 100.
pub const MAX_DISCOVERY_ITEMS: usize = 1_024;

#[cfg(all(test, feature = "alloc"))]
mod tests;
