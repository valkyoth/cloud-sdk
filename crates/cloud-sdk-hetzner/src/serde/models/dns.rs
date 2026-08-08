//! Dedicated DNS response models and dispatch.

mod rrset;
mod zone;

use alloc::vec::Vec;
use core::fmt;

use crate::serde::strict_json::Value;

use super::ResponseModelError;

pub use rrset::{DnsRecord, DnsRrset, DnsRrsetProtection, DnsRrsetType};
pub use zone::{
    AuthoritativeNameservers, DnsTsigAlgorithm, PrimaryNameserver, Zone, ZoneDelegationStatus,
    ZoneMode, ZoneProtection, ZoneRegistrar, ZoneStatus,
};

/// Dedicated DNS resource family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum DnsResourceKind {
    /// DNS zone.
    Zone,
    /// DNS resource-record set.
    Rrset,
}

/// Source-complete DNS resource returned by the checked decoder.
#[derive(PartialEq)]
#[non_exhaustive]
pub enum DnsResource {
    /// DNS zone.
    Zone(Zone),
    /// DNS resource-record set.
    Rrset(DnsRrset),
}

impl DnsResource {
    /// Returns the exact DNS resource family.
    #[must_use]
    pub const fn kind(&self) -> DnsResourceKind {
        match self {
            Self::Zone(_) => DnsResourceKind::Zone,
            Self::Rrset(_) => DnsResourceKind::Rrset,
        }
    }
}

impl fmt::Debug for DnsResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DnsResource")
            .field("kind", &self.kind())
            .field("fields", &"[redacted]")
            .finish()
    }
}

pub(crate) fn is_dns_resource_root(root: &str) -> bool {
    matches!(root, "zone" | "zones" | "rrset" | "rrsets")
}

pub(crate) fn parse_dns_resource(
    root: &str,
    value: &mut Value,
) -> Result<DnsResource, ResponseModelError> {
    match root {
        "zone" | "zones" => zone::parse_zone(value).map(DnsResource::Zone),
        "rrset" | "rrsets" => rrset::parse_rrset(value).map(DnsResource::Rrset),
        _ => Err(ResponseModelError::EnvelopeMismatch),
    }
}

pub(crate) fn parse_dns_resources(
    root: &str,
    value: &mut Value,
) -> Result<Vec<DnsResource>, ResponseModelError> {
    let values = value.as_array_mut().ok_or(ResponseModelError::WrongType)?;
    if values.len() > 1_024 {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut resources = Vec::new();
    resources
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        resources.push(parse_dns_resource(root, value)?);
    }
    Ok(resources)
}
