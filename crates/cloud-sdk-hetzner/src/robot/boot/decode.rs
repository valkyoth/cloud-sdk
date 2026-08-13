use alloc::vec::Vec;

use cloud_sdk::operation::CheckedResponse;
use cloud_sdk::transport::{ResponseDecodeWorkspace, StatusCode};

use super::model::*;
use super::value::{MAX_ROBOT_BOOT_KEY_BYTES, MAX_ROBOT_BOOT_VALUE_BYTES};
use crate::robot::server::identity::DecimalServerNumberError;
use crate::robot::{RobotCancellationValueError, RobotIpAddress, RobotServerNumber};
use crate::serde::SensitiveText;
use crate::serde::strict_json::{JsonError, Map, Value, parse_with_scratch};

const MAX_OPTIONS: usize = 256;

/// Failure while decoding a source-locked Robot boot response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotBootDecodeError {
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
    /// A selector, password, or key violated its explicit bound.
    InvalidProtectedValue,
    /// An option or key collection exceeded its bound or contained duplicates.
    InvalidCollection,
    /// A response contradicted the canonical request identity.
    ResponseIdentityMismatch,
    /// An activation/deactivation acknowledgement contradicted requested state.
    MutationOutcomeMismatch,
    /// Bounded protected result storage could not be allocated.
    Allocation,
}

impl_static_error!(RobotBootDecodeError,
    Self::UnexpectedStatus => "Robot boot response status is unexpected",
    Self::MalformedPayload => "Robot boot response JSON is malformed",
    Self::InvalidEnvelope => "Robot boot response envelope is invalid",
    Self::InvalidAddress => "Robot boot response address is invalid",
    Self::InvalidServerNumber => "Robot boot response server number is invalid",
    Self::InvalidProtectedValue => "Robot boot response protected value is invalid",
    Self::InvalidCollection => "Robot boot response collection is invalid",
    Self::ResponseIdentityMismatch => "Robot boot response identity does not match the request",
    Self::MutationOutcomeMismatch => "Robot boot response contradicts the requested state",
    Self::Allocation => "Robot boot response allocation failed",
);

