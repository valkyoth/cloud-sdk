//! Placement group request domains.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::{ApiBaseUrl, EndpointPath};

use super::super::shared::{
    CloudLabels, CloudName, CloudQueryWriter, CloudRequestError, CloudResourceId, static_path,
    write_id_path, write_static_path,
};

/// Placement group endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[EndpointGroup::PlacementGroups];

/// Placement group identifier.
pub type PlacementGroupId = CloudResourceId;
/// Placement group name.
pub type PlacementGroupName<'a> = CloudName<'a>;
/// Placement group labels.
pub type PlacementGroupLabels<'a> = CloudLabels<'a>;
/// Placement group request error.
pub type PlacementGroupRequestError = CloudRequestError;

/// Placement group type admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlacementGroupType {
    /// Spread placement group.
    Spread,
}

impl PlacementGroupType {
    /// Parses an API placement group type string.
    pub fn from_api_str(value: &str) -> Result<Self, PlacementGroupRequestError> {
        match value {
            "spread" => Ok(Self::Spread),
            _ => Err(PlacementGroupRequestError::InvalidType),
        }
    }

    /// Returns the API string.
    #[must_use]
    pub const fn as_api_str(self) -> &'static str {
        match self {
            Self::Spread => "spread",
        }
    }
}

/// Placement group sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlacementGroupSortField {
    /// Sort by ID.
    Id,
    /// Sort by name.
    Name,
    /// Sort by creation timestamp.
    Created,
}

/// Placement group CRUD endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlacementGroupEndpoint {
    /// `GET /placement_groups`.
    List,
    /// `POST /placement_groups`.
    Create,
    /// `GET /placement_groups/{id}`.
    Get(PlacementGroupId),
    /// `PUT /placement_groups/{id}`.
    Update(PlacementGroupId),
    /// `DELETE /placement_groups/{id}`.
    Delete(PlacementGroupId),
}

impl PlacementGroupEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::List | Self::Get(_) => Method::Get,
            Self::Create => Method::Post,
            Self::Update(_) => Method::Put,
            Self::Delete(_) => Method::Delete,
        }
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::PlacementGroups
    }

    /// Returns a static endpoint path when no ID is required.
    pub fn static_path(self) -> Option<Result<EndpointPath<'static>, PlacementGroupRequestError>> {
        match self {
            Self::List | Self::Create => Some(static_path("/placement_groups")),
            Self::Get(_) | Self::Update(_) | Self::Delete(_) => None,
        }
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, PlacementGroupRequestError> {
        match self {
            Self::List | Self::Create => write_static_path(output, "/placement_groups"),
            Self::Get(id) | Self::Update(id) | Self::Delete(id) => {
                write_id_path(output, "/placement_groups/", id, "")
            }
        }
    }
}

/// Placement group list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlacementGroupListRequest<'a> {
    label_selector: Option<LabelSelector<'a>>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(PlacementGroupSortField, SortDirection)>,
}

impl<'a> PlacementGroupListRequest<'a> {
    /// Creates an empty list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            label_selector: None,
            page: None,
            per_page: None,
            sort: None,
        }
    }

    /// Sets label-selector filtering.
    #[must_use]
    pub const fn with_label_selector(mut self, selector: LabelSelector<'a>) -> Self {
        self.label_selector = Some(selector);
        self
    }

    /// Sets the page value.
    #[must_use]
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the per_page value.
    #[must_use]
    pub const fn with_per_page(mut self, per_page: PerPage) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Sets source-locked sorting.
    #[must_use]
    pub const fn with_sort(
        mut self,
        field: PlacementGroupSortField,
        direction: SortDirection,
    ) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, PlacementGroupRequestError> {
        let mut writer = CloudQueryWriter::new(output);
        if let Some(selector) = self.label_selector {
            writer.pair("label_selector", selector.as_str())?;
        }
        if let Some(page) = self.page {
            writer.u64_pair("page", page.get())?;
        }
        if let Some(per_page) = self.per_page {
            writer.u64_pair("per_page", u64::from(per_page.get()))?;
        }
        if let Some((field, direction)) = self.sort {
            writer.pair("sort", placement_group_sort_value(field, direction))?;
        }
        Ok(writer.len())
    }
}

impl Default for PlacementGroupListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Placement group create request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlacementGroupCreateRequest<'a> {
    name: PlacementGroupName<'a>,
    placement_group_type: PlacementGroupType,
    labels: Option<PlacementGroupLabels<'a>>,
}

impl<'a> PlacementGroupCreateRequest<'a> {
    /// Creates a validated create request.
    #[must_use]
    pub const fn new(
        name: PlacementGroupName<'a>,
        placement_group_type: PlacementGroupType,
    ) -> Self {
        Self {
            name,
            placement_group_type,
            labels: None,
        }
    }

    /// Sets labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: PlacementGroupLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> PlacementGroupEndpoint {
        PlacementGroupEndpoint::Create
    }

    /// Returns the placement group type.
    #[must_use]
    pub const fn placement_group_type(self) -> PlacementGroupType {
        self.placement_group_type
    }
}

/// Placement group update request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlacementGroupUpdateRequest<'a> {
    id: PlacementGroupId,
    name: Option<PlacementGroupName<'a>>,
    labels: Option<PlacementGroupLabels<'a>>,
}

impl<'a> PlacementGroupUpdateRequest<'a> {
    /// Creates an update request.
    #[must_use]
    pub const fn new(id: PlacementGroupId) -> Self {
        Self {
            id,
            name: None,
            labels: None,
        }
    }

    /// Sets replacement name.
    #[must_use]
    pub const fn with_name(mut self, name: PlacementGroupName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets replacement labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: PlacementGroupLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> PlacementGroupEndpoint {
        PlacementGroupEndpoint::Update(self.id)
    }
}

const fn placement_group_sort_value(
    field: PlacementGroupSortField,
    direction: SortDirection,
) -> &'static str {
    match (field, direction) {
        (PlacementGroupSortField::Id, SortDirection::Asc) => "id:asc",
        (PlacementGroupSortField::Id, SortDirection::Desc) => "id:desc",
        (PlacementGroupSortField::Name, SortDirection::Asc) => "name:asc",
        (PlacementGroupSortField::Name, SortDirection::Desc) => "name:desc",
        (PlacementGroupSortField::Created, SortDirection::Asc) => "created:asc",
        (PlacementGroupSortField::Created, SortDirection::Desc) => "created:desc",
    }
}

#[cfg(test)]
mod tests;
