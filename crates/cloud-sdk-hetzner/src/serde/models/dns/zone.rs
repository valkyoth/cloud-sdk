//! Typed DNS zone response model with protected TSIG material.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::net::IpAddr;
use core::str::FromStr;

use crate::dns::zones::MAX_TSIG_KEY_BYTES;
use crate::serde::strict_json::Value;

use super::super::cloud_schema::validate_model;
use super::super::{
    Labels, ResponseModelError, SensitiveText, UtcTimestamp, object, parse_labels, required,
    value_text,
};

const MAX_PROVIDER_ID: u64 = 9_007_199_254_740_991;
const MAX_ZONE_NAME_BYTES: usize = 255;
const MAX_NAMESERVER_TEXT_BYTES: usize = 255;
const MAX_NAMESERVERS: usize = 64;
const MIN_TTL: u64 = 60;
const MAX_TTL: u64 = 2_147_483_647;

/// DNS zone mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoneMode {
    /// Hetzner is authoritative primary.
    Primary,
    /// Hetzner transfers from caller primaries.
    Secondary,
}
/// DNS zone publication status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoneStatus {
    /// Published.
    Ok,
    /// Publication in progress.
    Updating,
    /// Publication failed.
    Error,
}
/// Domain registrar classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoneRegistrar {
    /// Registered at Hetzner.
    Hetzner,
    /// Registered elsewhere.
    Other,
    /// Registrar unknown.
    Unknown,
}
/// Delegation status reported by Hetzner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoneDelegationStatus {
    /// Parent delegation matches.
    Valid,
    /// Parent includes additional Hetzner nameservers.
    PartiallyValid,
    /// Parent does not match.
    Invalid,
    /// Delegated nameservers do not know the zone.
    Lame,
    /// Domain is unregistered.
    Unregistered,
    /// Status not known yet.
    Unknown,
}
/// TSIG algorithm returned by the API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DnsTsigAlgorithm {
    /// Legacy HMAC-MD5 response value; outbound request policy does not admit it.
    HmacMd5,
    /// Legacy HMAC-SHA1 response value; outbound request policy does not admit it.
    HmacSha1,
    /// Hardened HMAC-SHA256 value.
    HmacSha256,
}

/// Zone deletion-protection state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZoneProtection {
    delete: bool,
}
impl ZoneProtection {
    /// Reports whether deletion is protected.
    #[must_use]
    pub const fn delete(self) -> bool {
        self.delete
    }
}

/// Authoritative and delegated nameserver state.
#[derive(Eq, PartialEq)]
pub struct AuthoritativeNameservers {
    assigned: Vec<String>,
    delegated: Vec<String>,
    delegation_last_check: Option<UtcTimestamp>,
    delegation_status: Option<ZoneDelegationStatus>,
}
impl AuthoritativeNameservers {
    /// Returns assigned Hetzner nameservers.
    #[must_use]
    pub fn assigned(&self) -> &[String] {
        &self.assigned
    }
    /// Returns nameservers delegated by the parent zone.
    #[must_use]
    pub fn delegated(&self) -> &[String] {
        &self.delegated
    }
    /// Returns the last delegation check timestamp.
    #[must_use]
    pub const fn delegation_last_check(&self) -> Option<&UtcTimestamp> {
        self.delegation_last_check.as_ref()
    }
    /// Returns delegation status when supplied.
    #[must_use]
    pub const fn delegation_status(&self) -> Option<ZoneDelegationStatus> {
        self.delegation_status
    }
}
impl fmt::Debug for AuthoritativeNameservers {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthoritativeNameservers")
            .field("assigned_count", &self.assigned.len())
            .field("delegated_count", &self.delegated.len())
            .field("delegation_status", &self.delegation_status)
            .field("values", &"[redacted]")
            .finish()
    }
}

