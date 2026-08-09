use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use cloud_sdk_sanitization::sanitize_string;

use super::super::{Labels, ResponseModelError, UtcTimestamp, required};
use super::common::{MAX_CONSOLE_ITEMS, parse_model_labels};
use super::parse::{
    boolean, number, object_mut, positive, take_text, take_text_allow_empty, take_timestamp,
};
use crate::serde::strict_json::Value;

/// Storage usage counters for one snapshot.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxSnapshotStats {
    /// Current storage requirement in bytes.
    pub size: u64,
    /// Compressed filesystem size in bytes.
    pub size_filesystem: u64,
}

/// One source-complete Storage Box snapshot.
///
/// Whole-model equality is intentionally unavailable because dynamic provider
/// text must not acquire a variable-time comparison API.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::serde::StorageBoxSnapshot;
/// fn compare(left: &StorageBoxSnapshot, right: &StorageBoxSnapshot) -> bool { left == right }
/// ```
#[non_exhaustive]
pub struct StorageBoxSnapshot {
    storage_box: u64,
    id: u64,
    name: String,
    description: String,
    labels: Labels,
    stats: StorageBoxSnapshotStats,
    is_automatic: bool,
    created: UtcTimestamp,
}

impl StorageBoxSnapshot {
    /// Returns the parent Storage Box identifier.
    #[must_use]
    pub const fn storage_box(&self) -> u64 {
        self.storage_box
    }

    /// Returns the snapshot identifier.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the snapshot name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the snapshot description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns user-defined labels.
    #[must_use]
    pub const fn labels(&self) -> &Labels {
        &self.labels
    }

    /// Returns storage statistics.
    #[must_use]
    pub const fn stats(&self) -> StorageBoxSnapshotStats {
        self.stats
    }

    /// Reports whether the snapshot was created automatically.
    #[must_use]
    pub const fn is_automatic(&self) -> bool {
        self.is_automatic
    }

    /// Returns the canonical creation timestamp.
    #[must_use]
    pub fn created(&self) -> &str {
        self.created.as_str()
    }
}

impl fmt::Debug for StorageBoxSnapshot {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StorageBoxSnapshot")
            .field("storage_box", &"[redacted]")
            .field("id", &"[redacted]")
            .field("is_automatic", &self.is_automatic)
            .field("text", &"[redacted]")
            .field("labels", &self.labels)
            .finish()
    }
}

impl Drop for StorageBoxSnapshot {
    fn drop(&mut self) {
        sanitize_string(&mut self.name);
        sanitize_string(&mut self.description);
    }
}

pub(crate) fn parse_storage_box_snapshot(
    value: &mut Value,
) -> Result<StorageBoxSnapshot, ResponseModelError> {
    let fields = object_mut(value)?;
    let description = take_text_allow_empty(fields, "description", 1_000)?;
    if !description.as_str().bytes().all(valid_description_byte) {
        return Err(ResponseModelError::InvalidText);
    }
    let storage_box = positive(fields, "storage_box")?;
    let id = positive(fields, "id")?;
    let name = take_text(fields, "name", 256)?;
    let labels = parse_model_labels(required(fields, "labels")?)?;
    let stats = parse_stats(required(fields, "stats")?)?;
    let is_automatic = boolean(fields, "is_automatic")?;
    let created = take_timestamp(fields, "created")?;
    Ok(StorageBoxSnapshot {
        storage_box,
        id,
        name: name.into_inner(),
        description: description.into_inner(),
        labels,
        stats,
        is_automatic,
        created,
    })
}

pub(crate) fn parse_storage_box_snapshots(
    value: &mut Value,
) -> Result<Vec<StorageBoxSnapshot>, ResponseModelError> {
    let values = value.as_array_mut().ok_or(ResponseModelError::WrongType)?;
    if values.len() > MAX_CONSOLE_ITEMS {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut snapshots = Vec::new();
    snapshots
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        snapshots.push(parse_storage_box_snapshot(value)?);
    }
    Ok(snapshots)
}

fn parse_stats(value: &Value) -> Result<StorageBoxSnapshotStats, ResponseModelError> {
    let fields = value.as_object().ok_or(ResponseModelError::WrongType)?;
    Ok(StorageBoxSnapshotStats {
        size: number(fields, "size")?,
        size_filesystem: number(fields, "size_filesystem")?,
    })
}

fn valid_description_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'-' | b'_'
                | b','
                | b':'
                | b'<'
                | b'>'
                | b'+'
                | b'#'
                | b'!'
                | b'('
                | b')'
                | b'['
                | b']'
                | b'{'
                | b'}'
                | b' '
        )
}
