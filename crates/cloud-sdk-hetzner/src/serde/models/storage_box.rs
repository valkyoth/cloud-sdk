//! Source-complete Storage Box response models.

use alloc::string::String;
use alloc::vec::Vec;

use super::location::parse_location;
use super::{
    Labels, Location, ResponseModelError, object, parse_labels, parse_pagination, required,
    value_text,
};
use crate::pagination::{MAX_PER_PAGE, PaginationMetadata};
use crate::serde::strict_json::{Map, Value};

const MAX_PRICES: usize = MAX_PER_PAGE as usize;
const MAX_LABELS: usize = 64;

/// Decimal monetary amount preserved as provider text.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Money {
    /// Amount without VAT.
    pub net: String,
    /// Amount including VAT.
    pub gross: String,
}

/// Storage Box price at one location.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Price {
    /// Location name.
    pub location: String,
    /// Hourly price.
    pub hourly: Money,
    /// Monthly price.
    pub monthly: Money,
    /// One-time setup fee.
    pub setup_fee: Money,
}

/// Provider deprecation interval.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deprecation {
    /// Removal timestamp.
    pub unavailable_after: String,
    /// Announcement timestamp.
    pub announced: String,
}

/// Source-complete Storage Box type embedded in a box response.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StorageBoxType {
    /// Provider resource identifier.
    pub id: u64,
    /// Provider type name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Manual snapshot limit.
    pub snapshot_limit: Option<u64>,
    /// Automatic snapshot limit.
    pub automatic_snapshot_limit: Option<u64>,
    /// Subaccount limit.
    pub subaccounts_limit: u64,
    /// Available storage in bytes.
    pub size: u64,
    /// Location-specific prices.
    pub prices: Vec<Price>,
    /// Deprecation interval, if any.
    pub deprecation: Option<Deprecation>,
}

/// Storage Box access switches.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccessSettings {
    /// Whether access outside Hetzner's network is allowed.
    pub reachable_externally: bool,
    /// Whether Samba is enabled.
    pub samba_enabled: bool,
    /// Whether SSH is enabled.
    pub ssh_enabled: bool,
    /// Whether WebDAV is enabled.
    pub webdav_enabled: bool,
    /// Whether the ZFS snapshot folder is visible.
    pub zfs_enabled: bool,
}

/// Active automatic snapshot schedule.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotPlan {
    /// Retained snapshot limit.
    pub max_snapshots: u64,
    /// UTC minute.
    pub minute: u8,
    /// UTC hour.
    pub hour: u8,
    /// Optional ISO weekday, Monday = 1.
    pub day_of_week: Option<u8>,
    /// Optional day of month.
    pub day_of_month: Option<u8>,
}

/// Storage Box deletion protection.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Protection {
    /// Whether deletion is prevented.
    pub delete: bool,
}

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
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct StorageBox {
    /// Provider resource identifier.
    pub id: u64,
    /// Resource name.
    pub name: String,
    /// Embedded type.
    pub storage_box_type: StorageBoxType,
    /// Physical location.
    pub location: Location,
    /// Access switches.
    pub access_settings: AccessSettings,
    /// Active snapshot plan.
    pub snapshot_plan: Option<SnapshotPlan>,
    /// Deletion protection.
    pub protection: Protection,
    /// User labels.
    pub labels: Labels,
    /// Lifecycle state.
    pub status: StorageBoxStatus,
    /// Primary username when initialized.
    pub username: Option<String>,
    /// Service FQDN when initialized.
    pub server: Option<String>,
    /// Host system when initialized.
    pub system: Option<String>,
    /// Current disk usage.
    pub stats: StorageBoxStats,
    /// Creation timestamp text.
    pub created: String,
}

/// Source-complete `list_storage_boxes` response.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct StorageBoxPage {
    /// Boxes returned on this page.
    pub storage_boxes: Vec<StorageBox>,
    /// Validated one-based pagination metadata.
    pub pagination: PaginationMetadata,
}

