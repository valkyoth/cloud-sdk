//! RRSet action endpoint and request domains.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::request::ApiBaseUrl;

use super::path::write_rrset_path;
use super::{RecordUpdates, Records, RrsetReference, RrsetRequestError, RrsetTtl};

/// RRSet action endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RrsetActionEndpoint<'a> {
    /// Change mutation protection.
    ChangeProtection(RrsetReference<'a>),
    /// Change or inherit TTL.
    ChangeTtl(RrsetReference<'a>),
    /// Replace all records.
    SetRecords(RrsetReference<'a>),
    /// Add records, creating the RRSet when absent.
    AddRecords(RrsetReference<'a>),
    /// Remove records by value.
    RemoveRecords(RrsetReference<'a>),
    /// Update comments by record value.
    UpdateRecords(RrsetReference<'a>),
}

impl RrsetActionEndpoint<'_> {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        Method::Post
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::ZoneRrsetActions
    }

    /// Returns the Cloud v1 base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes the action endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, RrsetRequestError> {
        let (rrset, suffix) = match self {
            Self::ChangeProtection(rrset) => (rrset, "/actions/change_protection"),
            Self::ChangeTtl(rrset) => (rrset, "/actions/change_ttl"),
            Self::SetRecords(rrset) => (rrset, "/actions/set_records"),
            Self::AddRecords(rrset) => (rrset, "/actions/add_records"),
            Self::RemoveRecords(rrset) => (rrset, "/actions/remove_records"),
            Self::UpdateRecords(rrset) => (rrset, "/actions/update_records"),
        };
        write_rrset_path(output, rrset, suffix)
    }
}

/// Change-protection action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RrsetProtectionRequest<'a> {
    rrset: RrsetReference<'a>,
    change: bool,
}

impl<'a> RrsetProtectionRequest<'a> {
    /// Creates explicit mutation-protection intent.
    #[must_use]
    pub const fn new(rrset: RrsetReference<'a>, change: bool) -> Self {
        Self { rrset, change }
    }

    /// Returns the action endpoint.
    #[must_use]
    pub const fn endpoint(self) -> RrsetActionEndpoint<'a> {
        RrsetActionEndpoint::ChangeProtection(self.rrset)
    }

    /// Returns mutation-protection intent.
    #[must_use]
    pub const fn change(self) -> bool {
        self.change
    }
}

/// Change-TTL action body with mandatory explicit intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RrsetTtlRequest<'a> {
    rrset: RrsetReference<'a>,
    ttl: RrsetTtl,
}

impl<'a> RrsetTtlRequest<'a> {
    /// Creates explicit bounded-TTL or JSON-null inheritance intent.
    #[must_use]
    pub const fn new(rrset: RrsetReference<'a>, ttl: RrsetTtl) -> Self {
        Self { rrset, ttl }
    }

    /// Returns the action endpoint.
    #[must_use]
    pub const fn endpoint(self) -> RrsetActionEndpoint<'a> {
        RrsetActionEndpoint::ChangeTtl(self.rrset)
    }

    /// Returns mandatory TTL intent.
    #[must_use]
    pub const fn ttl(self) -> RrsetTtl {
        self.ttl
    }
}

/// Set-records action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RrsetSetRecordsRequest<'a> {
    rrset: RrsetReference<'a>,
    records: Records<'a>,
}

impl<'a> RrsetSetRecordsRequest<'a> {
    /// Creates an explicit complete replacement.
    #[must_use]
    pub const fn new(rrset: RrsetReference<'a>, records: Records<'a>) -> Self {
        Self { rrset, records }
    }

    /// Returns the action endpoint.
    #[must_use]
    pub const fn endpoint(self) -> RrsetActionEndpoint<'a> {
        RrsetActionEndpoint::SetRecords(self.rrset)
    }

    /// Returns replacement records.
    #[must_use]
    pub const fn records(self) -> Records<'a> {
        self.records
    }
}

/// Add-records action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RrsetAddRecordsRequest<'a> {
    rrset: RrsetReference<'a>,
    records: Records<'a>,
    ttl: Option<RrsetTtl>,
}

impl<'a> RrsetAddRecordsRequest<'a> {
    /// Creates an add request. TTL is initially omitted.
    #[must_use]
    pub const fn new(rrset: RrsetReference<'a>, records: Records<'a>) -> Self {
        Self {
            rrset,
            records,
            ttl: None,
        }
    }

    /// Supplies TTL intent used when the action creates a missing RRSet.
    #[must_use]
    pub const fn with_ttl(mut self, ttl: RrsetTtl) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Returns the action endpoint.
    #[must_use]
    pub const fn endpoint(self) -> RrsetActionEndpoint<'a> {
        RrsetActionEndpoint::AddRecords(self.rrset)
    }

    /// Returns records to add.
    #[must_use]
    pub const fn records(self) -> Records<'a> {
        self.records
    }

    /// Returns TTL intent. `None` means the field is omitted.
    #[must_use]
    pub const fn ttl(self) -> Option<RrsetTtl> {
        self.ttl
    }
}

/// Remove-records action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RrsetRemoveRecordsRequest<'a> {
    rrset: RrsetReference<'a>,
    records: Records<'a>,
}

impl<'a> RrsetRemoveRecordsRequest<'a> {
    /// Creates a removal request identified by record values.
    #[must_use]
    pub const fn new(rrset: RrsetReference<'a>, records: Records<'a>) -> Self {
        Self { rrset, records }
    }

    /// Returns the action endpoint.
    #[must_use]
    pub const fn endpoint(self) -> RrsetActionEndpoint<'a> {
        RrsetActionEndpoint::RemoveRecords(self.rrset)
    }

    /// Returns records to remove by value.
    #[must_use]
    pub const fn records(self) -> Records<'a> {
        self.records
    }
}

/// Update-record-comments action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RrsetUpdateRecordsRequest<'a> {
    rrset: RrsetReference<'a>,
    records: RecordUpdates<'a>,
}

impl<'a> RrsetUpdateRecordsRequest<'a> {
    /// Creates a comment-update request.
    #[must_use]
    pub const fn new(rrset: RrsetReference<'a>, records: RecordUpdates<'a>) -> Self {
        Self { rrset, records }
    }

    /// Returns the action endpoint.
    #[must_use]
    pub const fn endpoint(self) -> RrsetActionEndpoint<'a> {
        RrsetActionEndpoint::UpdateRecords(self.rrset)
    }

    /// Returns record comment updates.
    #[must_use]
    pub const fn records(self) -> RecordUpdates<'a> {
        self.records
    }
}
