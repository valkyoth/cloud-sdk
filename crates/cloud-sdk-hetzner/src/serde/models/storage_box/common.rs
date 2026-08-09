use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use cloud_sdk_sanitization::sanitize_string;

use super::super::{ResponseModelError, UtcTimestamp, WipeString, object, parse_labels};
use super::parse::{
    boolean, nullable_ranged_u8, nullable_u64, number, object_mut, positive, ranged_u8,
    required_mut, take_text, take_text_allow_empty, take_timestamp,
};
use crate::pagination::MAX_PER_PAGE;
use crate::serde::models::Labels;
use crate::serde::strict_json::{Map, Value};

pub(super) const MAX_CONSOLE_ITEMS: usize = 1_024;
pub(super) const MAX_LABELS: usize = 64;
const MAX_PRICES: usize = MAX_PER_PAGE as usize;

/// Decimal monetary amount preserved as provider text.
#[non_exhaustive]
#[derive(Eq, PartialEq)]
pub struct Money {
    net: String,
    gross: String,
}

impl Money {
    /// Returns the amount without VAT.
    #[must_use]
    pub fn net(&self) -> &str {
        &self.net
    }

    /// Returns the amount including VAT.
    #[must_use]
    pub fn gross(&self) -> &str {
        &self.gross
    }
}

impl fmt::Debug for Money {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Money([redacted])")
    }
}

impl Drop for Money {
    fn drop(&mut self) {
        sanitize_string(&mut self.net);
        sanitize_string(&mut self.gross);
    }
}

/// Storage Box price at one location.
#[non_exhaustive]
#[derive(Eq, PartialEq)]
pub struct Price {
    location: String,
    hourly: Money,
    monthly: Money,
    setup_fee: Money,
}

impl Price {
    /// Returns the location name.
    #[must_use]
    pub fn location(&self) -> &str {
        &self.location
    }

    /// Returns the hourly price.
    #[must_use]
    pub const fn hourly(&self) -> &Money {
        &self.hourly
    }

    /// Returns the monthly price.
    #[must_use]
    pub const fn monthly(&self) -> &Money {
        &self.monthly
    }

    /// Returns the one-time setup fee.
    #[must_use]
    pub const fn setup_fee(&self) -> &Money {
        &self.setup_fee
    }
}

impl fmt::Debug for Price {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Price")
            .field("location", &"[redacted]")
            .field("amounts", &"[redacted]")
            .finish()
    }
}

impl Drop for Price {
    fn drop(&mut self) {
        sanitize_string(&mut self.location);
    }
}

/// Provider deprecation interval.
#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub struct Deprecation {
    unavailable_after: UtcTimestamp,
    announced: UtcTimestamp,
}

impl Deprecation {
    /// Returns the removal timestamp.
    #[must_use]
    pub fn unavailable_after(&self) -> &str {
        self.unavailable_after.as_str()
    }

    /// Returns the announcement timestamp.
    #[must_use]
    pub fn announced(&self) -> &str {
        self.announced.as_str()
    }
}

/// Source-complete Storage Box type.
#[non_exhaustive]
#[derive(PartialEq)]
pub struct StorageBoxType {
    id: u64,
    name: String,
    description: String,
    snapshot_limit: Option<u64>,
    automatic_snapshot_limit: Option<u64>,
    subaccounts_limit: u64,
    size: u64,
    prices: Vec<Price>,
    deprecation: Option<Deprecation>,
}

impl StorageBoxType {
    /// Returns the provider resource identifier.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the provider type name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the human-readable description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the manual snapshot limit.
    #[must_use]
    pub const fn snapshot_limit(&self) -> Option<u64> {
        self.snapshot_limit
    }

    /// Returns the automatic snapshot limit.
    #[must_use]
    pub const fn automatic_snapshot_limit(&self) -> Option<u64> {
        self.automatic_snapshot_limit
    }

    /// Returns the subaccount limit.
    #[must_use]
    pub const fn subaccounts_limit(&self) -> u64 {
        self.subaccounts_limit
    }

    /// Returns available storage in bytes.
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// Returns location-specific prices.
    #[must_use]
    pub fn prices(&self) -> &[Price] {
        &self.prices
    }

