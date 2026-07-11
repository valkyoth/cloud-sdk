//! DNS Zone CRUD and zone-file export requests.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::cloud::shared::{CloudQueryWriter, write_static_path};
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::ApiBaseUrl;

use super::{
    PrimaryNameservers, ZoneFile, ZoneLabels, ZoneMode, ZoneName, ZoneReference, ZoneRequestError,
    ZoneTtl, write_zone_path,
};

/// Source-locked Zone sort fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoneSortField {
    /// Sort by ID.
    Id,
    /// Sort by name.
    Name,
    /// Sort by creation timestamp.
    Created,
}

/// Zone CRUD and zone-file export endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoneEndpoint<'a> {
    /// `GET /zones`.
    List,
    /// `POST /zones`.
    Create,
    /// `GET /zones/{id_or_name}`.
    Get(ZoneReference<'a>),
    /// `PUT /zones/{id_or_name}`.
    Update(ZoneReference<'a>),
    /// `DELETE /zones/{id_or_name}`.
    Delete(ZoneReference<'a>),
    /// `GET /zones/{id_or_name}/zonefile`.
    ExportZoneFile(ZoneReference<'a>),
}

impl ZoneEndpoint<'_> {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::List | Self::Get(_) | Self::ExportZoneFile(_) => Method::Get,
            Self::Create => Method::Post,
            Self::Update(_) => Method::Put,
            Self::Delete(_) => Method::Delete,
        }
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::Zones
    }

    /// Returns the Cloud v1 base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, ZoneRequestError> {
        let len = match self {
            Self::List | Self::Create => write_static_path(output, "/zones")?,
            Self::Get(zone) | Self::Update(zone) | Self::Delete(zone) => {
                write_zone_path(output, zone, "")?
            }
            Self::ExportZoneFile(zone) => write_zone_path(output, zone, "/zonefile")?,
        };
        Ok(len)
    }
}

/// Zone list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZoneListRequest<'a> {
    name: Option<ZoneName<'a>>,
    mode: Option<ZoneMode>,
    label_selector: Option<LabelSelector<'a>>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(ZoneSortField, SortDirection)>,
}

impl<'a> ZoneListRequest<'a> {
    /// Creates an empty Zone list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            mode: None,
            label_selector: None,
            page: None,
            per_page: None,
            sort: None,
        }
    }

    /// Filters by exact name.
    #[must_use]
    pub const fn with_name(mut self, name: ZoneName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Filters by mode.
    #[must_use]
    pub const fn with_mode(mut self, mode: ZoneMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Filters by labels.
    #[must_use]
    pub const fn with_label_selector(mut self, selector: LabelSelector<'a>) -> Self {
        self.label_selector = Some(selector);
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
    pub const fn with_sort(mut self, field: ZoneSortField, direction: SortDirection) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes a deterministic query string.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, ZoneRequestError> {
        let mut writer = CloudQueryWriter::new(output);
        if let Some(selector) = self.label_selector {
            writer.pair("label_selector", selector.as_str())?;
        }
        if let Some(mode) = self.mode {
            writer.pair("mode", mode.as_api_str())?;
        }
        if let Some(name) = self.name {
            writer.pair("name", name.as_str())?;
        }
        if let Some(page) = self.page {
            writer.u64_pair("page", u64::from(page.get()))?;
        }
        if let Some(per_page) = self.per_page {
            writer.u64_pair("per_page", u64::from(per_page.get()))?;
        }
        if let Some((field, direction)) = self.sort {
            writer.pair("sort", zone_sort_value(field, direction))?;
        }
        Ok(writer.len())
    }
}

impl Default for ZoneListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Structurally valid Zone creation mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ZoneCreateMode<'a> {
    /// Create a primary Zone hosted by Hetzner.
    Primary,
    /// Create a secondary Zone with caller-owned primary nameservers.
    Secondary(PrimaryNameservers<'a>),
}

impl ZoneCreateMode<'_> {
    /// Returns the API mode.
    #[must_use]
    pub const fn mode(self) -> ZoneMode {
        match self {
            Self::Primary => ZoneMode::Primary,
            Self::Secondary(_) => ZoneMode::Secondary,
        }
    }
}

/// Zone create request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZoneCreateRequest<'a> {
    name: ZoneName<'a>,
    mode: ZoneCreateMode<'a>,
    ttl: Option<ZoneTtl>,
    labels: Option<ZoneLabels<'a>>,
    zonefile: Option<ZoneFile<'a>>,
}

