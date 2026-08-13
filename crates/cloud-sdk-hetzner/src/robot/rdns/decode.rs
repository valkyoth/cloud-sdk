use alloc::vec::Vec;

use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::{ResponseDecodeWorkspace, StatusCode};

use super::model::*;
use super::prepare::{MAX_ROBOT_RDNS_ITEM_RESPONSE_BYTES, MAX_ROBOT_RDNS_LIST_RESPONSE_BYTES};
use crate::robot::duplicates::{DuplicateError, reject_duplicates_by_cmp};
use crate::robot::{
    RobotCancellationValueError, RobotIpAddress, RobotRdnsName, RobotRdnsNameError,
};
use crate::serde::strict_json::{JsonError, Map, Value, parse_with_scratch};

/// Failure while decoding a source-locked Robot reverse-DNS success response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotRdnsDecodeError {
    /// The checked status was not admitted for the operation.
    UnexpectedStatus,
    /// The body exceeded the operation's independent decode limit.
    ResponseTooLarge,
    /// JSON syntax, UTF-8, nesting, duplicates, or parser bounds were invalid.
    MalformedPayload,
    /// Required or extra response fields violated the exact source shape.
    InvalidEnvelope,
    /// An address was malformed or not in canonical provider form.
    InvalidAddress,
    /// A PTR target was malformed or not in canonical DNS form.
    InvalidPtr,
    /// A list exceeded its bound or repeated an address identity.
    InvalidList,
    /// The response address did not match the exact request.
    ResponseIdentityMismatch,
    /// A successful mutation contradicted the requested PTR target.
    MutationOutcomeMismatch,
    /// Bounded protected result storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotRdnsDecodeError,
    Self::UnexpectedStatus => "Robot reverse-DNS response status is unexpected",
    Self::ResponseTooLarge => "Robot reverse-DNS response exceeds its operation limit",
    Self::MalformedPayload => "Robot reverse-DNS response JSON is malformed",
    Self::InvalidEnvelope => "Robot reverse-DNS response envelope is invalid",
    Self::InvalidAddress => "Robot reverse-DNS response address is invalid",
    Self::InvalidPtr => "Robot reverse-DNS response PTR target is invalid",
    Self::InvalidList => "Robot reverse-DNS response list is invalid",
    Self::ResponseIdentityMismatch => "Robot reverse-DNS response identity does not match the request",
    Self::MutationOutcomeMismatch => "Robot reverse-DNS mutation response contradicts the request",
    Self::Allocation => "Robot reverse-DNS response allocation failed",
);

/// Decodes the checked `GET /rdns` result.
pub fn decode_robot_rdns_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotRdnsList, RobotRdnsDecodeError> {
    if checked.status() != StatusCode::OK {
        return Err(RobotRdnsDecodeError::UnexpectedStatus);
    }
    require_body_limit(checked, MAX_ROBOT_RDNS_LIST_RESPONSE_BYTES)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let values = root
        .take_array()
        .ok_or(RobotRdnsDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_RDNS_LIST_ITEMS {
        return Err(RobotRdnsDecodeError::InvalidList);
    }
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(values.len())
        .map_err(|_| RobotRdnsDecodeError::Allocation)?;
    for mut value in values {
        entries.push(parse_wrapper(&mut value)?);
    }
    reject_duplicates_by_cmp(&entries, |left, right| {
        left.address
            .with_addr(|left| right.address.with_addr(|right| left.cmp(&right)))
    })
    .map_err(|error| match error {
        DuplicateError::Duplicate => RobotRdnsDecodeError::InvalidList,
        DuplicateError::Allocation => RobotRdnsDecodeError::Allocation,
    })?;
    Ok(RobotRdnsList(entries))
}

/// Decodes one item response and binds it to the exact request address.
pub fn decode_robot_rdns(
    checked: CheckedResponse<'_>,
    expected: &RobotIpAddress,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotRdns, RobotRdnsDecodeError> {
    if !matches!(checked.status(), StatusCode::OK | StatusCode::CREATED) {
        return Err(RobotRdnsDecodeError::UnexpectedStatus);
    }
    require_body_limit(checked, MAX_ROBOT_RDNS_ITEM_RESPONSE_BYTES)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let result = parse_wrapper(&mut root)?;
    if &result.address == expected {
        Ok(result)
    } else {
        Err(RobotRdnsDecodeError::ResponseIdentityMismatch)
    }
}

fn parse_wrapper(value: &mut Value) -> Result<RobotRdns, RobotRdnsDecodeError> {
    let wrapper = value
        .as_object_mut()
        .ok_or(RobotRdnsDecodeError::InvalidEnvelope)?;
    require_fields(wrapper, &["rdns"])?;
    let object = wrapper
        .get_mut("rdns")
        .and_then(Value::as_object_mut)
        .ok_or(RobotRdnsDecodeError::InvalidEnvelope)?;
    require_fields(object, &["ip", "ptr"])?;
    let address = parse_address(object)?;
    let ptr = parse_ptr(object)?;
    Ok(RobotRdns { address, ptr })
}

fn parse_address(object: &Map) -> Result<RobotIpAddress, RobotRdnsDecodeError> {
    object
        .get("ip")
        .ok_or(RobotRdnsDecodeError::InvalidEnvelope)?
        .try_with_str(|text| RobotIpAddress::new(text).map_err(map_address_error))
        .map_err(|_| RobotRdnsDecodeError::InvalidAddress)?
        .ok_or(RobotRdnsDecodeError::InvalidEnvelope)?
}

fn parse_ptr(object: &Map) -> Result<RobotRdnsName, RobotRdnsDecodeError> {
    object
        .get("ptr")
        .ok_or(RobotRdnsDecodeError::InvalidEnvelope)?
        .try_with_str(|text| RobotRdnsName::new(text).map_err(map_ptr_error))
        .map_err(|_| RobotRdnsDecodeError::InvalidPtr)?
        .ok_or(RobotRdnsDecodeError::InvalidEnvelope)?
}

fn require_body_limit(
    checked: CheckedResponse<'_>,
    maximum: usize,
) -> Result<(), RobotRdnsDecodeError> {
    if checked.body().len() <= maximum {
        Ok(())
    } else {
        Err(RobotRdnsDecodeError::ResponseTooLarge)
    }
}

fn require_fields(object: &Map, fields: &[&str]) -> Result<(), RobotRdnsDecodeError> {
    if object.len() == fields.len() && fields.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotRdnsDecodeError::InvalidEnvelope)
    }
}

const fn map_address_error(error: RobotCancellationValueError) -> RobotRdnsDecodeError {
    match error {
        RobotCancellationValueError::Invalid => RobotRdnsDecodeError::InvalidAddress,
        RobotCancellationValueError::Allocation => RobotRdnsDecodeError::Allocation,
    }
}

const fn map_ptr_error(error: RobotRdnsNameError) -> RobotRdnsDecodeError {
    match error {
        RobotRdnsNameError::Invalid => RobotRdnsDecodeError::InvalidPtr,
        RobotRdnsNameError::Allocation => RobotRdnsDecodeError::Allocation,
    }
}

const fn map_json_error(error: JsonError) -> RobotRdnsDecodeError {
    if matches!(error, JsonError::Allocation) {
        RobotRdnsDecodeError::Allocation
    } else {
        RobotRdnsDecodeError::MalformedPayload
    }
}
