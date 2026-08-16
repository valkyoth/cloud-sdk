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
use crate::robot::ordering::{RobotOrderRequestError, RobotOrderTransactionId};

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
const ACCEPT: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
const MAX_TARGET_BYTES: usize = 4_096;

/// Maximum accepted bytes for one transaction-list response.
pub const MAX_ROBOT_ORDER_TRANSACTION_LIST_RESPONSE_BYTES: usize = 4_194_304;
/// Maximum accepted bytes for one transaction-detail response.
pub const MAX_ROBOT_ORDER_TRANSACTION_ITEM_RESPONSE_BYTES: usize = 1_048_576;

#[derive(Clone, Copy)]
pub(super) enum Kind<'a> {
    StandardList,
    StandardGet(&'a RobotOrderTransactionId),
    MarketList,
    MarketGet(&'a RobotOrderTransactionId),
    AddonList,
    AddonGet(&'a RobotOrderTransactionId),
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
    RobotStandardTransactionListRequest,
    self,
    Kind::StandardList
);
prepare_operation!(
    RobotStandardTransactionGetRequest,
    self,
    Kind::StandardGet(&self.id)
);
prepare_operation!(RobotMarketTransactionListRequest, self, Kind::MarketList);
prepare_operation!(
    RobotMarketTransactionGetRequest,
    self,
    Kind::MarketGet(&self.id)
);
prepare_operation!(RobotAddonTransactionListRequest, self, Kind::AddonList);
prepare_operation!(
    RobotAddonTransactionGetRequest,
    self,
    Kind::AddonGet(&self.id)
);

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
    let target_text = target_storage
        .get(..target_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .unwrap_or_else(|| unreachable!("validated Robot transaction target lost UTF-8"));
    let target = RequestTarget::new(target_text)
        .unwrap_or_else(|_| unreachable!("validated Robot transaction target became invalid"));
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
    .unwrap_or_else(|_| unreachable!("prevalidated Robot transaction policy changed"));
    Ok(prepared.with_operation_id(operation_id))
}

fn encode_target(
    kind: Kind<'_>,
    encoder: &mut SnapshotEncoder<'_, RobotOrderRequestError>,
) -> Result<(), RobotOrderRequestError> {
    match kind {
        Kind::StandardList => encoder.string("/order/server/transaction"),
        Kind::StandardGet(id) => transaction_target("/order/server/transaction/", id, encoder),
        Kind::MarketList => encoder.string("/order/server_market/transaction"),
        Kind::MarketGet(id) => transaction_target("/order/server_market/transaction/", id, encoder),
        Kind::AddonList => encoder.string("/order/server_addon/transaction"),
        Kind::AddonGet(id) => transaction_target("/order/server_addon/transaction/", id, encoder),
    }
}

fn transaction_target(
    prefix: &str,
    id: &RobotOrderTransactionId,
    encoder: &mut SnapshotEncoder<'_, RobotOrderRequestError>,
) -> Result<(), RobotOrderRequestError> {
    encoder.string(prefix)?;
    id.with_text(|value| encoder.percent_encoded(value))
}

const fn operation_id(kind: Kind<'_>) -> &'static str {
    match kind {
        Kind::StandardList => "robot_list_server_transactions",
        Kind::StandardGet(_) => "robot_get_server_transaction",
        Kind::MarketList => "robot_list_server_market_transactions",
        Kind::MarketGet(_) => "robot_get_server_market_transaction",
        Kind::AddonList => "robot_list_server_addon_transactions",
        Kind::AddonGet(_) => "robot_get_server_addon_transaction",
    }
}

const fn maximum_response_bytes(kind: Kind<'_>) -> usize {
    match kind {
        Kind::StandardList | Kind::MarketList | Kind::AddonList => {
            MAX_ROBOT_ORDER_TRANSACTION_LIST_RESPONSE_BYTES
        }
        Kind::StandardGet(_) | Kind::MarketGet(_) | Kind::AddonGet(_) => {
            MAX_ROBOT_ORDER_TRANSACTION_ITEM_RESPONSE_BYTES
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
