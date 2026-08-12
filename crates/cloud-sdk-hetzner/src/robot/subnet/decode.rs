use alloc::vec::Vec;
use core::net::IpAddr;

use cloud_sdk::operation::{CheckedResponse, CheckedResponseGuard};
use cloud_sdk::transport::{ResponseDecodeWorkspace, StatusCode};

use super::model::*;
use super::{RobotSubnetGetRequest, RobotSubnetListRequest};
use crate::robot::duplicates::{DuplicateError, reject_duplicates_by_cmp};
use crate::robot::server::identity::DecimalServerNumberError;
use crate::robot::{
    RobotCancellationValueError, RobotIpAddress, RobotMacAddress, RobotMacAddressError,
    RobotServerNumber, RobotSubnetAddress,
};
use crate::serde::strict_json::{JsonError, Map, Value, parse_with_scratch};

/// Failure while decoding a source-locked Robot subnet success response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotSubnetDecodeError {
    /// The checked status was not the source-locked success status.
    UnexpectedStatus,
    /// JSON syntax, UTF-8, nesting, duplicates, or parser bounds were invalid.
    MalformedPayload,
    /// Required, nullable, or extra response fields violated the source shape.
    InvalidEnvelope,
    /// An IP address was malformed, noncanonical, or had the wrong family.
    InvalidAddress,
    /// A prefix or gateway conflicted with the addressed subnet.
    InvalidNetwork,
    /// A server number was zero, malformed, or outside `u64`.
    InvalidServerNumber,
    /// A MAC was malformed or noncanonical.
    InvalidMac,
    /// A list or MAC-choice map violated its source-locked bound or identity rules.
    InvalidList,
    /// The response identity did not match the exact request.
    ResponseIdentityMismatch,
    /// A successful mutation contradicted the requested state transition.
    MutationOutcomeMismatch,
    /// Bounded protected result storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotSubnetDecodeError,
    Self::UnexpectedStatus => "Robot subnet response status is unexpected",
    Self::MalformedPayload => "Robot subnet response JSON is malformed",
    Self::InvalidEnvelope => "Robot subnet response envelope is invalid",
    Self::InvalidAddress => "Robot subnet response address is invalid",
    Self::InvalidNetwork => "Robot subnet response network is inconsistent",
    Self::InvalidServerNumber => "Robot subnet response server number is invalid",
    Self::InvalidMac => "Robot subnet response MAC is invalid",
    Self::InvalidList => "Robot subnet response collection is invalid",
    Self::ResponseIdentityMismatch => "Robot subnet response identity does not match the request",
    Self::MutationOutcomeMismatch => "Robot subnet mutation response contradicts the request",
    Self::Allocation => "Robot subnet response allocation failed",
);

