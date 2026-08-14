use alloc::vec::Vec;
use core::cmp::Ordering;
use core::net::IpAddr;

use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::{ResponseDecodeWorkspace, StatusCode};

use super::model::*;
use super::prepare::{
    MAX_ROBOT_VSWITCH_ITEM_RESPONSE_BYTES, MAX_ROBOT_VSWITCH_LIST_RESPONSE_BYTES,
};
use super::{RobotVSwitchId, RobotVSwitchName, RobotVSwitchValueError, RobotVlanId};
use crate::robot::duplicates::{DuplicateError, reject_duplicates_by_cmp};
use crate::robot::server::identity::DecimalServerNumberError;
use crate::robot::{RobotCancellationValueError, RobotIpAddress, RobotServerNumber};
use crate::serde::strict_json::{JsonError, Map, Value, parse_with_scratch};

/// Failure while decoding a source-locked Robot vSwitch success response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotVSwitchDecodeError {
    /// The checked status was not admitted for this operation.
    UnexpectedStatus,
    /// The body exceeded this operation's independent decode limit.
    ResponseTooLarge,
    /// JSON syntax, UTF-8, nesting, duplicates, or parser bounds were invalid.
    MalformedPayload,
    /// Required or extra fields violated the exact source shape.
    InvalidEnvelope,
    /// A vSwitch identity, VLAN, or name was invalid.
    InvalidVSwitch,
    /// A server membership was invalid or contradictory.
    InvalidServer,
    /// A subnet or Cloud Network route was invalid or noncanonical.
    InvalidNetwork,
    /// A bounded collection exceeded its limit or repeated an identity.
    InvalidList,
    /// The response vSwitch did not match the exact request.
    ResponseIdentityMismatch,
    /// A successful creation contradicted the requested configuration.
    MutationOutcomeMismatch,
    /// Bounded protected result storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotVSwitchDecodeError,
    Self::UnexpectedStatus => "Robot vSwitch response status is unexpected",
    Self::ResponseTooLarge => "Robot vSwitch response exceeds its operation limit",
    Self::MalformedPayload => "Robot vSwitch response JSON is malformed",
    Self::InvalidEnvelope => "Robot vSwitch response envelope is invalid",
    Self::InvalidVSwitch => "Robot vSwitch response identity or configuration is invalid",
    Self::InvalidServer => "Robot vSwitch server membership is invalid",
    Self::InvalidNetwork => "Robot vSwitch network route is invalid",
    Self::InvalidList => "Robot vSwitch response collection is invalid",
    Self::ResponseIdentityMismatch => "Robot vSwitch response identity does not match the request",
    Self::MutationOutcomeMismatch => "Robot vSwitch mutation response contradicts the request",
    Self::Allocation => "Robot vSwitch response allocation failed",
);

pub(crate) fn decode_robot_vswitch_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotVSwitchList, RobotVSwitchDecodeError> {
    require_status(checked, StatusCode::OK)?;
    require_body_limit(checked, MAX_ROBOT_VSWITCH_LIST_RESPONSE_BYTES)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let values = root
        .take_array()
        .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_VSWITCH_LIST_ITEMS {
        return Err(RobotVSwitchDecodeError::InvalidList);
    }
    let mut summaries = Vec::new();
    summaries
        .try_reserve_exact(values.len())
        .map_err(|_| RobotVSwitchDecodeError::Allocation)?;
    for mut value in values {
        summaries.push(parse_summary(&mut value)?);
    }
    reject_duplicates_by_cmp(&summaries, |left, right| left.id.cmp(&right.id))
        .map_err(map_duplicate)?;
    reject_duplicates_by_cmp(&summaries, |left, right| left.vlan.cmp(&right.vlan))
        .map_err(map_duplicate)?;
    Ok(RobotVSwitchList(summaries))
}

