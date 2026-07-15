//! Server endpoint request domains.

mod shared;

pub mod placement_groups;

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::{ApiBaseUrl, EndpointPath};

use self::shared::{ResourceId, ServerQueryError, static_path, write_id_path, write_query_pair};

/// Server endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::Servers,
    EndpointGroup::ServerActions,
    EndpointGroup::PlacementGroups,
];

/// Server identifier.
pub type ServerId = ResourceId;

/// Server-adjacent resource identifier.
pub type ServerResourceId = ResourceId;

/// Server request error.
pub use self::shared::ServerRequestError;

/// Server name validated as a conservative RFC 1123 hostname.
pub use self::shared::ServerName;

/// ID-or-name server create/action reference.
pub use self::shared::ServerReference;

/// Bounded server action/query text value.
pub use self::shared::TextValue;

/// RFC3339 metrics timestamp.
pub use self::shared::TimestampValue;

/// Cloud-init user data value capped by the source-locked API limit.
pub use self::shared::UserData;

/// Server list status filter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerStatus {
    /// Running.
    Running,
    /// Initializing.
    Initializing,
    /// Starting.
    Starting,
    /// Stopping.
    Stopping,
    /// Off.
    Off,
    /// Deleting.
    Deleting,
    /// Migrating.
    Migrating,
    /// Rebuilding.
    Rebuilding,
    /// Unknown.
    Unknown,
}

impl ServerStatus {
    const fn as_api_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Initializing => "initializing",
            Self::Starting => "starting",
            Self::Stopping => "stopping",
            Self::Off => "off",
            Self::Deleting => "deleting",
            Self::Migrating => "migrating",
            Self::Rebuilding => "rebuilding",
            Self::Unknown => "unknown",
        }
    }
}

/// Server sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerSortField {
    /// Sort by ID.
    Id,
    /// Sort by name.
    Name,
    /// Sort by creation timestamp.
    Created,
}

/// Server CRUD and metrics endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerEndpoint {
    /// `GET /servers`.
    List,
    /// `POST /servers`.
    Create,
    /// `GET /servers/{id}`.
    Get(ServerId),
    /// `PUT /servers/{id}`.
    Update(ServerId),
    /// `DELETE /servers/{id}`.
    Delete(ServerId),
    /// `GET /servers/{id}/metrics`.
    Metrics(ServerId),
}

impl ServerEndpoint {
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

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::Servers
    }

    /// Returns a static endpoint path when no ID is required.
    pub fn static_path(self) -> Option<Result<EndpointPath<'static>, ServerRequestError>> {
        match self {
            Self::List | Self::Create => Some(static_path("/servers")),
            Self::Get(_) | Self::Update(_) | Self::Delete(_) | Self::Metrics(_) => None,
        }
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, ServerRequestError> {
        match self {
            Self::List | Self::Create => shared::write_static_path(output, "/servers"),
            Self::Get(id) | Self::Update(id) | Self::Delete(id) => {
                write_id_path(output, "/servers/", id, "")
            }
            Self::Metrics(id) => write_id_path(output, "/servers/", id, "/metrics"),
        }
    }
}

/// Server list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerListRequest<'a> {
    name: Option<ServerName<'a>>,
    label_selector: Option<LabelSelector<'a>>,
    status: Option<ServerStatus>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(ServerSortField, SortDirection)>,
}

impl<'a> ServerListRequest<'a> {
    /// Creates an empty list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            label_selector: None,
            status: None,
            page: None,
            per_page: None,
            sort: None,
        }
    }

    /// Sets exact name filtering.
    #[must_use]
    pub const fn with_name(mut self, name: ServerName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets label-selector filtering.
    #[must_use]
    pub const fn with_label_selector(mut self, selector: LabelSelector<'a>) -> Self {
        self.label_selector = Some(selector);
        self
    }

    /// Sets status filtering.
    #[must_use]
    pub const fn with_status(mut self, status: ServerStatus) -> Self {
        self.status = Some(status);
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
    pub const fn with_sort(mut self, field: ServerSortField, direction: SortDirection) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, ServerRequestError> {
        let mut writer = ServerQueryError::new(output);
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
        if let Some(status) = self.status {
            writer.pair("status", status.as_api_str())?;
        }
        Ok(writer.len())
    }
}

impl Default for ServerListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Public network primary IP selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimaryIpSelection {
    /// Omit the field and let Hetzner auto-create when enabled.
    Auto,
    /// Use an existing primary IP ID.
    Id(ServerResourceId),
    /// Explicit JSON null semantics for future body serialization.
    Null,
}

/// Server public network create options.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerPublicNet {
    pub(crate) enable_ipv4: bool,
    pub(crate) enable_ipv6: bool,
    pub(crate) ipv4: PrimaryIpSelection,
    pub(crate) ipv6: PrimaryIpSelection,
}

