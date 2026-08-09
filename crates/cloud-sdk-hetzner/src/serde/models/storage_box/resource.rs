use core::fmt;

use super::super::ResponseModelError;
use super::box_model::{StorageBox, parse_storage_box};
use super::parse::{object_mut, positive};
use crate::serde::strict_json::Value;

/// Minimal snapshot identity returned by `create_storage_box_snapshot`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxSnapshotReference {
    id: u64,
    storage_box: u64,
}

impl StorageBoxSnapshotReference {
    /// Returns the snapshot identifier.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the parent Storage Box identifier.
    #[must_use]
    pub const fn storage_box(&self) -> u64 {
        self.storage_box
    }
}

/// Minimal subaccount identity returned by `create_storage_box_subaccount`.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxSubaccountReference {
    id: u64,
    storage_box: u64,
}

impl StorageBoxSubaccountReference {
    /// Returns the subaccount identifier.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the parent Storage Box identifier.
    #[must_use]
    pub const fn storage_box(&self) -> u64 {
        self.storage_box
    }
}

/// Source-locked resource supplied by a Console create composite.
#[non_exhaustive]
pub enum StorageBoxResource {
    /// Complete Storage Box returned by `create_storage_box`.
    StorageBox(StorageBox),
    /// Minimal snapshot identity returned while creation is asynchronous.
    SnapshotReference(StorageBoxSnapshotReference),
    /// Minimal subaccount identity returned while creation is asynchronous.
    SubaccountReference(StorageBoxSubaccountReference),
}

impl fmt::Debug for StorageBoxResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StorageBox(value) => formatter.debug_tuple("StorageBox").field(value).finish(),
            Self::SnapshotReference(value) => formatter
                .debug_tuple("SnapshotReference")
                .field(value)
                .finish(),
            Self::SubaccountReference(value) => formatter
                .debug_tuple("SubaccountReference")
                .field(value)
                .finish(),
        }
    }
}

pub(crate) fn parse_storage_box_composite_resource(
    operation: &str,
    value: &mut Value,
) -> Result<StorageBoxResource, ResponseModelError> {
    match operation {
        "create_storage_box" => parse_storage_box(value).map(StorageBoxResource::StorageBox),
        "create_storage_box_snapshot" => {
            parse_snapshot_reference(value).map(StorageBoxResource::SnapshotReference)
        }
        "create_storage_box_subaccount" => {
            parse_subaccount_reference(value).map(StorageBoxResource::SubaccountReference)
        }
        _ => Err(ResponseModelError::EnvelopeMismatch),
    }
}

fn parse_snapshot_reference(
    value: &mut Value,
) -> Result<StorageBoxSnapshotReference, ResponseModelError> {
    let fields = object_mut(value)?;
    Ok(StorageBoxSnapshotReference {
        id: positive(fields, "id")?,
        storage_box: positive(fields, "storage_box")?,
    })
}

fn parse_subaccount_reference(
    value: &mut Value,
) -> Result<StorageBoxSubaccountReference, ResponseModelError> {
    let fields = object_mut(value)?;
    Ok(StorageBoxSubaccountReference {
        id: positive(fields, "id")?,
        storage_box: positive(fields, "storage_box")?,
    })
}