pub(crate) fn decode_robot_vswitch(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotVSwitch, RobotVSwitchDecodeError> {
    if !matches!(checked.status(), StatusCode::OK | StatusCode::CREATED) {
        return Err(RobotVSwitchDecodeError::UnexpectedStatus);
    }
    require_body_limit(checked, MAX_ROBOT_VSWITCH_ITEM_RESPONSE_BYTES)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    parse_detail(&mut root)
}

fn parse_summary(value: &mut Value) -> Result<RobotVSwitchSummary, RobotVSwitchDecodeError> {
    let object = value
        .as_object_mut()
        .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?;
    require_fields(object, &["id", "name", "vlan", "cancelled"])?;
    Ok(RobotVSwitchSummary {
        id: parse_id(object, "id")?,
        name: parse_name(object, "name")?,
        vlan: parse_vlan(object, "vlan")?,
        cancelled: parse_bool(object, "cancelled")?,
    })
}

fn parse_detail(value: &mut Value) -> Result<RobotVSwitch, RobotVSwitchDecodeError> {
    let object = value
        .as_object_mut()
        .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?;
    require_fields(
        object,
        &[
            "id",
            "name",
            "vlan",
            "cancelled",
            "server",
            "subnet",
            "cloud_network",
        ],
    )?;
    let id = parse_id(object, "id")?;
    let name = parse_name(object, "name")?;
    let vlan = parse_vlan(object, "vlan")?;
    let cancelled = parse_bool(object, "cancelled")?;
    let servers = parse_servers(object)?;
    let subnets = parse_subnets(object)?;
    let cloud_networks = parse_cloud_networks(object)?;
    Ok(RobotVSwitch {
        id,
        name,
        vlan,
        cancelled,
        servers,
        subnets,
        cloud_networks,
    })
}

fn parse_servers(object: &mut Map) -> Result<Vec<RobotVSwitchServer>, RobotVSwitchDecodeError> {
    let values = object
        .get_mut("server")
        .and_then(Value::take_array)
        .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_VSWITCH_MEMBER_SERVERS {
        return Err(RobotVSwitchDecodeError::InvalidList);
    }
    let mut servers = Vec::new();
    servers
        .try_reserve_exact(values.len())
        .map_err(|_| RobotVSwitchDecodeError::Allocation)?;
    for mut value in values {
        let member = value
            .as_object_mut()
            .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?;
        require_fields(
            member,
            &["server_ip", "server_ipv6_net", "server_number", "status"],
        )?;
        let ipv4 = parse_address(member, "server_ip")?;
        let ipv6_network = parse_address(member, "server_ipv6_net")?;
        if !ipv4.with_addr(|value| value.is_ipv4())
            || !ipv6_network.with_addr(|value| value.is_ipv6())
        {
            return Err(RobotVSwitchDecodeError::InvalidServer);
        }
        let number = parse_server_number(member)?;
        let status = parse_server_status(member)?;
        servers.push(RobotVSwitchServer {
            ipv4,
            ipv6_network,
            number,
            status,
        });
    }
    reject_duplicates_by_cmp(&servers, |left, right| left.number.cmp(&right.number))
        .map_err(map_duplicate)?;
    Ok(servers)
}

fn parse_subnets(object: &mut Map) -> Result<Vec<RobotVSwitchSubnet>, RobotVSwitchDecodeError> {
    let values = object
        .get_mut("subnet")
        .and_then(Value::take_array)
        .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_VSWITCH_SUBNETS {
        return Err(RobotVSwitchDecodeError::InvalidList);
    }
    let mut subnets = Vec::new();
    subnets
        .try_reserve_exact(values.len())
        .map_err(|_| RobotVSwitchDecodeError::Allocation)?;
    for mut value in values {
        let route = value
            .as_object_mut()
            .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?;
        require_fields(route, &["ip", "mask", "gateway"])?;
        let (network, prefix, gateway) = parse_route(route)?;
        subnets.push(RobotVSwitchSubnet {
            network,
            prefix,
            gateway,
        });
    }
    reject_duplicates_by_cmp(&subnets, |left, right| {
        compare_address(&left.network, &right.network)
    })
    .map_err(map_duplicate)?;
    Ok(subnets)
}

fn parse_cloud_networks(
    object: &mut Map,
) -> Result<Vec<RobotVSwitchCloudNetwork>, RobotVSwitchDecodeError> {
    let values = object
        .get_mut("cloud_network")
        .and_then(Value::take_array)
        .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_VSWITCH_CLOUD_NETWORKS {
        return Err(RobotVSwitchDecodeError::InvalidList);
    }
    let mut networks = Vec::new();
    networks
        .try_reserve_exact(values.len())
        .map_err(|_| RobotVSwitchDecodeError::Allocation)?;
    for mut value in values {
        let route = value
            .as_object_mut()
            .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?;
        require_fields(route, &["id", "ip", "mask", "gateway"])?;
        let id = route
            .get("id")
            .and_then(Value::as_u64)
            .filter(|id| *id != 0)
            .ok_or(RobotVSwitchDecodeError::InvalidNetwork)?;
        let (network, prefix, gateway) = parse_route(route)?;
        networks.push(RobotVSwitchCloudNetwork {
            id,
            network,
            prefix,
            gateway,
        });
    }
    reject_duplicates_by_cmp(&networks, |left, right| left.id.cmp(&right.id))
        .map_err(map_duplicate)?;
    Ok(networks)
}

fn parse_route(
    object: &Map,
) -> Result<(RobotIpAddress, u8, RobotIpAddress), RobotVSwitchDecodeError> {
    let network = parse_address(object, "ip")?;
    let prefix = object
        .get("mask")
        .and_then(Value::as_u64)
        .and_then(|value| u8::try_from(value).ok())
        .ok_or(RobotVSwitchDecodeError::InvalidNetwork)?;
    let gateway = parse_address(object, "gateway")?;
    let valid = network
        .with_addr(|network| gateway.with_addr(|gateway| valid_route(network, prefix, gateway)));
    if valid {
        Ok((network, prefix, gateway))
    } else {
        Err(RobotVSwitchDecodeError::InvalidNetwork)
    }
}

fn valid_route(network: IpAddr, prefix: u8, gateway: IpAddr) -> bool {
    match (network, gateway) {
        (IpAddr::V4(network), IpAddr::V4(gateway)) if prefix <= 32 => {
            let Some(mask) = route_mask_v4(prefix) else {
                return false;
            };
            u32::from(network) & mask == u32::from(network)
                && u32::from(gateway) & mask == u32::from(network)
        }
        (IpAddr::V6(network), IpAddr::V6(gateway)) if prefix <= 128 => {
            let Some(mask) = route_mask_v6(prefix) else {
                return false;
            };
            u128::from(network) & mask == u128::from(network)
                && u128::from(gateway) & mask == u128::from(network)
        }
        _ => false,
    }
}

fn route_mask_v4(prefix: u8) -> Option<u32> {
    if prefix == 0 {
        Some(0)
    } else {
        32_u32
            .checked_sub(u32::from(prefix))
            .and_then(|shift| u32::MAX.checked_shl(shift))
    }
}

fn route_mask_v6(prefix: u8) -> Option<u128> {
    if prefix == 0 {
        Some(0)
    } else {
        128_u32
            .checked_sub(u32::from(prefix))
            .and_then(|shift| u128::MAX.checked_shl(shift))
    }
}

fn parse_id(object: &Map, field: &str) -> Result<RobotVSwitchId, RobotVSwitchDecodeError> {
    object
        .get(field)
        .and_then(Value::as_u64)
        .ok_or(RobotVSwitchDecodeError::InvalidVSwitch)
        .and_then(|value| RobotVSwitchId::new(value).map_err(map_value_error))
}

fn parse_vlan(object: &Map, field: &str) -> Result<RobotVlanId, RobotVSwitchDecodeError> {
    object
        .get(field)
        .and_then(Value::as_u64)
        .and_then(|value| u16::try_from(value).ok())
        .ok_or(RobotVSwitchDecodeError::InvalidVSwitch)
        .and_then(|value| RobotVlanId::new(value).map_err(map_value_error))
}

fn parse_name(object: &Map, field: &str) -> Result<RobotVSwitchName, RobotVSwitchDecodeError> {
    object
        .get(field)
        .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?
        .try_with_str(RobotVSwitchName::new)
        .map_err(|_| RobotVSwitchDecodeError::InvalidVSwitch)?
        .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?
        .map_err(map_value_error)
}

fn parse_bool(object: &Map, field: &str) -> Result<bool, RobotVSwitchDecodeError> {
    object
        .get(field)
        .and_then(Value::as_bool)
        .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)
}

