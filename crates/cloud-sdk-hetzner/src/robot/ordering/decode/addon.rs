use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::ResponseDecodeWorkspace;

use super::super::model::{MAX_ROBOT_ADDON_PRODUCTS, RobotAddonProduct, RobotAddonProductList};
use super::shared::*;
use super::{RobotOrderCatalogDecodeError, parse, require_ok};
use crate::robot::duplicates::reject_duplicates_by_cmp;

use super::super::prepare::MAX_ROBOT_ORDER_LIST_RESPONSE_BYTES;

pub(in crate::robot::ordering) fn decode_addon_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotAddonProductList, RobotOrderCatalogDecodeError> {
    require_ok(checked, MAX_ROBOT_ORDER_LIST_RESPONSE_BYTES)?;
    let mut root = parse(checked, workspace)?;
    let values = root
        .take_array()
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_ADDON_PRODUCTS {
        return Err(RobotOrderCatalogDecodeError::InvalidList);
    }
    let mut products = reserved(values.len())?;
    for mut value in values {
        let product = product_object(&mut value)?;
        require_fields(product, &["id", "name", "type", "price"])?;
        let price = product
            .get_mut("price")
            .and_then(crate::serde::strict_json::Value::as_object_mut)
            .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)
            .and_then(parse_price)?;
        products.push(RobotAddonProduct {
            id: product_id(product, "id")?,
            name: text(product, "name")?,
            kind: text(product, "type")?,
            price,
        });
    }
    reject_duplicates_by_cmp(&products, |left, right| {
        compare_product_id(left.id(), right.id())
    })
    .map_err(map_duplicate)?;
    Ok(RobotAddonProductList(products))
}
