use alloc::vec::Vec;

use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::{ResponseDecodeWorkspace, StatusCode};

use super::RobotResetType;
use super::model::*;
use crate::robot::duplicates::{DuplicateError, reject_duplicates, reject_duplicates_by};
use crate::robot::server::identity::DecimalServerNumberError;
use crate::robot::{RobotCancellationValueError, RobotIpAddress, RobotServerNumber};
use crate::serde::SensitiveText;
use crate::serde::strict_json::{JsonError, Map, Value, parse_with_scratch};

/// Failure while decoding a source-locked Robot reset success response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotResetDecodeError {
    /// The checked status was not `200 OK`.
    UnexpectedStatus,
    /// JSON syntax, UTF-8, nesting, or duplicate keys were invalid.
    MalformedPayload,
    /// Required, optional, extra, or typed fields violated the source shape.
    InvalidEnvelope,
    /// An address was malformed, noncanonical, or had the wrong family.
    InvalidAddress,
    /// A server number was zero, malformed, or outside `u64`.
    InvalidServerNumber,
    /// A reset type was unknown, duplicated, or absent.
    InvalidResetTypes,
    /// A list exceeded its explicit bound or repeated a server identity.
    InvalidList,
    /// A detail or action response contradicted its exact request.
    ResponseIdentityMismatch,
    /// A successful action acknowledged a different reset type.
    MutationOutcomeMismatch,
    /// Bounded protected result storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotResetDecodeError,
    Self::UnexpectedStatus => "Robot reset response status is unexpected",
    Self::MalformedPayload => "Robot reset response JSON is malformed",
    Self::InvalidEnvelope => "Robot reset response envelope is invalid",
    Self::InvalidAddress => "Robot reset response address is invalid",
    Self::InvalidServerNumber => "Robot reset response server number is invalid",
    Self::InvalidResetTypes => "Robot reset response capabilities are invalid",
    Self::InvalidList => "Robot reset response collection is invalid",
    Self::ResponseIdentityMismatch => "Robot reset response identity does not match the request",
    Self::MutationOutcomeMismatch => "Robot reset action contradicts the requested intent",
    Self::Allocation => "Robot reset response allocation failed",
);

