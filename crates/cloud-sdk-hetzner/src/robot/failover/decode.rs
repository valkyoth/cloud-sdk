use alloc::vec::Vec;
use core::net::IpAddr;

use cloud_sdk::operation::{CheckedResponse, CheckedResponseGuard};
use cloud_sdk::transport::{ResponseDecodeWorkspace, StatusCode};

use super::model::*;
use super::request::{RobotFailoverGetRequest, RobotFailoverListRequest};
use crate::robot::duplicates::{DuplicateError, reject_duplicates_by_cmp};
use crate::robot::server::identity::DecimalServerNumberError;
use crate::robot::{RobotCancellationValueError, RobotIpAddress, RobotServerNumber};
use crate::serde::strict_json::{JsonError, Map, Value, parse_with_scratch};

/// Failure while decoding a source-locked Robot failover success response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotFailoverDecodeError {
    /// The checked status was not the source-locked success status.
    UnexpectedStatus,
    /// JSON syntax, UTF-8, nesting, duplicates, or parser bounds were invalid.
    MalformedPayload,
    /// Required, nullable, or extra response fields violated the source shape.
    InvalidEnvelope,
    /// An address was malformed, noncanonical, or had the wrong family.
    InvalidAddress,
    /// The route netmask was noncontiguous or contradicted the route identity.
    InvalidRoute,
    /// A server number was zero, malformed, or outside `u64`.
    InvalidServerNumber,
    /// A list exceeded its bound or contained duplicate route identities.
    InvalidList,
    /// The response route did not match the exact request.
    ResponseIdentityMismatch,
    /// A successful mutation contradicted the requested routing state.
    MutationOutcomeMismatch,
    /// Bounded protected result storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotFailoverDecodeError,
    Self::UnexpectedStatus => "Robot failover response status is unexpected",
    Self::MalformedPayload => "Robot failover response JSON is malformed",
    Self::InvalidEnvelope => "Robot failover response envelope is invalid",
    Self::InvalidAddress => "Robot failover response address is invalid",
    Self::InvalidRoute => "Robot failover route and netmask are inconsistent",
    Self::InvalidServerNumber => "Robot failover server number is invalid",
    Self::InvalidList => "Robot failover response list is invalid",
    Self::ResponseIdentityMismatch => "Robot failover response identity does not match the request",
    Self::MutationOutcomeMismatch => "Robot failover mutation response contradicts the request",
    Self::Allocation => "Robot failover response allocation failed",
);

