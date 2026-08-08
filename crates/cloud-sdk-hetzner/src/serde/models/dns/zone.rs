//! Typed DNS zone response model with protected TSIG material.

mod parser;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use cloud_sdk_sanitization::sanitize_string;

use super::super::{Labels, SensitiveText, UtcTimestamp};

pub(super) use parser::parse_zone;

const MAX_PROVIDER_ID: u64 = 9_007_199_254_740_991;
const MAX_ZONE_NAME_BYTES: usize = 255;
const MAX_NAMESERVER_TEXT_BYTES: usize = 255;
const MAX_NAMESERVERS: usize = 64;
const MIN_TTL: u64 = 60;
const MAX_TTL: u64 = 2_147_483_647;
/// Conservative checked-response ceiling for one zone's provider record count.
///
/// Hetzner documents a default limit of 500 records that may be raised on
/// request. This larger local envelope admits substantial custom limits while
/// preventing unchecked provider counts from driving caller allocations.
pub const MAX_ZONE_RECORD_COUNT: u64 = 1_000_000;

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

impl Drop for AuthoritativeNameservers {
    fn drop(&mut self) {
        for value in self.assigned.iter_mut().chain(self.delegated.iter_mut()) {
            sanitize_string(value);
        }
    }
}

/// Primary nameserver used by a secondary zone.
///
/// Ordinary equality is unavailable because this value can contain a TSIG
/// key. Compare reviewed public metadata through the accessors instead.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::serde::PrimaryNameserver;
/// fn compare(left: PrimaryNameserver, right: PrimaryNameserver) -> bool {
///     left == right
/// }
/// ```
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

impl Drop for PrimaryNameserver {
    fn drop(&mut self) {
        sanitize_string(&mut self.address);
    }
}

/// Source-complete DNS zone response.
///
/// Ordinary equality is unavailable because secondary-zone state can contain
/// TSIG keys.
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

impl Drop for Zone {
    fn drop(&mut self) {
        sanitize_string(&mut self.name);
    }
}