/// Primary nameserver used by a secondary zone.
#[derive(Eq, PartialEq)]
pub struct PrimaryNameserver {
    address: String,
    port: Option<u16>,
    tsig_key: Option<SensitiveText>,
    tsig_algorithm: Option<DnsTsigAlgorithm>,
}
impl PrimaryNameserver {
    /// Returns the validated IP address text.
    #[must_use]
    pub fn address(&self) -> &str {
        &self.address
    }
    /// Returns the explicitly supplied port.
    #[must_use]
    pub const fn port(&self) -> Option<u16> {
        self.port
    }
    /// Returns the effective port, applying Hetzner's default of 53.
    #[must_use]
    pub const fn effective_port(&self) -> u16 {
        match self.port {
            Some(port) => port,
            None => 53,
        }
    }
    /// Returns the optional response algorithm.
    #[must_use]
    pub const fn tsig_algorithm(&self) -> Option<DnsTsigAlgorithm> {
        self.tsig_algorithm
    }
    /// Runs a closure with the protected TSIG key when present.
    pub fn try_with_tsig_key<R>(
        &self,
        inspect: impl FnOnce(Option<&str>) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        match &self.tsig_key {
            Some(key) => key.try_with_secret(|value| inspect(Some(value))),
            None => Ok(inspect(None)),
        }
    }
}
impl fmt::Debug for PrimaryNameserver {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PrimaryNameserver")
            .field("address", &"[redacted]")
            .field("port", &self.port)
            .field("tsig_key", &self.tsig_key.as_ref().map(|_| "[redacted]"))
            .field("tsig_algorithm", &self.tsig_algorithm)
            .finish()
    }
}

/// Source-complete DNS zone response.
#[derive(PartialEq)]
pub struct Zone {
    id: u64,
    name: String,
    created: UtcTimestamp,
    mode: ZoneMode,
    status: ZoneStatus,
    ttl: u32,
    record_count: u64,
    labels: Labels,
    protection: ZoneProtection,
    authoritative_nameservers: AuthoritativeNameservers,
    primary_nameservers: Vec<PrimaryNameserver>,
    registrar: ZoneRegistrar,
}
impl Zone {
    /// Returns the provider identifier.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }
    /// Returns the canonical zone name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns creation time.
    #[must_use]
    pub const fn created(&self) -> &UtcTimestamp {
        &self.created
    }
    /// Returns primary or secondary mode.
    #[must_use]
    pub const fn mode(&self) -> ZoneMode {
        self.mode
    }
    /// Returns publication status.
    #[must_use]
    pub const fn status(&self) -> ZoneStatus {
        self.status
    }
    /// Returns the zone default TTL.
    #[must_use]
    pub const fn ttl(&self) -> u32 {
        self.ttl
    }
    /// Returns the provider record count.
    #[must_use]
    pub const fn record_count(&self) -> u64 {
        self.record_count
    }
    /// Returns provider labels.
    #[must_use]
    pub const fn labels(&self) -> &Labels {
        &self.labels
    }
    /// Returns deletion protection.
    #[must_use]
    pub const fn protection(&self) -> ZoneProtection {
        self.protection
    }
    /// Returns authoritative delegation state.
    #[must_use]
    pub const fn authoritative_nameservers(&self) -> &AuthoritativeNameservers {
        &self.authoritative_nameservers
    }
    /// Returns primary nameservers supplied for secondary mode.
    #[must_use]
    pub fn primary_nameservers(&self) -> &[PrimaryNameserver] {
        &self.primary_nameservers
    }
    /// Returns registrar classification.
    #[must_use]
    pub const fn registrar(&self) -> ZoneRegistrar {
        self.registrar
    }
}
impl fmt::Debug for Zone {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Zone")
            .field("mode", &self.mode)
            .field("status", &self.status)
            .field("record_count", &self.record_count)
            .field("primary_nameserver_count", &self.primary_nameservers.len())
            .field("fields", &"[redacted]")
            .finish()
    }
}