/// Decodes the checked `GET /subnet` result.
pub fn decode_robot_subnet_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotSubnetList, RobotSubnetDecodeError> {
    require_ok(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let values = root
        .take_array()
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_SUBNET_LIST_ITEMS {
        return Err(RobotSubnetDecodeError::InvalidList);
    }
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(values.len())
        .map_err(|_| RobotSubnetDecodeError::Allocation)?;
    for mut value in values {
        let wrapper = value
            .as_object_mut()
            .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?;
        require_fields(wrapper, &["subnet"])?;
        let object = wrapper
            .get_mut("subnet")
            .and_then(Value::as_object_mut)
            .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?;
        entries.push(parse_subnet(object)?);
    }
    reject_duplicates_by_cmp(&entries, |left, right| {
        left.address
            .with_addr(|left| right.address.with_addr(|right| left.cmp(&right)))
    })
    .map_err(map_duplicate_error)?;
    Ok(RobotSubnetList(entries))
}

/// Decodes and identity-checks one subnet resource.
pub fn decode_robot_subnet(
    checked: CheckedResponse<'_>,
    expected: &RobotSubnetAddress,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotSubnet, RobotSubnetDecodeError> {
    require_ok(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let wrapper = root
        .as_object_mut()
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?;
    require_fields(wrapper, &["subnet"])?;
    let object = wrapper
        .get_mut("subnet")
        .and_then(Value::as_object_mut)
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?;
    let subnet = parse_subnet(object)?;
    if &subnet.address != expected {
        return Err(RobotSubnetDecodeError::ResponseIdentityMismatch);
    }
    Ok(subnet)
}

/// Decodes and identity-checks one subnet-MAC response.
pub fn decode_robot_subnet_mac(
    checked: CheckedResponse<'_>,
    expected: &RobotSubnetAddress,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotSubnetMac, RobotSubnetDecodeError> {
    require_ok(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let wrapper = root
        .as_object_mut()
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?;
    require_fields(wrapper, &["mac"])?;
    let object = wrapper
        .get_mut("mac")
        .and_then(Value::as_object_mut)
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?;
    require_fields(object, &["ip", "mask", "mac", "possible_mac"])?;
    let address = parse_subnet_address(object, "ip")?;
    if &address != expected {
        return Err(RobotSubnetDecodeError::ResponseIdentityMismatch);
    }
    let prefix = parse_string_prefix(object, "mask", &address)?;
    let mac = parse_mac(
        object
            .get("mac")
            .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?,
    )?;
    let possible_object = object
        .get("possible_mac")
        .and_then(Value::as_object)
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?;
    if possible_object.len() == 0 || possible_object.len() > MAX_ROBOT_SUBNET_MAC_OPTIONS {
        return Err(RobotSubnetDecodeError::InvalidList);
    }
    let mut possible = Vec::new();
    possible
        .try_reserve_exact(possible_object.len())
        .map_err(|_| RobotSubnetDecodeError::Allocation)?;
    possible_object.try_for_each(|key, value| {
        let address = RobotIpAddress::new(key).map_err(map_address_error)?;
        let option_mac = parse_mac(value)?;
        possible.push(RobotSubnetMacOption {
            address,
            mac: option_mac,
        });
        Ok::<(), RobotSubnetDecodeError>(())
    })?;
    let advertised = possible.iter().any(|option| option.mac == mac);
    if !advertised {
        return Err(RobotSubnetDecodeError::InvalidMac);
    }
    Ok(RobotSubnetMac {
        address,
        prefix,
        mac,
        possible,
    })
}

fn parse_subnet(object: &mut Map) -> Result<RobotSubnet, RobotSubnetDecodeError> {
    require_fields(
        object,
        &[
            "ip",
            "mask",
            "gateway",
            "server_ip",
            "server_number",
            "failover",
            "locked",
            "traffic_warnings",
            "traffic_hourly",
            "traffic_daily",
            "traffic_monthly",
        ],
    )?;
    let address = parse_subnet_address(object, "ip")?;
    let prefix = required_u64(object, "mask").and_then(|value| {
        u8::try_from(value).map_err(|_| RobotSubnetDecodeError::InvalidNetwork)
    })?;
    let gateway = parse_address(object, "gateway")?;
    validate_network(&address, &gateway, prefix)?;
    let server_address = parse_optional_address(object, "server_ip")?;
    if server_address
        .as_ref()
        .is_some_and(|value| !value.with_addr(|value| value.is_ipv4()))
    {
        return Err(RobotSubnetDecodeError::InvalidAddress);
    }
    let server_number = object
        .get("server_number")
        .and_then(|value| {
            value.try_with_unsigned_lexical(|digits| {
                RobotServerNumber::from_decimal_bytes(digits.as_bytes())
            })
        })
        .ok_or(RobotSubnetDecodeError::InvalidServerNumber)?
        .map_err(map_server_number)?;
    let traffic = RobotSubnetTrafficPolicy {
        enabled: required_bool(object, "traffic_warnings")?,
        hourly_megabytes: required_u64(object, "traffic_hourly")?,
        daily_megabytes: required_u64(object, "traffic_daily")?,
        monthly_gigabytes: required_u64(object, "traffic_monthly")?,
    };
    Ok(RobotSubnet {
        address,
        prefix,
        gateway,
        server_address,
        server_number,
        failover: required_bool(object, "failover")?,
        locked: required_bool(object, "locked")?,
        traffic,
    })
}

fn parse_subnet_address(
    object: &Map,
    field: &str,
) -> Result<RobotSubnetAddress, RobotSubnetDecodeError> {
    object
        .get(field)
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?
        .try_with_str(|text| RobotSubnetAddress::new(text).map_err(map_address_error))
        .map_err(|_| RobotSubnetDecodeError::InvalidAddress)?
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?
}

fn parse_address(object: &Map, field: &str) -> Result<RobotIpAddress, RobotSubnetDecodeError> {
    object
        .get(field)
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?
        .try_with_str(|text| RobotIpAddress::new(text).map_err(map_address_error))
        .map_err(|_| RobotSubnetDecodeError::InvalidAddress)?
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?
}

fn parse_optional_address(
    object: &Map,
    field: &str,
) -> Result<Option<RobotIpAddress>, RobotSubnetDecodeError> {
    let value = object
        .get(field)
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        return Ok(None);
    }
    parse_address(object, field).map(Some)
}

fn parse_mac(value: &Value) -> Result<RobotMacAddress, RobotSubnetDecodeError> {
    value
        .try_with_str(|text| RobotMacAddress::new(text).map_err(map_mac_error))
        .map_err(|_| RobotSubnetDecodeError::InvalidMac)?
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?
}

fn parse_string_prefix(
    object: &Map,
    field: &str,
    address: &RobotSubnetAddress,
) -> Result<u8, RobotSubnetDecodeError> {
    object
        .get(field)
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?
        .try_with_str(|text| {
            let bytes = text.as_bytes();
            if bytes.is_empty()
                || bytes.len() > 3
                || (bytes.len() > 1 && bytes.first() == Some(&b'0'))
                || bytes.iter().any(|byte| !byte.is_ascii_digit())
            {
                return Err(RobotSubnetDecodeError::InvalidNetwork);
            }
            let prefix = text
                .parse::<u8>()
                .map_err(|_| RobotSubnetDecodeError::InvalidNetwork)?;
            validate_prefix(address, prefix)?;
            Ok(prefix)
        })
        .map_err(|_| RobotSubnetDecodeError::InvalidNetwork)?
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)?
}

fn validate_prefix(address: &RobotSubnetAddress, prefix: u8) -> Result<(), RobotSubnetDecodeError> {
    if address.with_addr(|address| match address {
        IpAddr::V4(_) => prefix <= 32,
        IpAddr::V6(_) => prefix <= 128,
    }) {
        Ok(())
    } else {
        Err(RobotSubnetDecodeError::InvalidNetwork)
    }
}

fn validate_network(
    address: &RobotSubnetAddress,
    gateway: &RobotIpAddress,
    prefix: u8,
) -> Result<(), RobotSubnetDecodeError> {
    validate_prefix(address, prefix)?;
    let address = address.with_addr(|value| value);
    let gateway = gateway.with_addr(|value| value);
    let same_network = match (address, gateway) {
        (IpAddr::V4(address), IpAddr::V4(gateway)) => {
            let Some(mask) = prefix_mask_v4(prefix) else {
                return Err(RobotSubnetDecodeError::InvalidNetwork);
            };
            u32::from(address) & mask == u32::from(gateway) & mask
        }
        (IpAddr::V6(address), IpAddr::V6(gateway)) => {
            let Some(mask) = prefix_mask_v6(prefix) else {
                return Err(RobotSubnetDecodeError::InvalidNetwork);
            };
            u128::from(address) & mask == u128::from(gateway) & mask
        }
        _ => false,
    };
    if same_network {
        Ok(())
    } else {
        Err(RobotSubnetDecodeError::InvalidNetwork)
    }
}

fn required_bool(object: &Map, field: &str) -> Result<bool, RobotSubnetDecodeError> {
    object
        .get(field)
        .and_then(Value::as_bool)
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)
}

fn required_u64(object: &Map, field: &str) -> Result<u64, RobotSubnetDecodeError> {
    object
        .get(field)
        .and_then(Value::as_u64)
        .ok_or(RobotSubnetDecodeError::InvalidEnvelope)
}

fn require_ok(checked: CheckedResponse<'_>) -> Result<(), RobotSubnetDecodeError> {
    if checked.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(RobotSubnetDecodeError::UnexpectedStatus)
    }
}

fn require_fields(object: &Map, fields: &[&str]) -> Result<(), RobotSubnetDecodeError> {
    if object.len() == fields.len() && fields.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotSubnetDecodeError::InvalidEnvelope)
    }
}

