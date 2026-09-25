//! Anonymous public users, teams, statistics and ownership inspection.
//! Returned identities are metadata, never mutation authority.
#[cfg(feature = "alloc")]
pub mod personal;
mod request;
#[cfg(feature = "alloc")]
pub mod tokens;
pub use crate::discovery::DiscoveryError as AccountError;
pub use request::{AccountOperation, AccountRequest};
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
pub use crate::discovery::DiscoveryExecutionError as AccountExecutionError;
#[cfg(any(feature = "blocking", feature = "async"))]
pub use client::AccountClient;
#[cfg(all(test, feature = "alloc"))]
mod tests;

/// SDK cap for one unpaginated owner response. Oversize lists fail, never truncate.
pub const MAX_OWNERS: usize = 256;
/// SDK cap for the explicitly requested public linked-account list.
pub const MAX_LINKED_ACCOUNTS: usize = 16;
