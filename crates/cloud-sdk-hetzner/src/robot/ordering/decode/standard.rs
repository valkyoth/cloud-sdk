use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::ResponseDecodeWorkspace;

use super::super::model::{
    MAX_ROBOT_STANDARD_PRODUCTS, RobotStandardProduct, RobotStandardProductList,
};
use super::shared::*;
use super::{RobotOrderCatalogDecodeError, parse, require_ok};
use crate::robot::duplicates::reject_duplicates_by_cmp;

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
    "location",
    "prices",
    "orderable_addons",
];

pub(in crate::robot::ordering) fn decode_standard_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotStandardProductList, RobotOrderCatalogDecodeError> {
    require_ok(checked, MAX_ROBOT_ORDER_LIST_RESPONSE_BYTES)?;
    let mut root = parse(checked, workspace)?;
    let values = root
        .take_array()
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_STANDARD_PRODUCTS {
        return Err(RobotOrderCatalogDecodeError::InvalidList);
    }
    let mut products = reserved(values.len())?;
    for mut value in values {
        products.push(parse_product(product_object(&mut value)?)?);
    }
    reject_duplicates_by_cmp(&products, |left, right| {
        compare_product_id(left.id(), right.id())
    })
    .map_err(map_duplicate)?;
    Ok(RobotStandardProductList(products))
}

pub(in crate::robot::ordering) fn decode_standard(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotStandardProduct, RobotOrderCatalogDecodeError> {
    require_ok(checked, MAX_ROBOT_ORDER_ITEM_RESPONSE_BYTES)?;
    let mut root = parse(checked, workspace)?;
    parse_product(product_object(&mut root)?)
}

fn parse_product(
    product: &mut crate::serde::strict_json::Map,
) -> Result<RobotStandardProduct, RobotOrderCatalogDecodeError> {
    require_fields(product, FIELDS)?;
    validate_architectures(product)?;
    let locations = location_list(product, "location")?;
    let prices = prices(product, "prices")?;
    if locations.is_empty()
        || prices.is_empty()
        || prices
            .iter()
            .any(|price| !locations.contains(price.location()))
    {
        return Err(RobotOrderCatalogDecodeError::InvalidPrice);
    }
    let addons = addons(product)?;
    if addons
        .iter()
        .flat_map(|addon| addon.prices())
        .any(|price| !locations.contains(price.location()))
    {
        return Err(RobotOrderCatalogDecodeError::InvalidPrice);
    }
    Ok(RobotStandardProduct {
        id: product_id(product, "id")?,
        name: text(product, "name")?,
        description: text_list(product, "description")?,
        traffic: text(product, "traffic")?,
        distributions: choice_list(product, "dist")?,
        languages: choice_list(product, "lang")?,
        locations,
        prices,
        addons,
    })
}