pub(crate) fn parse_storage_box_page(value: &Value) -> Result<StorageBoxPage, ResponseModelError> {
    let envelope = object(value)?;
    let pagination = parse_pagination(required(envelope, "meta")?)?;
    let values = required(envelope, "storage_boxes")?
        .as_array()
        .ok_or(ResponseModelError::WrongType)?;
    if values.len() > usize::from(pagination.per_page().get()) {
        return Err(ResponseModelError::InvalidPagination);
    }
    let mut storage_boxes = Vec::new();
    storage_boxes
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        storage_boxes.push(parse_box(value)?);
    }
    Ok(StorageBoxPage {
        storage_boxes,
        pagination,
    })
}

fn parse_box(value: &Value) -> Result<StorageBox, ResponseModelError> {
    let fields = object(value)?;
    Ok(StorageBox {
        id: positive(fields, "id")?,
        name: text(fields, "name", 256)?,
        storage_box_type: parse_type(required(fields, "storage_box_type")?)?,
        location: parse_location(required(fields, "location")?)?,
        access_settings: parse_access(required(fields, "access_settings")?)?,
        snapshot_plan: parse_snapshot_plan(required(fields, "snapshot_plan")?)?,
        protection: parse_protection(required(fields, "protection")?)?,
        labels: parse_labels(required(fields, "labels")?, MAX_LABELS)?,
        status: parse_status(required(fields, "status")?)?,
        username: nullable_text(fields, "username", 256)?,
        server: nullable_text(fields, "server", 512)?,
        system: nullable_text(fields, "system", 256)?,
        stats: parse_stats(required(fields, "stats")?)?,
        created: text(fields, "created", 64)?,
    })
}