/// Decodes the checked `GET /failover` result.
pub fn decode_robot_failover_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailoverList, RobotFailoverDecodeError> {
    require_ok(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let values = root
        .take_array()
        .ok_or(RobotFailoverDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_FAILOVER_LIST_ITEMS {
        return Err(RobotFailoverDecodeError::InvalidList);
    }
    let mut entries = Vec::new();
    entries
        .try_reserve_exact(values.len())
        .map_err(|_| RobotFailoverDecodeError::Allocation)?;
    for mut value in values {
        entries.push(parse_wrapper(&mut value)?);
    }
    reject_duplicates_by_cmp(&entries, |left, right| {
        left.route
            .with_addr(|left| right.route.with_addr(|right| left.cmp(&right)))
    })
    .map_err(|error| match error {
        DuplicateError::Duplicate => RobotFailoverDecodeError::InvalidList,
        DuplicateError::Allocation => RobotFailoverDecodeError::Allocation,
    })?;
    Ok(RobotFailoverList(entries))
}

/// Decodes and identity-checks one failover route.
pub fn decode_robot_failover(
    checked: CheckedResponse<'_>,
    expected: &RobotIpAddress,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFailover, RobotFailoverDecodeError> {
    require_ok(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let result = parse_wrapper(&mut root)?;
    if &result.route == expected {
        Ok(result)
    } else {
        Err(RobotFailoverDecodeError::ResponseIdentityMismatch)
    }
}

fn parse_wrapper(value: &mut Value) -> Result<RobotFailover, RobotFailoverDecodeError> {
    let wrapper = value
        .as_object_mut()
        .ok_or(RobotFailoverDecodeError::InvalidEnvelope)?;
    require_fields(wrapper, &["failover"])?;
    let object = wrapper
        .get_mut("failover")
        .and_then(Value::as_object_mut)
        .ok_or(RobotFailoverDecodeError::InvalidEnvelope)?;
    parse_failover(object)
}

fn parse_failover(object: &mut Map) -> Result<RobotFailover, RobotFailoverDecodeError> {
    require_fields(
        object,
        &[
            "ip",
            "netmask",
            "server_ip",
            "server_ipv6_net",
            "server_number",
            "active_server_ip",
        ],
    )?;
    let route = parse_address(object, "ip")?;
    let netmask = parse_address(object, "netmask")?;
    let prefix = validate_route(&route, &netmask)?;
    let server_ipv4 = parse_address(object, "server_ip")?;
    if !server_ipv4.with_addr(|value| value.is_ipv4()) {
        return Err(RobotFailoverDecodeError::InvalidAddress);
    }
    let server_ipv6_network = parse_address(object, "server_ipv6_net")?;
    if !server_ipv6_network.with_addr(|value| value.is_ipv6()) {
        return Err(RobotFailoverDecodeError::InvalidAddress);
    }
    let server_number = object
        .get("server_number")
        .and_then(|value| {
            value.try_with_unsigned_lexical(|digits| {
                RobotServerNumber::from_decimal_bytes(digits.as_bytes())
            })
        })
        .ok_or(RobotFailoverDecodeError::InvalidServerNumber)?
        .map_err(map_server_number)?;
    let active_server = parse_optional_address(object, "active_server_ip")?;
    if active_server.as_ref().is_some_and(|active| {
        route.with_addr(|route| active.with_addr(|active| route.is_ipv4() != active.is_ipv4()))
    }) {
        return Err(RobotFailoverDecodeError::InvalidAddress);
    }
    Ok(RobotFailover {
        route,
        prefix,
        server_ipv4,
        server_ipv6_network,
        server_number,
        active_server,
    })
}

fn parse_address(object: &Map, field: &str) -> Result<RobotIpAddress, RobotFailoverDecodeError> {
    object
        .get(field)
        .ok_or(RobotFailoverDecodeError::InvalidEnvelope)?
        .try_with_str(|text| RobotIpAddress::new(text).map_err(map_address_error))
        .map_err(|_| RobotFailoverDecodeError::InvalidAddress)?
        .ok_or(RobotFailoverDecodeError::InvalidEnvelope)?
}

fn parse_optional_address(
    object: &Map,
    field: &str,
) -> Result<Option<RobotIpAddress>, RobotFailoverDecodeError> {
    let value = object
        .get(field)
        .ok_or(RobotFailoverDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        return Ok(None);
    }
    value
        .try_with_str(|text| RobotIpAddress::new(text).map_err(map_address_error))
        .map_err(|_| RobotFailoverDecodeError::InvalidAddress)?
        .ok_or(RobotFailoverDecodeError::InvalidEnvelope)?
        .map(Some)
}

fn validate_route(
    route: &RobotIpAddress,
    netmask: &RobotIpAddress,
) -> Result<u8, RobotFailoverDecodeError> {
    route.with_addr(|route| {
        netmask.with_addr(|netmask| match (route, netmask) {
            (IpAddr::V4(route), IpAddr::V4(mask)) => {
                let mask = u32::from(mask);
                if !contiguous_u32(mask) || u32::from(route) & mask != u32::from(route) {
                    return Err(RobotFailoverDecodeError::InvalidRoute);
                }
                u8::try_from(mask.count_ones()).map_err(|_| RobotFailoverDecodeError::InvalidRoute)
            }
            (IpAddr::V6(route), IpAddr::V6(mask)) => {
                let mask = u128::from(mask);
                if !contiguous_u128(mask) || u128::from(route) & mask != u128::from(route) {
                    return Err(RobotFailoverDecodeError::InvalidRoute);
                }
                u8::try_from(mask.count_ones()).map_err(|_| RobotFailoverDecodeError::InvalidRoute)
            }
            _ => Err(RobotFailoverDecodeError::InvalidRoute),
        })
    })
}

const fn contiguous_u32(mask: u32) -> bool {
    let inverse = !mask;
    inverse & inverse.wrapping_add(1) == 0
}

const fn contiguous_u128(mask: u128) -> bool {
    let inverse = !mask;
    inverse & inverse.wrapping_add(1) == 0
}

fn require_ok(checked: CheckedResponse<'_>) -> Result<(), RobotFailoverDecodeError> {
    if checked.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(RobotFailoverDecodeError::UnexpectedStatus)
    }
}

fn require_fields(object: &Map, fields: &[&str]) -> Result<(), RobotFailoverDecodeError> {
    if object.len() == fields.len() && fields.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotFailoverDecodeError::InvalidEnvelope)
    }
}

const fn map_address_error(error: RobotCancellationValueError) -> RobotFailoverDecodeError {
    match error {
        RobotCancellationValueError::Invalid => RobotFailoverDecodeError::InvalidAddress,
        RobotCancellationValueError::Allocation => RobotFailoverDecodeError::Allocation,
    }
}

const fn map_server_number(error: DecimalServerNumberError) -> RobotFailoverDecodeError {
    match error {
        DecimalServerNumberError::Invalid => RobotFailoverDecodeError::InvalidServerNumber,
        DecimalServerNumberError::Allocation => RobotFailoverDecodeError::Allocation,
    }
}

const fn map_json_error(error: JsonError) -> RobotFailoverDecodeError {
    if matches!(error, JsonError::Allocation) {
        RobotFailoverDecodeError::Allocation
    } else {
        RobotFailoverDecodeError::MalformedPayload
    }
}

impl RobotFailoverListRequest {
    /// Decodes and clears a response admitted by this request's policy.
    pub fn decode_response(
        self,
        checked: CheckedResponseGuard<'_>,
    ) -> Result<RobotFailoverList, RobotFailoverDecodeError> {
        checked.decode_owned_with_workspace(decode_robot_failover_list)
    }
}

impl RobotFailoverGetRequest {
    /// Decodes, identity-checks, and clears this request's response.
    pub fn decode_response(
        self,
        checked: CheckedResponseGuard<'_>,
    ) -> Result<RobotFailover, RobotFailoverDecodeError> {
        checked.decode_owned_with_workspace(|response, workspace| {
            decode_robot_failover(response, &self.route, workspace)
        })
    }
}
