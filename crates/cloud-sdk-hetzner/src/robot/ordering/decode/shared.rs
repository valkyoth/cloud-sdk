use alloc::vec::Vec;
use core::cmp::Ordering;

use super::super::model::{
    RobotOrderPrice, RobotOrderPricePair, RobotOrderText, RobotOrderableAddon,
};
use super::super::{
    RobotOrderChoice, RobotOrderDecimal, RobotOrderLocation, RobotOrderProductId,
    RobotOrderValueError,
};
use super::RobotOrderCatalogDecodeError;
use crate::robot::duplicates::{DuplicateError, reject_duplicates_by_cmp};
use crate::serde::strict_json::{Map, Value};

pub(super) const MAX_NESTED_ITEMS: usize = 4_096;

pub(super) fn require_fields(
    object: &Map,
    fields: &[&str],
) -> Result<(), RobotOrderCatalogDecodeError> {
    if object.len() == fields.len() && fields.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotOrderCatalogDecodeError::InvalidEnvelope)
    }
}

pub(super) fn require_fields_with_optional(
    object: &Map,
    fields: &[&str],
    optional: &str,
) -> Result<(), RobotOrderCatalogDecodeError> {
    if (object.len() == fields.len()
        || (object.len() == fields.len().saturating_add(1) && object.get(optional).is_some()))
        && fields.iter().all(|field| object.get(field).is_some())
    {
        Ok(())
    } else {
        Err(RobotOrderCatalogDecodeError::InvalidEnvelope)
    }
}

pub(super) fn product_object(value: &mut Value) -> Result<&mut Map, RobotOrderCatalogDecodeError> {
    let wrapper = value
        .as_object_mut()
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?;
    require_fields(wrapper, &["product"])?;
    wrapper
        .get_mut("product")
        .and_then(Value::as_object_mut)
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)
}

pub(super) fn text(
    object: &mut Map,
    field: &str,
) -> Result<RobotOrderText, RobotOrderCatalogDecodeError> {
    object
        .get_mut(field)
        .and_then(Value::take_string)
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)
        .and_then(RobotOrderText::from_provider)
}

pub(super) fn text_list(
    object: &mut Map,
    field: &str,
) -> Result<Vec<RobotOrderText>, RobotOrderCatalogDecodeError> {
    let values = array(object, field)?;
    let mut result = reserved(values.len())?;
    for mut value in values {
        let text = value
            .take_string()
            .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?;
        result.push(RobotOrderText::from_provider(text)?);
    }
    Ok(result)
}

pub(super) fn choice_list(
    object: &mut Map,
    field: &str,
) -> Result<Vec<RobotOrderChoice>, RobotOrderCatalogDecodeError> {
    let values = array(object, field)?;
    let mut result = reserved(values.len())?;
    for value in values {
        result.push(
            value
                .try_with_str(RobotOrderChoice::new)
                .map_err(|_| RobotOrderCatalogDecodeError::InvalidText)?
                .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?
                .map_err(map_value_error)?,
        );
    }
    reject_duplicates_by_cmp(&result, compare_choice).map_err(map_duplicate)?;
    Ok(result)
}

pub(super) fn location_list(
    object: &mut Map,
    field: &str,
) -> Result<Vec<RobotOrderLocation>, RobotOrderCatalogDecodeError> {
    let values = array(object, field)?;
    let mut result = reserved(values.len())?;
    for value in values {
        result.push(
            value
                .try_with_str(RobotOrderLocation::new)
                .map_err(|_| RobotOrderCatalogDecodeError::InvalidText)?
                .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?
                .map_err(map_value_error)?,
        );
    }
    reject_duplicates_by_cmp(&result, compare_location).map_err(map_duplicate)?;
    Ok(result)
}

pub(super) fn product_id(
    object: &Map,
    field: &str,
) -> Result<RobotOrderProductId, RobotOrderCatalogDecodeError> {
    object
        .get(field)
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?
        .try_with_str(RobotOrderProductId::new)
        .map_err(|_| RobotOrderCatalogDecodeError::InvalidProduct)?
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?
        .map_err(map_value_error)
}

pub(super) fn decimal(
    object: &Map,
    field: &str,
) -> Result<RobotOrderDecimal, RobotOrderCatalogDecodeError> {
    object
        .get(field)
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?
        .try_with_str(RobotOrderDecimal::new)
        .map_err(|_| RobotOrderCatalogDecodeError::InvalidPrice)?
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?
        .map_err(map_value_error)
}

pub(super) fn optional_decimal(
    object: &Map,
    field: &str,
) -> Result<Option<RobotOrderDecimal>, RobotOrderCatalogDecodeError> {
    let value = object
        .get(field)
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        Ok(None)
    } else {
        value
            .try_with_str(RobotOrderDecimal::new)
            .map_err(|_| RobotOrderCatalogDecodeError::InvalidPrice)?
            .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?
            .map(Some)
            .map_err(map_value_error)
    }
}

pub(super) fn prices(
    object: &mut Map,
    field: &str,
) -> Result<Vec<RobotOrderPrice>, RobotOrderCatalogDecodeError> {
    let values = array(object, field)?;
    let mut result = reserved(values.len())?;
    for mut value in values {
        let price = value
            .as_object_mut()
            .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?;
        result.push(parse_price(price)?);
    }
    reject_duplicates_by_cmp(&result, |left, right| {
        compare_location(left.location(), right.location())
    })
    .map_err(map_duplicate)?;
    Ok(result)
}

