use alloc::vec::Vec;

use cloud_sdk::operation::{CheckedResponse, CheckedResponseGuard};
use cloud_sdk::transport::{ResponseDecodeWorkspace, StatusCode};

use super::duplicates::{reject_duplicates, reject_duplicates_by};
use super::identity::{DecimalServerNumberError, RobotServerNumber};
use super::model::{
    MAX_ROBOT_SERVER_ADDRESSES, MAX_ROBOT_SERVER_LIST_ITEMS, RobotServer, RobotServerList,
    RobotServerSummary,
};
use super::protected::{
    ProtectedFlag, ProtectedIpAddr, RobotServerCapabilities, RobotServerDate, RobotServerStatus,
    RobotServerSubnet, RobotStorageBoxNumber,
};
use super::request::{RobotServerGetRequest, RobotServerListRequest, RobotServerUpdateRequest};
use crate::serde::SensitiveText;
use crate::serde::strict_json::{JsonError, Map, Value, parse_with_scratch};

const MAX_SERVER_TEXT_BYTES: usize = 4_096;

/// Failure while decoding a source-locked Robot server success response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotServerDecodeError {
    /// A checked response did not carry `200 OK`.
    UnexpectedStatus,
    /// JSON syntax, UTF-8, duplicate keys, or parser bounds were invalid.
    MalformedPayload,
    /// Required, optional, extra, or typed fields violated the source lock.
    InvalidEnvelope,
    /// A provider resource identifier was invalid.
    InvalidIdentifier,
    /// Bounded provider text was empty or unsafe for diagnostics.
    InvalidText,
    /// A single address was malformed or contradicted the main address.
    InvalidAddress,
    /// A subnet family, prefix, or canonical network was invalid.
    InvalidSubnet,
    /// A paid-through date was malformed or calendar-invalid.
    InvalidDate,
    /// Robot returned a server status outside the source-locked set.
    UnknownStatus,
    /// A list exceeded its explicit maximum.
    TooManyItems,
    /// A server, address, or subnet identity was duplicated.
    DuplicateIdentity,
    /// A detail response returned a different canonical server number.
    ResponseIdentityMismatch,
    /// Bounded owned result storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotServerDecodeError,
    Self::UnexpectedStatus => "Robot server response status is unexpected",
    Self::MalformedPayload => "Robot server response JSON is malformed",
    Self::InvalidEnvelope => "Robot server response envelope is invalid",
    Self::InvalidIdentifier => "Robot server response identifier is invalid",
    Self::InvalidText => "Robot server response text is invalid",
    Self::InvalidAddress => "Robot server response address is invalid",
    Self::InvalidSubnet => "Robot server response subnet is invalid",
    Self::InvalidDate => "Robot server response date is invalid",
    Self::UnknownStatus => "Robot server response status value is unknown",
    Self::TooManyItems => "Robot server response exceeds a collection limit",
    Self::DuplicateIdentity => "Robot server response contains a duplicate identity",
    Self::ResponseIdentityMismatch => "Robot server response identity does not match the request",
    Self::Allocation => "Robot server response allocation failed",
);

/// Decodes the checked `GET /server` result.
pub fn decode_robot_server_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotServerList, RobotServerDecodeError> {
    require_ok(checked)?;
    let mut value = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let values = value
        .take_array()
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_SERVER_LIST_ITEMS {
        return Err(RobotServerDecodeError::TooManyItems);
    }
    let mut servers = Vec::new();
    servers
        .try_reserve_exact(values.len())
        .map_err(|_| RobotServerDecodeError::Allocation)?;
    for mut value in values {
        let wrapper = value
            .as_object_mut()
            .ok_or(RobotServerDecodeError::InvalidEnvelope)?;
        require_fields(wrapper, &["server"])?;
        let server = wrapper
            .get_mut("server")
            .and_then(Value::as_object_mut)
            .ok_or(RobotServerDecodeError::InvalidEnvelope)?;
        let summary = parse_summary(server, SummaryShape::List)?;
        servers.push(summary);
    }
    reject_duplicates_by(&servers, RobotServerSummary::number)?;
    Ok(RobotServerList(servers))
}

