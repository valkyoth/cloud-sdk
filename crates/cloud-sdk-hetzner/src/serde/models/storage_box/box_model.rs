use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use cloud_sdk_sanitization::sanitize_string;

use super::super::{Labels, Location, ResponseModelError, UtcTimestamp, parse_location, required};
use super::common::{
    AccessSettings, MAX_CONSOLE_ITEMS, Protection, SnapshotPlan, StorageBoxType, parse_access,
    parse_model_labels, parse_protection, parse_snapshot_plan, parse_type,
};
use super::parse::{
    number, object_mut, positive, required_mut, take_optional_text, take_text, take_timestamp,
};
use crate::pagination::PaginationMetadata;
use crate::serde::models::parse_pagination;
use crate::serde::strict_json::Value;

/// Current Storage Box lifecycle state.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBoxStatus {
    /// Ready for use.
    Active,
    /// Still being provisioned.
    Initializing,
    /// Administratively locked.
    Locked,
}

/// Storage usage counters in bytes.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxStats {
    /// Total current disk use.
    pub size: u64,
    /// Data disk use.
    pub size_data: u64,
    /// Snapshot disk use.
    pub size_snapshots: u64,
}

/// One source-complete Hetzner Console Storage Box.
///
/// Whole-model equality is intentionally unavailable because dynamic provider
/// text must not acquire a variable-time comparison API.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::serde::StorageBox;
/// fn compare(left: &StorageBox, right: &StorageBox) -> bool { left == right }
/// ```
#[non_exhaustive]
pub struct StorageBox {
    id: u64,
    name: String,
    storage_box_type: StorageBoxType,
    location: Location,
    access_settings: AccessSettings,
    snapshot_plan: Option<SnapshotPlan>,
    protection: Protection,
    labels: Labels,
    status: StorageBoxStatus,
    username: Option<String>,
    server: Option<String>,
    system: Option<String>,
    stats: StorageBoxStats,
    created: UtcTimestamp,
}

impl StorageBox {
    /// Returns the provider identifier.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the resource name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the embedded product type.
    #[must_use]
    pub const fn storage_box_type(&self) -> &StorageBoxType {
        &self.storage_box_type
    }

    /// Returns the physical location.
    #[must_use]
    pub const fn location(&self) -> &Location {
        &self.location
    }

    /// Returns access settings.
    #[must_use]
    pub const fn access_settings(&self) -> AccessSettings {
        self.access_settings
    }

    /// Returns the active snapshot plan.
    #[must_use]
    pub const fn snapshot_plan(&self) -> Option<SnapshotPlan> {
        self.snapshot_plan
    }

    /// Returns deletion protection.
    #[must_use]
    pub const fn protection(&self) -> Protection {
        self.protection
    }

    /// Returns user-defined labels.
    #[must_use]
    pub const fn labels(&self) -> &Labels {
        &self.labels
    }

    /// Returns lifecycle status.
    #[must_use]
    pub const fn status(&self) -> StorageBoxStatus {
        self.status
    }

    /// Returns the primary username when initialized.
    #[must_use]
    pub fn username(&self) -> Option<&str> {
        self.username.as_deref()
    }

    /// Returns the service FQDN when initialized.
    #[must_use]
    pub fn server(&self) -> Option<&str> {
        self.server.as_deref()
    }

    /// Returns the host system when initialized.
    #[must_use]
    pub fn system(&self) -> Option<&str> {
        self.system.as_deref()
    }

    /// Returns current disk usage.
    #[must_use]
    pub const fn stats(&self) -> StorageBoxStats {
        self.stats
    }

    /// Returns the canonical creation timestamp.
    #[must_use]
    pub fn created(&self) -> &str {
        self.created.as_str()
    }
}

impl fmt::Debug for StorageBox {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StorageBox")
            .field("id", &"[redacted]")
            .field("status", &self.status)
            .field("name", &"[redacted]")
            .field("connection_identity", &"[redacted]")
            .field("labels", &self.labels)
            .finish()
    }
}

impl Drop for StorageBox {
    fn drop(&mut self) {
        sanitize_string(&mut self.name);
        for value in [&mut self.username, &mut self.server, &mut self.system]
            .into_iter()
            .flatten()
        {
            sanitize_string(value);
        }
    }
}

