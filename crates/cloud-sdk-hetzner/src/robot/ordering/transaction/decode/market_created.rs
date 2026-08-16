use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::ResponseDecodeWorkspace;

use super::super::model::{RobotMarketCreatedProduct, RobotMarketCreatedTransaction};
use super::common::*;
use super::{RobotOrderTransactionDecodeError, parse, require_item};
use crate::robot::ordering::{RobotMarketProductId, RobotOrderProductId};
use crate::serde::strict_json::Value;

const FIELDS: &[&str] = &[
    "id",
    "date",
    "status",
    "server_number",
    "server_ip",
    "authorized_key",
    "host_key",
    "comment",
    "product",
    "addons",
];

const PRODUCT_FIELDS: &[&str] = &[
    "id",
    "name",
    "description",
    "traffic",
    "dist",
    "@deprecated arch",
    "lang",
    "cpu",
    "cpu_benchmark",
    "memory_size",
    "hdd_size",
    "hdd_text",
    "hdd_count",
    "datacenter",
    "network_speed",
];

pub(in crate::robot::ordering) fn decode_market_created(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotMarketCreatedTransaction, RobotOrderTransactionDecodeError> {
    require_item(checked)?;
    let mut root = parse(checked, workspace)?;
    let object = transaction_object(&mut root)?;
    require_fields(object, FIELDS)?;
    let common = parse_common(object)?;
    let product = object
        .get_mut("product")
        .and_then(Value::as_object_mut)
        .ok_or(RobotOrderTransactionDecodeError::InvalidProduct)?;
    require_fields(product, PRODUCT_FIELDS)?;
    let id = product
        .get("id")
        .and_then(Value::as_u64)
        .ok_or(RobotOrderTransactionDecodeError::InvalidProduct)
        .and_then(|value| {
            RobotMarketProductId::new(value)
                .map_err(|_| RobotOrderTransactionDecodeError::InvalidProduct)
        })?;
    let product = RobotMarketCreatedProduct {
        id,
        name: text(product, "name")?,
        description: text_list(product, "description")?,
        traffic: text(product, "traffic")?,
        distribution: choice(product, "dist")?,
        architecture: architecture(product)?,
        language: choice(product, "lang")?,
        cpu: text(product, "cpu")?,
        cpu_benchmark: unsigned(product, "cpu_benchmark")?,
        memory_size: unsigned(product, "memory_size")?,
        hdd_size: unsigned(product, "hdd_size")?,
        hdd_text: text(product, "hdd_text")?,
        hdd_count: unsigned(product, "hdd_count")?,
        datacenter: text(product, "datacenter")?,
        network_speed: text(product, "network_speed")?,
    };
    let values = array(object, "addons", 4_096)?;
    let mut addons = reserved(values.len())?;
    for value in values {
        addons.push(
            value
                .try_with_str(RobotOrderProductId::new)
                .map_err(|_| RobotOrderTransactionDecodeError::InvalidProduct)?
                .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
                .map_err(map_value_error)?,
        );
    }
    Ok(RobotMarketCreatedTransaction {
        id: common.id,
        date: common.date,
        status: common.status,
        server_number: common.server_number,
        server_ip: common.server_ip,
        authorized_keys: common.authorized_keys,
        host_keys: common.host_keys,
        comment: common.comment,
        product,
        addons,
    })
}

fn unsigned(
    product: &crate::serde::strict_json::Map,
    field: &str,
) -> Result<u64, RobotOrderTransactionDecodeError> {
    product
        .get(field)
        .and_then(Value::as_u64)
        .ok_or(RobotOrderTransactionDecodeError::InvalidProduct)
}