/// Decodes the checked `GET /reset` result.
pub fn decode_robot_reset_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotResetList, RobotResetDecodeError> {
    require_ok(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let values = root
        .take_array()
        .ok_or(RobotResetDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_RESET_LIST_ITEMS {
        return Err(RobotResetDecodeError::InvalidList);
    }
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(values.len())
        .map_err(|_| RobotResetDecodeError::Allocation)?;
    for mut value in values {
        let wrapper = value
            .as_object_mut()
            .ok_or(RobotResetDecodeError::InvalidEnvelope)?;
        require_fields(wrapper, &["reset"])?;
        let object = wrapper
            .get_mut("reset")
            .and_then(Value::as_object_mut)
            .ok_or(RobotResetDecodeError::InvalidEnvelope)?;
        require_fields(
            object,
            &["server_ip", "server_ipv6_net", "server_number", "type"],
        )?;
        entries.push(parse_summary(object)?);
    }
    reject_duplicates_by(&entries, RobotResetSummary::number).map_err(map_duplicate_error)?;
    Ok(RobotResetList(entries))
}

/// Decodes and identity-checks one reset capability resource.
pub fn decode_robot_reset(
    checked: CheckedResponse<'_>,
    expected: &RobotServerNumber,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotReset, RobotResetDecodeError> {
    require_ok(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let wrapper = root
        .as_object_mut()
        .ok_or(RobotResetDecodeError::InvalidEnvelope)?;
    require_fields(wrapper, &["reset"])?;
    let object = wrapper
        .get_mut("reset")
        .and_then(Value::as_object_mut)
        .ok_or(RobotResetDecodeError::InvalidEnvelope)?;
    require_fields(
        object,
        &[
            "server_ip",
            "server_ipv6_net",
            "server_number",
            "type",
            "operating_status",
        ],
    )?;
    let summary = parse_summary(object)?;
    if summary.number() != expected {
        return Err(RobotResetDecodeError::ResponseIdentityMismatch);
    }
    let status = object
        .get_mut("operating_status")
        .and_then(Value::take_string)
        .map(SensitiveText::new)
        .ok_or(RobotResetDecodeError::InvalidEnvelope)?;
    let operating_status = RobotResetOperatingStatus::new(status)
        .map_err(|()| RobotResetDecodeError::InvalidEnvelope)?;
    Ok(RobotReset {
        summary,
        operating_status,
    })
}

/// Decodes one reset execution acknowledgement.
pub fn decode_robot_reset_action(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotResetAction, RobotResetDecodeError> {
    require_ok(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let wrapper = root
        .as_object_mut()
        .ok_or(RobotResetDecodeError::InvalidEnvelope)?;
    require_fields(wrapper, &["reset"])?;
    let object = wrapper
        .get_mut("reset")
        .and_then(Value::as_object_mut)
        .ok_or(RobotResetDecodeError::InvalidEnvelope)?;
    let required = ["server_ip", "server_ipv6_net", "type"];
    let valid_shape = (object.len() == required.len()
        && required.iter().all(|field| object.get(field).is_some()))
        || (object.len() == required.len().saturating_add(1)
            && required.iter().all(|field| object.get(field).is_some())
            && object.get("server_number").is_some());
    if !valid_shape {
        return Err(RobotResetDecodeError::InvalidEnvelope);
    }
    let server_ipv4 = parse_address(object, "server_ip", true)?;
    let server_ipv6_network = parse_address(object, "server_ipv6_net", false)?;
    let number = object
        .get("server_number")
        .map(parse_server_number)
        .transpose()?;
    let reset_type = parse_type_value(
        object
            .get("type")
            .ok_or(RobotResetDecodeError::InvalidEnvelope)?,
    )?;
    Ok(RobotResetAction {
        server_ipv4,
        server_ipv6_network,
        number,
        reset_type,
    })
}

fn parse_summary(object: &mut Map) -> Result<RobotResetSummary, RobotResetDecodeError> {
    let server_ipv4 = parse_address(object, "server_ip", true)?;
    let server_ipv6_network = parse_address(object, "server_ipv6_net", false)?;
    let number = parse_server_number(
        object
            .get("server_number")
            .ok_or(RobotResetDecodeError::InvalidEnvelope)?,
    )?;
    let values = object
        .get_mut("type")
        .and_then(Value::take_array)
        .ok_or(RobotResetDecodeError::InvalidEnvelope)?;
    if values.is_empty() || values.len() > 5 {
        return Err(RobotResetDecodeError::InvalidResetTypes);
    }
    let mut types = Vec::new();
    types
        .try_reserve_exact(values.len())
        .map_err(|_| RobotResetDecodeError::Allocation)?;
    for value in &values {
        types.push(parse_type_value(value)?);
    }
    reject_duplicates(&types).map_err(|error| match error {
        DuplicateError::Duplicate => RobotResetDecodeError::InvalidResetTypes,
        DuplicateError::Allocation => RobotResetDecodeError::Allocation,
    })?;
    Ok(RobotResetSummary {
        server_ipv4,
        server_ipv6_network,
        number,
        types,
    })
}

fn parse_type_value(value: &Value) -> Result<RobotResetType, RobotResetDecodeError> {
    value
        .try_with_str(RobotResetType::parse)
        .map_err(|_| RobotResetDecodeError::InvalidResetTypes)?
        .flatten()
        .ok_or(RobotResetDecodeError::InvalidResetTypes)
}

fn parse_server_number(value: &Value) -> Result<RobotServerNumber, RobotResetDecodeError> {
    value
        .try_with_unsigned_lexical(|digits| {
            RobotServerNumber::from_decimal_bytes(digits.as_bytes())
        })
        .ok_or(RobotResetDecodeError::InvalidServerNumber)?
        .map_err(map_server_number)
}

fn parse_address(
    object: &Map,
    field: &str,
    ipv4: bool,
) -> Result<RobotIpAddress, RobotResetDecodeError> {
    let address = object
        .get(field)
        .ok_or(RobotResetDecodeError::InvalidEnvelope)?
        .try_with_str(|text| RobotIpAddress::new(text).map_err(map_address_error))
        .map_err(|_| RobotResetDecodeError::InvalidAddress)?
        .ok_or(RobotResetDecodeError::InvalidEnvelope)??;
    if address.with_addr(|value| value.is_ipv4()) != ipv4 {
        return Err(RobotResetDecodeError::InvalidAddress);
    }
    Ok(address)
}

fn require_ok(checked: CheckedResponse<'_>) -> Result<(), RobotResetDecodeError> {
    if checked.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(RobotResetDecodeError::UnexpectedStatus)
    }
}

fn require_fields(object: &Map, fields: &[&str]) -> Result<(), RobotResetDecodeError> {
    if object.len() == fields.len() && fields.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotResetDecodeError::InvalidEnvelope)
    }
}

const fn map_duplicate_error(error: DuplicateError) -> RobotResetDecodeError {
    match error {
        DuplicateError::Duplicate => RobotResetDecodeError::InvalidList,
        DuplicateError::Allocation => RobotResetDecodeError::Allocation,
    }
}

const fn map_address_error(error: RobotCancellationValueError) -> RobotResetDecodeError {
    match error {
        RobotCancellationValueError::Invalid => RobotResetDecodeError::InvalidAddress,
        RobotCancellationValueError::Allocation => RobotResetDecodeError::Allocation,
    }
}

const fn map_server_number(error: DecimalServerNumberError) -> RobotResetDecodeError {
    match error {
        DecimalServerNumberError::Invalid => RobotResetDecodeError::InvalidServerNumber,
        DecimalServerNumberError::Allocation => RobotResetDecodeError::Allocation,
    }
}

const fn map_json_error(error: JsonError) -> RobotResetDecodeError {
    if matches!(error, JsonError::Allocation) {
        RobotResetDecodeError::Allocation
    } else {
        RobotResetDecodeError::MalformedPayload
    }
}
