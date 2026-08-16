use alloc::vec;

use cloud_sdk::authentication::{CREDENTIAL_BINDING_BYTES, CredentialBinding};
use cloud_sdk::operation::{PreparationStorage, PreparationStorageGuard};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::*;
use crate::robot::RobotServerNumber;
use crate::robot::ordering::{
    CheckedRobotOrderCatalog, CredentialObserved, PreparedRobotOrderCatalog, RobotAddonCatalog,
    RobotAddonOrderPlan, RobotAddonProductListRequest, RobotAddonTransactionList,
    RobotAddonTransactionListRequest, RobotMarketOrderPlan, RobotMarketProduct,
    RobotMarketProductGetRequest, RobotMarketProductId, RobotOrderCurrency,
    RobotOrderCurrencyRequest, RobotOrderProductId, RobotStandardAddonSelection,
    RobotStandardOrderPlan, RobotStandardProduct, RobotStandardProductGetRequest,
};

mod adversarial;
mod basic;
mod permit;

const STANDARD: &[u8] =
    include_bytes!("../../../../../../tests/fixtures/robot-ordering/standard.json");
const STANDARD_MULTISET: &[u8] =
    include_bytes!("../../../../../../tests/fixtures/robot-order-mutations/standard-multiset.json");
const MARKET: &[u8] = include_bytes!("../../../../../../tests/fixtures/robot-ordering/market.json");
const ADDONS: &[u8] = include_bytes!("../../../../../../tests/fixtures/robot-ordering/addons.json");
const CURRENCY: &[u8] =
    include_bytes!("../../../../../../tests/fixtures/robot-ordering/currency.json");

fn credential(byte: u8) -> CredentialBinding {
    CredentialBinding::new([byte; CREDENTIAL_BINDING_BYTES])
        .unwrap_or_else(|_| unreachable!("credential fixture is nonzero"))
}

fn authorization<R: RobotOrderPermitRequest + ?Sized>(
    request: &R,
) -> RobotOrderAuthorizationEvidence<'static> {
    RobotOrderAuthorizationEvidence::for_request(
        RobotOrderAccount::new(b"robot-account").unwrap_or_else(|_| unreachable!()),
        request,
    )
}

fn observed<T>(value: T) -> CredentialObserved<T> {
    observed_with(value, 0x5a)
}

fn observed_with<T>(value: T, byte: u8) -> CredentialObserved<T> {
    CredentialObserved::from_parts(value, credential(byte))
}

fn addon_parameters() -> RobotAddonOrderParameters<'static> {
    RobotAddonOrderParameters::Ip {
        reason: RobotRipeReason::new("VPS").unwrap_or_else(|_| unreachable!()),
    }
}

fn multiset_standard_product() -> CredentialObserved<RobotStandardProduct> {
    standard_product_from_with(STANDARD_MULTISET, 0x5a)
}

fn standard_product_from_with(
    body: &[u8],
    credential_byte: u8,
) -> CredentialObserved<RobotStandardProduct> {
    let request = RobotStandardProductGetRequest::new(product_id("EX40"));
    let mut target = [0_u8; 256];
    let mut response_storage = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut response_storage))
        .unwrap_or_else(|_| unreachable!("standard catalog preparation failed"));
    observed_with(
        with_catalog_json(prepared, body, |checked| checked.decode_response())
            .unwrap_or_else(|_| unreachable!("standard catalog fixture failed")),
        credential_byte,
    )
}

fn market_product() -> CredentialObserved<RobotMarketProduct> {
    let request = RobotMarketProductGetRequest::new(
        RobotMarketProductId::new(282_323)
            .unwrap_or_else(|_| unreachable!("market product fixture failed")),
    );
    let mut target = [0_u8; 256];
    let mut body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("market catalog preparation failed"));
    observed(
        with_catalog_json(prepared, MARKET, |checked| checked.decode_response())
            .unwrap_or_else(|_| unreachable!("market catalog fixture failed")),
    )
}

fn currency() -> CredentialObserved<RobotOrderCurrency> {
    currency_with(0x5a)
}

fn currency_with(credential_byte: u8) -> CredentialObserved<RobotOrderCurrency> {
    let request = RobotOrderCurrencyRequest::new();
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("currency preparation failed"));
    observed_with(
        with_catalog_json(prepared, CURRENCY, |checked| checked.decode_response())
            .unwrap_or_else(|_| unreachable!("currency fixture failed")),
        credential_byte,
    )
}

fn with_standard_plan<R>(inspect: impl FnOnce(&RobotStandardOrderPlan<'_>) -> R) -> R {
    with_standard_plan_with(0x5a, inspect)
}

fn with_standard_plan_with<R>(
    credential_byte: u8,
    inspect: impl FnOnce(&RobotStandardOrderPlan<'_>) -> R,
) -> R {
    let product = standard_product_from_with(STANDARD, credential_byte);
    let currency = currency_with(credential_byte);
    let addon = product
        .value()
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
    with_addon_plan_at(0, inspect)
}

fn with_addon_plan_at<R>(
    product_index: usize,
    inspect: impl FnOnce(&RobotAddonOrderPlan<'_, '_>) -> R,
) -> R {
    let request = RobotAddonProductListRequest::new(server(321));
    let mut target = [0_u8; 256];
    let mut body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("addon catalog preparation failed"));
    let catalog: RobotAddonCatalog<'_> =
        with_catalog_json(prepared, ADDONS, |checked| checked.decode_response())
            .unwrap_or_else(|_| unreachable!("addon catalog fixture failed"));
    let catalog = observed(catalog);
    let currency = currency();
    let plan = RobotAddonOrderPlan::new(&catalog, product_index, &currency)
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

fn addon_history(body: &[u8]) -> CredentialObserved<RobotAddonTransactionList> {
    let request = RobotAddonTransactionListRequest::new();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let mut guard = PreparationStorageGuard::new(&mut target, &mut request_body);
    let prepared = request
        .prepare_bound(&mut guard)
        .unwrap_or_else(|_| unreachable!("addon history preparation failed"));
    let mut response_body = vec![0_u8; body.len()];
    let mut response_headers = [0_u8; 128];
    let response = json_response(
        &mut response_body,
        &mut response_headers,
        StatusCode::OK,
        body,
    );
    observed(
        prepared
            .validate_response(response)
            .unwrap_or_else(|_| unreachable!("addon history response policy failed"))
            .decode_response()
            .unwrap_or_else(|_| unreachable!("addon history fixture failed")),
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