impl<'a> ZoneCreateRequest<'a> {
    /// Creates a request with the two required fields.
    pub fn try_new(
        name: Option<ZoneName<'a>>,
        mode: Option<ZoneCreateMode<'a>>,
    ) -> Result<Self, ZoneRequestError> {
        Ok(Self {
            name: name.ok_or(ZoneRequestError::MissingRequiredField)?,
            mode: mode.ok_or(ZoneRequestError::MissingRequiredField)?,
            ttl: None,
            labels: None,
            zonefile: None,
        })
    }

    /// Sets an explicit default TTL.
    #[must_use]
    pub const fn with_ttl(mut self, ttl: ZoneTtl) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Sets labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: ZoneLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Sets a primary Zone file. Secondary Zones reject this field.
    pub const fn with_zonefile(mut self, zonefile: ZoneFile<'a>) -> Result<Self, ZoneRequestError> {
        if matches!(self.mode, ZoneCreateMode::Secondary(_)) {
            return Err(ZoneRequestError::InvalidModeConfiguration);
        }
        self.zonefile = Some(zonefile);
        Ok(self)
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> ZoneEndpoint<'a> {
        ZoneEndpoint::Create
    }
    /// Returns the name.
    #[must_use]
    pub const fn name(self) -> ZoneName<'a> {
        self.name
    }
    /// Returns mode-specific configuration.
    #[must_use]
    pub const fn mode(self) -> ZoneCreateMode<'a> {
        self.mode
    }
    /// Returns explicit TTL intent.
    #[must_use]
    pub const fn ttl(self) -> Option<ZoneTtl> {
        self.ttl
    }
    /// Returns labels when supplied.
    #[must_use]
    pub const fn labels(self) -> Option<ZoneLabels<'a>> {
        self.labels
    }
    /// Returns a primary Zone file when supplied.
    #[must_use]
    pub const fn zonefile(self) -> Option<ZoneFile<'a>> {
        self.zonefile
    }
}

/// Partial Zone update request. The current API only updates labels here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZoneUpdateRequest<'a> {
    zone: ZoneReference<'a>,
    labels: Option<ZoneLabels<'a>>,
}

impl<'a> ZoneUpdateRequest<'a> {
    /// Creates an empty update request.
    #[must_use]
    pub const fn new(zone: ZoneReference<'a>) -> Self {
        Self { zone, labels: None }
    }

    /// Replaces all labels, including with an explicitly empty set.
    #[must_use]
    pub const fn with_labels(mut self, labels: ZoneLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> ZoneEndpoint<'a> {
        ZoneEndpoint::Update(self.zone)
    }

    /// Returns replacement labels.
    #[must_use]
    pub const fn labels(self) -> Option<ZoneLabels<'a>> {
        self.labels
    }
}

const fn zone_sort_value(field: ZoneSortField, direction: SortDirection) -> &'static str {
    match (field, direction) {
        (ZoneSortField::Id, SortDirection::Asc) => "id:asc",
        (ZoneSortField::Id, SortDirection::Desc) => "id:desc",
        (ZoneSortField::Name, SortDirection::Asc) => "name:asc",
        (ZoneSortField::Name, SortDirection::Desc) => "name:desc",
        (ZoneSortField::Created, SortDirection::Asc) => "created:asc",
        (ZoneSortField::Created, SortDirection::Desc) => "created:desc",
    }
}
