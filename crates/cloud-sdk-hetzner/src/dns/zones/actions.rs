//! DNS Zone action endpoints and request bodies.

use cloud_sdk::{Method, buffer};

use crate::EndpointGroup;
use crate::actions::ActionStatus;
use crate::cloud::shared::{CloudQueryWriter, CloudRequestError, write_static_path};
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::{ApiBaseUrl, EndpointPath};

use super::{
    PrimaryNameservers, ZoneActionId, ZoneFile, ZoneReference, ZoneRequestError, ZoneTtl,
    write_zone_path,
};

/// Source-locked Zone action sort fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoneActionSortField {
    /// Sort by action ID.
    Id,
    /// Sort by command.
    Command,
    /// Sort by status.
    Status,
    /// Sort by start timestamp.
    Started,
    /// Sort by completion timestamp.
    Finished,
}

/// Zone action endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoneActionEndpoint<'a> {
    /// `GET /zones/actions`.
    ListAll,
    /// `GET /zones/actions/{id}`.
    Get(ZoneActionId),
    /// `GET /zones/{id_or_name}/actions`.
    ListForZone(ZoneReference<'a>),
    /// Change primary nameservers.
    ChangePrimaryNameservers(ZoneReference<'a>),
    /// Change deletion protection.
    ChangeProtection(ZoneReference<'a>),
    /// Change the Zone default TTL.
    ChangeTtl(ZoneReference<'a>),
    /// Import a complete Zone file.
    ImportZoneFile(ZoneReference<'a>),
}

impl ZoneActionEndpoint<'_> {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::ListAll | Self::Get(_) | Self::ListForZone(_) => Method::Get,
            Self::ChangePrimaryNameservers(_)
            | Self::ChangeProtection(_)
            | Self::ChangeTtl(_)
            | Self::ImportZoneFile(_) => Method::Post,
        }
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::ZoneActions
    }

    /// Returns the Cloud v1 base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, ZoneRequestError> {
        let len = match self {
            Self::ListAll => write_static_path(output, "/zones/actions")?,
            Self::Get(id) => write_action_id_path(output, id)?,
            Self::ListForZone(zone) => write_zone_path(output, zone, "/actions")?,
            Self::ChangePrimaryNameservers(zone) => {
                action_path(output, zone, "change_primary_nameservers")?
            }
            Self::ChangeProtection(zone) => action_path(output, zone, "change_protection")?,
            Self::ChangeTtl(zone) => action_path(output, zone, "change_ttl")?,
            Self::ImportZoneFile(zone) => action_path(output, zone, "import_zonefile")?,
        };
        Ok(len)
    }
}

/// Zone action list filters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZoneActionListRequest {
    id: Option<ZoneActionId>,
    status: Option<ActionStatus>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(ZoneActionSortField, SortDirection)>,
}

impl ZoneActionListRequest {
    /// Creates an empty action list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            id: None,
            status: None,
            page: None,
            per_page: None,
            sort: None,
        }
    }

    /// Filters the global list by one action ID.
    #[must_use]
    pub const fn with_id(mut self, id: ZoneActionId) -> Self {
        self.id = Some(id);
        self
    }

    /// Filters by one action status.
    #[must_use]
    pub const fn with_status(mut self, status: ActionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets pagination.
    #[must_use]
    pub const fn with_page(mut self, page: Page, per_page: PerPage) -> Self {
        self.page = Some(page);
        self.per_page = Some(per_page);
        self
    }

    /// Sets one source-locked sort value.
    #[must_use]
    pub const fn with_sort(mut self, field: ZoneActionSortField, direction: SortDirection) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes deterministic query parameters for a list endpoint.
    pub fn write_query(
        self,
        endpoint: ZoneActionEndpoint<'_>,
        output: &mut [u8],
    ) -> Result<usize, ZoneRequestError> {
        if !matches!(
            endpoint,
            ZoneActionEndpoint::ListAll | ZoneActionEndpoint::ListForZone(_)
        ) {
            return Err(ZoneRequestError::InvalidActionFilter);
        }
        if self.id.is_some() && !matches!(endpoint, ZoneActionEndpoint::ListAll) {
            return Err(ZoneRequestError::InvalidActionFilter);
        }
        let mut writer = CloudQueryWriter::new(output);
        if let Some(id) = self.id {
            writer.u64_pair("id", id.get())?;
        }
        if let Some(page) = self.page {
            writer.u64_pair("page", u64::from(page.get()))?;
        }
        if let Some(per_page) = self.per_page {
            writer.u64_pair("per_page", u64::from(per_page.get()))?;
        }
        if let Some((field, direction)) = self.sort {
            writer.pair("sort", action_sort_value(field, direction))?;
        }
        if let Some(status) = self.status {
            writer.pair("status", status.as_api_str())?;
        }
        Ok(writer.len())
    }
}

impl Default for ZoneActionListRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Change-primary-nameservers action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZonePrimaryNameserversRequest<'a> {
    zone: ZoneReference<'a>,
    nameservers: PrimaryNameservers<'a>,
}

