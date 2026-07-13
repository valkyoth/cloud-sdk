//! Network CRUD and list request builders.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::cloud::ip::NetworkIpRange;
use crate::cloud::shared::{CloudQueryWriter, write_id_path, write_static_path};
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::ApiBaseUrl;

use super::types::{
    NetworkId, NetworkLabels, NetworkName, NetworkRequestError, NetworkRoute, NetworkSortField,
    NetworkSubnet,
};

/// Network CRUD endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkEndpoint {
    /// `GET /networks`.
    List,
    /// `POST /networks`.
    Create,
    /// `GET /networks/{id}`.
    Get(NetworkId),
    /// `PUT /networks/{id}`.
    Update(NetworkId),
    /// `DELETE /networks/{id}`.
    Delete(NetworkId),
}

impl NetworkEndpoint {
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

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::Networks
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, NetworkRequestError> {
        match self {
            Self::List | Self::Create => write_static_path(output, "/networks"),
            Self::Get(id) | Self::Update(id) | Self::Delete(id) => {
                write_id_path(output, "/networks/", id, "")
            }
        }
    }
}

/// Network list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkListRequest<'a> {
    name: Option<NetworkName<'a>>,
    label_selector: Option<LabelSelector<'a>>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(NetworkSortField, SortDirection)>,
}

impl<'a> NetworkListRequest<'a> {
    /// Creates an empty list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            label_selector: None,
            page: None,
            per_page: None,
            sort: None,
        }
    }

    /// Filters by exact name.
    #[must_use]
    pub const fn with_name(mut self, name: NetworkName<'a>) -> Self {
        self.name = Some(name);
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

    /// Sets source-locked sorting.
    #[must_use]
    pub const fn with_sort(mut self, field: NetworkSortField, direction: SortDirection) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, NetworkRequestError> {
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
            writer.pair("sort", network_sort_value(field, direction))?;
        }
        Ok(writer.len())
    }
}

impl Default for NetworkListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Network create request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkCreateRequest<'a> {
    name: NetworkName<'a>,
    ip_range: NetworkIpRange<'a>,
    labels: Option<NetworkLabels<'a>>,
    subnets: Option<&'a [NetworkSubnet<'a>]>,
    routes: Option<&'a [NetworkRoute<'a>]>,
    expose_routes_to_vswitch: bool,
}

impl<'a> NetworkCreateRequest<'a> {
    /// Creates a request with required name and private IPv4 range.
    pub fn try_new(
        name: Option<NetworkName<'a>>,
        ip_range: Option<NetworkIpRange<'a>>,
    ) -> Result<Self, NetworkRequestError> {
        Ok(Self {
            name: name.ok_or(NetworkRequestError::MissingRequiredField)?,
            ip_range: ip_range.ok_or(NetworkRequestError::MissingRequiredField)?,
            labels: None,
            subnets: None,
            routes: None,
            expose_routes_to_vswitch: false,
        })
    }

    /// Sets labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: NetworkLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Sets initial subnets.
    #[must_use]
    pub const fn with_subnets(mut self, subnets: &'a [NetworkSubnet<'a>]) -> Self {
        self.subnets = Some(subnets);
        self
    }

    /// Sets initial routes.
    #[must_use]
    pub const fn with_routes(mut self, routes: &'a [NetworkRoute<'a>]) -> Self {
        self.routes = Some(routes);
        self
    }

    /// Sets whether routes are exposed to a configured vSwitch.
    #[must_use]
    pub const fn with_expose_routes_to_vswitch(mut self, expose: bool) -> Self {
        self.expose_routes_to_vswitch = expose;
        self
    }

    /// Returns the required range.
    #[must_use]
    pub const fn ip_range(self) -> NetworkIpRange<'a> {
        self.ip_range
    }

    /// Returns the required name.
    #[must_use]
    pub const fn name(self) -> NetworkName<'a> {
        self.name
    }

    /// Returns labels when supplied.
    #[must_use]
    pub const fn labels(self) -> Option<NetworkLabels<'a>> {
        self.labels
    }

    /// Returns initial subnets when supplied.
    #[must_use]
    pub const fn subnets(self) -> Option<&'a [NetworkSubnet<'a>]> {
        self.subnets
    }

    /// Returns initial routes when supplied.
    #[must_use]
    pub const fn routes(self) -> Option<&'a [NetworkRoute<'a>]> {
        self.routes
    }

    /// Returns whether route exposure is enabled.
    #[must_use]
    pub const fn expose_routes_to_vswitch(self) -> bool {
        self.expose_routes_to_vswitch
    }
}

/// Network update request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkUpdateRequest<'a> {
    id: NetworkId,
    name: Option<NetworkName<'a>>,
    labels: Option<NetworkLabels<'a>>,
    expose_routes_to_vswitch: Option<bool>,
}

impl<'a> NetworkUpdateRequest<'a> {
    /// Creates an update request.
    #[must_use]
    pub const fn new(id: NetworkId) -> Self {
        Self {
            id,
            name: None,
            labels: None,
            expose_routes_to_vswitch: None,
        }
    }

    /// Sets the replacement name.
    #[must_use]
    pub const fn with_name(mut self, name: NetworkName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets replacement labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: NetworkLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Sets route exposure explicitly.
    #[must_use]
    pub const fn with_expose_routes_to_vswitch(mut self, expose: bool) -> Self {
        self.expose_routes_to_vswitch = Some(expose);
        self
    }

    /// Returns the update endpoint.
    #[must_use]
    pub const fn endpoint(self) -> NetworkEndpoint {
        NetworkEndpoint::Update(self.id)
    }

    /// Returns the replacement name when supplied.
    #[must_use]
    pub const fn name(self) -> Option<NetworkName<'a>> {
        self.name
    }

    /// Returns replacement labels when supplied.
    #[must_use]
    pub const fn labels(self) -> Option<NetworkLabels<'a>> {
        self.labels
    }

    /// Returns the explicit route-exposure setting when supplied.
    #[must_use]
    pub const fn expose_routes_to_vswitch(self) -> Option<bool> {
        self.expose_routes_to_vswitch
    }
}

const fn network_sort_value(field: NetworkSortField, direction: SortDirection) -> &'static str {
    match (field, direction) {
        (NetworkSortField::Id, SortDirection::Asc) => "id:asc",
        (NetworkSortField::Id, SortDirection::Desc) => "id:desc",
        (NetworkSortField::Name, SortDirection::Asc) => "name:asc",
        (NetworkSortField::Name, SortDirection::Desc) => "name:desc",
        (NetworkSortField::Created, SortDirection::Asc) => "created:asc",
        (NetworkSortField::Created, SortDirection::Desc) => "created:desc",
    }
}
