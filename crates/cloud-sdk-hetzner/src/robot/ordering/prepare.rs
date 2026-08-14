use cloud_sdk::Method;
use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::buffer::{SnapshotEncoder, encode_snapshot_bounded};
use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationImpact, OperationMetadata,
    PreparationStorage, PrepareOperation, PreparedRequest, ProviderService, RequestBodySensitivity,
    RequestIdPolicy, RequestSemantics, ResponseBodyPolicy, ResponsePolicy, RetryEligibility,
};
use cloud_sdk::transport::{
    HeaderName, MAX_INFORMATIONAL_RESPONSES, MediaType, RawResponsePolicy, RequestHeader,
    RequestHeaders, RequestTarget, ResponseMediaPolicy, StatusCode, TransportRequest,
};
use cloud_sdk_sanitization::sanitize_bytes;

use super::request::*;
use crate::endpoint::{official_robot_endpoint_identity, official_robot_endpoint_policy};
use crate::identity::{HETZNER_PROVIDER_ID, ROBOT_SERVICE_ID, RobotService};

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
const ACCEPT: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
const MAX_TARGET_BYTES: usize = 4_096;
/// Maximum accepted bytes for one ordering-catalog list response.
pub const MAX_ROBOT_ORDER_LIST_RESPONSE_BYTES: usize = 4_194_304;
/// Maximum accepted bytes for one ordering-catalog item response.
pub const MAX_ROBOT_ORDER_ITEM_RESPONSE_BYTES: usize = 1_048_576;