impl<'a> ZonePrimaryNameserversRequest<'a> {
    /// Creates an explicit replacement list.
    #[must_use]
    pub const fn new(zone: ZoneReference<'a>, nameservers: PrimaryNameservers<'a>) -> Self {
        Self { zone, nameservers }
    }
    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> ZoneActionEndpoint<'a> {
        ZoneActionEndpoint::ChangePrimaryNameservers(self.zone)
    }
    /// Returns replacement nameservers.
    #[must_use]
    pub const fn nameservers(self) -> PrimaryNameservers<'a> {
        self.nameservers
    }
}

/// Change-protection action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZoneProtectionRequest<'a> {
    zone: ZoneReference<'a>,
    delete: bool,
}

impl<'a> ZoneProtectionRequest<'a> {
    /// Creates explicit deletion-protection intent.
    #[must_use]
    pub const fn new(zone: ZoneReference<'a>, delete: bool) -> Self {
        Self { zone, delete }
    }
    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> ZoneActionEndpoint<'a> {
        ZoneActionEndpoint::ChangeProtection(self.zone)
    }
    /// Returns deletion-protection intent.
    #[must_use]
    pub const fn delete(self) -> bool {
        self.delete
    }
}

/// Change-TTL action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZoneTtlRequest<'a> {
    zone: ZoneReference<'a>,
    ttl: ZoneTtl,
}

impl<'a> ZoneTtlRequest<'a> {
    /// Creates explicit TTL intent. Omission is not representable.
    #[must_use]
    pub const fn new(zone: ZoneReference<'a>, ttl: ZoneTtl) -> Self {
        Self { zone, ttl }
    }
    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> ZoneActionEndpoint<'a> {
        ZoneActionEndpoint::ChangeTtl(self.zone)
    }
    /// Returns the TTL.
    #[must_use]
    pub const fn ttl(self) -> ZoneTtl {
        self.ttl
    }
}

/// Import-zonefile action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZoneFileImportRequest<'a> {
    zone: ZoneReference<'a>,
    zonefile: ZoneFile<'a>,
}

impl<'a> ZoneFileImportRequest<'a> {
    /// Creates a bounded import request.
    #[must_use]
    pub const fn new(zone: ZoneReference<'a>, zonefile: ZoneFile<'a>) -> Self {
        Self { zone, zonefile }
    }
    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> ZoneActionEndpoint<'a> {
        ZoneActionEndpoint::ImportZoneFile(self.zone)
    }
    /// Returns the redacted zone-file marker.
    #[must_use]
    pub const fn zonefile(self) -> ZoneFile<'a> {
        self.zonefile
    }
}

fn action_path(
    output: &mut [u8],
    zone: ZoneReference<'_>,
    action: &str,
) -> Result<usize, ZoneRequestError> {
    let mut suffix = [0_u8; 64];
    let mut len = 0;
    buffer::write_str(
        &mut suffix,
        &mut len,
        "/actions/",
        CloudRequestError::PathBufferTooSmall,
    )?;
    buffer::write_str(
        &mut suffix,
        &mut len,
        action,
        CloudRequestError::PathBufferTooSmall,
    )?;
    let value = core::str::from_utf8(
        suffix
            .get(..len)
            .ok_or(CloudRequestError::PathBufferTooSmall)?,
    )
    .map_err(|_| CloudRequestError::PathEncodingFailed)?;
    write_zone_path(output, zone, value)
}

fn write_action_id_path(output: &mut [u8], id: ZoneActionId) -> Result<usize, ZoneRequestError> {
    let mut len = 0;
    buffer::write_str(
        output,
        &mut len,
        "/zones/actions/",
        CloudRequestError::PathBufferTooSmall,
    )?;
    buffer::write_u64(
        output,
        &mut len,
        id.get(),
        CloudRequestError::PathBufferTooSmall,
    )?;
    let path = core::str::from_utf8(
        output
            .get(..len)
            .ok_or(CloudRequestError::PathBufferTooSmall)?,
    )
    .map_err(|_| CloudRequestError::PathEncodingFailed)?;
    EndpointPath::new(path).map_err(CloudRequestError::InvalidPath)?;
    Ok(len)
}

const fn action_sort_value(field: ZoneActionSortField, direction: SortDirection) -> &'static str {
    match (field, direction) {
        (ZoneActionSortField::Id, SortDirection::Asc) => "id:asc",
        (ZoneActionSortField::Id, SortDirection::Desc) => "id:desc",
        (ZoneActionSortField::Command, SortDirection::Asc) => "command:asc",
        (ZoneActionSortField::Command, SortDirection::Desc) => "command:desc",
        (ZoneActionSortField::Status, SortDirection::Asc) => "status:asc",
        (ZoneActionSortField::Status, SortDirection::Desc) => "status:desc",
        (ZoneActionSortField::Started, SortDirection::Asc) => "started:asc",
        (ZoneActionSortField::Started, SortDirection::Desc) => "started:desc",
        (ZoneActionSortField::Finished, SortDirection::Asc) => "finished:asc",
        (ZoneActionSortField::Finished, SortDirection::Desc) => "finished:desc",
    }
}