/// Decodes a checked canonical get or update result and binds its identity.
pub fn decode_robot_server(
    checked: CheckedResponse<'_>,
    expected: &RobotServerNumber,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotServer, RobotServerDecodeError> {
    require_ok(checked)?;
    let mut value = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let root = value
        .as_object_mut()
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?;
    require_fields(root, &["server"])?;
    let object = root
        .get_mut("server")
        .and_then(Value::as_object_mut)
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?;
    let detailed_fields = [
        "server_ip",
        "server_ipv6_net",
        "server_number",
        "server_name",
        "product",
        "dc",
        "traffic",
        "status",
        "cancelled",
        "paid_until",
        "ip",
        "subnet",
        "reset",
        "rescue",
        "vnc",
        "windows",
        "plesk",
        "cpanel",
        "wol",
        "hot_swap",
    ];
    require_fields_with_optional(object, &detailed_fields, "linked_storagebox")?;
    let summary = parse_summary(object, SummaryShape::Detail)?;
    if summary.number() != expected {
        return Err(RobotServerDecodeError::ResponseIdentityMismatch);
    }
    let capability_fields = [
        "reset", "rescue", "vnc", "windows", "plesk", "cpanel", "wol", "hot_swap",
    ];
    if !capability_fields
        .iter()
        .all(|field| object.get(field).is_some_and(Value::is_bool))
    {
        return Err(RobotServerDecodeError::InvalidEnvelope);
    }
    let capabilities = RobotServerCapabilities::from_protected(|index, destination| {
        object
            .get(capability_fields.get(index).copied().unwrap_or(""))
            .and_then(|value| value.copy_bool_byte_to(destination))
            .unwrap_or_else(|| unreachable!("validated Robot capability changed type"));
    })
    .map_err(|_| RobotServerDecodeError::Allocation)?;
    let linked_storage_box = object
        .get("linked_storagebox")
        .map(|value| {
            value
                .try_with_unsigned_lexical(|digits| {
                    RobotStorageBoxNumber::from_decimal_bytes(digits.as_bytes())
                        .map_err(|_| RobotServerDecodeError::InvalidIdentifier)
                })
                .ok_or(RobotServerDecodeError::InvalidIdentifier)?
        })
        .transpose()?
        .flatten();
    Ok(RobotServer {
        summary,
        capabilities,
        linked_storage_box,
    })
}

impl RobotServerListRequest {
    /// Decodes and clears a response admitted by this request's prepared policy.
    pub fn decode_response(
        self,
        checked: CheckedResponseGuard<'_>,
    ) -> Result<RobotServerList, RobotServerDecodeError> {
        checked.decode_owned_with_workspace(decode_robot_server_list)
    }
}

impl RobotServerGetRequest {
    /// Decodes, identity-checks, and clears this request's response.
    pub fn decode_response(
        self,
        checked: CheckedResponseGuard<'_>,
    ) -> Result<RobotServer, RobotServerDecodeError> {
        checked.decode_owned_with_workspace(|response, workspace| {
            decode_robot_server(response, self.number(), workspace)
        })
    }
}

impl RobotServerUpdateRequest<'_> {
    /// Decodes, identity-checks, and clears this request's response.
    pub fn decode_response(
        self,
        checked: CheckedResponseGuard<'_>,
    ) -> Result<RobotServer, RobotServerDecodeError> {
        checked.decode_owned_with_workspace(|response, workspace| {
            decode_robot_server(response, self.number(), workspace)
        })
    }
}

#[derive(Clone, Copy)]
enum SummaryShape {
    List,
    Detail,
}

fn parse_summary(
    object: &mut Map,
    shape: SummaryShape,
) -> Result<RobotServerSummary, RobotServerDecodeError> {
    let summary_fields = [
        "server_ip",
        "server_ipv6_net",
        "server_number",
        "server_name",
        "product",
        "dc",
        "traffic",
        "status",
        "cancelled",
        "paid_until",
        "ip",
        "subnet",
    ];
    if matches!(shape, SummaryShape::List) {
        require_fields(object, &summary_fields)?;
    }
    let number = object
        .get("server_number")
        .and_then(|value| {
            value.try_with_unsigned_lexical(|digits| {
                RobotServerNumber::from_decimal_bytes(digits.as_bytes())
            })
        })
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?
        .map_err(map_number_error)?;
    let main_ipv4 = parse_text(object, "server_ip", |value| {
        ProtectedIpAddr::parse_ipv4(value).map_err(|_| RobotServerDecodeError::InvalidAddress)
    })?;
    let main_ipv6_network = parse_text(object, "server_ipv6_net", |value| {
        ProtectedIpAddr::parse_ipv6(value).map_err(|_| RobotServerDecodeError::InvalidAddress)
    })?;
    let name = take_text(object, "server_name")?;
    let product = take_text(object, "product")?;
    let datacenter = take_text(object, "dc")?;
    let traffic = take_text(object, "traffic")?;
    let status = parse_text(object, "status", |value| match value {
        "ready" => RobotServerStatus::ready().map_err(|_| RobotServerDecodeError::Allocation),
        "in process" => {
            RobotServerStatus::in_process().map_err(|_| RobotServerDecodeError::Allocation)
        }
        _ => Err(RobotServerDecodeError::UnknownStatus),
    })?;
    let cancelled_value = object
        .get("cancelled")
        .filter(|value| value.is_bool())
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?;
    let cancelled = ProtectedFlag::from_protected(|destination| {
        cancelled_value
            .copy_bool_byte_to(destination)
            .unwrap_or_else(|| unreachable!("validated Robot cancellation changed type"));
    })
    .map_err(|_| RobotServerDecodeError::Allocation)?;
    let paid_until = parse_text(object, "paid_until", |value| {
        RobotServerDate::parse(value).map_err(|_| RobotServerDecodeError::InvalidDate)
    })?;
    let addresses = parse_addresses(object)?;
    if !addresses.iter().any(|address| address == &main_ipv4) {
        return Err(RobotServerDecodeError::InvalidAddress);
    }
    let subnets = parse_subnets(object)?;
    Ok(RobotServerSummary {
        number,
        main_ipv4,
        main_ipv6_network,
        name,
        product,
        datacenter,
        traffic,
        status,
        cancelled,
        paid_until,
        addresses,
        subnets,
    })
}

