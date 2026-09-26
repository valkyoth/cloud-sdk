#[cfg(feature = "blocking")]
mod bundled;
#[cfg(feature = "async")]
mod bundled_async;
mod metadata;
mod request;
mod response;
mod schema_table;
mod stream;
mod target;
mod validation;
pub use crate::discovery::DiscoveryError as PublishError;
pub use metadata::PublishMetadata;
pub use request::{PublishPermit, PublishRequest};
pub use response::PublishResponse;
pub use stream::{PublishUpload, SlicePackage};
#[cfg(any(feature = "blocking", feature = "async"))]
mod client;
#[cfg(any(feature = "blocking", feature = "async"))]
pub use crate::discovery::DiscoveryExecutionError as PublishExecutionError;
#[cfg(any(feature = "blocking", feature = "async"))]
pub use client::{PublishBuffers, PublishClient};
#[cfg(test)]
mod tests;

/// Local metadata ceiling, lower than the registry's one-MiB framing ceiling.
pub const MAX_PUBLISH_METADATA_BYTES: usize = 131_072;
/// Local compressed archive ceiling; registry/account limits may be lower.
pub const MAX_PUBLISH_ARCHIVE_BYTES: u64 = 536_870_912;
