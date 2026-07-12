//! Validated RRSet request values.

mod records;

pub use records::{
    MAX_RECORD_COMMENT_BYTES, MAX_RECORD_VALUE_BYTES, MAX_RECORDS_PER_REQUEST, Record,
    RecordComment, RecordUpdate, RecordUpdates, RecordValue, Records,
};

use crate::cloud::shared::{CloudLabels, CloudRequestError};
use crate::dns::zones::{ZoneReference, ZoneTtl};

/// Maximum RRSet name bytes admitted by the SDK.
pub const MAX_RRSET_NAME_BYTES: usize = 253;
/// Number of source-locked RR types.
pub const RRSET_TYPE_COUNT: usize = 16;

/// RRSet request validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RrsetRequestError {
    /// A shared Cloud request operation failed.
    Cloud(CloudRequestError),
    /// A required request field was omitted.
    MissingRequiredField,
    /// RRSet name validation failed.
    InvalidName,
    /// Record value validation failed.
    InvalidRecordValue,
    /// Record comment validation failed.
    InvalidRecordComment,
    /// At least one record is required.
    EmptyRecords,
    /// The record count exceeds the source-locked request limit.
    TooManyRecords,
    /// Record values must be unique within one request.
    DuplicateRecord,
    /// RR type filters must be nonempty and unique.
    InvalidTypeFilter,
    /// A body output buffer is too small.
    BodyBufferTooSmall,
}

impl From<CloudRequestError> for RrsetRequestError {
    fn from(value: CloudRequestError) -> Self {
        Self::Cloud(value)
    }
}

/// RRSet labels.
pub type RrsetLabels<'a> = CloudLabels<'a>;

/// Relative, lowercase RRSet name or the Zone apex marker `@`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RrsetName<'a>(&'a str);

impl<'a> RrsetName<'a> {
    /// Validates a conservative ASCII or ACE-encoded relative DNS name.
    pub fn new(value: &'a str) -> Result<Self, RrsetRequestError> {
        if value == "@" {
            return Ok(Self(value));
        }
        if value.is_empty()
            || value.len() > MAX_RRSET_NAME_BYTES
            || value.starts_with('.')
            || value.ends_with('.')
        {
            return Err(RrsetRequestError::InvalidName);
        }
        for (index, label) in value.split('.').enumerate() {
            if !valid_name_label(label, index) {
                return Err(RrsetRequestError::InvalidName);
            }
        }
        Ok(Self(value))
    }

    pub(crate) const fn as_str(self) -> &'a str {
        self.0
    }
}

#[cfg(feature = "serde")]
impl ::serde::Serialize for RrsetName<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_str(self.0)
    }
}

/// Source-locked RR type.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RrsetType {
    /// IPv4 address.
    A,
    /// IPv6 address.
    Aaaa,
    /// Certification Authority Authorization.
    Caa,
    /// Canonical name.
    Cname,
    /// DNSSEC delegation signer.
    Ds,
    /// Host information.
    Hinfo,
    /// HTTPS service binding.
    Https,
    /// Mail exchanger.
    Mx,
    /// Authoritative nameserver.
    Ns,
    /// Reverse pointer.
    Ptr,
    /// Responsible person.
    Rp,
    /// Start of authority.
    Soa,
    /// Service locator.
    Srv,
    /// Service binding.
    Svcb,
    /// TLS association.
    Tlsa,
    /// Text record.
    Txt,
}

impl RrsetType {
    /// Returns the source-locked API value.
    #[must_use]
    pub const fn as_api_str(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::Aaaa => "AAAA",
            Self::Caa => "CAA",
            Self::Cname => "CNAME",
            Self::Ds => "DS",
            Self::Hinfo => "HINFO",
            Self::Https => "HTTPS",
            Self::Mx => "MX",
            Self::Ns => "NS",
            Self::Ptr => "PTR",
            Self::Rp => "RP",
            Self::Soa => "SOA",
            Self::Srv => "SRV",
            Self::Svcb => "SVCB",
            Self::Tlsa => "TLSA",
            Self::Txt => "TXT",
        }
    }
}

#[cfg(feature = "serde")]
impl ::serde::Serialize for RrsetType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_str(self.as_api_str())
    }
}

/// Nonempty, unique RR type filter list.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RrsetTypeFilter<'a>(&'a [RrsetType]);

impl<'a> RrsetTypeFilter<'a> {
    /// Validates filter count and uniqueness.
    pub fn new(values: &'a [RrsetType]) -> Result<Self, RrsetRequestError> {
        if values.is_empty() || values.len() > RRSET_TYPE_COUNT {
            return Err(RrsetRequestError::InvalidTypeFilter);
        }
        for (index, value) in values.iter().enumerate() {
            if values
                .get(..index)
                .is_none_or(|previous| previous.contains(value))
            {
                return Err(RrsetRequestError::InvalidTypeFilter);
            }
        }
        Ok(Self(values))
    }

    /// Returns source-ordered filter values.
    #[must_use]
    pub const fn entries(self) -> &'a [RrsetType] {
        self.0
    }
}

/// Explicit RRSet TTL intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RrsetTtl {
    /// Emit JSON `null` to inherit the Zone default TTL.
    InheritZoneDefault,
    /// Emit an explicit bounded TTL.
    Explicit(ZoneTtl),
}

#[cfg(feature = "serde")]
impl ::serde::Serialize for RrsetTtl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        match self {
            Self::InheritZoneDefault => serializer.serialize_none(),
            Self::Explicit(ttl) => ttl.serialize(serializer),
        }
    }
}

/// Complete RRSet path selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RrsetReference<'a> {
    zone: ZoneReference<'a>,
    name: RrsetName<'a>,
    rr_type: RrsetType,
}

impl<'a> RrsetReference<'a> {
    /// Creates a complete RRSet selector.
    #[must_use]
    pub const fn new(zone: ZoneReference<'a>, name: RrsetName<'a>, rr_type: RrsetType) -> Self {
        Self {
            zone,
            name,
            rr_type,
        }
    }

    pub(crate) const fn parts(self) -> (ZoneReference<'a>, RrsetName<'a>, RrsetType) {
        (self.zone, self.name, self.rr_type)
    }
}

fn valid_name_label(label: &str, index: usize) -> bool {
    if label == "*" {
        return index == 0;
    }
    if label.is_empty() || label.len() > 63 {
        return false;
    }
    let boundary = |byte: &u8| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_';
    label.as_bytes().first().is_some_and(boundary)
        && label.as_bytes().last().is_some_and(boundary)
        && label.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}
