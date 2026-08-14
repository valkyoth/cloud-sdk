use alloc::{string::ToString, vec::Vec};
use core::net::Ipv4Addr;
use core::str::FromStr;

use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::{ResponseDecodeWorkspace, StatusCode};

use super::decode_support::*;
use super::model::*;
use super::prepare::MAX_ROBOT_FIREWALL_TEMPLATE_LIST_RESPONSE_BYTES;
use crate::robot::{
    MAX_ROBOT_FIREWALL_RULES_PER_DIRECTION, RobotFirewallAction, RobotFirewallCidr,
    RobotFirewallIpVersion, RobotFirewallPortRange, RobotFirewallProtocol, RobotFirewallRule,
    RobotFirewallTcpFlags, RobotFirewallTemplateId, RobotFirewallTemplateName, RobotServerNumber,
};
use crate::serde::SensitiveText;
use crate::serde::strict_json::{Map, Value, parse_with_scratch};

/// Failure while decoding a source-locked Robot firewall success response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotFirewallDecodeError {
    /// The checked status was not admitted for the operation.
    UnexpectedStatus,
    /// The body exceeded the independent operation limit.
    ResponseTooLarge,
    /// JSON syntax, UTF-8, nesting, duplicates, or parser bounds were invalid.
    MalformedPayload,
    /// Required, null, or extra response fields violated the exact source shape.
    InvalidEnvelope,
    /// A firewall identity, state, rule, or template field was invalid.
    InvalidValue,
    /// A list or rule direction exceeded its bound or contained a duplicate.
    InvalidCollection,
    /// The response identity did not match the exact request.
    ResponseIdentityMismatch,
    /// A successful mutation contradicted the requested replacement.
    MutationOutcomeMismatch,
    /// Bounded protected result storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotFirewallDecodeError,
    Self::UnexpectedStatus => "Robot firewall response status is unexpected",
    Self::ResponseTooLarge => "Robot firewall response exceeds its operation limit",
    Self::MalformedPayload => "Robot firewall response JSON is malformed",
    Self::InvalidEnvelope => "Robot firewall response envelope is invalid",
    Self::InvalidValue => "Robot firewall response value is invalid",
    Self::InvalidCollection => "Robot firewall response collection is invalid",
    Self::ResponseIdentityMismatch => "Robot firewall response identity does not match the request",
    Self::MutationOutcomeMismatch => "Robot firewall mutation response contradicts the request",
    Self::Allocation => "Robot firewall response allocation failed",
);

