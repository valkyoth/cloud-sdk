//! Explicit crate/version settings patches. No automatic retries or lost-update protection.
mod decode;
mod request;
mod schema_table;
pub use crate::discovery::DiscoveryError as SettingsError;
pub use decode::SettingsResponse;
pub use request::{SettingsOperation, SettingsPermit, SettingsRequest, YankMessage};
#[cfg(feature = "blocking")]
mod client;
#[cfg(feature = "blocking")]
pub use crate::discovery::DiscoveryExecutionError as SettingsExecutionError;
#[cfg(feature = "blocking")]
pub use client::{SettingsBuffers, SettingsClient};
#[cfg(test)]
mod tests;

/// Local UTF-8 byte limit, not a claim about an upstream database limit.
pub const MAX_YANK_MESSAGE_BYTES: usize = 4096;
/// Worst-case escaped JSON plus envelope; caller storage is bounded and cleared.
pub const MAX_SETTINGS_BODY_BYTES: usize = 25_000;
