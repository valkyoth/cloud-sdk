mod addon;
mod common;
mod server;

use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::{ResponseDecodeWorkspace, StatusCode};

pub(super) use addon::{decode_addon, decode_addon_list};
pub(super) use server::{decode_market, decode_market_list, decode_standard, decode_standard_list};

use super::prepare::{
    MAX_ROBOT_ORDER_TRANSACTION_ITEM_RESPONSE_BYTES,
    MAX_ROBOT_ORDER_TRANSACTION_LIST_RESPONSE_BYTES,
};
use crate::serde::strict_json::{JsonError, Value, parse_with_scratch};

/// Failure while decoding a source-locked Robot transaction response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotOrderTransactionDecodeError {
    /// The checked status was not admitted for this operation.
    UnexpectedStatus,
    /// The body exceeded this operation's independent decode limit.
    ResponseTooLarge,
    /// JSON syntax, UTF-8, nesting, duplicates, or parser bounds were invalid.
    MalformedPayload,
    /// Required, nullable, or extra fields violated the source shape.
    InvalidEnvelope,
    /// A transaction identity was invalid or repeated.
    InvalidTransaction,
    /// A transaction timestamp was malformed or calendar-invalid.
    InvalidTimestamp,
    /// A transaction state or state-dependent field relationship was invalid.
    InvalidStatus,
    /// A resulting server identity was malformed or inconsistent with state.
    InvalidServer,
    /// SSH key metadata was malformed, oversized, or repeated.
    InvalidKey,
    /// Product data was malformed or internally inconsistent.
    InvalidProduct,
    /// Provider-owned text was empty, oversized, or invalid.
    InvalidText,
    /// Exact price data was malformed or internally inconsistent.
    InvalidPrice,
    /// A bounded collection overflowed or contained duplicate identities.
    InvalidList,
    /// An addon resource identity was malformed or repeated.
    InvalidResource,
    /// A detail response did not match the exact requested transaction.
    ResponseIdentityMismatch,
    /// Protected or bounded response storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotOrderTransactionDecodeError,
    Self::UnexpectedStatus => "Robot order transaction response status is unexpected",
    Self::ResponseTooLarge => "Robot order transaction response exceeds its operation limit",
    Self::MalformedPayload => "Robot order transaction response JSON is malformed",
    Self::InvalidEnvelope => "Robot order transaction response envelope is invalid",
    Self::InvalidTransaction => "Robot order transaction identity is invalid",
    Self::InvalidTimestamp => "Robot order transaction timestamp is invalid",
    Self::InvalidStatus => "Robot order transaction state is inconsistent",
    Self::InvalidServer => "Robot order transaction server identity is invalid",
    Self::InvalidKey => "Robot order transaction SSH key metadata is invalid",
    Self::InvalidProduct => "Robot order transaction product is invalid",
    Self::InvalidText => "Robot order transaction text is invalid",
    Self::InvalidPrice => "Robot order transaction price is invalid",
    Self::InvalidList => "Robot order transaction collection is invalid",
    Self::InvalidResource => "Robot order transaction resource is invalid",
    Self::ResponseIdentityMismatch => "Robot order transaction identity does not match the request",
    Self::Allocation => "Robot order transaction response allocation failed",
);

fn parse(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<Value, RobotOrderTransactionDecodeError> {
    parse_with_scratch(checked.body(), workspace.decoder_scratch_mut()).map_err(map_json_error)
}

fn require_list(checked: CheckedResponse<'_>) -> Result<(), RobotOrderTransactionDecodeError> {
    require_ok(checked, MAX_ROBOT_ORDER_TRANSACTION_LIST_RESPONSE_BYTES)
}

fn require_item(checked: CheckedResponse<'_>) -> Result<(), RobotOrderTransactionDecodeError> {
    require_ok(checked, MAX_ROBOT_ORDER_TRANSACTION_ITEM_RESPONSE_BYTES)
}

fn require_ok(
    checked: CheckedResponse<'_>,
    maximum: usize,
) -> Result<(), RobotOrderTransactionDecodeError> {
    if checked.status() != StatusCode::OK {
        Err(RobotOrderTransactionDecodeError::UnexpectedStatus)
    } else if checked.body().len() > maximum {
        Err(RobotOrderTransactionDecodeError::ResponseTooLarge)
    } else {
        Ok(())
    }
}

const fn map_json_error(error: JsonError) -> RobotOrderTransactionDecodeError {
    if matches!(error, JsonError::Allocation) {
        RobotOrderTransactionDecodeError::Allocation
    } else {
        RobotOrderTransactionDecodeError::MalformedPayload
    }
}