pub(crate) fn decode_robot_firewall(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFirewall, RobotFirewallDecodeError> {
    require_item(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let wrapper = object_mut(&mut root)?;
    require_fields(wrapper, &["firewall"])?;
    let object = wrapper
        .get_mut("firewall")
        .and_then(Value::as_object_mut)
        .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?;
    require_fields(
        object,
        &[
            "server_ip",
            "server_number",
            "status",
            "filter_ipv6",
            "whitelist_hos",
            "port",
            "rules",
        ],
    )?;
    let server_ip = take_text(object, "server_ip", |value| {
        let address =
            Ipv4Addr::from_str(value).map_err(|_| RobotFirewallDecodeError::InvalidValue)?;
        if address.to_string() == value {
            Ok(())
        } else {
            Err(RobotFirewallDecodeError::InvalidValue)
        }
    })?;
    let server_number = RobotServerNumber::new(required_u64(object, "server_number")?).map_err(
        |error| match error {
            crate::robot::RobotServerNumberError::Zero => RobotFirewallDecodeError::InvalidValue,
            crate::robot::RobotServerNumberError::Allocation => {
                RobotFirewallDecodeError::Allocation
            }
        },
    )?;
    let status = required_text(object, "status", parse_status)?;
    let filter_ipv6 = required_bool(object, "filter_ipv6")?;
    let whitelist_hos = required_bool(object, "whitelist_hos")?;
    let port = required_text(object, "port", parse_port)?;
    let rules = parse_rules(
        object
            .get_mut("rules")
            .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?,
    )?;
    Ok(RobotFirewall {
        server_ip,
        server_number,
        status,
        filter_ipv6,
        whitelist_hos,
        port,
        rules,
    })
}

pub(crate) fn decode_robot_firewall_template_list(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFirewallTemplateList, RobotFirewallDecodeError> {
    require_status(checked, StatusCode::OK)?;
    require_limit(checked, MAX_ROBOT_FIREWALL_TEMPLATE_LIST_RESPONSE_BYTES)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let values = root
        .take_array()
        .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_FIREWALL_TEMPLATE_LIST_ITEMS {
        return Err(RobotFirewallDecodeError::InvalidCollection);
    }
    let mut summaries = Vec::new();
    summaries
        .try_reserve_exact(values.len())
        .map_err(|_| RobotFirewallDecodeError::Allocation)?;
    for mut value in values {
        let wrapper = object_mut(&mut value)?;
        require_fields(wrapper, &["firewall_template"])?;
        let object = wrapper
            .get_mut("firewall_template")
            .and_then(Value::as_object_mut)
            .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?;
        summaries.push(parse_summary(object)?);
    }
    let mut remaining = summaries.as_slice();
    while let Some((summary, tail)) = remaining.split_first() {
        if tail.iter().any(|candidate| candidate.id == summary.id) {
            return Err(RobotFirewallDecodeError::InvalidCollection);
        }
        remaining = tail;
    }
    Ok(RobotFirewallTemplateList(summaries))
}

pub(crate) fn decode_robot_firewall_template(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotFirewallTemplate, RobotFirewallDecodeError> {
    require_item(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let wrapper = object_mut(&mut root)?;
    require_fields(wrapper, &["firewall_template"])?;
    let object = wrapper
        .get_mut("firewall_template")
        .and_then(Value::as_object_mut)
        .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?;
    require_fields(
        object,
        &[
            "id",
            "name",
            "filter_ipv6",
            "whitelist_hos",
            "is_default",
            "rules",
        ],
    )?;
    let id = template_id(object)?;
    let name = template_name(object)?;
    let filter_ipv6 = required_bool(object, "filter_ipv6")?;
    let whitelist_hos = required_bool(object, "whitelist_hos")?;
    let is_default = required_bool(object, "is_default")?;
    let rules = parse_rules(
        object
            .get_mut("rules")
            .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?,
    )?;
    Ok(RobotFirewallTemplate {
        summary: RobotFirewallTemplateSummary {
            id,
            name,
            filter_ipv6,
            whitelist_hos,
            is_default,
        },
        rules,
    })
}

fn parse_summary(
    object: &mut Map,
) -> Result<RobotFirewallTemplateSummary, RobotFirewallDecodeError> {
    require_fields(
        object,
        &["id", "name", "filter_ipv6", "whitelist_hos", "is_default"],
    )?;
    Ok(RobotFirewallTemplateSummary {
        id: template_id(object)?,
        name: template_name(object)?,
        filter_ipv6: required_bool(object, "filter_ipv6")?,
        whitelist_hos: required_bool(object, "whitelist_hos")?,
        is_default: required_bool(object, "is_default")?,
    })
}

fn parse_rules(value: &mut Value) -> Result<RobotFirewallRuleSet, RobotFirewallDecodeError> {
    let object = object_mut(value)?;
    if object.len() == 0 {
        return Ok(RobotFirewallRuleSet {
            input: Vec::new(),
            output: Vec::new(),
        });
    }
    require_fields(object, &["input", "output"])?;
    let input = parse_direction(
        object
            .get_mut("input")
            .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?,
    )?;
    let output = parse_direction(
        object
            .get_mut("output")
            .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?,
    )?;
    Ok(RobotFirewallRuleSet { input, output })
}

fn parse_direction(
    value: &mut Value,
) -> Result<Vec<RobotFirewallRuleModel>, RobotFirewallDecodeError> {
    let values = value
        .take_array()
        .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_ROBOT_FIREWALL_RULES_PER_DIRECTION {
        return Err(RobotFirewallDecodeError::InvalidCollection);
    }
    let mut rules = Vec::new();
    rules
        .try_reserve_exact(values.len())
        .map_err(|_| RobotFirewallDecodeError::Allocation)?;
    for mut value in values {
        rules.push(parse_rule(object_mut(&mut value)?)?);
    }
    let mut remaining = rules.as_slice();
    while let Some((rule, tail)) = remaining.split_first() {
        if tail.iter().any(|candidate| rules_equal(rule, candidate)) {
            return Err(RobotFirewallDecodeError::InvalidCollection);
        }
        remaining = tail;
    }
    Ok(rules)
}

fn parse_rule(object: &mut Map) -> Result<RobotFirewallRuleModel, RobotFirewallDecodeError> {
    require_fields(
        object,
        &[
            "ip_version",
            "name",
            "dst_ip",
            "src_ip",
            "dst_port",
            "src_port",
            "protocol",
            "tcp_flags",
            "action",
        ],
    )?;
    let ip_version = optional_text_enum(object, "ip_version", parse_ip_version)?;
    let name = take_optional_text(object, "name", |value| {
        RobotFirewallRule::new(RobotFirewallAction::Accept)
            .with_name(value)
            .map(|_| ())
            .map_err(|_| RobotFirewallDecodeError::InvalidValue)
    })?;
    let destination_ip = take_optional_text(object, "dst_ip", validate_cidr)?;
    let source_ip = take_optional_text(object, "src_ip", validate_cidr)?;
    let destination_port = take_optional_text(object, "dst_port", validate_port_range)?;
    let source_port = take_optional_text(object, "src_port", validate_port_range)?;
    let protocol = optional_text_enum(object, "protocol", parse_protocol)?;
    let tcp_flags = take_optional_text(object, "tcp_flags", validate_tcp_flags)?;
    let action = required_text(object, "action", parse_action)?;
    let has_ip = destination_ip.is_some() || source_ip.is_some();
    let has_port = destination_port.is_some() || source_port.is_some();
    if has_ip && ip_version != Some(RobotFirewallIpVersion::Ipv4)
        || ip_version.is_none() && protocol.is_some()
        || has_port
            && !matches!(
                protocol,
                Some(RobotFirewallProtocol::Tcp | RobotFirewallProtocol::Udp)
            )
        || tcp_flags.is_some() && protocol != Some(RobotFirewallProtocol::Tcp)
    {
        return Err(RobotFirewallDecodeError::InvalidValue);
    }
    Ok(RobotFirewallRuleModel {
        ip_version,
        name,
        destination_ip,
        source_ip,
        destination_port,
        source_port,
        protocol,
        tcp_flags,
        action,
    })
}

fn template_id(object: &Map) -> Result<RobotFirewallTemplateId, RobotFirewallDecodeError> {
    RobotFirewallTemplateId::new(required_u64(object, "id")?)
        .map_err(|_| RobotFirewallDecodeError::InvalidValue)
}

fn template_name(object: &mut Map) -> Result<SensitiveText, RobotFirewallDecodeError> {
    take_text(object, "name", |value| {
        RobotFirewallTemplateName::new(value)
            .map(|_| ())
            .map_err(|_| RobotFirewallDecodeError::InvalidValue)
    })
}

fn take_text(
    object: &mut Map,
    field: &str,
    validate: impl FnOnce(&str) -> Result<(), RobotFirewallDecodeError>,
) -> Result<SensitiveText, RobotFirewallDecodeError> {
    let value = object
        .get_mut(field)
        .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?;
    value
        .try_with_str(validate)
        .map_err(|_| RobotFirewallDecodeError::InvalidValue)?
        .ok_or(RobotFirewallDecodeError::InvalidEnvelope)??;
    value
        .take_string()
        .map(SensitiveText::new)
        .ok_or(RobotFirewallDecodeError::InvalidEnvelope)
}

fn take_optional_text(
    object: &mut Map,
    field: &str,
    validate: impl FnOnce(&str) -> Result<(), RobotFirewallDecodeError>,
) -> Result<Option<SensitiveText>, RobotFirewallDecodeError> {
    let value = object
        .get_mut(field)
        .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        return Ok(None);
    }
    take_text(object, field, validate).map(Some)
}

fn optional_text_enum<T>(
    object: &Map,
    field: &str,
    parse: impl FnOnce(&str) -> Result<T, RobotFirewallDecodeError>,
) -> Result<Option<T>, RobotFirewallDecodeError> {
    let value = object
        .get(field)
        .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        return Ok(None);
    }
    Ok(Some(
        value
            .try_with_str(parse)
            .map_err(|_| RobotFirewallDecodeError::InvalidValue)?
            .ok_or(RobotFirewallDecodeError::InvalidEnvelope)??,
    ))
}

fn required_text<T>(
    object: &Map,
    field: &str,
    parse: impl FnOnce(&str) -> Result<T, RobotFirewallDecodeError>,
) -> Result<T, RobotFirewallDecodeError> {
    object
        .get(field)
        .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?
        .try_with_str(parse)
        .map_err(|_| RobotFirewallDecodeError::InvalidValue)?
        .ok_or(RobotFirewallDecodeError::InvalidEnvelope)?
}

fn required_u64(object: &Map, field: &str) -> Result<u64, RobotFirewallDecodeError> {
    object
        .get(field)
        .and_then(Value::as_u64)
        .ok_or(RobotFirewallDecodeError::InvalidValue)
}

fn required_bool(object: &Map, field: &str) -> Result<bool, RobotFirewallDecodeError> {
    object
        .get(field)
        .and_then(Value::as_bool)
        .ok_or(RobotFirewallDecodeError::InvalidValue)
}

fn parse_status(value: &str) -> Result<RobotFirewallRuntimeStatus, RobotFirewallDecodeError> {
    match value {
        "active" => Ok(RobotFirewallRuntimeStatus::Active),
        "disabled" => Ok(RobotFirewallRuntimeStatus::Disabled),
        "in process" => Ok(RobotFirewallRuntimeStatus::InProcess),
        _ => Err(RobotFirewallDecodeError::InvalidValue),
    }
}

fn parse_port(value: &str) -> Result<RobotFirewallPort, RobotFirewallDecodeError> {
    match value {
        "main" => Ok(RobotFirewallPort::Main),
        "kvm" => Ok(RobotFirewallPort::Kvm),
        _ => Err(RobotFirewallDecodeError::InvalidValue),
    }
}

fn parse_ip_version(value: &str) -> Result<RobotFirewallIpVersion, RobotFirewallDecodeError> {
    match value {
        "ipv4" => Ok(RobotFirewallIpVersion::Ipv4),
        "ipv6" => Ok(RobotFirewallIpVersion::Ipv6),
        _ => Err(RobotFirewallDecodeError::InvalidValue),
    }
}

fn parse_protocol(value: &str) -> Result<RobotFirewallProtocol, RobotFirewallDecodeError> {
    match value {
        "tcp" => Ok(RobotFirewallProtocol::Tcp),
        "udp" => Ok(RobotFirewallProtocol::Udp),
        "gre" => Ok(RobotFirewallProtocol::Gre),
        "icmp" => Ok(RobotFirewallProtocol::Icmp),
        "ipip" => Ok(RobotFirewallProtocol::Ipip),
        "ah" => Ok(RobotFirewallProtocol::Ah),
        "esp" => Ok(RobotFirewallProtocol::Esp),
        _ => Err(RobotFirewallDecodeError::InvalidValue),
    }
}

fn parse_action(value: &str) -> Result<RobotFirewallAction, RobotFirewallDecodeError> {
    match value {
        "accept" => Ok(RobotFirewallAction::Accept),
        "discard" => Ok(RobotFirewallAction::Discard),
        _ => Err(RobotFirewallDecodeError::InvalidValue),
    }
}

fn validate_cidr(value: &str) -> Result<(), RobotFirewallDecodeError> {
    RobotFirewallCidr::new(value)
        .map(|_| ())
        .map_err(|_| RobotFirewallDecodeError::InvalidValue)
}

fn validate_port_range(value: &str) -> Result<(), RobotFirewallDecodeError> {
    RobotFirewallPortRange::new(value)
        .map(|_| ())
        .map_err(|_| RobotFirewallDecodeError::InvalidValue)
}

fn validate_tcp_flags(value: &str) -> Result<(), RobotFirewallDecodeError> {
    RobotFirewallTcpFlags::new(value)
        .map(|_| ())
        .map_err(|_| RobotFirewallDecodeError::InvalidValue)
}