fn parse_address(object: &Map, field: &str) -> Result<RobotIpAddress, RobotVSwitchDecodeError> {
    object
        .get(field)
        .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?
        .try_with_str(RobotIpAddress::new)
        .map_err(|_| RobotVSwitchDecodeError::InvalidNetwork)?
        .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?
        .map_err(map_address_error)
}

fn parse_server_number(object: &Map) -> Result<RobotServerNumber, RobotVSwitchDecodeError> {
    object
        .get("server_number")
        .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?
        .try_with_unsigned_lexical(|digits| {
            RobotServerNumber::from_decimal_bytes(digits.as_bytes())
        })
        .ok_or(RobotVSwitchDecodeError::InvalidServer)?
        .map_err(map_decimal_server_error)
}

fn parse_server_status(object: &Map) -> Result<RobotVSwitchServerStatus, RobotVSwitchDecodeError> {
    object
        .get("status")
        .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?
        .try_with_str(|status| match status {
            "ready" => Ok(RobotVSwitchServerStatus::Ready),
            "in process" => Ok(RobotVSwitchServerStatus::InProcess),
            "failed" => Ok(RobotVSwitchServerStatus::Failed),
            _ => Err(RobotVSwitchDecodeError::InvalidServer),
        })
        .map_err(|_| RobotVSwitchDecodeError::InvalidServer)?
        .ok_or(RobotVSwitchDecodeError::InvalidEnvelope)?
}

