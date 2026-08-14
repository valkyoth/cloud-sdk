use alloc::{format, vec};

use cloud_sdk::Method;
use cloud_sdk::operation::{
    OperationImpact, PreparationStorage, PrepareOperation, RequestBodySensitivity,
    RequestSemantics, RetryEligibility,
};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::robot::RobotServerNumber;

const STANDARD: &[u8] =
    include_bytes!("../../../../../tests/fixtures/robot-ordering/standard.json");
const MARKET: &[u8] = include_bytes!("../../../../../tests/fixtures/robot-ordering/market.json");
const ADDONS: &[u8] = include_bytes!("../../../../../tests/fixtures/robot-ordering/addons.json");
const CURRENCY: &[u8] =
    include_bytes!("../../../../../tests/fixtures/robot-ordering/currency.json");

#[test]
fn exact_decimal_and_selector_values_are_bounded_and_redacted() {
    let one = decimal("1.0");
    let same = decimal("1.0000");
    let greater = decimal("1.0001");
    assert_eq!(one, same);
    assert!(greater > same);
    assert_eq!(greater.scale(), 4);
    assert_eq!(format!("{greater:?}"), "RobotOrderDecimal([redacted])");

    for invalid in ["", ".1", "1.", "01.0", "-1", "+1", "1.00001", "1e2"] {
        assert_eq!(
            RobotOrderDecimal::new(invalid).err(),
            Some(RobotOrderValueError::InvalidDecimal)
        );
    }
    assert!(RobotOrderProductId::new("EX40").is_ok());
    assert!(RobotOrderProductId::new("EX/40").is_err());
    assert!(RobotOrderLocation::new("FSN1").is_ok());
    assert!(RobotOrderChoice::new("Debian 12 minimal").is_ok());
    assert!(RobotOrderCurrency::new("EUR").is_ok());
    assert!(RobotOrderCurrency::new("eur").is_err());
}

#[test]
fn prepares_all_six_read_only_operations_and_exact_filters() {
    let filters = RobotStandardProductFilters::new(
        Some(decimal("10.0000")),
        Some(decimal("99.5000")),
        Some(decimal("0")),
        Some(decimal("25.0000")),
        Some(location("FSN1")),
    )
    .unwrap_or_else(|_| unreachable!("filter fixture failed"));
    assert_prepared(
        RobotStandardProductListRequest::new(filters),
        "/order/server/product?min_price=10.0000&max_price=99.5000&min_price_setup=0&max_price_setup=25.0000&location=FSN1",
        "robot_list_server_products",
        MAX_ROBOT_ORDER_LIST_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotStandardProductGetRequest::new(product_id("EX40")),
        "/order/server/product/EX40",
        "robot_get_server_product",
        MAX_ROBOT_ORDER_ITEM_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotMarketProductListRequest::new(),
        "/order/server_market/product",
        "robot_list_server_market_products",
        MAX_ROBOT_ORDER_LIST_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotMarketProductGetRequest::new(market_id()),
        "/order/server_market/product/282323",
        "robot_get_server_market_product",
        MAX_ROBOT_ORDER_ITEM_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotAddonProductListRequest::new(server()),
        "/order/server_addon/321/product",
        "robot_list_server_addon_products",
        MAX_ROBOT_ORDER_LIST_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotOrderCurrencyRequest::new(),
        "/order/currency",
        "robot_list_order_currencies",
        MAX_ROBOT_ORDER_ITEM_RESPONSE_BYTES,
    );
}

#[test]
fn filter_ranges_fail_before_target_writes() {
    assert_eq!(
        RobotStandardProductFilters::new(Some(decimal("2")), Some(decimal("1")), None, None, None,)
            .err(),
        Some(RobotOrderRequestError::InvalidPriceRange)
    );
    let request = RobotStandardProductGetRequest::new(product_id("EX40"));
    let mut target = [0x5a_u8; 8];
    let mut body = [0x5a_u8; 8];
    assert!(
        request
            .prepare(PreparationStorage::new(&mut target, &mut body))
            .is_err()
    );
    assert!(target.iter().all(|byte| *byte == 0));
    assert!(body.iter().all(|byte| *byte == 0));
}

