//! Strict parser for source-complete DNS zone responses.

use alloc::vec::Vec;
use core::net::IpAddr;
use core::str::FromStr;

use crate::dns::zones::MAX_TSIG_KEY_BYTES;
use crate::serde::strict_json::Value;

use super::super::super::cloud_schema::validate_model;
use super::super::super::wipe_string::{WipeString, WipeStrings};
use super::super::super::{ResponseModelError, object, parse_labels, required, value_text};
use super::*;

pub(in crate::serde::models::dns) fn parse_zone(
    value: &mut Value,
) -> Result<Zone, ResponseModelError> {
    validate_model("zone", value)?;
    let fields = object(value)?;
    let id = bounded_id(required(fields, "id")?)?;
    let name = WipeString::new(value_text(required(fields, "name")?, MAX_ZONE_NAME_BYTES)?);
    let created = timestamp(required(fields, "created")?)?;
    let mode = parse_mode(required(fields, "mode")?)?;
    let status = parse_status(required(fields, "status")?)?;
    let ttl = parse_ttl(required(fields, "ttl")?)?;
    let record_count = required(fields, "record_count")?
        .as_u64()
        .filter(|count| *count <= MAX_ZONE_RECORD_COUNT)
        .ok_or(ResponseModelError::InvalidNumber)?;
    let labels = parse_labels(required(fields, "labels")?, 256)?;
    let protection = parse_protection(required(fields, "protection")?)?;
    let authoritative_nameservers =
        parse_authoritative(required(fields, "authoritative_nameservers")?)?;
    let registrar = parse_registrar(required(fields, "registrar")?)?;
    let primary_nameservers = parse_primary_nameservers(value)?;
    validate_zone_mode(mode, &primary_nameservers)?;
    Ok(Zone {
        id,
        name: name.into_inner(),
        created,
        mode,
        status,
        ttl,
        record_count,
        labels,
        protection,
        authoritative_nameservers,
        primary_nameservers,
        registrar,
    })
}

fn validate_zone_mode(
    mode: ZoneMode,
    primary_nameservers: &[PrimaryNameserver],
) -> Result<(), ResponseModelError> {
    match mode {
        ZoneMode::Primary if !primary_nameservers.is_empty() => {
            Err(ResponseModelError::EnvelopeMismatch)
        }
        ZoneMode::Secondary if primary_nameservers.is_empty() => {
            Err(ResponseModelError::EnvelopeMismatch)
        }
        _ => Ok(()),
    }
}

fn parse_primary_nameservers(
    value: &mut Value,
) -> Result<Vec<PrimaryNameserver>, ResponseModelError> {
    let fields = value.as_object_mut().ok_or(ResponseModelError::WrongType)?;
    let Some(value) = fields.get_mut("primary_nameservers") else {
        return Ok(Vec::new());
    };
    let values = value.as_array_mut().ok_or(ResponseModelError::WrongType)?;
    if values.len() > MAX_NAMESERVERS {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        output.push(parse_primary_nameserver(value)?);
    }
    if !primary_addresses_are_unique(&output)? {
        return Err(ResponseModelError::EnvelopeMismatch);
    }
    Ok(output)
}

