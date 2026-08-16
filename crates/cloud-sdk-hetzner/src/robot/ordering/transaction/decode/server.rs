use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::ResponseDecodeWorkspace;

use super::super::model::{
    MAX_ROBOT_ORDER_TRANSACTION_ITEMS, RobotMarketTransaction, RobotMarketTransactionList,
    RobotMarketTransactionProduct, RobotStandardTransaction, RobotStandardTransactionList,
    RobotStandardTransactionProduct,
};
use super::common::*;
use super::{RobotOrderTransactionDecodeError, parse, require_item, require_list};
use crate::robot::duplicates::reject_duplicates_by_cmp;
use crate::robot::ordering::{RobotMarketProductId, RobotOrderProductId};
use crate::serde::strict_json::{Map, Value};

const STANDARD_FIELDS: &[&str] = &[
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
const MARKET_FIELDS: &[&str] = &[
    "id",
    "date",
    "status",
    "server_number",
    "server_ip",
    "authorized_key",
    "host_key",
    "comment",
    "product",
];

pub(in crate::robot::ordering) fn decode_standard_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotStandardTransactionList, RobotOrderTransactionDecodeError> {
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
        transactions.push(parse_standard(transaction_object(&mut value)?)?);
    }
    reject_transaction_duplicates(&transactions, RobotStandardTransaction::id)?;
    Ok(RobotStandardTransactionList(transactions))
}

pub(in crate::robot::ordering) fn decode_standard(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotStandardTransaction, RobotOrderTransactionDecodeError> {
    require_item(checked)?;
    let mut root = parse(checked, workspace)?;
    parse_standard(transaction_object(&mut root)?)
}

fn parse_standard(
    object: &mut Map,
) -> Result<RobotStandardTransaction, RobotOrderTransactionDecodeError> {
    require_fields(object, STANDARD_FIELDS)?;
    let common = parse_common(object)?;
    let product = object
        .get_mut("product")
        .and_then(Value::as_object_mut)
        .ok_or(RobotOrderTransactionDecodeError::InvalidProduct)
        .and_then(parse_standard_product)?;
    let values = array(object, "addons", 4_096)?;
    let mut addons = reserved(values.len())?;
    for value in values {
        addons.push(
            value
                .try_with_str(crate::robot::ordering::RobotOrderProductId::new)
                .map_err(|_| RobotOrderTransactionDecodeError::InvalidProduct)?
                .ok_or(RobotOrderTransactionDecodeError::InvalidEnvelope)?
                .map_err(map_value_error)?,
        );
    }
    reject_duplicates_by_cmp(&addons, compare_product_id).map_err(map_duplicate)?;
    Ok(RobotStandardTransaction {
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

fn parse_standard_product(
    product: &mut Map,
) -> Result<RobotStandardTransactionProduct, RobotOrderTransactionDecodeError> {
    require_fields(
        product,
        &[
            "id",
            "name",
            "description",
            "traffic",
            "dist",
            "@deprecated arch",
            "lang",
            "location",
        ],
    )?;
    Ok(RobotStandardTransactionProduct {
        id: product_id(product, "id")?,
        name: text(product, "name")?,
        description: text_list(product, "description")?,
        traffic: text(product, "traffic")?,
        distribution: choice(product, "dist")?,
        architecture: architecture(product)?,
        language: choice(product, "lang")?,
        location: nullable_location(product, "location")?,
    })
}

pub(in crate::robot::ordering) fn decode_market_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotMarketTransactionList, RobotOrderTransactionDecodeError> {
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
        transactions.push(parse_market(transaction_object(&mut value)?)?);
    }
    reject_transaction_duplicates(&transactions, RobotMarketTransaction::id)?;
    Ok(RobotMarketTransactionList(transactions))
}

pub(in crate::robot::ordering) fn decode_market(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotMarketTransaction, RobotOrderTransactionDecodeError> {
    require_item(checked)?;
    let mut root = parse(checked, workspace)?;
    parse_market(transaction_object(&mut root)?)
}

fn parse_market(
    object: &mut Map,
) -> Result<RobotMarketTransaction, RobotOrderTransactionDecodeError> {
    require_fields(object, MARKET_FIELDS)?;
    let common = parse_common(object)?;
    let product = object
        .get_mut("product")
        .and_then(Value::as_object_mut)
        .ok_or(RobotOrderTransactionDecodeError::InvalidProduct)
        .and_then(parse_market_product)?;
    Ok(RobotMarketTransaction {
        id: common.id,
        date: common.date,
        status: common.status,
        server_number: common.server_number,
        server_ip: common.server_ip,
        authorized_keys: common.authorized_keys,
        host_keys: common.host_keys,
        comment: common.comment,
        product,
    })
}

fn parse_market_product(
    product: &mut Map,
) -> Result<RobotMarketTransactionProduct, RobotOrderTransactionDecodeError> {
    require_fields(
        product,
        &[
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
            "fixed_price",
            "next_reduce",
            "next_reduce_date",
        ],
    )?;
    let id = product
        .get("id")
        .and_then(Value::as_u64)
        .ok_or(RobotOrderTransactionDecodeError::InvalidProduct)
        .and_then(|id| {
            RobotMarketProductId::new(id)
                .map_err(|_| RobotOrderTransactionDecodeError::InvalidProduct)
        })?;
    let cpu_benchmark = unsigned(product, "cpu_benchmark")?;
    let memory_size = unsigned(product, "memory_size")?;
    let hdd_size = unsigned(product, "hdd_size")?;
    let hdd_count = unsigned(product, "hdd_count")?;
    let next_reduce_at = text(product, "next_reduce_date")?;
    if !valid_legacy_timestamp(&next_reduce_at) {
        return Err(RobotOrderTransactionDecodeError::InvalidTimestamp);
    }
    Ok(RobotMarketTransactionProduct {
        id,
        name: text(product, "name")?,
        description: text_list(product, "description")?,
        traffic: text(product, "traffic")?,
        distribution: choice(product, "dist")?,
        architecture: architecture(product)?,
        language: choice(product, "lang")?,
        cpu: text(product, "cpu")?,
        cpu_benchmark,
        memory_size,
        hdd_size,
        hdd_text: text(product, "hdd_text")?,
        hdd_count,
        datacenter: text(product, "datacenter")?,
        network_speed: text(product, "network_speed")?,
        fixed_price: product
            .get("fixed_price")
            .and_then(Value::as_bool)
            .ok_or(RobotOrderTransactionDecodeError::InvalidProduct)?,
        next_reduce_seconds: product
            .get("next_reduce")
            .and_then(Value::as_i64)
            .ok_or(RobotOrderTransactionDecodeError::InvalidProduct)?,
        next_reduce_at,
    })
}

fn unsigned(object: &Map, field: &str) -> Result<u64, RobotOrderTransactionDecodeError> {
    object
        .get(field)
        .and_then(Value::as_u64)
        .ok_or(RobotOrderTransactionDecodeError::InvalidProduct)
}

fn compare_product_id(
    left: &RobotOrderProductId,
    right: &RobotOrderProductId,
) -> core::cmp::Ordering {
    left.with_text(|left| right.with_text(|right| left.cmp(right)))
}
