use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::ResponseDecodeWorkspace;

use super::super::RobotMarketProductId;
use super::super::model::{MAX_ROBOT_MARKET_PRODUCTS, RobotMarketProduct, RobotMarketProductList};
use super::shared::*;
use super::{RobotOrderCatalogDecodeError, parse, require_ok};
use crate::robot::duplicates::reject_duplicates_by_cmp;
use crate::serde::strict_json::Map;

use super::super::prepare::{
    MAX_ROBOT_ORDER_ITEM_RESPONSE_BYTES, MAX_ROBOT_ORDER_LIST_RESPONSE_BYTES,
};

const FIELDS: &[&str] = &[
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
    "price",
    "price_hourly",
    "price_setup",
    "price_vat",
    "price_hourly_vat",
    "price_setup_vat",
    "fixed_price",
    "next_reduce",
    "next_reduce_date",
    "orderable_addons",
];

pub(in crate::robot::ordering) fn decode_market_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotMarketProductList, RobotOrderCatalogDecodeError> {
    require_ok(checked, MAX_ROBOT_ORDER_LIST_RESPONSE_BYTES)?;
    let mut root = parse(checked, workspace)?;
    let values = root
        .take_array()
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_MARKET_PRODUCTS {
        return Err(RobotOrderCatalogDecodeError::InvalidList);
    }
    let mut products = reserved(values.len())?;
    for mut value in values {
        products.push(parse_product(product_object(&mut value)?)?);
    }
    reject_duplicates_by_cmp(&products, |left, right| left.id().cmp(&right.id()))
        .map_err(map_duplicate)?;
    Ok(RobotMarketProductList(products))
}

pub(in crate::robot::ordering) fn decode_market(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotMarketProduct, RobotOrderCatalogDecodeError> {
    require_ok(checked, MAX_ROBOT_ORDER_ITEM_RESPONSE_BYTES)?;
    let mut root = parse(checked, workspace)?;
    parse_product(product_object(&mut root)?)
}

fn parse_product(product: &mut Map) -> Result<RobotMarketProduct, RobotOrderCatalogDecodeError> {
    require_fields(product, FIELDS)?;
    validate_architectures(product)?;
    let id = RobotMarketProductId::new(unsigned(product, "id")?).map_err(map_value_error)?;
    let hourly_net = optional_decimal(product, "price_hourly")?;
    let hourly_gross = optional_decimal(product, "price_hourly_vat")?;
    if hourly_net.is_some() != hourly_gross.is_some() {
        return Err(RobotOrderCatalogDecodeError::InvalidPrice);
    }
    let fixed_price = product
        .get("fixed_price")
        .and_then(crate::serde::strict_json::Value::as_bool)
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?;
    let next_reduce_seconds = product
        .get("next_reduce")
        .and_then(crate::serde::strict_json::Value::as_i64)
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?;
    Ok(RobotMarketProduct {
        id,
        name: text(product, "name")?,
        description: text_list(product, "description")?,
        traffic: text(product, "traffic")?,
        distributions: choice_list(product, "dist")?,
        languages: choice_list(product, "lang")?,
        cpu: text(product, "cpu")?,
        cpu_benchmark: unsigned(product, "cpu_benchmark")?,
        memory_size: unsigned(product, "memory_size")?,
        hdd_size: unsigned(product, "hdd_size")?,
        hdd_text: text(product, "hdd_text")?,
        hdd_count: unsigned(product, "hdd_count")?,
        datacenter: text(product, "datacenter")?,
        network_speed: text(product, "network_speed")?,
        monthly_net: decimal(product, "price")?,
        hourly_net,
        setup_net: decimal(product, "price_setup")?,
        monthly_gross: decimal(product, "price_vat")?,
        hourly_gross,
        setup_gross: decimal(product, "price_setup_vat")?,
        fixed_price,
        next_reduce_seconds,
        next_reduce_at: text(product, "next_reduce_date")?,
        addons: addons(product)?,
    })
}
