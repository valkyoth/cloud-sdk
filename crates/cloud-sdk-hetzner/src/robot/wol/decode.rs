use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::{ResponseDecodeWorkspace, StatusCode};

use super::{MAX_ROBOT_WOL_RESPONSE_BYTES, RobotWol};
use crate::robot::server::identity::DecimalServerNumberError;
use crate::robot::{RobotCancellationValueError, RobotIpAddress, RobotServerNumber};
use crate::serde::strict_json::{JsonError, Map, Value, parse_with_scratch};

/// Failure while decoding a source-locked Robot Wake-on-LAN success response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotWolDecodeError {
    /// The checked status was not `200 OK`.
    UnexpectedStatus,
    /// The body exceeded the source-specific independent limit.
    ResponseTooLarge,
    /// JSON syntax, UTF-8, nesting, or duplicate keys were invalid.
    MalformedPayload,
    /// Required, extra, or typed fields violated the source shape.
    InvalidEnvelope,
    /// An address was malformed, noncanonical, or had the wrong family.
    InvalidAddress,
    /// A server number was zero, malformed, or outside `u64`.
    InvalidServerNumber,
    /// The response identified a different server than the request.
    ResponseIdentityMismatch,
    /// Bounded protected result storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotWolDecodeError,
    Self::UnexpectedStatus => "Robot Wake-on-LAN response status is unexpected",
    Self::ResponseTooLarge => "Robot Wake-on-LAN response exceeds its bound",
    Self::MalformedPayload => "Robot Wake-on-LAN response JSON is malformed",
    Self::InvalidEnvelope => "Robot Wake-on-LAN response envelope is invalid",
    Self::InvalidAddress => "Robot Wake-on-LAN response address is invalid",
    Self::InvalidServerNumber => "Robot Wake-on-LAN server number is invalid",
    Self::ResponseIdentityMismatch => "Robot Wake-on-LAN response identity does not match",
    Self::Allocation => "Robot Wake-on-LAN response allocation failed",
);

/// Decodes one checked WOL response and binds it to an expected server number.
pub fn decode_robot_wol(
    checked: CheckedResponse<'_>,
    expected: &RobotServerNumber,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotWol, RobotWolDecodeError> {
    if checked.status() != StatusCode::OK {
        return Err(RobotWolDecodeError::UnexpectedStatus);
    }
    if checked.body().len() > MAX_ROBOT_WOL_RESPONSE_BYTES {
        return Err(RobotWolDecodeError::ResponseTooLarge);
    }
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let wrapper = root
        .as_object_mut()
        .ok_or(RobotWolDecodeError::InvalidEnvelope)?;
    require_fields(wrapper, &["wol"])?;
    let object = wrapper
        .get_mut("wol")
        .and_then(Value::as_object_mut)
        .ok_or(RobotWolDecodeError::InvalidEnvelope)?;
    require_fields(object, &["server_ip", "server_ipv6_net", "server_number"])?;
    let server_ipv4 = parse_address(object, "server_ip", true)?;
    let server_ipv6_network = parse_address(object, "server_ipv6_net", false)?;
    let number = parse_server_number(
        object
            .get("server_number")
            .ok_or(RobotWolDecodeError::InvalidEnvelope)?,
    )?;
    if &number != expected {
        return Err(RobotWolDecodeError::ResponseIdentityMismatch);
    }
    Ok(RobotWol {
        server_ipv4,
        server_ipv6_network,
        number,
    })
}

fn parse_server_number(value: &Value) -> Result<RobotServerNumber, RobotWolDecodeError> {
    value
        .try_with_unsigned_lexical(|digits| {
            RobotServerNumber::from_decimal_bytes(digits.as_bytes())
        })
        .ok_or(RobotWolDecodeError::InvalidServerNumber)?
        .map_err(map_server_number)
}

fn parse_address(
    object: &Map,
    field: &str,
    ipv4: bool,
) -> Result<RobotIpAddress, RobotWolDecodeError> {
    let address = object
        .get(field)
        .ok_or(RobotWolDecodeError::InvalidEnvelope)?
        .try_with_str(|text| RobotIpAddress::new(text).map_err(map_address_error))
        .map_err(|_| RobotWolDecodeError::InvalidAddress)?
        .ok_or(RobotWolDecodeError::InvalidEnvelope)??;
    if address.with_addr(|value| value.is_ipv4()) != ipv4 {
        return Err(RobotWolDecodeError::InvalidAddress);
    }
    Ok(address)
}

fn require_fields(object: &Map, fields: &[&str]) -> Result<(), RobotWolDecodeError> {
    if object.len() == fields.len() && fields.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotWolDecodeError::InvalidEnvelope)
    }
}

const fn map_address_error(error: RobotCancellationValueError) -> RobotWolDecodeError {
    match error {
        RobotCancellationValueError::Invalid => RobotWolDecodeError::InvalidAddress,
        RobotCancellationValueError::Allocation => RobotWolDecodeError::Allocation,
    }
}

const fn map_server_number(error: DecimalServerNumberError) -> RobotWolDecodeError {
    match error {
        DecimalServerNumberError::Invalid => RobotWolDecodeError::InvalidServerNumber,
        DecimalServerNumberError::Allocation => RobotWolDecodeError::Allocation,
    }
}

const fn map_json_error(error: JsonError) -> RobotWolDecodeError {
    if matches!(error, JsonError::Allocation) {
        RobotWolDecodeError::Allocation
    } else {
        RobotWolDecodeError::MalformedPayload
    }
}