pub(super) fn parse_price(
    object: &mut Map,
) -> Result<RobotOrderPrice, RobotOrderCatalogDecodeError> {
    require_fields(object, &["location", "price", "price_setup"])?;
    let location = object
        .get("location")
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?
        .try_with_str(RobotOrderLocation::new)
        .map_err(|_| RobotOrderCatalogDecodeError::InvalidText)?
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?
        .map_err(map_value_error)?;
    let recurring = object
        .get("price")
        .and_then(Value::as_object)
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?;
    require_fields(recurring, &["net", "gross", "hourly_net", "hourly_gross"])?;
    let hourly_net = optional_decimal(recurring, "hourly_net")?;
    let hourly_gross = optional_decimal(recurring, "hourly_gross")?;
    let hourly = match (hourly_net, hourly_gross) {
        (Some(net), Some(gross)) => Some(RobotOrderPricePair { net, gross }),
        (None, None) => None,
        _ => return Err(RobotOrderCatalogDecodeError::InvalidPrice),
    };
    let setup = object
        .get("price_setup")
        .and_then(Value::as_object)
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?;
    require_fields(setup, &["net", "gross"])?;
    Ok(RobotOrderPrice {
        location,
        recurring: RobotOrderPricePair {
            net: decimal(recurring, "net")?,
            gross: decimal(recurring, "gross")?,
        },
        hourly,
        setup: RobotOrderPricePair {
            net: decimal(setup, "net")?,
            gross: decimal(setup, "gross")?,
        },
    })
}

pub(super) fn addons(
    object: &mut Map,
) -> Result<Vec<RobotOrderableAddon>, RobotOrderCatalogDecodeError> {
    let values = array(object, "orderable_addons")?;
    let mut result = reserved(values.len())?;
    for mut value in values {
        let addon = value
            .as_object_mut()
            .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?;
        require_fields_with_optional(addon, &["id", "name", "min", "max", "prices"], "location")?;
        if let Some(location) = addon.get("location") {
            location
                .try_with_str(RobotOrderLocation::new)
                .map_err(|_| RobotOrderCatalogDecodeError::InvalidText)?
                .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?
                .map_err(map_value_error)?;
        }
        let minimum = unsigned(addon, "min")?;
        let maximum = unsigned(addon, "max")?;
        if minimum > maximum {
            return Err(RobotOrderCatalogDecodeError::InvalidPrice);
        }
        result.push(RobotOrderableAddon {
            id: product_id(addon, "id")?,
            name: text(addon, "name")?,
            minimum,
            maximum,
            prices: prices(addon, "prices")?,
        });
    }
    reject_duplicates_by_cmp(&result, |left, right| {
        compare_product_id(&left.id, &right.id)
    })
    .map_err(map_duplicate)?;
    Ok(result)
}

pub(super) fn validate_architectures(object: &mut Map) -> Result<(), RobotOrderCatalogDecodeError> {
    let values = array(object, "@deprecated arch")?;
    if values.is_empty()
        || values
            .iter()
            .any(|value| !matches!(value.as_u64(), Some(32 | 64)))
    {
        return Err(RobotOrderCatalogDecodeError::InvalidEnvelope);
    }
    Ok(())
}

pub(super) fn unsigned(object: &Map, field: &str) -> Result<u64, RobotOrderCatalogDecodeError> {
    object
        .get(field)
        .and_then(Value::as_u64)
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)
}

pub(super) fn array(
    object: &mut Map,
    field: &str,
) -> Result<Vec<Value>, RobotOrderCatalogDecodeError> {
    let values = object
        .get_mut(field)
        .and_then(Value::take_array)
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_NESTED_ITEMS {
        Err(RobotOrderCatalogDecodeError::InvalidList)
    } else {
        Ok(values)
    }
}

pub(super) fn reserved<T>(length: usize) -> Result<Vec<T>, RobotOrderCatalogDecodeError> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(length)
        .map_err(|_| RobotOrderCatalogDecodeError::Allocation)?;
    Ok(result)
}

pub(super) fn compare_product_id(
    left: &RobotOrderProductId,
    right: &RobotOrderProductId,
) -> Ordering {
    left.with_text(|left| right.with_text(|right| left.cmp(right)))
}

fn compare_choice(left: &RobotOrderChoice, right: &RobotOrderChoice) -> Ordering {
    left.with_text(|left| right.with_text(|right| left.cmp(right)))
}

fn compare_location(left: &RobotOrderLocation, right: &RobotOrderLocation) -> Ordering {
    left.with_text(|left| right.with_text(|right| left.cmp(right)))
}

pub(super) const fn map_value_error(error: RobotOrderValueError) -> RobotOrderCatalogDecodeError {
    if matches!(error, RobotOrderValueError::Allocation) {
        RobotOrderCatalogDecodeError::Allocation
    } else if matches!(error, RobotOrderValueError::InvalidDecimal) {
        RobotOrderCatalogDecodeError::InvalidPrice
    } else {
        RobotOrderCatalogDecodeError::InvalidText
    }
}

pub(super) const fn map_duplicate(error: DuplicateError) -> RobotOrderCatalogDecodeError {
    match error {
        DuplicateError::Duplicate => RobotOrderCatalogDecodeError::InvalidList,
        DuplicateError::Allocation => RobotOrderCatalogDecodeError::Allocation,
    }
}
