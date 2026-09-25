mod decode;
mod identity;
mod preflight;
mod request;
pub use crate::discovery::DiscoveryError as OwnerChangeError;
pub use decode::{OwnerChangeResponse, OwnershipOutcome};
pub use identity::OwnerSelector;
pub use preflight::{RemovalSnapshot, SelfRemoval};
pub use request::{OwnerChangeOperation, OwnerChangePermit, OwnerChangeRequest};
#[cfg(feature = "blocking")]
mod client;
#[cfg(feature = "blocking")]
pub use crate::discovery::DiscoveryExecutionError as OwnerChangeExecutionError;
#[cfg(feature = "blocking")]
pub use client::{OwnerChangeBuffers, OwnerChangeClient};
#[cfg(test)]
mod tests;

/// Source add limit; also the SDK's conservative removal-batch limit.
pub const MAX_OWNER_CHANGES: usize = 10;
/// Bounded JSON body storage for ten validated selectors.
pub const MAX_OWNER_CHANGE_BODY_BYTES: usize = 4096;
