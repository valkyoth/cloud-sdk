use alloc::vec::Vec;
use core::net::IpAddr;

use cloud_sdk::operation::{CheckedResponse, CheckedResponseGuard};
use cloud_sdk::transport::{ResponseDecodeWorkspace, StatusCode};

use super::model::*;
use super::{RobotMacAddress, RobotMacAddressError};
use crate::robot::duplicates::{DuplicateError, reject_duplicates_by_cmp};
use crate::robot::server::identity::DecimalServerNumberError;
use crate::robot::{RobotCancellationValueError, RobotIpAddress, RobotServerNumber};
use crate::serde::strict_json::{JsonError, Map, Value, parse_with_scratch};

/// Failure while decoding a source-locked Robot IP success response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotIpDecodeError {
    /// The checked status was not the source-locked success status.
    UnexpectedStatus,
    /// JSON syntax, UTF-8, nesting, duplicates, or parser bounds were invalid.
    MalformedPayload,
    /// Required, optional, or extra response fields violated the source shape.
    InvalidEnvelope,
    /// An IP address was malformed, noncanonical, or had the wrong family.
    InvalidAddress,
    /// A prefix, gateway, or broadcast conflicted with the addressed network.
    InvalidNetwork,
    /// A server number was zero, malformed, or outside `u64`.
    InvalidServerNumber,
    /// A separate MAC was malformed or noncanonical.
    InvalidMac,
    /// A list exceeded its source-locked bound or contained duplicate addresses.
    InvalidList,
    /// The response identity did not match the exact request.
    ResponseIdentityMismatch,
    /// A successful mutation contradicted the requested state transition.
    MutationOutcomeMismatch,
    /// Bounded protected result storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotIpDecodeError,
    Self::UnexpectedStatus => "Robot IP response status is unexpected",
    Self::MalformedPayload => "Robot IP response JSON is malformed",
    Self::InvalidEnvelope => "Robot IP response envelope is invalid",
    Self::InvalidAddress => "Robot IP response address is invalid",
    Self::InvalidNetwork => "Robot IP response network is inconsistent",
    Self::InvalidServerNumber => "Robot IP response server number is invalid",
    Self::InvalidMac => "Robot IP response MAC is invalid",
    Self::InvalidList => "Robot IP response list is invalid",
    Self::ResponseIdentityMismatch => "Robot IP response identity does not match the request",
    Self::MutationOutcomeMismatch => "Robot IP mutation response contradicts the request",
    Self::Allocation => "Robot IP response allocation failed",
);