fn parse_addresses(object: &mut Map) -> Result<Vec<ProtectedIpAddr>, RobotServerDecodeError> {
    let values = object
        .get_mut("ip")
        .and_then(Value::take_array)
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?;
    if values.is_empty() || values.len() > MAX_ROBOT_SERVER_ADDRESSES {
        return Err(RobotServerDecodeError::TooManyItems);
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(values.len())
        .map_err(|_| RobotServerDecodeError::Allocation)?;
    for value in values {
        let address = value
            .try_with_str(|text| {
                ProtectedIpAddr::parse(text).map_err(|_| RobotServerDecodeError::InvalidAddress)
            })
            .map_err(|_| RobotServerDecodeError::InvalidAddress)?
            .ok_or(RobotServerDecodeError::InvalidEnvelope)??;
        result.push(address);
    }
    reject_duplicates(&result)?;
    Ok(result)
}

fn parse_subnets(
    object: &mut Map,
) -> Result<Option<Vec<RobotServerSubnet>>, RobotServerDecodeError> {
    let value = object
        .get_mut("subnet")
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        return Ok(None);
    }
    let values = value
        .take_array()
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_SERVER_ADDRESSES {
        return Err(RobotServerDecodeError::TooManyItems);
    }
    let mut result = Vec::new();
    result
        .try_reserve_exact(values.len())
        .map_err(|_| RobotServerDecodeError::Allocation)?;
    for mut value in values {
        let subnet = value
            .as_object_mut()
            .ok_or(RobotServerDecodeError::InvalidEnvelope)?;
        require_fields(subnet, &["ip", "mask"])?;
        result.push(parse_subnet(subnet)?);
    }
    reject_duplicates(&result)?;
    Ok(Some(result))
}

fn parse_subnet(object: &Map) -> Result<RobotServerSubnet, RobotServerDecodeError> {
    let network = object
        .get("ip")
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?;
    let prefix = object
        .get("mask")
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?;
    network
        .try_with_str(|network| {
            prefix.try_with_str(|prefix| {
                RobotServerSubnet::parse(network, prefix)
                    .map_err(|_| RobotServerDecodeError::InvalidSubnet)
            })
        })
        .map_err(|_| RobotServerDecodeError::InvalidSubnet)?
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?
        .map_err(|_| RobotServerDecodeError::InvalidSubnet)?
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?
}

fn map_number_error(error: DecimalServerNumberError) -> RobotServerDecodeError {
    match error {
        DecimalServerNumberError::Invalid => RobotServerDecodeError::InvalidIdentifier,
        DecimalServerNumberError::Allocation => RobotServerDecodeError::Allocation,
    }
}

fn take_text(object: &mut Map, field: &str) -> Result<SensitiveText, RobotServerDecodeError> {
    let text = object
        .get_mut(field)
        .and_then(Value::take_string)
        .map(SensitiveText::new)
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?;
    text.validate(MAX_SERVER_TEXT_BYTES)
        .map_err(|_| RobotServerDecodeError::InvalidText)?;
    Ok(text)
}

fn parse_text<T>(
    object: &Map,
    field: &str,
    parse: impl FnOnce(&str) -> Result<T, RobotServerDecodeError>,
) -> Result<T, RobotServerDecodeError> {
    object
        .get(field)
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?
        .try_with_str(parse)
        .map_err(|_| RobotServerDecodeError::InvalidText)?
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?
}

fn require_ok(checked: CheckedResponse<'_>) -> Result<(), RobotServerDecodeError> {
    if checked.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(RobotServerDecodeError::UnexpectedStatus)
    }
}
fn require_fields(object: &Map, fields: &[&str]) -> Result<(), RobotServerDecodeError> {
    if object.len() == fields.len() && fields.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotServerDecodeError::InvalidEnvelope)
    }
}
fn require_fields_with_optional(
    object: &Map,
    required: &[&str],
    optional: &str,
) -> Result<(), RobotServerDecodeError> {
    let expected = required
        .len()
        .checked_add(usize::from(object.get(optional).is_some()))
        .ok_or(RobotServerDecodeError::InvalidEnvelope)?;
    if object.len() == expected && required.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotServerDecodeError::InvalidEnvelope)
    }
}
fn map_json_error(error: JsonError) -> RobotServerDecodeError {
    if error == JsonError::Allocation {
        RobotServerDecodeError::Allocation
    } else {
        RobotServerDecodeError::MalformedPayload
    }
}
