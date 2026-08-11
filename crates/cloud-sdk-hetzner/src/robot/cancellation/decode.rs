use alloc::vec::Vec;

use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::{ResponseDecodeWorkspace, StatusCode};

use super::model::*;
use super::{RobotCancellationDate, RobotIpAddress, RobotSubnetAddress};
use crate::robot::server::identity::{DecimalServerNumberError, RobotServerNumber};
use crate::robot::server::protected_parse;
use crate::serde::SensitiveText;
use crate::serde::strict_json::{JsonError, Map, Value, parse_with_scratch};

/// Failure while decoding a source-locked Robot cancellation response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotCancellationDecodeError {
    /// A checked response did not carry `200 OK`.
    UnexpectedStatus,
    /// JSON syntax, UTF-8, duplicate keys, or parser bounds were invalid.
    MalformedPayload,
    /// Required, extra, nullable, or typed fields violated the source lock.
    InvalidEnvelope,
    /// A provider identity or subnet was malformed.
    InvalidIdentifier,
    /// A date was malformed, contradictory, or before the earliest date.
    InvalidDate,
    /// Cancellation state, reason shape, or reservation flags contradicted each other.
    StateConflict,
    /// The response identity did not match the request target.
    ResponseIdentityMismatch,
    /// A mutation acknowledgement did not match the authorized request intent.
    MutationOutcomeMismatch,
    /// A response collection exceeded its explicit bound.
    TooManyItems,
    /// Stable protected response storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotCancellationDecodeError,
    Self::UnexpectedStatus => "Robot cancellation response status is unexpected",
    Self::MalformedPayload => "Robot cancellation response JSON is malformed",
    Self::InvalidEnvelope => "Robot cancellation response envelope is invalid",
    Self::InvalidIdentifier => "Robot cancellation response identity is invalid",
    Self::InvalidDate => "Robot cancellation response date is invalid",
    Self::StateConflict => "Robot cancellation response state is contradictory",
    Self::ResponseIdentityMismatch => "Robot cancellation response identity does not match the request",
    Self::MutationOutcomeMismatch => "Robot cancellation mutation outcome does not match the request",
    Self::TooManyItems => "Robot cancellation response exceeds a collection limit",
    Self::Allocation => "Robot cancellation response allocation failed",
);

