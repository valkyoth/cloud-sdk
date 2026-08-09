use alloc::vec::Vec;

use super::super::{ResponseModelError, parse_pagination, required};
use super::common::{MAX_CONSOLE_ITEMS, StorageBoxType, parse_type};
use super::parse::{object_mut, required_mut};
use crate::pagination::PaginationMetadata;
use crate::serde::strict_json::Value;

/// Source-complete `list_storage_box_types` response.
#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub struct StorageBoxTypePage {
    storage_box_types: Vec<StorageBoxType>,
    pagination: PaginationMetadata,
}

impl StorageBoxTypePage {
    /// Returns types in source order.
    #[must_use]
    pub fn storage_box_types(&self) -> &[StorageBoxType] {
        &self.storage_box_types
    }

    /// Returns validated pagination metadata.
    #[must_use]
    pub const fn pagination(&self) -> PaginationMetadata {
        self.pagination
    }
}

pub(crate) fn parse_storage_box_type_page(
    value: &mut Value,
) -> Result<StorageBoxTypePage, ResponseModelError> {
    let envelope = object_mut(value)?;
    let pagination = parse_pagination(required(envelope, "meta")?)?;
    let values = required_mut(envelope, "storage_box_types")?
        .as_array_mut()
        .ok_or(ResponseModelError::WrongType)?;
    if values.len() > usize::from(pagination.per_page().get()) || values.len() > MAX_CONSOLE_ITEMS {
        return Err(ResponseModelError::InvalidPagination);
    }
    let mut storage_box_types = Vec::new();
    storage_box_types
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        storage_box_types.push(parse_type(value)?);
    }
    Ok(StorageBoxTypePage {
        storage_box_types,
        pagination,
    })
}

pub(crate) fn parse_storage_box_type(
    value: &mut Value,
) -> Result<StorageBoxType, ResponseModelError> {
    parse_type(value)
}
