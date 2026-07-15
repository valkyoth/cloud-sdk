//! Load Balancer CRUD endpoints and request builders.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::cloud::shared::{CloudQueryWriter, write_id_path, write_static_path};
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::ApiBaseUrl;

use super::{
    LoadBalancerAddTargetRequest, LoadBalancerAlgorithm, LoadBalancerId, LoadBalancerLabels,
    LoadBalancerLocation, LoadBalancerName, LoadBalancerNetworkId, LoadBalancerNetworkZone,
    LoadBalancerRequestError, LoadBalancerService, LoadBalancerType,
};

/// Source-locked Load Balancer list sort fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadBalancerSortField {
    /// Sort by ID.
    Id,
    /// Sort by name.
    Name,
    /// Sort by creation timestamp.
    Created,
}

/// Load Balancer CRUD and metrics endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadBalancerEndpoint {
    /// `GET /load_balancers`.
    List,
    /// `POST /load_balancers`.
    Create,
    /// `GET /load_balancers/{id}`.
    Get(LoadBalancerId),
    /// `PUT /load_balancers/{id}`.
    Update(LoadBalancerId),
    /// `DELETE /load_balancers/{id}`.
    Delete(LoadBalancerId),
    /// `GET /load_balancers/{id}/metrics`.
    Metrics(LoadBalancerId),
}

impl LoadBalancerEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::List | Self::Get(_) | Self::Metrics(_) => Method::Get,
            Self::Create => Method::Post,
            Self::Update(_) => Method::Put,
            Self::Delete(_) => Method::Delete,
        }
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::LoadBalancers
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, LoadBalancerRequestError> {
        let len = match self {
            Self::List | Self::Create => write_static_path(output, "/load_balancers")?,
            Self::Get(id) | Self::Update(id) | Self::Delete(id) => {
                write_id_path(output, "/load_balancers/", id, "")?
            }
            Self::Metrics(id) => write_id_path(output, "/load_balancers/", id, "/metrics")?,
        };
        Ok(len)
    }
}

/// Load Balancer list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerListRequest<'a> {
    name: Option<LoadBalancerName<'a>>,
    label_selector: Option<LabelSelector<'a>>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(LoadBalancerSortField, SortDirection)>,
}

impl<'a> LoadBalancerListRequest<'a> {
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
    pub const fn with_name(mut self, name: LoadBalancerName<'a>) -> Self {
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

    /// Sets sorting.
    #[must_use]
    pub const fn with_sort(
        mut self,
        field: LoadBalancerSortField,
        direction: SortDirection,
    ) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes a deterministic query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, LoadBalancerRequestError> {
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
        Ok(writer.len())
    }
}

impl Default for LoadBalancerListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Optional creation placement. The enum prevents conflicting location fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadBalancerPlacement<'a> {
    /// Place in a concrete location.
    Location(LoadBalancerLocation<'a>),
    /// Let Hetzner choose a location in a network zone.
    NetworkZone(LoadBalancerNetworkZone<'a>),
}

/// Load Balancer create request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerCreateRequest<'a> {
    name: LoadBalancerName<'a>,
    load_balancer_type: LoadBalancerType<'a>,
    algorithm: Option<LoadBalancerAlgorithm>,
    labels: Option<LoadBalancerLabels<'a>>,
    public_interface: Option<bool>,
    network: Option<LoadBalancerNetworkId>,
    placement: Option<LoadBalancerPlacement<'a>>,
    services: Option<&'a [LoadBalancerService<'a>]>,
    targets: Option<&'a [LoadBalancerAddTargetRequest<'a>]>,
}

impl<'a> LoadBalancerCreateRequest<'a> {
    /// Creates a request with its two required fields.
    #[must_use]
    pub const fn new(name: LoadBalancerName<'a>, load_balancer_type: LoadBalancerType<'a>) -> Self {
        Self {
            name,
            load_balancer_type,
            algorithm: None,
            labels: None,
            public_interface: None,
            network: None,
            placement: None,
            services: None,
            targets: None,
        }
    }

    /// Sets the balancing algorithm.
    #[must_use]
    pub const fn with_algorithm(mut self, algorithm: LoadBalancerAlgorithm) -> Self {
        self.algorithm = Some(algorithm);
        self
    }

    /// Sets labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: LoadBalancerLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Explicitly enables or disables the public interface.
    #[must_use]
    pub const fn with_public_interface(mut self, enabled: bool) -> Self {
        self.public_interface = Some(enabled);
        self
    }