/// Decodes and identity-checks one server cancellation response.
pub fn decode_robot_server_cancellation(
    checked: CheckedResponse<'_>,
    expected: &RobotServerNumber,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotServerCancellation, RobotCancellationDecodeError> {
    let mut value = parse_checked(checked, workspace)?;
    let object = envelope(&mut value)?;
    require_fields(
        object,
        &[
            "server_ip",
            "server_ipv6_net",
            "server_number",
            "server_name",
            "earliest_cancellation_date",
            "cancelled",
            "reservation_possible",
            "reserved",
            "cancellation_date",
            "cancellation_reason",
        ],
    )?;
    let server_number = parse_server_number(object, "server_number")?;
    if &server_number != expected {
        return Err(RobotCancellationDecodeError::ResponseIdentityMismatch);
    }
    let server_ip = parse_ip(object, "server_ip")?;
    let server_ipv6_network = parse_ip(object, "server_ipv6_net")?;
    if !server_ipv6_network.with_addr(|address| address.is_ipv6()) {
        return Err(RobotCancellationDecodeError::InvalidIdentifier);
    }
    let server_name = take_text(object, "server_name")?;
    let earliest_date = parse_date(object, "earliest_cancellation_date")?;
    let cancelled = parse_flag(object, "cancelled")?;
    let reservation_possible = parse_flag(object, "reservation_possible")?;
    let reserved = parse_flag(object, "reserved")?;
    let cancellation_date = parse_nullable_date(object.get("cancellation_date"))?;
    validate_date_state(cancelled.get(), &earliest_date, cancellation_date.as_ref())?;
    if reserved.get() && (!reservation_possible.get() || !cancelled.get()) {
        return Err(RobotCancellationDecodeError::StateConflict);
    }
    let reason = parse_reason(object, cancelled.get())?;
    Ok(RobotServerCancellation {
        server_number,
        server_ip,
        server_ipv6_network,
        server_name,
        earliest_date,
        cancelled,
        reservation_possible,
        reserved,
        cancellation_date,
        reason,
    })
}

/// Decodes and identity-checks one IP cancellation response.
pub fn decode_robot_ip_cancellation(
    checked: CheckedResponse<'_>,
    expected: &RobotIpAddress,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotIpCancellation, RobotCancellationDecodeError> {
    let mut value = parse_checked(checked, workspace)?;
    let object = envelope(&mut value)?;
    require_common_fields(
        object,
        &[
            "ip",
            "server_number",
            "earliest_cancellation_date",
            "cancelled",
        ],
    )?;
    let ip = parse_ip(object, "ip")?;
    if &ip != expected {
        return Err(RobotCancellationDecodeError::ResponseIdentityMismatch);
    }
    let server_number = parse_server_number(object, "server_number")?;
    let earliest_date = parse_date(object, "earliest_cancellation_date")?;
    let cancelled = parse_flag(object, "cancelled")?;
    let cancellation_date = parse_variant_date(object)?;
    validate_date_state(cancelled.get(), &earliest_date, cancellation_date.as_ref())?;
    Ok(RobotIpCancellation {
        ip,
        server_number,
        earliest_date,
        cancelled,
        cancellation_date,
    })
}

/// Decodes and identity-checks one subnet cancellation response.
pub fn decode_robot_subnet_cancellation(
    checked: CheckedResponse<'_>,
    expected: &RobotSubnetAddress,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotSubnetCancellation, RobotCancellationDecodeError> {
    let mut value = parse_checked(checked, workspace)?;
    let object = envelope(&mut value)?;
    require_common_fields(
        object,
        &[
            "ip",
            "mask",
            "server_number",
            "earliest_cancellation_date",
            "cancelled",
        ],
    )?;
    let subnet = parse_subnet(object, "ip")?;
    if &subnet != expected {
        return Err(RobotCancellationDecodeError::ResponseIdentityMismatch);
    }
    let prefix = parse_prefix(object)?;
    let valid_network = object
        .get("ip")
        .and_then(|value| {
            value
                .try_with_str(|ip| {
                    object.get("mask").and_then(|value| {
                        value
                            .try_with_str(|mask| protected_parse::subnet(ip, mask).is_ok())
                            .ok()
                            .flatten()
                    })
                })
                .ok()
                .flatten()
        })
        .flatten()
        .unwrap_or(false);
    if !valid_network {
        return Err(RobotCancellationDecodeError::InvalidIdentifier);
    }
    let server_number = parse_server_number(object, "server_number")?;
    let earliest_date = parse_date(object, "earliest_cancellation_date")?;
    let cancelled = parse_flag(object, "cancelled")?;
    let cancellation_date = parse_variant_date(object)?;
    validate_date_state(cancelled.get(), &earliest_date, cancellation_date.as_ref())?;
    Ok(RobotSubnetCancellation {
        subnet,
        prefix: ProtectedPrefix::new(prefix)
            .map_err(|_| RobotCancellationDecodeError::Allocation)?,
        server_number,
        earliest_date,
        cancelled,
        cancellation_date,
    })
}

fn parse_checked(
    checked: CheckedResponse<'_>,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<Value, RobotCancellationDecodeError> {
    if checked.status() != StatusCode::OK {
        return Err(RobotCancellationDecodeError::UnexpectedStatus);
    }
    parse_with_scratch(checked.body(), workspace.decoder_scratch_mut()).map_err(map_json_error)
}

fn envelope(value: &mut Value) -> Result<&mut Map, RobotCancellationDecodeError> {
    let root = value
        .as_object_mut()
        .ok_or(RobotCancellationDecodeError::InvalidEnvelope)?;
    require_fields(root, &["cancellation"])?;
    root.get_mut("cancellation")
        .and_then(Value::as_object_mut)
        .ok_or(RobotCancellationDecodeError::InvalidEnvelope)
}

fn require_fields(object: &Map, fields: &[&str]) -> Result<(), RobotCancellationDecodeError> {
    if object.len() == fields.len() && fields.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotCancellationDecodeError::InvalidEnvelope)
    }
}

fn require_common_fields(
    object: &Map,
    required: &[&str],
) -> Result<(), RobotCancellationDecodeError> {
    let underscore = object.get("cancellation_date").is_some();
    let hyphen = object.get("cancellation-date").is_some();
    if underscore == hyphen
        || object.len() != required.len().saturating_add(1)
        || !required.iter().all(|field| object.get(field).is_some())
    {
        return Err(RobotCancellationDecodeError::InvalidEnvelope);
    }
    Ok(())
}

fn parse_server_number(
    object: &Map,
    field: &str,
) -> Result<RobotServerNumber, RobotCancellationDecodeError> {
    let value = object
        .get(field)
        .ok_or(RobotCancellationDecodeError::InvalidEnvelope)?;
    let parsed = value
        .try_with_unsigned_lexical(|text| RobotServerNumber::from_decimal_bytes(text.as_bytes()))
        .or_else(|| {
            value
                .try_with_str(|text| RobotServerNumber::from_decimal_bytes(text.as_bytes()))
                .ok()
                .flatten()
        })
        .ok_or(RobotCancellationDecodeError::InvalidIdentifier)?;
    parsed.map_err(|error| match error {
        DecimalServerNumberError::Invalid => RobotCancellationDecodeError::InvalidIdentifier,
        DecimalServerNumberError::Allocation => RobotCancellationDecodeError::Allocation,
    })
}

fn parse_ip(object: &Map, field: &str) -> Result<RobotIpAddress, RobotCancellationDecodeError> {
    parse_text(object, field, |text| {
        RobotIpAddress::new(text).map_err(map_value_error)
    })
}
fn parse_subnet(
    object: &Map,
    field: &str,
) -> Result<RobotSubnetAddress, RobotCancellationDecodeError> {
    parse_text(object, field, |text| {
        RobotSubnetAddress::new(text).map_err(map_value_error)
    })
}
fn parse_date(
    object: &Map,
    field: &str,
) -> Result<RobotCancellationDate, RobotCancellationDecodeError> {
    parse_text(object, field, |text| {
        RobotCancellationDate::new(text).map_err(map_date_error)
    })
}

fn parse_text<T>(
    object: &Map,
    field: &str,
    parse: impl FnOnce(&str) -> Result<T, RobotCancellationDecodeError>,
) -> Result<T, RobotCancellationDecodeError> {
    object
        .get(field)
        .ok_or(RobotCancellationDecodeError::InvalidEnvelope)?
        .try_with_str(parse)
        .map_err(|_| RobotCancellationDecodeError::InvalidEnvelope)?
        .ok_or(RobotCancellationDecodeError::InvalidEnvelope)?
}

fn take_text(object: &mut Map, field: &str) -> Result<SensitiveText, RobotCancellationDecodeError> {
    let text = object
        .get_mut(field)
        .and_then(Value::take_string)
        .map(SensitiveText::new)
        .ok_or(RobotCancellationDecodeError::InvalidEnvelope)?;
    text.validate(MAX_ROBOT_CANCELLATION_REASON_BYTES)
        .map_err(|_| RobotCancellationDecodeError::InvalidEnvelope)?;
    Ok(text)
}

fn parse_flag(object: &Map, field: &str) -> Result<ProtectedFlag, RobotCancellationDecodeError> {
    let value = object
        .get(field)
        .filter(|value| value.is_bool())
        .ok_or(RobotCancellationDecodeError::InvalidEnvelope)?;
    ProtectedFlag::from_value(value).map_err(|_| RobotCancellationDecodeError::Allocation)
}

fn parse_nullable_date(
    value: Option<&Value>,
) -> Result<Option<RobotCancellationDate>, RobotCancellationDecodeError> {
    let value = value.ok_or(RobotCancellationDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        return Ok(None);
    }
    value
        .try_with_str(|text| RobotCancellationDate::new(text).map_err(map_date_error))
        .map_err(|_| RobotCancellationDecodeError::InvalidEnvelope)?
        .ok_or(RobotCancellationDecodeError::InvalidEnvelope)?
        .map(Some)
}

fn parse_variant_date(
    object: &Map,
) -> Result<Option<RobotCancellationDate>, RobotCancellationDecodeError> {
    parse_nullable_date(
        object
            .get("cancellation_date")
            .or_else(|| object.get("cancellation-date")),
    )
}

fn parse_reason(
    object: &mut Map,
    cancelled: bool,
) -> Result<RobotServerCancellationReason, RobotCancellationDecodeError> {
    let value = object
        .get_mut("cancellation_reason")
        .ok_or(RobotCancellationDecodeError::InvalidEnvelope)?;
    if cancelled {
        if value.is_null() {
            return Ok(RobotServerCancellationReason::Selected(None));
        }
        return take_sensitive_value(value)
            .map(|value| RobotServerCancellationReason::Selected(Some(value)));
    }
    let values = value
        .take_array()
        .ok_or(RobotCancellationDecodeError::StateConflict)?;
    if values.len() > MAX_ROBOT_CANCELLATION_REASONS {
        return Err(RobotCancellationDecodeError::TooManyItems);
    }
    let mut reasons = Vec::new();
    reasons
        .try_reserve_exact(values.len())
        .map_err(|_| RobotCancellationDecodeError::Allocation)?;
    for mut value in values {
        reasons.push(take_sensitive_value(&mut value)?);
    }
    Ok(RobotServerCancellationReason::Available(reasons))
}

fn take_sensitive_value(value: &mut Value) -> Result<SensitiveText, RobotCancellationDecodeError> {
    let text = value
        .take_string()
        .map(SensitiveText::new)
        .ok_or(RobotCancellationDecodeError::StateConflict)?;
    text.validate(MAX_ROBOT_CANCELLATION_REASON_BYTES)
        .map_err(|_| RobotCancellationDecodeError::StateConflict)?;
    Ok(text)
}

fn parse_prefix(object: &Map) -> Result<u8, RobotCancellationDecodeError> {
    object
        .get("mask")
        .and_then(|value| {
            value
                .try_with_str(|text| {
                    if text.is_empty() || (text.len() > 1 && text.starts_with('0')) {
                        return None;
                    }
                    text.parse::<u8>().ok().filter(|value| *value <= 128)
                })
                .ok()
                .flatten()
        })
        .flatten()
        .ok_or(RobotCancellationDecodeError::InvalidIdentifier)
}

fn validate_date_state(
    cancelled: bool,
    earliest: &RobotCancellationDate,
    date: Option<&RobotCancellationDate>,
) -> Result<(), RobotCancellationDecodeError> {
    if cancelled != date.is_some() {
        return Err(RobotCancellationDecodeError::StateConflict);
    }
    if date.is_some_and(|value| value < earliest) {
        return Err(RobotCancellationDecodeError::InvalidDate);
    }
    Ok(())
}

fn map_json_error(error: JsonError) -> RobotCancellationDecodeError {
    if error == JsonError::Allocation {
        RobotCancellationDecodeError::Allocation
    } else {
        RobotCancellationDecodeError::MalformedPayload
    }
}
fn map_value_error(error: super::RobotCancellationValueError) -> RobotCancellationDecodeError {
    match error {
        super::RobotCancellationValueError::Invalid => {
            RobotCancellationDecodeError::InvalidIdentifier
        }
        super::RobotCancellationValueError::Allocation => RobotCancellationDecodeError::Allocation,
    }
}

fn map_date_error(error: super::RobotCancellationValueError) -> RobotCancellationDecodeError {
    match error {
        super::RobotCancellationValueError::Invalid => RobotCancellationDecodeError::InvalidDate,
        super::RobotCancellationValueError::Allocation => RobotCancellationDecodeError::Allocation,
    }
}

#[cfg(test)]
mod tests {
    use super::{RobotCancellationDecodeError, map_date_error};
    use crate::robot::RobotCancellationValueError;

    #[test]
    fn date_failures_preserve_invalid_and_allocation_classes() {
        assert_eq!(
            map_date_error(RobotCancellationValueError::Invalid),
            RobotCancellationDecodeError::InvalidDate
        );
        assert_eq!(
            map_date_error(RobotCancellationValueError::Allocation),
            RobotCancellationDecodeError::Allocation
        );
    }
}