const fn map_duplicate_error(error: DuplicateError) -> RobotSubnetDecodeError {
    match error {
        DuplicateError::Duplicate => RobotSubnetDecodeError::InvalidList,
        DuplicateError::Allocation => RobotSubnetDecodeError::Allocation,
    }
}

const fn map_address_error(error: RobotCancellationValueError) -> RobotSubnetDecodeError {
    match error {
        RobotCancellationValueError::Invalid => RobotSubnetDecodeError::InvalidAddress,
        RobotCancellationValueError::Allocation => RobotSubnetDecodeError::Allocation,
    }
}

const fn map_mac_error(error: RobotMacAddressError) -> RobotSubnetDecodeError {
    match error {
        RobotMacAddressError::Invalid => RobotSubnetDecodeError::InvalidMac,
        RobotMacAddressError::Allocation => RobotSubnetDecodeError::Allocation,
    }
}

const fn map_server_number(error: DecimalServerNumberError) -> RobotSubnetDecodeError {
    match error {
        DecimalServerNumberError::Invalid => RobotSubnetDecodeError::InvalidServerNumber,
        DecimalServerNumberError::Allocation => RobotSubnetDecodeError::Allocation,
    }
}

const fn map_json_error(error: JsonError) -> RobotSubnetDecodeError {
    if matches!(error, JsonError::Allocation) {
        RobotSubnetDecodeError::Allocation
    } else {
        RobotSubnetDecodeError::MalformedPayload
    }
}

impl RobotSubnetListRequest {
    /// Decodes and clears a response admitted by this request's policy.
    pub fn decode_response(
        self,
        checked: CheckedResponseGuard<'_>,
    ) -> Result<RobotSubnetList, RobotSubnetDecodeError> {
        let result = checked.decode_owned_with_workspace(decode_robot_subnet_list)?;
        if let Some(expected) = self.server_address
            && result.as_slice().iter().any(|entry| {
                entry
                    .server_address
                    .as_ref()
                    .is_none_or(|actual| actual != &expected)
            })
        {
            return Err(RobotSubnetDecodeError::ResponseIdentityMismatch);
        }
        Ok(result)
    }
}

impl RobotSubnetGetRequest {
    /// Decodes, identity-checks, and clears this request's response.
    pub fn decode_response(
        self,
        checked: CheckedResponseGuard<'_>,
    ) -> Result<RobotSubnet, RobotSubnetDecodeError> {
        checked.decode_owned_with_workspace(|response, workspace| {
            decode_robot_subnet(response, &self.address, workspace)
        })
    }
}