impl ServerPublicNet {
    /// Creates validated public network options.
    pub fn new(
        enable_ipv4: bool,
        enable_ipv6: bool,
        ipv4: PrimaryIpSelection,
        ipv6: PrimaryIpSelection,
    ) -> Result<Self, ServerRequestError> {
        if !enable_ipv4 && matches!(ipv4, PrimaryIpSelection::Id(_)) {
            return Err(ServerRequestError::MutuallyExclusiveFields);
        }
        if !enable_ipv6 && matches!(ipv6, PrimaryIpSelection::Id(_)) {
            return Err(ServerRequestError::MutuallyExclusiveFields);
        }
        Ok(Self {
            enable_ipv4,
            enable_ipv6,
            ipv4,
            ipv6,
        })
    }
}

/// Server create request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerCreateRequest<'a> {
    pub(crate) name: ServerName<'a>,
    pub(crate) server_type: ServerReference<'a>,
    pub(crate) image: ServerReference<'a>,
    pub(crate) location: Option<ServerReference<'a>>,
    pub(crate) public_net: Option<ServerPublicNet>,
    pub(crate) user_data: Option<UserData<'a>>,
}

impl<'a> ServerCreateRequest<'a> {
    /// Creates a validated server create request.
    ///
    /// Required inputs cannot be omitted:
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::cloud::servers::ServerCreateRequest;
    ///
    /// let _ = ServerCreateRequest::new();
    /// ```
    #[must_use]
    pub const fn new(
        name: ServerName<'a>,
        server_type: ServerReference<'a>,
        image: ServerReference<'a>,
    ) -> Self {
        Self {
            name,
            server_type,
            image,
            location: None,
            public_net: None,
            user_data: None,
        }
    }

    /// Sets the location reference.
    #[must_use]
    pub const fn with_location(mut self, location: ServerReference<'a>) -> Self {
        self.location = Some(location);
        self
    }

    /// Sets public network options.
    #[must_use]
    pub const fn with_public_net(mut self, public_net: ServerPublicNet) -> Self {
        self.public_net = Some(public_net);
        self
    }

    /// Sets cloud-init user data.
    #[must_use]
    pub const fn with_user_data(mut self, user_data: UserData<'a>) -> Self {
        self.user_data = Some(user_data);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> ServerEndpoint {
        ServerEndpoint::Create
    }
}

/// Server update request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerUpdateRequest<'a> {
    id: ServerId,
    pub(crate) name: Option<ServerName<'a>>,
}

impl<'a> ServerUpdateRequest<'a> {
    /// Creates a server update request for an ID.
    #[must_use]
    pub const fn new(id: ServerId) -> Self {
        Self { id, name: None }
    }

    /// Sets the replacement name.
    #[must_use]
    pub const fn with_name(mut self, name: ServerName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> ServerEndpoint {
        ServerEndpoint::Update(self.id)
    }
}

/// Metrics type query value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerMetricType {
    /// CPU metrics.
    Cpu,
    /// Disk metrics.
    Disk,
    /// Network metrics.
    Network,
}

impl ServerMetricType {
    const fn as_api_str(self) -> &'static str {
        match self {
            Self::Cpu => "cpu",
            Self::Disk => "disk",
            Self::Network => "network",
        }
    }
}

/// Server metrics request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerMetricsRequest<'a> {
    id: ServerId,
    metric_type: ServerMetricType,
    start: TimestampValue<'a>,
    end: TimestampValue<'a>,
    step: Option<TextValue<'a>>,
}

impl<'a> ServerMetricsRequest<'a> {
    /// Creates a metrics request with required query fields.
    pub fn try_new(
        id: ServerId,
        metric_type: ServerMetricType,
        start: TimestampValue<'a>,
        end: TimestampValue<'a>,
    ) -> Result<Self, ServerRequestError> {
        if start.as_str() >= end.as_str() {
            return Err(ServerRequestError::InvalidTimeRange);
        }
        Ok(Self {
            id,
            metric_type,
            start,
            end,
            step: None,
        })
    }

    /// Sets the optional resolution step.
    #[must_use]
    pub const fn with_step(mut self, step: TextValue<'a>) -> Self {
        self.step = Some(step);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> ServerEndpoint {
        ServerEndpoint::Metrics(self.id)
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, ServerRequestError> {
        let mut len = 0;
        let mut first = true;
        write_query_pair(output, &mut len, &mut first, "end", self.end.as_str())?;
        if let Some(step) = self.step {
            write_query_pair(output, &mut len, &mut first, "step", step.as_str())?;
        }
        write_query_pair(output, &mut len, &mut first, "start", self.start.as_str())?;
        write_query_pair(
            output,
            &mut len,
            &mut first,
            "type",
            self.metric_type.as_api_str(),
        )?;
        Ok(len)
    }
}

const fn sort_value(field: ServerSortField, direction: SortDirection) -> &'static str {
    match (field, direction) {
        (ServerSortField::Id, SortDirection::Asc) => "id:asc",
        (ServerSortField::Id, SortDirection::Desc) => "id:desc",
        (ServerSortField::Name, SortDirection::Asc) => "name:asc",
        (ServerSortField::Name, SortDirection::Desc) => "name:desc",
        (ServerSortField::Created, SortDirection::Asc) => "created:asc",
        (ServerSortField::Created, SortDirection::Desc) => "created:desc",
    }
}

pub mod actions;

#[cfg(test)]
mod tests;
