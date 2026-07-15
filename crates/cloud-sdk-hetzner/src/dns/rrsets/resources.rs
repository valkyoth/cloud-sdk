//! RRSet CRUD request domains.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::cloud::shared::CloudQueryWriter;
use crate::dns::zones::ZoneReference;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::ApiBaseUrl;

use super::path::{write_collection_path, write_rrset_path};
use super::{
    Records, RrsetLabels, RrsetName, RrsetReference, RrsetRequestError, RrsetTtl, RrsetType,
    RrsetTypeFilter,
};

/// Source-locked RRSet sort fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RrsetSortField {
    /// Sort by ID.
    Id,
    /// Sort by relative name.
    Name,
    /// Sort by creation timestamp.
    Created,
}

/// RRSet CRUD endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RrsetEndpoint<'a> {
    /// `GET /zones/{id_or_name}/rrsets`.
    List(ZoneReference<'a>),
    /// `POST /zones/{id_or_name}/rrsets`.
    Create(ZoneReference<'a>),
    /// `GET /zones/{id_or_name}/rrsets/{rr_name}/{rr_type}`.
    Get(RrsetReference<'a>),
    /// `PUT /zones/{id_or_name}/rrsets/{rr_name}/{rr_type}`.
    Update(RrsetReference<'a>),
    /// `DELETE /zones/{id_or_name}/rrsets/{rr_name}/{rr_type}`.
    Delete(RrsetReference<'a>),
}

impl RrsetEndpoint<'_> {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::List(_) | Self::Get(_) => Method::Get,
            Self::Create(_) => Method::Post,
            Self::Update(_) => Method::Put,
            Self::Delete(_) => Method::Delete,
        }
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::ZoneRrsets
    }

    /// Returns the Cloud v1 base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, RrsetRequestError> {
        match self {
            Self::List(zone) | Self::Create(zone) => write_collection_path(output, zone),
            Self::Get(rrset) | Self::Update(rrset) | Self::Delete(rrset) => {
                write_rrset_path(output, rrset, "")
            }
        }
    }
}

/// RRSet list request with source-locked filters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RrsetListRequest<'a> {
    zone: ZoneReference<'a>,
    name: Option<RrsetName<'a>>,
    types: Option<RrsetTypeFilter<'a>>,
    label_selector: Option<LabelSelector<'a>>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(RrsetSortField, SortDirection)>,
}

impl<'a> RrsetListRequest<'a> {
    /// Creates an unfiltered list request for one Zone.
    #[must_use]
    pub const fn new(zone: ZoneReference<'a>) -> Self {
        Self {
            zone,
            name: None,
            types: None,
            label_selector: None,
            page: None,
            per_page: None,
            sort: None,
        }
    }

    /// Filters by exact relative name.
    #[must_use]
    pub const fn with_name(mut self, name: RrsetName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Filters by one or more unique RR types.
    #[must_use]
    pub const fn with_types(mut self, types: RrsetTypeFilter<'a>) -> Self {
        self.types = Some(types);
        self
    }

    /// Filters by labels.
    #[must_use]
    pub const fn with_label_selector(mut self, selector: LabelSelector<'a>) -> Self {
        self.label_selector = Some(selector);
        self
    }

    /// Sets pagination. RRSet lists permit up to 100 entries per page.
    #[must_use]
    pub const fn with_page(mut self, page: Page, per_page: PerPage) -> Self {
        self.page = Some(page);
        self.per_page = Some(per_page);
        self
    }

    /// Sets one source-locked sort value.
    #[must_use]
    pub const fn with_sort(mut self, field: RrsetSortField, direction: SortDirection) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Returns the list endpoint.
    #[must_use]
    pub const fn endpoint(self) -> RrsetEndpoint<'a> {
        RrsetEndpoint::List(self.zone)
    }

