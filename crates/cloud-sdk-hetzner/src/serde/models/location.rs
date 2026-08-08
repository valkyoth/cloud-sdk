//! Source-complete location response models.

use alloc::string::String;
use alloc::vec::Vec;

use super::{ResponseModelError, object, parse_pagination, required, value_text};
use crate::pagination::PaginationMetadata;
use crate::serde::strict_json::Value;

/// One source-complete Hetzner Cloud location.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    /// Provider resource identifier.
    pub id: u64,
    /// Unique location name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// ISO 3166-1 alpha-2 country code.
    pub country: String,
    /// Closest city.
    pub city: String,
    /// Geographic latitude.
    pub latitude: f64,
    /// Geographic longitude.
    pub longitude: f64,
    /// Provider network-zone name.
    pub network_zone: String,
}

/// Source-complete `list_locations` response.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct LocationPage {
    /// Locations returned on this page.
    pub locations: Vec<Location>,
    /// Validated one-based pagination metadata.
    pub pagination: PaginationMetadata,
}

pub(crate) fn parse_location_page(value: &Value) -> Result<LocationPage, ResponseModelError> {
    let envelope = object(value)?;
    let pagination = parse_pagination(required(envelope, "meta")?)?;
    let values = required(envelope, "locations")?
        .as_array()
        .ok_or(ResponseModelError::WrongType)?;
    if values.len() > usize::from(pagination.per_page().get()) {
        return Err(ResponseModelError::InvalidPagination);
    }
    let mut locations = Vec::new();
    locations
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        locations.push(parse_location(value)?);
    }
    Ok(LocationPage {
        locations,
        pagination,
    })
}

pub(crate) fn parse_location(value: &Value) -> Result<Location, ResponseModelError> {
    let fields = object(value)?;
    let id = positive_u64(fields, "id")?;
    let latitude = finite_number(fields, "latitude")?;
    let longitude = finite_number(fields, "longitude")?;
    if !(-90.0..=90.0).contains(&latitude) || !(-180.0..=180.0).contains(&longitude) {
        return Err(ResponseModelError::InvalidNumber);
    }
    Ok(Location {
        id,
        name: text(fields, "name", 128)?,
        description: text(fields, "description", 1_024)?,
        country: text(fields, "country", 2)?,
        city: text(fields, "city", 256)?,
        latitude,
        longitude,
        network_zone: text(fields, "network_zone", 128)?,
    })
}

fn text(
    fields: &crate::serde::strict_json::Map,
    key: &str,
    max: usize,
) -> Result<String, ResponseModelError> {
    value_text(required(fields, key)?, max)
}

fn positive_u64(
    fields: &crate::serde::strict_json::Map,
    key: &str,
) -> Result<u64, ResponseModelError> {
    required(fields, key)?
        .as_u64()
        .filter(|value| *value != 0 && *value <= 9_007_199_254_740_991)
        .ok_or(ResponseModelError::InvalidNumber)
}

fn finite_number(
    fields: &crate::serde::strict_json::Map,
    key: &str,
) -> Result<f64, ResponseModelError> {
    required(fields, key)?
        .as_f64()
        .filter(|value| value.is_finite())
        .ok_or(ResponseModelError::InvalidNumber)
}