fn parse_type(value: &Value) -> Result<StorageBoxType, ResponseModelError> {
    let fields = object(value)?;
    let prices = required(fields, "prices")?
        .as_array()
        .ok_or(ResponseModelError::WrongType)?;
    if prices.len() > MAX_PRICES {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut parsed_prices = Vec::new();
    parsed_prices
        .try_reserve_exact(prices.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for price in prices {
        parsed_prices.push(parse_price(price)?);
    }
    Ok(StorageBoxType {
        id: positive(fields, "id")?,
        name: text(fields, "name", 128)?,
        description: text(fields, "description", 256)?,
        snapshot_limit: nullable_u64(fields, "snapshot_limit")?,
        automatic_snapshot_limit: nullable_u64(fields, "automatic_snapshot_limit")?,
        subaccounts_limit: number(fields, "subaccounts_limit")?,
        size: number(fields, "size")?,
        prices: parsed_prices,
        deprecation: parse_deprecation(required(fields, "deprecation")?)?,
    })
}

fn parse_price(value: &Value) -> Result<Price, ResponseModelError> {
    let fields = object(value)?;
    Ok(Price {
        location: text(fields, "location", 128)?,
        hourly: parse_money(required(fields, "price_hourly")?)?,
        monthly: parse_money(required(fields, "price_monthly")?)?,
        setup_fee: parse_money(required(fields, "setup_fee")?)?,
    })
}

fn parse_money(value: &Value) -> Result<Money, ResponseModelError> {
    let fields = object(value)?;
    Ok(Money {
        net: decimal(fields, "net")?,
        gross: decimal(fields, "gross")?,
    })
}

fn parse_deprecation(value: &Value) -> Result<Option<Deprecation>, ResponseModelError> {
    if value.is_null() {
        return Ok(None);
    }
    let fields = object(value)?;
    Ok(Some(Deprecation {
        unavailable_after: text(fields, "unavailable_after", 64)?,
        announced: text(fields, "announced", 64)?,
    }))
}

fn parse_access(value: &Value) -> Result<AccessSettings, ResponseModelError> {
    let fields = object(value)?;
    Ok(AccessSettings {
        reachable_externally: boolean(fields, "reachable_externally")?,
        samba_enabled: boolean(fields, "samba_enabled")?,
        ssh_enabled: boolean(fields, "ssh_enabled")?,
        webdav_enabled: boolean(fields, "webdav_enabled")?,
        zfs_enabled: boolean(fields, "zfs_enabled")?,
    })
}

fn parse_snapshot_plan(value: &Value) -> Result<Option<SnapshotPlan>, ResponseModelError> {
    if value.is_null() {
        return Ok(None);
    }
    let fields = object(value)?;
    Ok(Some(SnapshotPlan {
        max_snapshots: positive(fields, "max_snapshots")?,
        minute: ranged_u8(fields, "minute", 0, 59)?,
        hour: ranged_u8(fields, "hour", 0, 23)?,
        day_of_week: nullable_ranged_u8(fields, "day_of_week", 1, 7)?,
        day_of_month: nullable_ranged_u8(fields, "day_of_month", 1, 31)?,
    }))
}

fn parse_protection(value: &Value) -> Result<Protection, ResponseModelError> {
    let fields = object(value)?;
    Ok(Protection {
        delete: boolean(fields, "delete")?,
    })
}

fn parse_stats(value: &Value) -> Result<StorageBoxStats, ResponseModelError> {
    let fields = object(value)?;
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

fn decimal(fields: &Map, key: &str) -> Result<String, ResponseModelError> {
    let value = text(fields, key, 128)?;
    let mut parts = value.split('.');
    let whole = parts.next().unwrap_or("");
    let fraction = parts.next();
    if whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || fraction
            .is_some_and(|part| part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()))
        || parts.next().is_some()
    {
        return Err(ResponseModelError::InvalidNumber);
    }
    Ok(value)
}

fn text(fields: &Map, key: &str, max: usize) -> Result<String, ResponseModelError> {
    value_text(required(fields, key)?, max)
}

fn nullable_text(
    fields: &Map,
    key: &str,
    max: usize,
) -> Result<Option<String>, ResponseModelError> {
    let value = required(fields, key)?;
    if value.is_null() {
        Ok(None)
    } else {
        value_text(value, max).map(Some)
    }
}

fn positive(fields: &Map, key: &str) -> Result<u64, ResponseModelError> {
    let value = number(fields, key)?;
    (1..=9_007_199_254_740_991)
        .contains(&value)
        .then_some(value)
        .ok_or(ResponseModelError::InvalidNumber)
}

fn number(fields: &Map, key: &str) -> Result<u64, ResponseModelError> {
    required(fields, key)?
        .as_u64()
        .ok_or(ResponseModelError::InvalidNumber)
}

fn nullable_u64(fields: &Map, key: &str) -> Result<Option<u64>, ResponseModelError> {
    let value = required(fields, key)?;
    if value.is_null() {
        Ok(None)
    } else {
        value
            .as_u64()
            .map(Some)
            .ok_or(ResponseModelError::InvalidNumber)
    }
}

fn boolean(fields: &Map, key: &str) -> Result<bool, ResponseModelError> {
    required(fields, key)?
        .as_bool()
        .ok_or(ResponseModelError::WrongType)
}

fn ranged_u8(fields: &Map, key: &str, min: u8, max: u8) -> Result<u8, ResponseModelError> {
    let value = number(fields, key)
        .and_then(|value| u8::try_from(value).map_err(|_| ResponseModelError::InvalidNumber))?;
    (min..=max)
        .contains(&value)
        .then_some(value)
        .ok_or(ResponseModelError::InvalidNumber)
}

fn nullable_ranged_u8(
    fields: &Map,
    key: &str,
    min: u8,
    max: u8,
) -> Result<Option<u8>, ResponseModelError> {
    nullable_u64(fields, key)?
        .map(|value| {
            let value = u8::try_from(value).map_err(|_| ResponseModelError::InvalidNumber)?;
            (min..=max)
                .contains(&value)
                .then_some(value)
                .ok_or(ResponseModelError::InvalidNumber)
        })
        .transpose()
}