    /// Writes a deterministic query string.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, RrsetRequestError> {
        let mut writer = CloudQueryWriter::new(output);
        if let Some(selector) = self.label_selector {
            writer.pair("label_selector", selector.as_str())?;
        }
        if let Some(name) = self.name {
            writer.pair("name", name.as_str())?;
        }
        if let Some(page) = self.page {
            writer.u64_pair("page", page.get())?;
        }
        if let Some(per_page) = self.per_page {
            writer.u64_pair("per_page", u64::from(per_page.get()))?;
        }
        if let Some((field, direction)) = self.sort {
            writer.pair("sort", sort_value(field, direction))?;
        }
        if let Some(types) = self.types {
            for rr_type in types.entries() {
                writer.pair("type", rr_type.as_api_str())?;
            }
        }
        Ok(writer.len())
    }
}

/// RRSet create request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RrsetCreateRequest<'a> {
    zone: ZoneReference<'a>,
    name: RrsetName<'a>,
    rr_type: RrsetType,
    records: Records<'a>,
    ttl: Option<RrsetTtl>,
    labels: Option<RrsetLabels<'a>>,
}

impl<'a> RrsetCreateRequest<'a> {
    /// Creates a request with all source-required fields.
    ///
    /// Required inputs cannot be omitted:
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::dns::rrsets::RrsetCreateRequest;
    ///
    /// let _ = RrsetCreateRequest::new();
    /// ```
    #[must_use]
    pub const fn new(
        zone: ZoneReference<'a>,
        name: RrsetName<'a>,
        rr_type: RrsetType,
        records: Records<'a>,
    ) -> Self {
        Self {
            zone,
            name,
            rr_type,
            records,
            ttl: None,
            labels: None,
        }
    }

    /// Sets explicit TTL or JSON-null inheritance intent.
    #[must_use]
    pub const fn with_ttl(mut self, ttl: RrsetTtl) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Sets labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: RrsetLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the create endpoint.
    #[must_use]
    pub const fn endpoint(self) -> RrsetEndpoint<'a> {
        RrsetEndpoint::Create(self.zone)
    }

    /// Returns the relative name.
    #[must_use]
    pub const fn name(self) -> RrsetName<'a> {
        self.name
    }

    /// Returns the RR type.
    #[must_use]
    pub const fn rr_type(self) -> RrsetType {
        self.rr_type
    }

    /// Returns records.
    #[must_use]
    pub const fn records(self) -> Records<'a> {
        self.records
    }

    /// Returns TTL intent. `None` means the field is omitted.
    #[must_use]
    pub const fn ttl(self) -> Option<RrsetTtl> {
        self.ttl
    }

    /// Returns labels when supplied.
    #[must_use]
    pub const fn labels(self) -> Option<RrsetLabels<'a>> {
        self.labels
    }
}

/// Partial RRSet update request. The current API only replaces labels here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RrsetUpdateRequest<'a> {
    rrset: RrsetReference<'a>,
    labels: Option<RrsetLabels<'a>>,
}

impl<'a> RrsetUpdateRequest<'a> {
    /// Creates an empty update request.
    #[must_use]
    pub const fn new(rrset: RrsetReference<'a>) -> Self {
        Self {
            rrset,
            labels: None,
        }
    }

    /// Replaces all labels, including with an explicitly empty set.
    #[must_use]
    pub const fn with_labels(mut self, labels: RrsetLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the update endpoint.
    #[must_use]
    pub const fn endpoint(self) -> RrsetEndpoint<'a> {
        RrsetEndpoint::Update(self.rrset)
    }

    /// Returns replacement labels.
    #[must_use]
    pub const fn labels(self) -> Option<RrsetLabels<'a>> {
        self.labels
    }
}

const fn sort_value(field: RrsetSortField, direction: SortDirection) -> &'static str {
    match (field, direction) {
        (RrsetSortField::Id, SortDirection::Asc) => "id:asc",
        (RrsetSortField::Id, SortDirection::Desc) => "id:desc",
        (RrsetSortField::Name, SortDirection::Asc) => "name:asc",
        (RrsetSortField::Name, SortDirection::Desc) => "name:desc",
        (RrsetSortField::Created, SortDirection::Asc) => "created:asc",
        (RrsetSortField::Created, SortDirection::Desc) => "created:desc",
    }
}