/// Decodes the complete `GET /boot/{server-number}` response.
pub fn decode_robot_boot(
    checked: CheckedResponse<'_>,
    expected: &RobotServerNumber,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotBoot, RobotBootDecodeError> {
    require_ok(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let wrapper = root
        .as_object_mut()
        .ok_or(RobotBootDecodeError::InvalidEnvelope)?;
    require_fields(wrapper, &["boot"])?;
    let boot = wrapper
        .get_mut("boot")
        .and_then(Value::as_object_mut)
        .ok_or(RobotBootDecodeError::InvalidEnvelope)?;
    require_fields(boot, &["rescue", "linux", "vnc", "windows"])?;
    let rescue = take_entry(boot, "rescue", RobotBootFamily::Rescue, expected)?;
    let linux = take_entry(boot, "linux", RobotBootFamily::Linux, expected)?;
    let vnc = take_entry(boot, "vnc", RobotBootFamily::Vnc, expected)?;
    let windows = take_entry(boot, "windows", RobotBootFamily::Windows, expected)?;
    if !same_identity(&rescue, &linux)
        || !same_identity(&rescue, &vnc)
        || !same_identity(&rescue, &windows)
    {
        return Err(RobotBootDecodeError::ResponseIdentityMismatch);
    }
    Ok(RobotBoot {
        rescue,
        linux,
        vnc,
        windows,
    })
}

/// Decodes one family-specific boot response and checks request identity.
pub fn decode_robot_boot_entry(
    checked: CheckedResponse<'_>,
    expected: &RobotServerNumber,
    family: RobotBootFamily,
    workspace: &mut ResponseDecodeWorkspace,
) -> Result<RobotBootEntry, RobotBootDecodeError> {
    require_ok(checked)?;
    let mut root = parse_with_scratch(checked.body(), workspace.decoder_scratch_mut())
        .map_err(map_json_error)?;
    let wrapper = root
        .as_object_mut()
        .ok_or(RobotBootDecodeError::InvalidEnvelope)?;
    let field = family_name(family);
    require_fields(wrapper, &[field])?;
    take_entry(wrapper, field, family, expected)
}

fn take_entry(
    wrapper: &mut Map,
    field: &str,
    family: RobotBootFamily,
    expected: &RobotServerNumber,
) -> Result<RobotBootEntry, RobotBootDecodeError> {
    let object = wrapper
        .get_mut(field)
        .and_then(Value::as_object_mut)
        .ok_or(RobotBootDecodeError::InvalidEnvelope)?;
    validate_fields(object, family)?;
    let server_ipv4 = parse_address(object, "server_ip", true)?;
    let server_ipv6_network = parse_address(object, "server_ipv6_net", false)?;
    let number = parse_number(
        object
            .get("server_number")
            .ok_or(RobotBootDecodeError::InvalidEnvelope)?,
    )?;
    if &number != expected {
        return Err(RobotBootDecodeError::ResponseIdentityMismatch);
    }
    let primary_name = if matches!(family, RobotBootFamily::Rescue | RobotBootFamily::Windows) {
        "os"
    } else {
        "dist"
    };
    let primary = parse_choice(
        object
            .get_mut(primary_name)
            .ok_or(RobotBootDecodeError::InvalidEnvelope)?,
        MAX_ROBOT_BOOT_VALUE_BYTES,
    )?;
    let languages = if matches!(family, RobotBootFamily::Rescue) {
        None
    } else {
        Some(parse_choice(
            object
                .get_mut("lang")
                .ok_or(RobotBootDecodeError::InvalidEnvelope)?,
            MAX_ROBOT_BOOT_VALUE_BYTES,
        )?)
    };
    let active = object
        .get("active")
        .and_then(Value::as_bool)
        .ok_or(RobotBootDecodeError::InvalidEnvelope)?;
    let password = take_optional_secret(object, "password", MAX_ROBOT_BOOT_KEY_BYTES)?;
    let (authorized_keys, host_keys) =
        if matches!(family, RobotBootFamily::Rescue | RobotBootFamily::Linux) {
            (
                take_secrets(object, "authorized_key", MAX_ROBOT_BOOT_KEY_BYTES)?,
                take_secrets(object, "host_key", MAX_ROBOT_BOOT_KEY_BYTES)?,
            )
        } else {
            (Vec::new(), Vec::new())
        };
    if !active && password.is_some() {
        return Err(RobotBootDecodeError::MutationOutcomeMismatch);
    }
    Ok(RobotBootEntry {
        family,
        server_ipv4,
        server_ipv6_network,
        number,
        primary,
        languages,
        active,
        password,
        authorized_keys,
        host_keys,
    })
}

fn validate_fields(object: &mut Map, family: RobotBootFamily) -> Result<(), RobotBootDecodeError> {
    let required: &[&str] = match family {
        RobotBootFamily::Rescue => &[
            "server_ip",
            "server_ipv6_net",
            "server_number",
            "os",
            "active",
            "password",
            "authorized_key",
            "host_key",
        ],
        RobotBootFamily::Linux => &[
            "server_ip",
            "server_ipv6_net",
            "server_number",
            "dist",
            "lang",
            "active",
            "password",
            "authorized_key",
            "host_key",
        ],
        RobotBootFamily::Vnc => &[
            "server_ip",
            "server_ipv6_net",
            "server_number",
            "dist",
            "lang",
            "active",
            "password",
        ],
        RobotBootFamily::Windows => &[
            "server_ip",
            "server_ipv6_net",
            "server_number",
            "os",
            "lang",
            "active",
            "password",
        ],
    };
    if required.iter().any(|field| object.get(field).is_none()) {
        return Err(RobotBootDecodeError::InvalidEnvelope);
    }
    let valid = object
        .try_for_each(|field, _| {
            if required.contains(&field)
                || field == "@deprecated arch"
                || (family == RobotBootFamily::Windows && field == "dist")
            {
                Ok(())
            } else {
                Err(())
            }
        })
        .is_ok();
    if !valid {
        return Err(RobotBootDecodeError::InvalidEnvelope);
    }
    if let Some(arch) = object.get("@deprecated arch") {
        validate_arch(arch)?;
    }
    if family == RobotBootFamily::Windows
        && let Some(dist) = object.get_mut("dist")
        && !dist.is_null()
    {
        drop(parse_choice(dist, MAX_ROBOT_BOOT_VALUE_BYTES)?);
    }
    Ok(())
}

fn validate_arch(value: &Value) -> Result<(), RobotBootDecodeError> {
    if matches!(value.as_u64(), Some(32 | 64)) {
        return Ok(());
    }
    let Some(values) = value.as_array() else {
        return Err(RobotBootDecodeError::InvalidEnvelope);
    };
    if values.len() > 2
        || values
            .iter()
            .any(|value| !matches!(value.as_u64(), Some(32 | 64)))
    {
        return Err(RobotBootDecodeError::InvalidEnvelope);
    }
    Ok(())
}

fn parse_choice(
    value: &mut Value,
    maximum: usize,
) -> Result<RobotBootChoice, RobotBootDecodeError> {
    if let Some(secret) = value.take_string() {
        return Ok(RobotBootChoice::Selected(validate_secret(secret, maximum)?));
    }
    let values = value
        .take_array()
        .ok_or(RobotBootDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_OPTIONS {
        return Err(RobotBootDecodeError::InvalidCollection);
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(values.len())
        .map_err(|_| RobotBootDecodeError::Allocation)?;
    for mut value in values {
        let secret = value
            .take_string()
            .ok_or(RobotBootDecodeError::InvalidEnvelope)?;
        output.push(validate_secret(secret, maximum)?);
    }
    reject_duplicate_secrets(&output)?;
    Ok(RobotBootChoice::Available(output))
}

fn take_optional_secret(
    object: &mut Map,
    field: &str,
    maximum: usize,
) -> Result<Option<RobotBootSecret>, RobotBootDecodeError> {
    let value = object
        .get_mut(field)
        .ok_or(RobotBootDecodeError::InvalidEnvelope)?;
    if value.is_null() {
        return Ok(None);
    }
    value
        .take_string()
        .ok_or(RobotBootDecodeError::InvalidEnvelope)
        .and_then(|secret| validate_secret(secret, maximum).map(Some))
}

fn take_secrets(
    object: &mut Map,
    field: &str,
    maximum: usize,
) -> Result<Vec<RobotBootSecret>, RobotBootDecodeError> {
    let values = object
        .get_mut(field)
        .and_then(Value::take_array)
        .ok_or(RobotBootDecodeError::InvalidEnvelope)?;
    if values.len() > MAX_OPTIONS {
        return Err(RobotBootDecodeError::InvalidCollection);
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(values.len())
        .map_err(|_| RobotBootDecodeError::Allocation)?;
    for mut value in values {
        output.push(validate_secret(
            value
                .take_string()
                .ok_or(RobotBootDecodeError::InvalidEnvelope)?,
            maximum,
        )?);
    }
    reject_duplicate_secrets(&output)?;
    Ok(output)
}

fn validate_secret(
    secret: cloud_sdk_sanitization::SecretString,
    maximum: usize,
) -> Result<RobotBootSecret, RobotBootDecodeError> {
    let text = SensitiveText::new(secret);
    let valid = text
        .try_with_secret(|value| {
            !value.is_empty() && value.len() <= maximum && !value.chars().any(char::is_control)
        })
        .map_err(|_| RobotBootDecodeError::InvalidProtectedValue)?;
    if !valid {
        return Err(RobotBootDecodeError::InvalidProtectedValue);
    }
    Ok(RobotBootSecret(text))
}

fn reject_duplicate_secrets(values: &[RobotBootSecret]) -> Result<(), RobotBootDecodeError> {
    for (index, value) in values.iter().enumerate() {
        let Some(previous) = values.get(..index) else {
            unreachable!("enumerated Robot boot prefix exceeded collection")
        };
        if previous.iter().any(|other| other.0 == value.0) {
            return Err(RobotBootDecodeError::InvalidCollection);
        }
    }
    Ok(())
}

fn parse_number(value: &Value) -> Result<RobotServerNumber, RobotBootDecodeError> {
    value
        .try_with_unsigned_lexical(|digits| {
            RobotServerNumber::from_decimal_bytes(digits.as_bytes())
        })
        .ok_or(RobotBootDecodeError::InvalidServerNumber)?
        .map_err(|error| match error {
            DecimalServerNumberError::Invalid => RobotBootDecodeError::InvalidServerNumber,
            DecimalServerNumberError::Allocation => RobotBootDecodeError::Allocation,
        })
}

fn parse_address(
    object: &Map,
    field: &str,
    ipv4: bool,
) -> Result<RobotIpAddress, RobotBootDecodeError> {
    let address = object
        .get(field)
        .ok_or(RobotBootDecodeError::InvalidEnvelope)?
        .try_with_str(RobotIpAddress::new)
        .map_err(|_| RobotBootDecodeError::InvalidAddress)?
        .ok_or(RobotBootDecodeError::InvalidEnvelope)?
        .map_err(|_: RobotCancellationValueError| RobotBootDecodeError::InvalidAddress)?;
    if address.with_addr(|value| value.is_ipv4()) != ipv4 {
        return Err(RobotBootDecodeError::InvalidAddress);
    }
    Ok(address)
}

fn same_identity(left: &RobotBootEntry, right: &RobotBootEntry) -> bool {
    left.number == right.number
        && left.server_ipv4 == right.server_ipv4
        && left.server_ipv6_network == right.server_ipv6_network
}

const fn family_name(family: RobotBootFamily) -> &'static str {
    match family {
        RobotBootFamily::Rescue => "rescue",
        RobotBootFamily::Linux => "linux",
        RobotBootFamily::Vnc => "vnc",
        RobotBootFamily::Windows => "windows",
    }
}

fn require_ok(checked: CheckedResponse<'_>) -> Result<(), RobotBootDecodeError> {
    if checked.status() == StatusCode::OK {
        Ok(())
    } else {
        Err(RobotBootDecodeError::UnexpectedStatus)
    }
}

fn require_fields(object: &Map, fields: &[&str]) -> Result<(), RobotBootDecodeError> {
    if object.len() == fields.len() && fields.iter().all(|field| object.get(field).is_some()) {
        Ok(())
    } else {
        Err(RobotBootDecodeError::InvalidEnvelope)
    }
}

const fn map_json_error(error: JsonError) -> RobotBootDecodeError {
    match error {
        JsonError::Allocation => RobotBootDecodeError::Allocation,
        _ => RobotBootDecodeError::MalformedPayload,
    }
}