#[derive(Clone, Copy)]
pub(super) enum Kind<'a> {
    StandardList(&'a RobotStandardProductFilters),
    StandardGet(&'a super::RobotOrderProductId),
    MarketList,
    MarketGet(super::RobotMarketProductId),
    AddonList(&'a crate::robot::RobotServerNumber),
    Currency,
}

macro_rules! prepare_operation {
    ($type:ty, $self:ident, $kind:expr) => {
        impl PrepareOperation for $type {
            type Error = RobotOrderRequestError;

            fn prepare<'storage>(
                &$self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRequest<'storage>, Self::Error> {
                prepare($kind, storage)
            }
        }
    };
}

prepare_operation!(
    RobotStandardProductListRequest,
    self,
    Kind::StandardList(&self.filters)
);
prepare_operation!(
    RobotStandardProductGetRequest,
    self,
    Kind::StandardGet(&self.id)
);
prepare_operation!(RobotMarketProductListRequest, self, Kind::MarketList);
prepare_operation!(RobotMarketProductGetRequest, self, Kind::MarketGet(self.id));
prepare_operation!(
    RobotAddonProductListRequest,
    self,
    Kind::AddonList(&self.server)
);
prepare_operation!(RobotOrderCurrencyRequest, self, Kind::Currency);

fn prepare<'storage>(
    kind: Kind<'_>,
    storage: PreparationStorage<'storage>,
) -> Result<PreparedRequest<'storage>, RobotOrderRequestError> {
    let (target_storage, body_storage) = storage.into_parts();
    sanitize_bytes(target_storage);
    sanitize_bytes(body_storage);
    macro_rules! admitted {
        ($result:expr) => {
            match $result {
                Ok(value) => value,
                Err(error) => {
                    sanitize_bytes(target_storage);
                    sanitize_bytes(body_storage);
                    return Err(error);
                }
            }
        };
    }

    let endpoint = admitted!(
        official_robot_endpoint_policy().map_err(RobotOrderRequestError::InvalidEndpoint)
    );
    let endpoint_identity = admitted!(
        official_robot_endpoint_identity().map_err(RobotOrderRequestError::InvalidEndpoint)
    );
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(HETZNER_PROVIDER_ID),
        ScopeRequirement::Required(ROBOT_SERVICE_ID),
        ScopeRequirement::Required(endpoint_identity),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let metadata = admitted!(metadata());
    let maximum = maximum_response_bytes(kind);
    let response = admitted!(
        ResponsePolicy::new(
            OK,
            ContentTypePolicy::Required(JSON),
            ResponseBodyPolicy::Required,
            maximum,
        )
        .map_err(RobotOrderRequestError::InvalidResponsePolicy)
    );
    let content =
        admitted!(HeaderName::new("content-type").map_err(RobotOrderRequestError::InvalidHeaders));
    let raw = admitted!(
        RawResponsePolicy::new(
            maximum,
            crate::robot::MAX_ROBOT_ERROR_BODY_BYTES,
            ResponseMediaPolicy::Required(JSON),
            ResponseMediaPolicy::Optional(JSON),
            &[content],
            MAX_INFORMATIONAL_RESPONSES,
        )
        .map_err(RobotOrderRequestError::InvalidRawPolicy)
    );
    admitted!(
        PreparedRequest::validate_construction_policy(Method::Get, metadata, raw)
            .map_err(RobotOrderRequestError::InvalidPreparedPolicy)
    );
    let target_len = admitted!(encode_snapshot_bounded(
        kind,
        target_storage,
        MAX_TARGET_BYTES,
        RobotOrderRequestError::Target,
        encode_target,
    ));
    let headers =
        admitted!(RequestHeaders::new(&ACCEPT).map_err(RobotOrderRequestError::InvalidHeaders));
    let operation_id = admitted!(
        OperationId::new(operation_id(kind)).map_err(RobotOrderRequestError::InvalidOperationId)
    );
    let target_valid = target_storage
        .get(..target_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .and_then(|text| RequestTarget::new(text).ok())
        .is_some();
    if !target_valid {
        sanitize_bytes(target_storage);
        sanitize_bytes(body_storage);
        return Err(RobotOrderRequestError::Target);
    }
    let Some(target_text) = target_storage
        .get(..target_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
    else {
        unreachable!("validated Robot order target lost UTF-8")
    };
    let Ok(target) = RequestTarget::new(target_text) else {
        unreachable!("validated Robot order target became invalid")
    };
    let service = ProviderService::from_marker::<RobotService>(endpoint);
    let request = TransportRequest::new(Method::Get, target).with_headers(headers);
    let prepared = PreparedRequest::new(
        request,
        service,
        metadata,
        response,
        authentication,
        raw,
        RequestBodySensitivity::Public,
    )
    .unwrap_or_else(|_| unreachable!("prevalidated Robot catalog policy changed during binding"));
    Ok(prepared.with_operation_id(operation_id))
}

fn encode_target(
    kind: Kind<'_>,
    encoder: &mut SnapshotEncoder<'_, RobotOrderRequestError>,
) -> Result<(), RobotOrderRequestError> {
    match kind {
        Kind::StandardList(filters) => {
            encoder.string("/order/server/product")?;
            encode_filters(filters, encoder)
        }
        Kind::StandardGet(id) => {
            encoder.string("/order/server/product/")?;
            id.with_text(|value| encoder.percent_encoded(value))
        }
        Kind::MarketList => encoder.string("/order/server_market/product"),
        Kind::MarketGet(id) => {
            encoder.string("/order/server_market/product/")?;
            encoder.u64(id.get())
        }
        Kind::AddonList(server) => {
            encoder.string("/order/server_addon/")?;
            server.with_decimal_bytes(|value| encoder.bytes(value))?;
            encoder.string("/product")
        }
        Kind::Currency => encoder.string("/order/currency"),
    }
}

fn encode_filters(
    filters: &RobotStandardProductFilters,
    encoder: &mut SnapshotEncoder<'_, RobotOrderRequestError>,
) -> Result<(), RobotOrderRequestError> {
    if filters.is_empty() {
        return Ok(());
    }
    encoder.byte(b'?')?;
    let mut first = true;
    decimal_pair(encoder, &mut first, "min_price", &filters.min_price)?;
    decimal_pair(encoder, &mut first, "max_price", &filters.max_price)?;
    decimal_pair(encoder, &mut first, "min_price_setup", &filters.min_setup)?;
    decimal_pair(encoder, &mut first, "max_price_setup", &filters.max_setup)?;
    if let Some(location) = &filters.location {
        location.with_text(|value| encoder.query_pair(&mut first, "location", value))?;
    }
    Ok(())
}

fn decimal_pair(
    encoder: &mut SnapshotEncoder<'_, RobotOrderRequestError>,
    first: &mut bool,
    name: &str,
    value: &Option<super::RobotOrderDecimal>,
) -> Result<(), RobotOrderRequestError> {
    if let Some(value) = value {
        value.with_text(|value| encoder.query_pair(first, name, value))?;
    }
    Ok(())
}

const fn operation_id(kind: Kind<'_>) -> &'static str {
    match kind {
        Kind::StandardList(_) => "robot_list_server_products",
        Kind::StandardGet(_) => "robot_get_server_product",
        Kind::MarketList => "robot_list_server_market_products",
        Kind::MarketGet(_) => "robot_get_server_market_product",
        Kind::AddonList(_) => "robot_list_server_addon_products",
        Kind::Currency => "robot_list_order_currencies",
    }
}

const fn maximum_response_bytes(kind: Kind<'_>) -> usize {
    match kind {
        Kind::StandardList(_) | Kind::MarketList | Kind::AddonList(_) => {
            MAX_ROBOT_ORDER_LIST_RESPONSE_BYTES
        }
        Kind::StandardGet(_) | Kind::MarketGet(_) | Kind::Currency => {
            MAX_ROBOT_ORDER_ITEM_RESPONSE_BYTES
        }
    }
}

fn metadata() -> Result<OperationMetadata, RobotOrderRequestError> {
    OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .map_err(RobotOrderRequestError::InvalidMetadata)
}