/// Source-complete `list_storage_boxes` response.
#[non_exhaustive]
#[derive(Debug)]
pub struct StorageBoxPage {
    storage_boxes: Vec<StorageBox>,
    pagination: PaginationMetadata,
}

impl StorageBoxPage {
    /// Returns boxes in source order.
    #[must_use]
    pub fn storage_boxes(&self) -> &[StorageBox] {
        &self.storage_boxes
    }

    /// Returns validated pagination metadata.
    #[must_use]
    pub const fn pagination(&self) -> PaginationMetadata {
        self.pagination
    }
}

pub(crate) fn parse_storage_box_page(
    value: &mut Value,
) -> Result<StorageBoxPage, ResponseModelError> {
    let envelope = object_mut(value)?;
    let pagination = parse_pagination(required(envelope, "meta")?)?;
    let values = required_mut(envelope, "storage_boxes")?
        .as_array_mut()
        .ok_or(ResponseModelError::WrongType)?;
    if values.len() > usize::from(pagination.per_page().get()) || values.len() > MAX_CONSOLE_ITEMS {
        return Err(ResponseModelError::InvalidPagination);
    }
    let mut storage_boxes = Vec::new();
    storage_boxes
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        storage_boxes.push(parse_storage_box(value)?);
    }
    Ok(StorageBoxPage {
        storage_boxes,
        pagination,
    })
}

pub(crate) fn parse_storage_box(value: &mut Value) -> Result<StorageBox, ResponseModelError> {
    let fields = object_mut(value)?;
    let status = parse_status(required(fields, "status")?)?;
    let username = take_optional_text(fields, "username", 256)?;
    let server = take_optional_text(fields, "server", 512)?;
    let system = take_optional_text(fields, "system", 256)?;
    let snapshot_plan = parse_snapshot_plan(required(fields, "snapshot_plan")?)?;
    let initialized_fields = [username.is_some(), server.is_some(), system.is_some()];
    if (status == StorageBoxStatus::Initializing && initialized_fields.iter().any(|set| *set))
        || (status == StorageBoxStatus::Initializing && snapshot_plan.is_some())
        || (status != StorageBoxStatus::Initializing && initialized_fields.iter().any(|set| !*set))
    {
        return Err(ResponseModelError::EnvelopeMismatch);
    }
    let id = positive(fields, "id")?;
    let name = take_text(fields, "name", 256)?;
    let storage_box_type = parse_type(required_mut(fields, "storage_box_type")?)?;
    let location = parse_location(required(fields, "location")?)?;
    let access_settings = parse_access(required(fields, "access_settings")?)?;
    let protection = parse_protection(required(fields, "protection")?)?;
    let labels = parse_model_labels(required(fields, "labels")?)?;
    let stats = parse_stats(required(fields, "stats")?)?;
    let created = take_timestamp(fields, "created")?;
    Ok(StorageBox {
        id,
        name: name.into_inner(),
        storage_box_type,
        location,
        access_settings,
        snapshot_plan,
        protection,
        labels,
        status,
        username: username.map(|value| value.into_inner()),
        server: server.map(|value| value.into_inner()),
        system: system.map(|value| value.into_inner()),
        stats,
        created,
    })
}

fn parse_stats(value: &Value) -> Result<StorageBoxStats, ResponseModelError> {
    let fields = value.as_object().ok_or(ResponseModelError::WrongType)?;
    Ok(StorageBoxStats {
        size: number(fields, "size")?,
        size_data: number(fields, "size_data")?,
        size_snapshots: number(fields, "size_snapshots")?,
    })
}

fn parse_status(value: &Value) -> Result<StorageBoxStatus, ResponseModelError> {
    value
        .try_with_str(|value| match value {
            "active" => Some(StorageBoxStatus::Active),
            "initializing" => Some(StorageBoxStatus::Initializing),
            "locked" => Some(StorageBoxStatus::Locked),
            _ => None,
        })
        .map_err(|_| ResponseModelError::InvalidText)?
        .flatten()
        .ok_or(ResponseModelError::UnknownEnumValue)
}
