mod addon;
mod market;
mod shared;
mod standard;

use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::{ResponseDecodeWorkspace, StatusCode};

pub(super) use addon::decode_addon_list;
pub(super) use market::{decode_market, decode_market_list};
pub(super) use standard::{decode_standard, decode_standard_list};

use super::RobotOrderCurrency;
use super::prepare::MAX_ROBOT_ORDER_ITEM_RESPONSE_BYTES;
use crate::serde::strict_json::{JsonError, Value, parse_with_scratch};

/// Failure while decoding a source-locked Robot ordering-catalog response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotOrderCatalogDecodeError {
    /// The checked status was not admitted for this operation.
    UnexpectedStatus,
    /// The body exceeded this operation's independent decode limit.
    ResponseTooLarge,
    /// JSON syntax, UTF-8, nesting, duplicates, or parser bounds were invalid.
    MalformedPayload,
    /// Required, optional, or extra fields violated the source shape.
    InvalidEnvelope,
    /// A product identity was invalid or repeated.
    InvalidProduct,
    /// Provider-owned text or a selection was invalid or repeated.
    InvalidText,
    /// A price, range, or billing relationship was invalid.
    InvalidPrice,
    /// A collection exceeded its local bound or repeated an identity.
    InvalidList,
    /// The response product did not match the exact requested identity.
    ResponseIdentityMismatch,
    /// Bounded protected result storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotOrderCatalogDecodeError,
    Self::UnexpectedStatus => "Robot order catalog response status is unexpected",
    Self::ResponseTooLarge => "Robot order catalog response exceeds its operation limit",
    Self::MalformedPayload => "Robot order catalog response JSON is malformed",
    Self::InvalidEnvelope => "Robot order catalog response envelope is invalid",
    Self::InvalidProduct => "Robot order catalog product is invalid",
    Self::InvalidText => "Robot order catalog text is invalid",
    Self::InvalidPrice => "Robot order catalog price is invalid",
    Self::InvalidList => "Robot order catalog collection is invalid",
    Self::ResponseIdentityMismatch => "Robot order catalog identity does not match the request",
    Self::Allocation => "Robot order catalog response allocation failed",
);

pub(super) fn decode_currency(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotOrderCurrency, RobotOrderCatalogDecodeError> {
    require_ok(checked, MAX_ROBOT_ORDER_ITEM_RESPONSE_BYTES)?;
    let root = parse(checked, workspace)?;
    let object = root
        .as_object()
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?;
    shared::require_fields(object, &["currency"])?;
    object
        .get("currency")
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?
        .try_with_str(RobotOrderCurrency::new)
        .map_err(|_| RobotOrderCatalogDecodeError::InvalidText)?
        .ok_or(RobotOrderCatalogDecodeError::InvalidEnvelope)?
        .map_err(shared::map_value_error)
}

pub(super) fn parse(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<Value, RobotOrderCatalogDecodeError> {
    parse_with_scratch(checked.body(), workspace.decoder_scratch_mut()).map_err(map_json_error)
}

pub(super) fn require_ok(
    checked: CheckedResponse<'_>,
    maximum: usize,
) -> Result<(), RobotOrderCatalogDecodeError> {
    if checked.status() != StatusCode::OK {
        Err(RobotOrderCatalogDecodeError::UnexpectedStatus)
    } else if checked.body().len() > maximum {
        Err(RobotOrderCatalogDecodeError::ResponseTooLarge)
    } else {
        Ok(())
    }
}

const fn map_json_error(error: JsonError) -> RobotOrderCatalogDecodeError {
    if matches!(error, JsonError::Allocation) {
        RobotOrderCatalogDecodeError::Allocation
    } else {
        RobotOrderCatalogDecodeError::MalformedPayload
    }
}
