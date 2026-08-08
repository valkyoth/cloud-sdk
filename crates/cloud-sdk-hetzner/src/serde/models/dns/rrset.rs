//! Typed DNS RRSet response model.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::dns::rrsets::{MAX_RECORD_COMMENT_BYTES, MAX_RECORD_VALUE_BYTES, RrsetType};
use crate::serde::strict_json::{Map, Value};

use super::super::cloud_schema::validate_model;
use super::super::{
    Labels, ResponseModelError, checked_text, object, parse_labels, required, validate_text,
    value_text,
};

const MAX_RRSET_ID_BYTES: usize = 1_024;
const MAX_RRSET_NAME_BYTES: usize = 255;
const MAX_RRSET_TYPE_BYTES: usize = 32;
const MAX_RRSET_RECORDS: usize = 4_096;
const MAX_PROVIDER_ID: u64 = 9_007_199_254_740_991;
const MIN_TTL: u64 = 60;
const MAX_TTL: u64 = 2_147_483_647;

/// Open RR type returned by Hetzner.
#[derive(Eq, PartialEq)]
pub struct DnsRrsetType {
    raw: String,
    known: Option<RrsetType>,
}

impl DnsRrsetType {
    /// Returns the exact provider value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Returns the source-known request type, or `None` for an additive future type.
    #[must_use]
    pub const fn known(&self) -> Option<RrsetType> {
        self.known
    }
}

impl fmt::Debug for DnsRrsetType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DnsRrsetType")
            .field("known", &self.known)
            .field("raw", &"[redacted]")
            .finish()
    }
}

/// One bounded record in an RRSet response.
#[derive(Eq, PartialEq)]
pub struct DnsRecord {
    value: String,
    comment: Option<String>,
}

impl DnsRecord {
    /// Returns the exact record value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Returns the optional provider comment, preserving an explicit empty comment.
    #[must_use]
    pub fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
}

impl fmt::Debug for DnsRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DnsRecord")
            .field("value", &"[redacted]")
            .field("comment", &self.comment.as_ref().map(|_| "[redacted]"))
            .finish()
    }
}

/// RRSet mutation-protection state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DnsRrsetProtection {
    change: bool,
}

impl DnsRrsetProtection {
    /// Reports whether RRSet changes are protected.
    #[must_use]
    pub const fn change(self) -> bool {
        self.change
    }
}

/// Source-complete DNS RRSet response.
#[derive(PartialEq)]
pub struct DnsRrset {
    id: String,
    name: String,
    record_type: DnsRrsetType,
    ttl: Option<u32>,
    labels: Labels,
    protection: DnsRrsetProtection,
    records: Vec<DnsRecord>,
    zone: u64,
}

impl DnsRrset {
    /// Returns the opaque provider RRSet identifier.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Returns the owner name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the open response record type.
    #[must_use]
    pub const fn record_type(&self) -> &DnsRrsetType {
        &self.record_type
    }
    /// Returns the explicit TTL, or `None` when the zone default applies.
    #[must_use]
    pub const fn ttl(&self) -> Option<u32> {
        self.ttl
    }
    /// Returns provider labels.
    #[must_use]
    pub const fn labels(&self) -> &Labels {
        &self.labels
    }
    /// Returns mutation protection.
    #[must_use]
    pub const fn protection(&self) -> DnsRrsetProtection {
        self.protection
    }
    /// Returns the nonempty, value-unique records.
    #[must_use]
    pub fn records(&self) -> &[DnsRecord] {
        &self.records
    }
    /// Returns the owning zone identifier.
    #[must_use]
    pub const fn zone(&self) -> u64 {
        self.zone
    }
}

impl fmt::Debug for DnsRrset {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DnsRrset")
            .field("record_type", &self.record_type.known())
            .field("record_count", &self.records.len())
            .field("fields", &"[redacted]")
            .finish()
    }
}

