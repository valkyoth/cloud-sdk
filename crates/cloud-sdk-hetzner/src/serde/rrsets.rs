//! Checked RRSet JSON request-body serialization.

use core::fmt;

use ::serde::Serialize;
use ::serde::ser::SerializeMap;

use crate::cloud::shared::CloudLabels;
use crate::dns::rrsets::{
    RecordUpdates, Records, RrsetAddRecordsRequest, RrsetCreateRequest, RrsetProtectionRequest,
    RrsetRemoveRecordsRequest, RrsetSetRecordsRequest, RrsetTtlRequest, RrsetUpdateRecordsRequest,
    RrsetUpdateRequest,
};

/// Maximum JSON body size admitted by the RRSet Serde boundary.
pub const MAX_RRSET_JSON_BODY_BYTES: usize = 1_048_576;

/// Error returned before an RRSet request becomes serializable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RrsetBodyError {
    /// Body-size arithmetic overflowed.
    SizeOverflow,
    /// The conservative JSON upper bound exceeds the SDK policy.
    BodyTooLarge,
}

/// Size-checked RRSet JSON request body.
#[derive(Clone, Copy)]
pub struct RrsetRequestBody<'a> {
    kind: RrsetBodyKind<'a>,
    size_upper_bound: usize,
}

#[derive(Clone, Copy)]
enum RrsetBodyKind<'a> {
    Create(RrsetCreateRequest<'a>),
    Update(RrsetUpdateRequest<'a>),
    Protection(RrsetProtectionRequest<'a>),
    ChangeTtl(RrsetTtlRequest<'a>),
    SetRecords(RrsetSetRecordsRequest<'a>),
    AddRecords(RrsetAddRecordsRequest<'a>),
    RemoveRecords(RrsetRemoveRecordsRequest<'a>),
    UpdateRecords(RrsetUpdateRecordsRequest<'a>),
}

impl fmt::Debug for RrsetRequestBody<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RrsetRequestBody")
            .field("body", &"[redacted]")
            .field("size_upper_bound", &self.size_upper_bound)
            .finish()
    }
}

impl<'a> RrsetRequestBody<'a> {
    /// Checks and wraps an RRSet create body.
    pub fn create(request: RrsetCreateRequest<'a>) -> Result<Self, RrsetBodyError> {
        Self::checked(RrsetBodyKind::Create(request))
    }

    /// Checks and wraps an RRSet labels update body.
    pub fn update(request: RrsetUpdateRequest<'a>) -> Result<Self, RrsetBodyError> {
        Self::checked(RrsetBodyKind::Update(request))
    }

    /// Checks and wraps an RRSet protection body.
    pub fn protection(request: RrsetProtectionRequest<'a>) -> Result<Self, RrsetBodyError> {
        Self::checked(RrsetBodyKind::Protection(request))
    }

    /// Checks and wraps an RRSet TTL body.
    pub fn change_ttl(request: RrsetTtlRequest<'a>) -> Result<Self, RrsetBodyError> {
        Self::checked(RrsetBodyKind::ChangeTtl(request))
    }

    /// Checks and wraps an RRSet complete replacement body.
    pub fn set_records(request: RrsetSetRecordsRequest<'a>) -> Result<Self, RrsetBodyError> {
        Self::checked(RrsetBodyKind::SetRecords(request))
    }

    /// Checks and wraps an RRSet add-records body.
    pub fn add_records(request: RrsetAddRecordsRequest<'a>) -> Result<Self, RrsetBodyError> {
        Self::checked(RrsetBodyKind::AddRecords(request))
    }

    /// Checks and wraps an RRSet remove-records body.
    pub fn remove_records(request: RrsetRemoveRecordsRequest<'a>) -> Result<Self, RrsetBodyError> {
        Self::checked(RrsetBodyKind::RemoveRecords(request))
    }

    /// Checks and wraps an RRSet comment-update body.
    pub fn update_records(request: RrsetUpdateRecordsRequest<'a>) -> Result<Self, RrsetBodyError> {
        Self::checked(RrsetBodyKind::UpdateRecords(request))
    }

    /// Returns the conservative JSON size upper bound checked at construction.
    #[must_use]
    pub const fn size_upper_bound(self) -> usize {
        self.size_upper_bound
    }

    fn checked(kind: RrsetBodyKind<'a>) -> Result<Self, RrsetBodyError> {
        let size_upper_bound = estimate_body(kind)?;
        if size_upper_bound > MAX_RRSET_JSON_BODY_BYTES {
            return Err(RrsetBodyError::BodyTooLarge);
        }
        Ok(Self {
            kind,
            size_upper_bound,
        })
    }
}

impl Serialize for RrsetRequestBody<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        match self.kind {
            RrsetBodyKind::Create(request) => serialize_create(request, serializer),
            RrsetBodyKind::Update(request) => serialize_update(request, serializer),
            RrsetBodyKind::Protection(request) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("change", &request.change())?;
                map.end()
            }
            RrsetBodyKind::ChangeTtl(request) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("ttl", &request.ttl())?;
                map.end()
            }
            RrsetBodyKind::SetRecords(request) => {
                serialize_records(request.records(), None, serializer)
            }
            RrsetBodyKind::AddRecords(request) => {
                serialize_records(request.records(), request.ttl(), serializer)
            }
            RrsetBodyKind::RemoveRecords(request) => {
                serialize_records(request.records(), None, serializer)
            }
            RrsetBodyKind::UpdateRecords(request) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("records", &request.records())?;
                map.end()
            }
        }
    }
}