fn primary_addresses_are_unique(
    nameservers: &[PrimaryNameserver],
) -> Result<bool, ResponseModelError> {
    let mut addresses = Vec::new();
    addresses
        .try_reserve_exact(nameservers.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for nameserver in nameservers {
        addresses.push(
            IpAddr::from_str(nameserver.address()).map_err(|_| ResponseModelError::InvalidText)?,
        );
    }
    addresses.sort_unstable();
    Ok(!addresses.windows(2).any(|pair| match pair {
        [left, right] => left == right,
        _ => false,
    }))
}

fn parse_primary_nameserver(value: &mut Value) -> Result<PrimaryNameserver, ResponseModelError> {
    let fields = value.as_object_mut().ok_or(ResponseModelError::WrongType)?;
    let address = WipeString::new(value_text(
        required(fields, "address")?,
        MAX_NAMESERVER_TEXT_BYTES,
    )?);
    IpAddr::from_str(address.as_str()).map_err(|_| ResponseModelError::InvalidText)?;
    let port = fields
        .get("port")
        .map(|value| {
            value
                .as_u64()
                .and_then(|value| u16::try_from(value).ok())
                .filter(|value| *value != 0)
                .ok_or(ResponseModelError::InvalidNumber)
        })
        .transpose()?;
    let tsig_algorithm = fields
        .get("tsig_algorithm")
        .map(parse_tsig_algorithm)
        .transpose()?;
    let tsig_key = fields
        .get_mut("tsig_key")
        .map(|value| {
            let key = value
                .take_string()
                .map(SensitiveText::new)
                .ok_or(ResponseModelError::WrongType)?;
            key.validate(MAX_TSIG_KEY_BYTES)?;
            key.try_with_secret(valid_base64)
                .map_err(|_| ResponseModelError::InvalidText)?
                .then_some(key)
                .ok_or(ResponseModelError::InvalidText)
        })
        .transpose()?;
    if tsig_key.is_some() != tsig_algorithm.is_some() {
        return Err(ResponseModelError::EnvelopeMismatch);
    }
    Ok(PrimaryNameserver {
        address: address.into_inner(),
        port,
        tsig_key,
        tsig_algorithm,
    })
}

fn parse_authoritative(value: &Value) -> Result<AuthoritativeNameservers, ResponseModelError> {
    let fields = object(value)?;
    let assigned = nameserver_texts(required(fields, "assigned")?)?;
    let delegated = nameserver_texts(required(fields, "delegated")?)?;
    let last = required(fields, "delegation_last_check")?;
    let delegation_last_check = if last.is_null() {
        None
    } else {
        Some(timestamp(last)?)
    };
    let delegation_status = fields
        .get("delegation_status")
        .map(parse_delegation_status)
        .transpose()?;
    Ok(AuthoritativeNameservers {
        assigned: assigned.into_inner(),
        delegated: delegated.into_inner(),
        delegation_last_check,
        delegation_status,
    })
}

fn nameserver_texts(value: &Value) -> Result<WipeStrings, ResponseModelError> {
    let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
    if values.len() > MAX_NAMESERVERS {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut output = WipeStrings::with_capacity(values.len())?;
    for value in values {
        output.push(WipeString::new(value_text(
            value,
            MAX_NAMESERVER_TEXT_BYTES,
        )?));
    }
    Ok(output)
}

fn bounded_id(value: &Value) -> Result<u64, ResponseModelError> {
    value
        .as_u64()
        .filter(|value| (1..=MAX_PROVIDER_ID).contains(value))
        .ok_or(ResponseModelError::InvalidIdentifier)
}

fn parse_ttl(value: &Value) -> Result<u32, ResponseModelError> {
    value
        .as_u64()
        .filter(|value| (MIN_TTL..=MAX_TTL).contains(value))
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(ResponseModelError::InvalidNumber)
}

fn timestamp(value: &Value) -> Result<UtcTimestamp, ResponseModelError> {
    value
        .try_with_str(UtcTimestamp::try_new)
        .map_err(|_| ResponseModelError::InvalidText)?
        .ok_or(ResponseModelError::WrongType)?
}

fn parse_protection(value: &Value) -> Result<ZoneProtection, ResponseModelError> {
    let delete = required(object(value)?, "delete")?
        .as_bool()
        .ok_or(ResponseModelError::WrongType)?;
    Ok(ZoneProtection { delete })
}

macro_rules! parse_enum {
    ($value:expr, {$($text:literal => $variant:expr),+ $(,)?}) => {{
        $value.try_with_str(|value| match value {
            $($text => Ok($variant),)+
            _ => Err(ResponseModelError::UnknownEnumValue),
        })
        .map_err(|_| ResponseModelError::InvalidText)?
        .ok_or(ResponseModelError::WrongType)?
    }};
}

fn parse_mode(value: &Value) -> Result<ZoneMode, ResponseModelError> {
    parse_enum!(value, {"primary"=>ZoneMode::Primary,"secondary"=>ZoneMode::Secondary})
}

fn parse_status(value: &Value) -> Result<ZoneStatus, ResponseModelError> {
    parse_enum!(value, {"ok"=>ZoneStatus::Ok,"updating"=>ZoneStatus::Updating,"error"=>ZoneStatus::Error})
}

fn parse_registrar(value: &Value) -> Result<ZoneRegistrar, ResponseModelError> {
    parse_enum!(value, {"hetzner"=>ZoneRegistrar::Hetzner,"other"=>ZoneRegistrar::Other,"unknown"=>ZoneRegistrar::Unknown})
}

fn parse_delegation_status(value: &Value) -> Result<ZoneDelegationStatus, ResponseModelError> {
    parse_enum!(value, {"valid"=>ZoneDelegationStatus::Valid,"partially-valid"=>ZoneDelegationStatus::PartiallyValid,"invalid"=>ZoneDelegationStatus::Invalid,"lame"=>ZoneDelegationStatus::Lame,"unregistered"=>ZoneDelegationStatus::Unregistered,"unknown"=>ZoneDelegationStatus::Unknown})
}

fn parse_tsig_algorithm(value: &Value) -> Result<DnsTsigAlgorithm, ResponseModelError> {
    parse_enum!(value, {"hmac-md5"=>DnsTsigAlgorithm::HmacMd5,"hmac-sha1"=>DnsTsigAlgorithm::HmacSha1,"hmac-sha256"=>DnsTsigAlgorithm::HmacSha256})
}

fn valid_base64(value: &str) -> bool {
    let padding = value
        .len()
        .saturating_sub(value.trim_end_matches('=').len());
    let data_len = value.len().saturating_sub(padding);
    let Some(data) = value.as_bytes().get(..data_len) else {
        return false;
    };
    let canonical_tail = match (padding, data.last().and_then(|byte| base64_sextet(*byte))) {
        (0, _) => true,
        (1, Some(value)) => value & 0b0000_0011 == 0,
        (2, Some(value)) => value & 0b0000_1111 == 0,
        _ => false,
    };
    !value.is_empty()
        && value.len().is_multiple_of(4)
        && padding <= 2
        && data
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/'))
        && canonical_tail
}

fn base64_sextet(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => byte.checked_sub(b'A'),
        b'a'..=b'z' => byte.checked_sub(b'a')?.checked_add(26),
        b'0'..=b'9' => byte.checked_sub(b'0')?.checked_add(52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