fn compare_address(left: &RobotIpAddress, right: &RobotIpAddress) -> Ordering {
    left.with_addr(|left| right.with_addr(|right| left.cmp(&right)))
}

fn require_status(
    checked: CheckedResponse<'_>,
    expected: StatusCode,
) -> Result<(), RobotVSwitchDecodeError> {
    if checked.status() == expected {
        Ok(())
    } else {
        Err(RobotVSwitchDecodeError::UnexpectedStatus)
    }
}

fn require_body_limit(
    checked: CheckedResponse<'_>,
    maximum: usize,
) -> Result<(), RobotVSwitchDecodeError> {
    if checked.body().len() <= maximum {
        Ok(())
    } else {
        Err(RobotVSwitchDecodeError::ResponseTooLarge)
    }
}

fn require_fields(object: &Map, fields: &[&str]) -> Result<(), RobotVSwitchDecodeError> {
    if object.len() == fields.len() && fields.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotVSwitchDecodeError::InvalidEnvelope)
    }
}

const fn map_json_error(error: JsonError) -> RobotVSwitchDecodeError {
    if matches!(error, JsonError::Allocation) {
        RobotVSwitchDecodeError::Allocation
    } else {
        RobotVSwitchDecodeError::MalformedPayload
    }
}

const fn map_value_error(error: RobotVSwitchValueError) -> RobotVSwitchDecodeError {
    if matches!(error, RobotVSwitchValueError::Allocation) {
        RobotVSwitchDecodeError::Allocation
    } else {
        RobotVSwitchDecodeError::InvalidVSwitch
    }
}

const fn map_address_error(error: RobotCancellationValueError) -> RobotVSwitchDecodeError {
    if matches!(error, RobotCancellationValueError::Allocation) {
        RobotVSwitchDecodeError::Allocation
    } else {
        RobotVSwitchDecodeError::InvalidNetwork
    }
}

const fn map_decimal_server_error(error: DecimalServerNumberError) -> RobotVSwitchDecodeError {
    match error {
        DecimalServerNumberError::Invalid => RobotVSwitchDecodeError::InvalidServer,
        DecimalServerNumberError::Allocation => RobotVSwitchDecodeError::Allocation,
    }
}

const fn map_duplicate(error: DuplicateError) -> RobotVSwitchDecodeError {
    match error {
        DuplicateError::Duplicate => RobotVSwitchDecodeError::InvalidList,
        DuplicateError::Allocation => RobotVSwitchDecodeError::Allocation,
    }
}