pub(super) fn parse_rrset(value: &mut Value) -> Result<DnsRrset, ResponseModelError> {
    validate_model("rrset", value)?;
    let fields = object(value)?;
    let id = value_text(required(fields, "id")?, MAX_RRSET_ID_BYTES)?;
    let name = value_text(required(fields, "name")?, MAX_RRSET_NAME_BYTES)?;
    let record_type = parse_type(required(fields, "type")?)?;
    let ttl = parse_ttl(required(fields, "ttl")?)?;
    let labels = parse_labels(required(fields, "labels")?, 256)?;
    let protection = parse_protection(required(fields, "protection")?)?;
    let records = parse_records(required(fields, "records")?)?;
    let zone = required(fields, "zone")?
        .as_u64()
        .filter(|value| (1..=MAX_PROVIDER_ID).contains(value))
        .ok_or(ResponseModelError::InvalidIdentifier)?;
    Ok(DnsRrset {
        id,
        name,
        record_type,
        ttl,
        labels,
        protection,
        records,
        zone,
    })
}

fn parse_type(value: &Value) -> Result<DnsRrsetType, ResponseModelError> {
    let raw = value_text(value, MAX_RRSET_TYPE_BYTES)?;
    if !raw
        .bytes()
        .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    {
        return Err(ResponseModelError::InvalidText);
    }
    let known = known_type(&raw);
    Ok(DnsRrsetType { raw, known })
}

fn known_type(value: &str) -> Option<RrsetType> {
    [
        RrsetType::A,
        RrsetType::Aaaa,
        RrsetType::Caa,
        RrsetType::Cname,
        RrsetType::Ds,
        RrsetType::Hinfo,
        RrsetType::Https,
        RrsetType::Mx,
        RrsetType::Ns,
        RrsetType::Ptr,
        RrsetType::Rp,
        RrsetType::Soa,
        RrsetType::Srv,
        RrsetType::Svcb,
        RrsetType::Tlsa,
        RrsetType::Txt,
    ]
    .into_iter()
    .find(|candidate| candidate.as_api_str() == value)
}

fn parse_ttl(value: &Value) -> Result<Option<u32>, ResponseModelError> {
    if value.is_null() {
        return Ok(None);
    }
    let ttl = value
        .as_u64()
        .filter(|value| (MIN_TTL..=MAX_TTL).contains(value))
        .ok_or(ResponseModelError::InvalidNumber)?;
    u32::try_from(ttl)
        .map(Some)
        .map_err(|_| ResponseModelError::InvalidNumber)
}

fn parse_protection(value: &Value) -> Result<DnsRrsetProtection, ResponseModelError> {
    let fields = object(value)?;
    let change = required(fields, "change")?
        .as_bool()
        .ok_or(ResponseModelError::WrongType)?;
    Ok(DnsRrsetProtection { change })
}

fn parse_records(value: &Value) -> Result<Vec<DnsRecord>, ResponseModelError> {
    let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
    if values.is_empty() || values.len() > MAX_RRSET_RECORDS {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut records = Vec::new();
    records
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        let record = parse_record(object(value)?)?;
        if records
            .iter()
            .any(|previous: &DnsRecord| previous.value == record.value)
        {
            return Err(ResponseModelError::InvalidText);
        }
        records.push(record);
    }
    Ok(records)
}

fn parse_record(fields: &Map) -> Result<DnsRecord, ResponseModelError> {
    let value = value_text(required(fields, "value")?, MAX_RECORD_VALUE_BYTES)?;
    let comment = fields.get("comment").map(copy_optional_text).transpose()?;
    Ok(DnsRecord { value, comment })
}

fn copy_optional_text(value: &Value) -> Result<String, ResponseModelError> {
    value
        .try_with_str(|text| {
            if text.is_empty() {
                return Ok(String::new());
            }
            validate_text(text, MAX_RECORD_COMMENT_BYTES)?;
            checked_text(text, MAX_RECORD_COMMENT_BYTES)
        })
        .map_err(|_| ResponseModelError::InvalidText)?
        .ok_or(ResponseModelError::WrongType)?
}
