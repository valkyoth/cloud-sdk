//! Explicit personal mutations. Permits bind one action to one credential.
//! Server-side identity, expiry and permission checks remain authoritative.
mod decode;
mod permit;
mod request;
mod schema_table;
pub use decode::PersonalResponse;
pub use permit::PersonalPermit;
pub use request::{EmailAddress, NotificationUpdate, PersonalOperation, PersonalRequest};
#[cfg(feature = "blocking")]
mod client;
#[cfg(feature = "blocking")]
pub use crate::discovery::DiscoveryExecutionError as PersonalExecutionError;
#[cfg(feature = "blocking")]
pub use client::{PersonalBuffers, PersonalClient};
#[cfg(test)]
mod tests;

/// Local cap for the deprecated, unpaginated notification update batch.
pub const MAX_NOTIFICATION_UPDATES: usize = 64;
/// Local serialized request bound, including escaped email text.
pub const MAX_PERSONAL_BODY_BYTES: usize = 4096;
