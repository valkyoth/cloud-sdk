use alloc::vec;

use cloud_sdk::operation::{PreparationStorage, PreparationStorageGuard};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::robot::RobotServerNumber;
use crate::robot::ordering::{
    CheckedRobotOrderCatalog, PreparedRobotOrderCatalog, RobotAddonCatalog, RobotAddonOrderPlan,
    RobotAddonProductListRequest, RobotMarketOrderPlan, RobotMarketProduct,
    RobotMarketProductGetRequest, RobotMarketProductId, RobotOrderCurrency,
    RobotOrderCurrencyRequest, RobotOrderProductId, RobotStandardAddonSelection,
    RobotStandardOrderPlan, RobotStandardProduct, RobotStandardProductGetRequest,
};

mod adversarial;
mod basic;
mod permit;

const STANDARD: &[u8] =
    include_bytes!("../../../../../../tests/fixtures/robot-ordering/standard.json");
const MARKET: &[u8] = include_bytes!("../../../../../../tests/fixtures/robot-ordering/market.json");
const ADDONS: &[u8] = include_bytes!("../../../../../../tests/fixtures/robot-ordering/addons.json");
const CURRENCY: &[u8] =
    include_bytes!("../../../../../../tests/fixtures/robot-ordering/currency.json");

fn standard_product() -> RobotStandardProduct {
    let request = RobotStandardProductGetRequest::new(product_id("EX40"));
    let mut target = [0_u8; 256];
    let mut body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("standard catalog preparation failed"));
    with_catalog_json(prepared, STANDARD, |checked| checked.decode_response())
        .unwrap_or_else(|_| unreachable!("standard catalog fixture failed"))
}

fn market_product() -> RobotMarketProduct {
    let request = RobotMarketProductGetRequest::new(
        RobotMarketProductId::new(282_323)
            .unwrap_or_else(|_| unreachable!("market product fixture failed")),
    );
    let mut target = [0_u8; 256];
    let mut body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("market catalog preparation failed"));
    with_catalog_json(prepared, MARKET, |checked| checked.decode_response())
        .unwrap_or_else(|_| unreachable!("market catalog fixture failed"))
}

fn currency() -> RobotOrderCurrency {
    let request = RobotOrderCurrencyRequest::new();
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("currency preparation failed"));
    with_catalog_json(prepared, CURRENCY, |checked| checked.decode_response())
        .unwrap_or_else(|_| unreachable!("currency fixture failed"))
}

fn with_standard_plan<R>(inspect: impl FnOnce(&RobotStandardOrderPlan<'_>) -> R) -> R {
    let product = standard_product();
    let currency = currency();
    let addon = product
        .orderable_addons()
        .first()
        .unwrap_or_else(|| unreachable!("standard addon fixture disappeared"));
    let selection = RobotStandardAddonSelection::new(addon, 0, 1)
        .unwrap_or_else(|_| unreachable!("standard addon selection failed"));
    let selections = [selection];
    let plan = RobotStandardOrderPlan::new(&product, &currency, 0, 0, 0, &selections)
        .unwrap_or_else(|_| unreachable!("standard plan failed"));
    inspect(&plan)
}

fn with_market_plan<R>(inspect: impl FnOnce(&RobotMarketOrderPlan<'_>) -> R) -> R {
    let product = market_product();
    let currency = currency();
    let plan = RobotMarketOrderPlan::new(&product, &currency, 0, 0)
        .unwrap_or_else(|_| unreachable!("market plan failed"));
    inspect(&plan)
}

fn with_addon_plan<R>(inspect: impl FnOnce(&RobotAddonOrderPlan<'_, '_>) -> R) -> R {
    let request = RobotAddonProductListRequest::new(server(321));
    let mut target = [0_u8; 256];
    let mut body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("addon catalog preparation failed"));
    let catalog: RobotAddonCatalog<'_> =
        with_catalog_json(prepared, ADDONS, |checked| checked.decode_response())
            .unwrap_or_else(|_| unreachable!("addon catalog fixture failed"));
    let currency = currency();
    let plan = RobotAddonOrderPlan::new(&catalog, 0, &currency)
        .unwrap_or_else(|_| unreachable!("addon plan failed"));
    inspect(&plan)
}

fn with_catalog_json<'request, R, O>(
    prepared: PreparedRobotOrderCatalog<'_, 'request, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotOrderCatalog<'_, 'request, R>) -> O,
) -> O {
    let mut storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 128];
    let response = json_response(&mut storage, &mut headers, StatusCode::OK, body);
    decode(
        prepared
            .validate_response(response)
            .unwrap_or_else(|_| unreachable!("catalog response policy failed")),
    )
}

fn json_response<'a>(
    storage: &'a mut [u8],
    headers: &'a mut [u8],
    status: StatusCode,
    body: &[u8],
) -> ResponseBuffer<'a> {
    let mut response = ResponseBuffer::new(storage, body.len(), headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("response attempt failed"));
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!("response headers failed"))
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!("content type failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("response body failed"))
        .copy_from_slice(body);
    attempt
        .commit(status, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("response commit failed"));
    drop(attempt);
    response
}

fn prepared_standard<'guard, 'request>(
    request: &'request RobotStandardOrderCreateRequest<'_>,
    guard: &'guard mut PreparationStorageGuard<'_>,
) -> PreparedRobotOrderMutation<'guard, 'request, RobotStandardOrderCreateRequest<'request>> {
    request
        .prepare_bound(guard)
        .unwrap_or_else(|_| unreachable!("standard mutation preparation failed"))
}

fn product_id(value: &str) -> RobotOrderProductId {
    RobotOrderProductId::new(value).unwrap_or_else(|_| unreachable!("product ID fixture failed"))
}

fn server(value: u64) -> RobotServerNumber {
    RobotServerNumber::new(value).unwrap_or_else(|_| unreachable!("server fixture failed"))
}