fn serialize_create<S>(request: RrsetCreateRequest<'_>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ::serde::Serializer,
{
    let fields = 3_usize
        .saturating_add(usize::from(request.ttl().is_some()))
        .saturating_add(usize::from(request.labels().is_some()));
    let mut map = serializer.serialize_map(Some(fields))?;
    map.serialize_entry("name", &request.name())?;
    map.serialize_entry("type", &request.rr_type())?;
    if let Some(ttl) = request.ttl() {
        map.serialize_entry("ttl", &ttl)?;
    }
    map.serialize_entry("records", &request.records())?;
    if let Some(labels) = request.labels() {
        map.serialize_entry("labels", &labels)?;
    }
    map.end()
}

fn serialize_update<S>(request: RrsetUpdateRequest<'_>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: ::serde::Serializer,
{
    let mut map = serializer.serialize_map(Some(usize::from(request.labels().is_some())))?;
    if let Some(labels) = request.labels() {
        map.serialize_entry("labels", &labels)?;
    }
    map.end()
}

fn serialize_records<S>(
    records: Records<'_>,
    ttl: Option<crate::dns::rrsets::RrsetTtl>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: ::serde::Serializer,
{
    let fields = 1_usize.saturating_add(usize::from(ttl.is_some()));
    let mut map = serializer.serialize_map(Some(fields))?;
    map.serialize_entry("records", &records)?;
    if let Some(ttl) = ttl {
        map.serialize_entry("ttl", &ttl)?;
    }
    map.end()
}

fn estimate_body(kind: RrsetBodyKind<'_>) -> Result<usize, RrsetBodyError> {
    let mut size = 256_usize;
    match kind {
        RrsetBodyKind::Create(request) => {
            add(&mut size, json_string_len(request.name().as_str())?)?;
            add(&mut size, records_len(request.records())?)?;
            if let Some(labels) = request.labels() {
                add(&mut size, labels_len(labels)?)?;
            }
        }
        RrsetBodyKind::Update(request) => {
            if let Some(labels) = request.labels() {
                add(&mut size, labels_len(labels)?)?;
            }
        }
        RrsetBodyKind::SetRecords(request) => add(&mut size, records_len(request.records())?)?,
        RrsetBodyKind::AddRecords(request) => add(&mut size, records_len(request.records())?)?,
        RrsetBodyKind::RemoveRecords(request) => add(&mut size, records_len(request.records())?)?,
        RrsetBodyKind::UpdateRecords(request) => {
            add(&mut size, updates_len(request.records())?)?;
        }
        RrsetBodyKind::Protection(_) | RrsetBodyKind::ChangeTtl(_) => {}
    }
    Ok(size)
}

fn records_len(records: Records<'_>) -> Result<usize, RrsetBodyError> {
    let mut size = 2_usize;
    for record in records.entries() {
        add(&mut size, 32)?;
        add(
            &mut size,
            record
                .value()
                .json_size_upper_bound()
                .ok_or(RrsetBodyError::SizeOverflow)?,
        )?;
        if let Some(comment) = record.comment() {
            add(&mut size, 16)?;
            add(
                &mut size,
                comment
                    .json_size_upper_bound()
                    .ok_or(RrsetBodyError::SizeOverflow)?,
            )?;
        }
    }
    Ok(size)
}

fn updates_len(records: RecordUpdates<'_>) -> Result<usize, RrsetBodyError> {
    let mut size = 2_usize;
    for record in records.entries() {
        add(&mut size, 48)?;
        add(
            &mut size,
            record
                .value()
                .json_size_upper_bound()
                .ok_or(RrsetBodyError::SizeOverflow)?,
        )?;
        add(
            &mut size,
            record
                .comment()
                .json_size_upper_bound()
                .ok_or(RrsetBodyError::SizeOverflow)?,
        )?;
    }
    Ok(size)
}

fn labels_len(labels: CloudLabels<'_>) -> Result<usize, RrsetBodyError> {
    let mut size = 2_usize;
    for (key, value) in labels.entries() {
        add(&mut size, 8)?;
        add(&mut size, json_string_len(key.as_str())?)?;
        add(&mut size, json_string_len(value.as_str())?)?;
    }
    Ok(size)
}

fn json_string_len(value: &str) -> Result<usize, RrsetBodyError> {
    let escapes = value
        .bytes()
        .filter(|byte| matches!(byte, b'"' | b'\\'))
        .count();
    value
        .len()
        .checked_add(escapes)
        .and_then(|length| length.checked_add(2))
        .ok_or(RrsetBodyError::SizeOverflow)
}

fn add(target: &mut usize, value: usize) -> Result<(), RrsetBodyError> {
    *target = target
        .checked_add(value)
        .ok_or(RrsetBodyError::SizeOverflow)?;
    Ok(())
}