/// Decodes the checked `GET /ip` result.
pub fn decode_robot_ip_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotIpList, RobotIpDecodeError> {
    require_ok(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let values = root
        .take_array()
        .ok_or(RobotIpDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_IP_LIST_ITEMS {
        return Err(RobotIpDecodeError::InvalidList);
    }
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(values.len())
        .map_err(|_| RobotIpDecodeError::Allocation)?;
    for mut value in values {
        let wrapper = value
            .as_object_mut()
            .ok_or(RobotIpDecodeError::InvalidEnvelope)?;
        require_fields(wrapper, &["ip"])?;
        let object = wrapper
            .get_mut("ip")
            .and_then(Value::as_object_mut)
            .ok_or(RobotIpDecodeError::InvalidEnvelope)?;
        entries.push(parse_summary(object)?);
    }
    reject_duplicates_by_cmp(&entries, |left, right| {
        left.address.with_addr(|left_address| {
            right
                .address
                .with_addr(|right_address| left_address.cmp(&right_address))
        })
    })
    .map_err(|error| match error {
        DuplicateError::Duplicate => RobotIpDecodeError::InvalidList,
        DuplicateError::Allocation => RobotIpDecodeError::Allocation,
    })?;
    Ok(RobotIpList(entries))
}

/// Decodes and identity-checks one detailed IP resource.
pub fn decode_robot_ip(
    checked: CheckedResponse<'_>,
    expected: &RobotIpAddress,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotIp, RobotIpDecodeError> {
    require_ok(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let wrapper = root
        .as_object_mut()
        .ok_or(RobotIpDecodeError::InvalidEnvelope)?;
    require_fields(wrapper, &["ip"])?;
    let object = wrapper
        .get_mut("ip")
        .and_then(Value::as_object_mut)
        .ok_or(RobotIpDecodeError::InvalidEnvelope)?;
    let detail_fields = [
        "ip",
        "gateway",
        "mask",
        "broadcast",
        "server_ip",
        "server_number",
        "locked",
        "separate_mac",
        "traffic_warnings",
        "traffic_hourly",
        "traffic_daily",
        "traffic_monthly",
    ];
    require_fields(object, &detail_fields)?;
    let summary = parse_summary_fields(object)?;
    if &summary.address != expected {
        return Err(RobotIpDecodeError::ResponseIdentityMismatch);
    }
    let gateway = parse_address(object, "gateway")?;
    let broadcast = parse_address(object, "broadcast")?;
    let prefix = required_u64(object, "mask")
        .and_then(|value| u8::try_from(value).map_err(|_| RobotIpDecodeError::InvalidNetwork))?;
    validate_network(&summary.address, &gateway, &broadcast, prefix)?;
    Ok(RobotIp {
        summary,
        gateway,
        prefix,
        broadcast,
    })
}

/// Decodes and identity-checks one separate-MAC response.
pub fn decode_robot_ip_mac(
    checked: CheckedResponse<'_>,
    expected: &RobotIpAddress,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotIpMac, RobotIpDecodeError> {
    require_ok(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let wrapper = root
        .as_object_mut()
        .ok_or(RobotIpDecodeError::InvalidEnvelope)?;
    require_fields(wrapper, &["mac"])?;
    let object = wrapper
        .get_mut("mac")
        .and_then(Value::as_object_mut)
        .ok_or(RobotIpDecodeError::InvalidEnvelope)?;
    require_fields(object, &["ip", "mac"])?;
    let address = parse_address(object, "ip")?;
    if &address != expected {
        return Err(RobotIpDecodeError::ResponseIdentityMismatch);
    }
    let mac = parse_optional_mac(object, "mac")?;
    Ok(RobotIpMac { address, mac })
}

fn parse_summary(object: &mut Map) -> Result<RobotIpSummary, RobotIpDecodeError> {
    require_fields(
        object,
        &[
            "ip",
            "server_ip",
            "server_number",
            "locked",
            "separate_mac",
            "traffic_warnings",
            "traffic_hourly",
            "traffic_daily",
            "traffic_monthly",
        ],
    )?;
    parse_summary_fields(object)
}

fn parse_summary_fields(object: &mut Map) -> Result<RobotIpSummary, RobotIpDecodeError> {
    let address = parse_address(object, "ip")?;
    let server_address = parse_address(object, "server_ip")?;
    if !server_address.with_addr(|value| value.is_ipv4()) {
        return Err(RobotIpDecodeError::InvalidAddress);
    }
    let server_number = object
        .get("server_number")
        .and_then(|value| {
            value.try_with_unsigned_lexical(|digits| {
                RobotServerNumber::from_decimal_bytes(digits.as_bytes())
            })
        })
        .ok_or(RobotIpDecodeError::InvalidServerNumber)?
        .map_err(map_server_number)?;
    let locked = required_bool(object, "locked")?;
    let separate_mac = parse_optional_mac(object, "separate_mac")?;
    let traffic = RobotIpTrafficPolicy {
        enabled: required_bool(object, "traffic_warnings")?,
        hourly_megabytes: required_u64(object, "traffic_hourly")?,
        daily_megabytes: required_u64(object, "traffic_daily")?,
        monthly_gigabytes: required_u64(object, "traffic_monthly")?,
    };
    Ok(RobotIpSummary {
        address,
        server_address,
        server_number,
        locked,
        separate_mac,
        traffic,
    })
}

fn parse_address(object: &Map, field: &str) -> Result<RobotIpAddress, RobotIpDecodeError> {
    object
        .get(field)
        .ok_or(RobotIpDecodeError::InvalidEnvelope)?
        .try_with_str(|text| RobotIpAddress::new(text).map_err(map_address_error))
        .map_err(|_| RobotIpDecodeError::InvalidAddress)?
        .ok_or(RobotIpDecodeError::InvalidEnvelope)?
}

fn parse_optional_mac(
    object: &Map,
    field: &str,
) -> Result<Option<RobotMacAddress>, RobotIpDecodeError> {
    let value = object
        .get(field)
        .ok_or(RobotIpDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        return Ok(None);
    }
    value
        .try_with_str(|text| RobotMacAddress::new(text).map_err(map_mac_error))
        .map_err(|_| RobotIpDecodeError::InvalidMac)?
        .ok_or(RobotIpDecodeError::InvalidEnvelope)?
        .map(Some)
}

fn validate_network(
    address: &RobotIpAddress,
    gateway: &RobotIpAddress,
    broadcast: &RobotIpAddress,
    prefix: u8,
) -> Result<(), RobotIpDecodeError> {
    let address = address.with_addr(|value| value);
    let gateway = gateway.with_addr(|value| value);
    let broadcast = broadcast.with_addr(|value| value);
    match (address, gateway, broadcast) {
        (IpAddr::V4(address), IpAddr::V4(gateway), IpAddr::V4(broadcast)) if prefix <= 32 => {
            let mask = if prefix == 0 {
                0
            } else {
                let shift = 32_u32
                    .checked_sub(u32::from(prefix))
                    .ok_or(RobotIpDecodeError::InvalidNetwork)?;
                u32::MAX
                    .checked_shl(shift)
                    .ok_or(RobotIpDecodeError::InvalidNetwork)?
            };
            let network = u32::from(address) & mask;
            if u32::from(gateway) & mask == network && u32::from(broadcast) == network | !mask {
                Ok(())
            } else {
                Err(RobotIpDecodeError::InvalidNetwork)
            }
        }
        (IpAddr::V6(address), IpAddr::V6(gateway), IpAddr::V6(broadcast)) if prefix <= 128 => {
            let mask = if prefix == 0 {
                0
            } else {
                let shift = 128_u32
                    .checked_sub(u32::from(prefix))
                    .ok_or(RobotIpDecodeError::InvalidNetwork)?;
                u128::MAX
                    .checked_shl(shift)
                    .ok_or(RobotIpDecodeError::InvalidNetwork)?
            };
            let network = u128::from(address) & mask;
            if u128::from(gateway) & mask == network && u128::from(broadcast) & mask == network {
                Ok(())
            } else {
                Err(RobotIpDecodeError::InvalidNetwork)
            }
        }
        _ => Err(RobotIpDecodeError::InvalidNetwork),
    }
}

fn required_bool(object: &Map, field: &str) -> Result<bool, RobotIpDecodeError> {
    object
        .get(field)
        .and_then(Value::as_bool)
        .ok_or(RobotIpDecodeError::InvalidEnvelope)
}

fn required_u64(object: &Map, field: &str) -> Result<u64, RobotIpDecodeError> {
    object
        .get(field)
        .and_then(Value::as_u64)
        .ok_or(RobotIpDecodeError::InvalidEnvelope)
}

fn require_ok(checked: CheckedResponse<'_>) -> Result<(), RobotIpDecodeError> {
    if checked.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(RobotIpDecodeError::UnexpectedStatus)
    }
}

fn require_fields(object: &Map, fields: &[&str]) -> Result<(), RobotIpDecodeError> {
    if object.len() == fields.len() && fields.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotIpDecodeError::InvalidEnvelope)
    }
}

const fn map_address_error(error: RobotCancellationValueError) -> RobotIpDecodeError {
    match error {
        RobotCancellationValueError::Invalid => RobotIpDecodeError::InvalidAddress,
        RobotCancellationValueError::Allocation => RobotIpDecodeError::Allocation,
    }
}

const fn map_mac_error(error: RobotMacAddressError) -> RobotIpDecodeError {
    match error {
        RobotMacAddressError::Invalid => RobotIpDecodeError::InvalidMac,
        RobotMacAddressError::Allocation => RobotIpDecodeError::Allocation,
    }
}

const fn map_server_number(error: DecimalServerNumberError) -> RobotIpDecodeError {
    match error {
        DecimalServerNumberError::Invalid => RobotIpDecodeError::InvalidServerNumber,
        DecimalServerNumberError::Allocation => RobotIpDecodeError::Allocation,
    }
}

const fn map_json_error(error: JsonError) -> RobotIpDecodeError {
    if matches!(error, JsonError::Allocation) {
        RobotIpDecodeError::Allocation
    } else {
        RobotIpDecodeError::MalformedPayload
    }
}

impl RobotIpListRequest {
    /// Decodes and clears a response admitted by this request's policy.
    pub fn decode_response(
        self,
        checked: CheckedResponseGuard<'_>,
    ) -> Result<RobotIpList, RobotIpDecodeError> {
        let result = checked.decode_owned_with_workspace(decode_robot_ip_list)?;
        if let Some(expected) = self.server_address
            && result
                .as_slice()
                .iter()
                .any(|entry| entry.server_address != expected)
        {
            return Err(RobotIpDecodeError::ResponseIdentityMismatch);
        }
        Ok(result)
    }
}

impl RobotIpGetRequest {
    /// Decodes, identity-checks, and clears this request's response.
    pub fn decode_response(
        self,
        checked: CheckedResponseGuard<'_>,
    ) -> Result<RobotIp, RobotIpDecodeError> {
        checked.decode_owned_with_workspace(|response, workspace| {
            decode_robot_ip(response, &self.address, workspace)
        })
    }
}

use super::request::{RobotIpGetRequest, RobotIpListRequest};