    /// Attaches the new Load Balancer to a network.
    #[must_use]
    pub const fn with_network(mut self, network: LoadBalancerNetworkId) -> Self {
        self.network = Some(network);
        self
    }

    /// Sets one non-conflicting placement selector.
    #[must_use]
    pub const fn with_placement(mut self, placement: LoadBalancerPlacement<'a>) -> Self {
        self.placement = Some(placement);
        self
    }

    /// Sets initial services. An empty slice is retained as explicit API intent.
    #[must_use]
    pub const fn with_services(mut self, services: &'a [LoadBalancerService<'a>]) -> Self {
        self.services = Some(services);
        self
    }

    /// Sets initial targets. An empty slice is retained as explicit API intent.
    #[must_use]
    pub const fn with_targets(mut self, targets: &'a [LoadBalancerAddTargetRequest<'a>]) -> Self {
        self.targets = Some(targets);
        self
    }

    /// Returns the create endpoint.
    #[must_use]
    pub const fn endpoint(self) -> LoadBalancerEndpoint {
        LoadBalancerEndpoint::Create
    }

    /// Returns the required name.
    #[must_use]
    pub const fn name(self) -> LoadBalancerName<'a> {
        self.name
    }
    /// Returns the required type reference.
    #[must_use]
    pub const fn load_balancer_type(self) -> LoadBalancerType<'a> {
        self.load_balancer_type
    }
    /// Returns the optional algorithm.
    #[must_use]
    pub const fn algorithm(self) -> Option<LoadBalancerAlgorithm> {
        self.algorithm
    }
    /// Returns labels when supplied.
    #[must_use]
    pub const fn labels(self) -> Option<LoadBalancerLabels<'a>> {
        self.labels
    }
    /// Returns explicit public-interface intent.
    #[must_use]
    pub const fn public_interface(self) -> Option<bool> {
        self.public_interface
    }
    /// Returns an attached network when supplied.
    #[must_use]
    pub const fn network(self) -> Option<LoadBalancerNetworkId> {
        self.network
    }
    /// Returns placement when supplied.
    #[must_use]
    pub const fn placement(self) -> Option<LoadBalancerPlacement<'a>> {
        self.placement
    }
    /// Returns initial services when supplied.
    #[must_use]
    pub const fn services(self) -> Option<&'a [LoadBalancerService<'a>]> {
        self.services
    }
    /// Returns initial targets when supplied.
    #[must_use]
    pub const fn targets(self) -> Option<&'a [LoadBalancerAddTargetRequest<'a>]> {
        self.targets
    }
}

/// Partial Load Balancer update request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerUpdateRequest<'a> {
    id: LoadBalancerId,
    name: Option<LoadBalancerName<'a>>,
    labels: Option<LoadBalancerLabels<'a>>,
}

impl<'a> LoadBalancerUpdateRequest<'a> {
    /// Creates an empty update for an ID.
    #[must_use]
    pub const fn new(id: LoadBalancerId) -> Self {
        Self {
            id,
            name: None,
            labels: None,
        }
    }
    /// Replaces the name.
    #[must_use]
    pub const fn with_name(mut self, name: LoadBalancerName<'a>) -> Self {
        self.name = Some(name);
        self
    }
    /// Replaces all labels, including with an explicitly empty set.
    #[must_use]
    pub const fn with_labels(mut self, labels: LoadBalancerLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }
    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> LoadBalancerEndpoint {
        LoadBalancerEndpoint::Update(self.id)
    }
    /// Returns the replacement name.
    #[must_use]
    pub const fn name(self) -> Option<LoadBalancerName<'a>> {
        self.name
    }
    /// Returns replacement labels.
    #[must_use]
    pub const fn labels(self) -> Option<LoadBalancerLabels<'a>> {
        self.labels
    }
}

const fn sort_value(field: LoadBalancerSortField, direction: SortDirection) -> &'static str {
    match (field, direction) {
        (LoadBalancerSortField::Id, SortDirection::Asc) => "id:asc",
        (LoadBalancerSortField::Id, SortDirection::Desc) => "id:desc",
        (LoadBalancerSortField::Name, SortDirection::Asc) => "name:asc",
        (LoadBalancerSortField::Name, SortDirection::Desc) => "name:desc",
        (LoadBalancerSortField::Created, SortDirection::Asc) => "created:asc",
        (LoadBalancerSortField::Created, SortDirection::Desc) => "created:desc",
    }
}