pub(super) fn parse_zone(value: &mut Value) -> Result<Zone, ResponseModelError> {
    validate_model("zone", value)?;
    let fields = object(value)?;
    let id = bounded_id(required(fields, "id")?)?;
    let name = value_text(required(fields, "name")?, MAX_ZONE_NAME_BYTES)?;
    let created = timestamp(required(fields, "created")?)?;
    let mode = parse_mode(required(fields, "mode")?)?;
    let status = parse_status(required(fields, "status")?)?;
    let ttl = parse_ttl(required(fields, "ttl")?)?;
    let record_count = required(fields, "record_count")?
        .as_u64()
        .ok_or(ResponseModelError::InvalidNumber)?;
    let labels = parse_labels(required(fields, "labels")?, 256)?;
    let protection = parse_protection(required(fields, "protection")?)?;
    let authoritative_nameservers =
        parse_authoritative(required(fields, "authoritative_nameservers")?)?;
    let registrar = parse_registrar(required(fields, "registrar")?)?;
    let primary_nameservers = parse_primary_nameservers(value)?;
    if mode == ZoneMode::Primary && !primary_nameservers.is_empty() {
        return Err(ResponseModelError::EnvelopeMismatch);
    }
    Ok(Zone {
        id,
        name,
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
    Ok(output)
}

fn parse_primary_nameserver(value: &mut Value) -> Result<PrimaryNameserver, ResponseModelError> {
    let fields = value.as_object_mut().ok_or(ResponseModelError::WrongType)?;
    let address = value_text(required(fields, "address")?, MAX_NAMESERVER_TEXT_BYTES)?;
    IpAddr::from_str(&address).map_err(|_| ResponseModelError::InvalidText)?;
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
    Ok(PrimaryNameserver {
        address,
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
        assigned,
        delegated,
        delegation_last_check,
        delegation_status,
    })
}

fn nameserver_texts(value: &Value) -> Result<Vec<String>, ResponseModelError> {
    let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
    if values.len() > MAX_NAMESERVERS {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        output.push(value_text(value, MAX_NAMESERVER_TEXT_BYTES)?);
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
    ($name:ident, $value:expr, {$($text:literal => $variant:expr),+ $(,)?}) => {{
        $value.try_with_str(|value| match value { $($text => Ok($variant),)+ _ => Err(ResponseModelError::UnknownEnumValue) })
            .map_err(|_| ResponseModelError::InvalidText)?.ok_or(ResponseModelError::WrongType)?
    }};
}
fn parse_mode(value: &Value) -> Result<ZoneMode, ResponseModelError> {
    parse_enum!(mode, value, {"primary"=>ZoneMode::Primary,"secondary"=>ZoneMode::Secondary})
}
fn parse_status(value: &Value) -> Result<ZoneStatus, ResponseModelError> {
    parse_enum!(status, value, {"ok"=>ZoneStatus::Ok,"updating"=>ZoneStatus::Updating,"error"=>ZoneStatus::Error})
}
fn parse_registrar(value: &Value) -> Result<ZoneRegistrar, ResponseModelError> {
    parse_enum!(registrar, value, {"hetzner"=>ZoneRegistrar::Hetzner,"other"=>ZoneRegistrar::Other,"unknown"=>ZoneRegistrar::Unknown})
}
fn parse_delegation_status(value: &Value) -> Result<ZoneDelegationStatus, ResponseModelError> {
    parse_enum!(delegation, value, {"valid"=>ZoneDelegationStatus::Valid,"partially-valid"=>ZoneDelegationStatus::PartiallyValid,"invalid"=>ZoneDelegationStatus::Invalid,"lame"=>ZoneDelegationStatus::Lame,"unregistered"=>ZoneDelegationStatus::Unregistered,"unknown"=>ZoneDelegationStatus::Unknown})
}
fn parse_tsig_algorithm(value: &Value) -> Result<DnsTsigAlgorithm, ResponseModelError> {
    parse_enum!(tsig, value, {"hmac-md5"=>DnsTsigAlgorithm::HmacMd5,"hmac-sha1"=>DnsTsigAlgorithm::HmacSha1,"hmac-sha256"=>DnsTsigAlgorithm::HmacSha256})
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
