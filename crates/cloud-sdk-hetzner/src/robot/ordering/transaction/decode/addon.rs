use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::ResponseDecodeWorkspace;

use super::super::model::{
    MAX_ROBOT_ORDER_TRANSACTION_ITEMS, MAX_ROBOT_ORDER_TRANSACTION_RESOURCES,
    RobotAddonTransaction, RobotAddonTransactionList, RobotAddonTransactionProduct,
    RobotOrderTransactionResource,
};
use super::common::*;
use super::{RobotOrderTransactionDecodeError, parse, require_item, require_list};
use crate::robot::duplicates::{DuplicateError, reject_duplicates_by_cmp};
use crate::robot::ordering::{RobotOrderDecimal, RobotOrderPrice, RobotOrderPricePair};
use crate::serde::strict_json::{Map, Value};

const FIELDS: &[&str] = &[
    "id",
    "date",
    "status",
    "server_number",
    "product",
    "resources",
];

pub(in crate::robot::ordering) fn decode_addon_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotAddonTransactionList, RobotOrderTransactionDecodeError> {
    require_list(checked)?;
    let mut root = parse(checked, workspace)?;
    let values = root
        .take_array()
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_ORDER_TRANSACTION_ITEMS {
        return Err(RobotOrderTransactionDecodeError::InvalidList);
    }
    let mut transactions = reserved(values.len())?;
    for mut value in values {
        transactions.push(parse_addon(transaction_object(&mut value)?)?);
    }
    reject_transaction_duplicates(&transactions, RobotAddonTransaction::id)?;
    Ok(RobotAddonTransactionList(transactions))
}

pub(in crate::robot::ordering) fn decode_addon(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotAddonTransaction, RobotOrderTransactionDecodeError> {
    require_item(checked)?;
    let mut root = parse(checked, workspace)?;
    parse_addon(transaction_object(&mut root)?)
}

fn parse_addon(
    object: &mut Map,
) -> Result<RobotAddonTransaction, RobotOrderTransactionDecodeError> {
    require_fields(object, FIELDS)?;
    let id = transaction_id(object, "id")?;
    let date = timestamp(object, "date")?;
    let status = status(object, "status")?;
    let server_number = object
        .get("server_number")
        .and_then(Value::as_u64)
        .ok_or(RobotOrderTransactionDecodeError::InvalidServer)
        .and_then(|number| {
            crate::robot::RobotServerNumber::new(number).map_err(|error| match error {
                crate::robot::RobotServerNumberError::Zero => {
                    RobotOrderTransactionDecodeError::InvalidServer
                }
                crate::robot::RobotServerNumberError::Allocation => {
                    RobotOrderTransactionDecodeError::Allocation
                }
            })
        })?;
    let product = object
        .get_mut("product")
        .and_then(Value::as_object_mut)
        .ok_or(RobotOrderTransactionDecodeError::InvalidProduct)
        .and_then(parse_product)?;
    let resources = parse_resources(object)?;
    Ok(RobotAddonTransaction {
        id,
        date,
        status,
        server_number,
        product,
        resources,
    })
}

fn parse_product(
    product: &mut Map,
) -> Result<RobotAddonTransactionProduct, RobotOrderTransactionDecodeError> {
    require_fields(product, &["id", "name", "price"])?;
    let price = product
        .get_mut("price")
        .and_then(Value::as_object_mut)
        .ok_or(RobotOrderTransactionDecodeError::InvalidPrice)
        .and_then(parse_price)?;
    Ok(RobotAddonTransactionProduct {
        id: product_id(product, "id")?,
        name: text(product, "name")?,
        price,
    })
}

fn parse_price(object: &mut Map) -> Result<RobotOrderPrice, RobotOrderTransactionDecodeError> {
    require_fields(object, &["location", "price", "price_setup"])?;
    let location = object
        .get("location")
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
        .try_with_str(crate::robot::ordering::RobotOrderLocation::new)
        .map_err(|_| RobotOrderTransactionDecodeError::InvalidText)?
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
        .map_err(map_value_error)?;
    let recurring = object
        .get("price")
        .and_then(Value::as_object)
        .ok_or(RobotOrderTransactionDecodeError::InvalidPrice)?;
    require_fields(recurring, &["net", "gross", "hourly_net", "hourly_gross"])?;
    let hourly_net = optional_decimal(recurring, "hourly_net")?;
    let hourly_gross = optional_decimal(recurring, "hourly_gross")?;
    let hourly = match (hourly_net, hourly_gross) {
        (Some(net), Some(gross)) => Some(RobotOrderPricePair { net, gross }),
        (None, None) => None,
        _ => return Err(RobotOrderTransactionDecodeError::InvalidPrice),
    };
    let setup = object
        .get("price_setup")
        .and_then(Value::as_object)
        .ok_or(RobotOrderTransactionDecodeError::InvalidPrice)?;
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

fn decimal(
    object: &Map,
    field: &str,
) -> Result<RobotOrderDecimal, RobotOrderTransactionDecodeError> {
    object
        .get(field)
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
        .try_with_str(RobotOrderDecimal::new)
        .map_err(|_| RobotOrderTransactionDecodeError::InvalidPrice)?
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
        .map_err(|error| match error {
            crate::robot::ordering::RobotOrderValueError::Allocation => {
                RobotOrderTransactionDecodeError::Allocation
            }
            _ => RobotOrderTransactionDecodeError::InvalidPrice,
        })
}

fn optional_decimal(
    object: &Map,
    field: &str,
) -> Result<Option<RobotOrderDecimal>, RobotOrderTransactionDecodeError> {
    let value = object
        .get(field)
        .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        Ok(None)
    } else {
        value
            .try_with_str(RobotOrderDecimal::new)
            .map_err(|_| RobotOrderTransactionDecodeError::InvalidPrice)?
            .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
            .map(Some)
            .map_err(|error| match error {
                crate::robot::ordering::RobotOrderValueError::Allocation => {
                    RobotOrderTransactionDecodeError::Allocation
                }
                _ => RobotOrderTransactionDecodeError::InvalidPrice,
            })
    }
}

fn parse_resources(
    object: &mut Map,
) -> Result<alloc::vec::Vec<RobotOrderTransactionResource>, RobotOrderTransactionDecodeError> {
    let values = array(object, "resources", MAX_ROBOT_ORDER_TRANSACTION_RESOURCES)?;
    let mut resources = reserved(values.len())?;
    for mut value in values {
        let resource = value
            .as_object_mut()
            .ok_or(RobotOrderTransactionDecodeError::InvalidResource)?;
        require_fields(resource, &["type", "id"])
            .map_err(|_| RobotOrderTransactionDecodeError::InvalidResource)?;
        resources.push(RobotOrderTransactionResource {
            kind: text(resource, "type")?,
            id: text(resource, "id")?,
        });
    }
    reject_duplicates_by_cmp(&resources, |left, right| {
        left.kind
            .compare(&right.kind)
            .then_with(|| left.id.compare(&right.id))
    })
    .map_err(|error| match error {
        DuplicateError::Duplicate => RobotOrderTransactionDecodeError::InvalidResource,
        DuplicateError::Allocation => RobotOrderTransactionDecodeError::Allocation,
    })?;
    Ok(resources)
}