    /// Returns the deprecation interval when present.
    #[must_use]
    pub const fn deprecation(&self) -> Option<&Deprecation> {
        self.deprecation.as_ref()
    }
}

impl fmt::Debug for StorageBoxType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StorageBoxType")
            .field("id", &self.id)
            .field("name", &"[redacted]")
            .field("price_count", &self.prices.len())
            .field("deprecated", &self.deprecation.is_some())
            .finish()
    }
}

impl Drop for StorageBoxType {
    fn drop(&mut self) {
        sanitize_string(&mut self.name);
        sanitize_string(&mut self.description);
    }
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

pub(super) fn parse_type(value: &mut Value) -> Result<StorageBoxType, ResponseModelError> {
    let fields = object_mut(value)?;
    let price_values = required_mut(fields, "prices")?
        .as_array_mut()
        .ok_or(ResponseModelError::WrongType)?;
    if price_values.len() > MAX_PRICES {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut prices = Vec::new();
    prices
        .try_reserve_exact(price_values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for price in price_values {
        prices.push(parse_price(price)?);
    }
    let id = positive(fields, "id")?;
    let name = take_text(fields, "name", 128)?;
    let description = take_text_allow_empty(fields, "description", 1_024)?;
    let snapshot_limit = nullable_u64(fields, "snapshot_limit")?;
    let automatic_snapshot_limit = nullable_u64(fields, "automatic_snapshot_limit")?;
    let subaccounts_limit = number(fields, "subaccounts_limit")?;
    let size = number(fields, "size")?;
    let deprecation = parse_deprecation(required_mut(fields, "deprecation")?)?;
    Ok(StorageBoxType {
        id,
        name: name.into_inner(),
        description: description.into_inner(),
        snapshot_limit,
        automatic_snapshot_limit,
        subaccounts_limit,
        size,
        prices,
        deprecation,
    })
}

fn parse_price(value: &mut Value) -> Result<Price, ResponseModelError> {
    let fields = object_mut(value)?;
    let location = take_text(fields, "location", 128)?;
    let hourly = parse_money(required_mut(fields, "price_hourly")?)?;
    let monthly = parse_money(required_mut(fields, "price_monthly")?)?;
    let setup_fee = parse_money(required_mut(fields, "setup_fee")?)?;
    Ok(Price {
        location: location.into_inner(),
        hourly,
        monthly,
        setup_fee,
    })
}

fn parse_money(value: &mut Value) -> Result<Money, ResponseModelError> {
    let fields = object_mut(value)?;
    let net = take_decimal(fields, "net")?;
    let gross = take_decimal(fields, "gross")?;
    Ok(Money {
        net: net.into_inner(),
        gross: gross.into_inner(),
    })
}

fn take_decimal(fields: &mut Map, key: &str) -> Result<WipeString, ResponseModelError> {
    let value = take_text(fields, key, 128)?;
    let mut parts = value.as_str().split('.');
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

fn parse_deprecation(value: &mut Value) -> Result<Option<Deprecation>, ResponseModelError> {
    if value.is_null() {
        return Ok(None);
    }
    let fields = object_mut(value)?;
    Ok(Some(Deprecation {
        unavailable_after: take_timestamp(fields, "unavailable_after")?,
        announced: take_timestamp(fields, "announced")?,
    }))
}

pub(super) fn parse_access(value: &Value) -> Result<AccessSettings, ResponseModelError> {
    let fields = object(value)?;
    Ok(AccessSettings {
        reachable_externally: boolean(fields, "reachable_externally")?,
        samba_enabled: boolean(fields, "samba_enabled")?,
        ssh_enabled: boolean(fields, "ssh_enabled")?,
        webdav_enabled: boolean(fields, "webdav_enabled")?,
        zfs_enabled: boolean(fields, "zfs_enabled")?,
    })
}

pub(super) fn parse_snapshot_plan(
    value: &Value,
) -> Result<Option<SnapshotPlan>, ResponseModelError> {
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

pub(super) fn parse_protection(value: &Value) -> Result<Protection, ResponseModelError> {
    let fields = object(value)?;
    Ok(Protection {
        delete: boolean(fields, "delete")?,
    })
}

pub(super) fn parse_model_labels(value: &Value) -> Result<Labels, ResponseModelError> {
    parse_labels(value, MAX_LABELS)
}