#[test]
fn decodes_complete_standard_catalog_and_non_executable_plan() {
    let product =
        decode_standard(STANDARD).unwrap_or_else(|_| unreachable!("standard fixture failed"));
    assert_eq!(product.prices().len(), 2);
    assert_eq!(product.orderable_addons().len(), 1);
    assert_eq!(product.distributions().len(), 2);
    assert_eq!(product.languages().len(), 1);
    assert_eq!(format!("{product:?}"), "RobotStandardProduct([redacted])");

    let currency = decode_currency_fixture(CURRENCY)
        .unwrap_or_else(|_| unreachable!("currency fixture failed"));
    let addon = product
        .orderable_addons()
        .first()
        .unwrap_or_else(|| unreachable!("standard addon fixture disappeared"));
    let selection = RobotStandardAddonSelection::new(addon, 0, 1)
        .unwrap_or_else(|_| unreachable!("addon selection failed"));
    let selections = [selection];
    let plan = RobotStandardOrderPlan::new(&product, &currency, 0, 1, 0, &selections)
        .unwrap_or_else(|_| unreachable!("standard plan failed"));
    assert_eq!(
        plan.price_warning(),
        RobotCatalogPriceWarning::RevalidateImmediatelyBeforePurchase
    );
    assert_eq!(plan.addons().len(), 1);
}

#[test]
fn decodes_complete_market_addon_and_currency_catalogs() {
    let product =
        decode_market_fixture(MARKET).unwrap_or_else(|_| unreachable!("market fixture failed"));
    assert_eq!(product.id(), market_id());
    assert_eq!(product.cpu_benchmark(), 8_944);
    assert_eq!(product.memory_size(), 24);
    assert_eq!(product.hdd_count(), 2);
    assert_eq!(product.next_reduce_seconds(), -10_800);
    assert!(product.hourly_net().is_some());
    assert!(!product.fixed_price());

    let currency = decode_currency_fixture(CURRENCY)
        .unwrap_or_else(|_| unreachable!("currency fixture failed"));
    let plan = RobotMarketOrderPlan::new(&product, &currency, 0, 0)
        .unwrap_or_else(|_| unreachable!("market plan failed"));
    assert_eq!(
        plan.price_warning(),
        RobotCatalogPriceWarning::RevalidateImmediatelyBeforePurchase
    );

    let products = decode_addons(ADDONS).unwrap_or_else(|_| unreachable!("addon fixture failed"));
    assert_eq!(products.products().len(), 2);
    let addon = products
        .products()
        .first()
        .unwrap_or_else(|| unreachable!("addon fixture disappeared"));
    let server = server();
    let addon_plan = RobotAddonOrderPlan::new(&server, addon, &currency);
    assert_eq!(
        addon_plan.price_warning(),
        RobotCatalogPriceWarning::RevalidateImmediatelyBeforePurchase
    );
}

#[test]
fn strict_decoding_rejects_identity_decimal_and_shape_drift() {
    assert_eq!(
        decode_standard_get(product_id("OTHER"), STANDARD).err(),
        Some(RobotOrderCatalogDecodeError::ResponseIdentityMismatch)
    );
    let duplicate = format!("[{},{}]", text(STANDARD), text(STANDARD));
    assert_eq!(
        decode_standard_list(duplicate.as_bytes()).err(),
        Some(RobotOrderCatalogDecodeError::InvalidList)
    );
    let invalid_decimal = text(STANDARD).replace("84.0300", "84.03001");
    assert_eq!(
        decode_standard(invalid_decimal.as_bytes()).err(),
        Some(RobotOrderCatalogDecodeError::InvalidPrice)
    );
    let mismatched_hourly = text(MARKET).replace(
        "\"price_hourly_vat\": \"0.1747\"",
        "\"price_hourly_vat\": null",
    );
    assert_eq!(
        decode_market_fixture(mismatched_hourly.as_bytes()).err(),
        Some(RobotOrderCatalogDecodeError::InvalidPrice)
    );
    let extra = text(CURRENCY).replace("}", ",\"future\":true}");
    assert_eq!(
        decode_currency_fixture(extra.as_bytes()).err(),
        Some(RobotOrderCatalogDecodeError::InvalidEnvelope)
    );
}

#[test]
fn plan_binding_rejects_wrong_location_quantity_and_duplicate_addons() {
    let product =
        decode_standard(STANDARD).unwrap_or_else(|_| unreachable!("standard fixture failed"));
    let currency = decode_currency_fixture(CURRENCY)
        .unwrap_or_else(|_| unreachable!("currency fixture failed"));
    let addon = product
        .orderable_addons()
        .first()
        .unwrap_or_else(|| unreachable!("addon fixture disappeared"));
    assert_eq!(
        RobotStandardAddonSelection::new(addon, 0, 2).err(),
        Some(RobotCatalogPlanError::InvalidQuantity)
    );
    let wrong_location = RobotStandardAddonSelection::new(addon, 1, 1)
        .unwrap_or_else(|_| unreachable!("location selection failed"));
    assert_eq!(
        RobotStandardOrderPlan::new(&product, &currency, 0, 0, 0, &[wrong_location]).err(),
        Some(RobotCatalogPlanError::LocationMismatch)
    );
    let first = RobotStandardAddonSelection::new(addon, 0, 1)
        .unwrap_or_else(|_| unreachable!("first addon selection failed"));
    let second = RobotStandardAddonSelection::new(addon, 0, 1)
        .unwrap_or_else(|_| unreachable!("second addon selection failed"));
    assert_eq!(
        RobotStandardOrderPlan::new(&product, &currency, 0, 0, 0, &[first, second]).err(),
        Some(RobotCatalogPlanError::DuplicateAddon)
    );
}

fn assert_prepared<O>(operation: O, target: &str, id: &str, maximum: usize)
where
    O: PrepareOperation<Error = RobotOrderRequestError>,
{
    let mut target_storage = [0_u8; 4_096];
    let mut body_storage = [0_u8; 1];
    let prepared = operation
        .prepare(PreparationStorage::new(
            &mut target_storage,
            &mut body_storage,
        ))
        .unwrap_or_else(|_| unreachable!("catalog preparation failed"));
    assert_eq!(prepared.transport_request().method(), Method::Get);
    assert_eq!(prepared.transport_request().target().as_str(), target);
    assert!(prepared.transport_request().body().is_empty());
    assert_eq!(
        prepared.operation_id().map(|value| value.as_str()),
        Some(id)
    );
    assert_eq!(prepared.metadata().impact(), OperationImpact::ReadOnly);
    assert_eq!(prepared.metadata().semantics(), RequestSemantics::Safe);
    assert_eq!(
        prepared.metadata().retry_eligibility(),
        RetryEligibility::ExplicitPolicy
    );
    assert_eq!(prepared.body_sensitivity(), RequestBodySensitivity::Public);
    assert_eq!(prepared.response_policy().max_body_bytes(), maximum);
}

fn decode_standard(body: &[u8]) -> Result<RobotStandardProduct, RobotOrderCatalogDecodeError> {
    decode_standard_get(product_id("EX40"), body)
}

fn decode_standard_get(
    id: RobotOrderProductId,
    body: &[u8],
) -> Result<RobotStandardProduct, RobotOrderCatalogDecodeError> {
    let request = RobotStandardProductGetRequest::new(id);
    let mut target = [0_u8; 256];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("standard preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_standard_list(
    body: &[u8],
) -> Result<RobotStandardProductList, RobotOrderCatalogDecodeError> {
    let request = RobotStandardProductListRequest::default();
    let mut target = [0_u8; 256];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("standard list preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_market_fixture(body: &[u8]) -> Result<RobotMarketProduct, RobotOrderCatalogDecodeError> {
    let request = RobotMarketProductGetRequest::new(market_id());
    let mut target = [0_u8; 256];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("market preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_addons(body: &[u8]) -> Result<RobotAddonProductList, RobotOrderCatalogDecodeError> {
    let request = RobotAddonProductListRequest::new(server());
    let mut target = [0_u8; 256];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("addon preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_currency_fixture(
    body: &[u8],
) -> Result<RobotOrderCurrency, RobotOrderCatalogDecodeError> {
    let request = RobotOrderCurrencyRequest::new();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("currency preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn with_json<R, O>(
    prepared: PreparedRobotOrderCatalog<'_, '_, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotOrderCatalog<'_, '_, R>) -> O,
) -> O {
    let mut storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut storage, body.len(), &mut headers);
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
        .commit(StatusCode::OK, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("response commit failed"));
    drop(attempt);
    let checked = prepared
        .validate_response(response)
        .unwrap_or_else(|_| unreachable!("response policy failed"));
    decode(checked)
}

fn decimal(value: &str) -> RobotOrderDecimal {
    RobotOrderDecimal::new(value).unwrap_or_else(|_| unreachable!("decimal fixture failed"))
}

fn product_id(value: &str) -> RobotOrderProductId {
    RobotOrderProductId::new(value).unwrap_or_else(|_| unreachable!("product ID fixture failed"))
}

fn location(value: &str) -> RobotOrderLocation {
    RobotOrderLocation::new(value).unwrap_or_else(|_| unreachable!("location fixture failed"))
}

fn market_id() -> RobotMarketProductId {
    RobotMarketProductId::new(282_323).unwrap_or_else(|_| unreachable!("market ID fixture failed"))
}

fn server() -> RobotServerNumber {
    RobotServerNumber::new(321).unwrap_or_else(|_| unreachable!("server fixture failed"))
}

fn text(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).unwrap_or_else(|_| unreachable!("fixture lost UTF-8"))
}
