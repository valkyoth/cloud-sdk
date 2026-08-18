//! Primary IP request domains.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::{ApiBaseUrl, EndpointPath};

use super::super::shared::{
    CloudLabels, CloudName, CloudRequestError, CloudResourceId, CloudText, encode_query,
    static_path, write_id_path, write_static_path,
};

/// Primary IP endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] =
    &[EndpointGroup::PrimaryIps, EndpointGroup::PrimaryIpActions];

/// Primary IP identifier.
pub type PrimaryIpId = CloudResourceId;
/// Primary IP assignee identifier.
pub type PrimaryIpAssigneeId = CloudResourceId;
/// Primary IP request name.
pub type PrimaryIpName<'a> = CloudName<'a>;
/// Primary IP location name.
pub type PrimaryIpLocation<'a> = CloudName<'a>;
/// Primary IP request labels.
pub type PrimaryIpLabels<'a> = CloudLabels<'a>;
/// Primary IP address text.
pub type PrimaryIpAddress<'a> = CloudText<'a>;
/// Primary IP DNS pointer value.
pub type PrimaryIpDnsPtr<'a> = CloudText<'a>;
/// Primary IP request error.
pub type PrimaryIpRequestError = CloudRequestError;

/// Primary IP address family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimaryIpType {
    /// IPv4.
    Ipv4,
    /// IPv6.
    Ipv6,
}

/// Primary IP sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimaryIpSortField {
    /// Sort by ID.
    Id,
    /// Sort by creation timestamp.
    Created,
}

/// Primary IP CRUD endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimaryIpEndpoint {
    /// `GET /primary_ips`.
    List,
    /// `POST /primary_ips`.
    Create,
    /// `GET /primary_ips/{id}`.
    Get(PrimaryIpId),
    /// `PUT /primary_ips/{id}`.
    Update(PrimaryIpId),
    /// `DELETE /primary_ips/{id}`.
    Delete(PrimaryIpId),
}

impl PrimaryIpEndpoint {
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
        EndpointGroup::PrimaryIps
    }

    /// Returns a static endpoint path when no ID is required.
    pub fn static_path(self) -> Option<Result<EndpointPath<'static>, PrimaryIpRequestError>> {
        match self {
            Self::List | Self::Create => Some(static_path("/primary_ips")),
            Self::Get(_) | Self::Update(_) | Self::Delete(_) => None,
        }
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, PrimaryIpRequestError> {
        match self {
            Self::List | Self::Create => write_static_path(output, "/primary_ips"),
            Self::Get(id) | Self::Update(id) | Self::Delete(id) => {
                write_id_path(output, "/primary_ips/", id, "")
            }
        }
    }
}

/// Primary IP list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrimaryIpListRequest<'a> {
    name: Option<PrimaryIpName<'a>>,
    ip: Option<PrimaryIpAddress<'a>>,
    label_selector: Option<LabelSelector<'a>>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(PrimaryIpSortField, SortDirection)>,
}

impl<'a> PrimaryIpListRequest<'a> {
    /// Creates an empty list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            ip: None,
            label_selector: None,
            page: None,
            per_page: None,
            sort: None,
        }
    }

    /// Sets exact name filtering.
    #[must_use]
    pub const fn with_name(mut self, name: PrimaryIpName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets exact IP-address filtering.
    #[must_use]
    pub const fn with_ip(mut self, ip: PrimaryIpAddress<'a>) -> Self {
        self.ip = Some(ip);
        self
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
    pub const fn with_sort(mut self, field: PrimaryIpSortField, direction: SortDirection) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, PrimaryIpRequestError> {
        encode_query(output, |writer, first| {
            if let Some(ip) = self.ip {
                writer.query_pair(first, "ip", ip.as_str())?;
            }
            if let Some(selector) = self.label_selector {
                writer.query_pair(first, "label_selector", selector.as_str())?;
            }
            if let Some(name) = self.name {
                writer.query_pair(first, "name", name.as_str())?;
            }
            if let Some(page) = self.page {
                writer.query_u64(first, "page", page.get())?;
            }
            if let Some(per_page) = self.per_page {
                writer.query_u64(first, "per_page", u64::from(per_page.get()))?;
            }
            if let Some((field, direction)) = self.sort {
                writer.query_pair(first, "sort", primary_ip_sort_value(field, direction))?;
            }
            Ok(())
        })
    }
}

impl Default for PrimaryIpListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Primary IP create request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrimaryIpCreateRequest<'a> {
    ip_type: PrimaryIpType,
    pub(crate) name: Option<PrimaryIpName<'a>>,
    pub(crate) assignee_id: Option<PrimaryIpAssigneeId>,
    pub(crate) auto_delete: bool,
    pub(crate) location: Option<PrimaryIpLocation<'a>>,
    pub(crate) labels: Option<PrimaryIpLabels<'a>>,
}

impl<'a> PrimaryIpCreateRequest<'a> {
    /// Creates a validated create request without deprecated datacenter fields.
    #[must_use]
    pub const fn new(ip_type: PrimaryIpType) -> Self {
        Self {
            ip_type,
            name: None,
            assignee_id: None,
            auto_delete: false,
            location: None,
            labels: None,
        }
    }

    /// Sets a resource name.
    #[must_use]
    pub const fn with_name(mut self, name: PrimaryIpName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets server assignment at create time.
    #[must_use]
    pub const fn with_assignee(mut self, assignee_id: PrimaryIpAssigneeId) -> Self {
        self.assignee_id = Some(assignee_id);
        self
    }

    /// Sets auto-delete behavior when assigned to a server.
    #[must_use]
    pub const fn with_auto_delete(mut self, auto_delete: bool) -> Self {
        self.auto_delete = auto_delete;
        self
    }

    /// Sets location by name. Deprecated datacenter fields are intentionally not modeled.
    #[must_use]
    pub const fn with_location(mut self, location: PrimaryIpLocation<'a>) -> Self {
        self.location = Some(location);
        self
    }

    /// Sets labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: PrimaryIpLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> PrimaryIpEndpoint {
        PrimaryIpEndpoint::Create
    }

    /// Returns the IP type.
    #[must_use]
    pub const fn ip_type(self) -> PrimaryIpType {
        self.ip_type
    }
}

/// Primary IP update request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrimaryIpUpdateRequest<'a> {
    id: PrimaryIpId,
    pub(crate) name: Option<PrimaryIpName<'a>>,
    pub(crate) labels: Option<PrimaryIpLabels<'a>>,
}

impl<'a> PrimaryIpUpdateRequest<'a> {
    /// Creates an update request.
    #[must_use]
    pub const fn new(id: PrimaryIpId) -> Self {
        Self {
            id,
            name: None,
            labels: None,
        }
    }

    /// Sets replacement name.
    #[must_use]
    pub const fn with_name(mut self, name: PrimaryIpName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets replacement labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: PrimaryIpLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> PrimaryIpEndpoint {
        PrimaryIpEndpoint::Update(self.id)
    }
}

/// Explicit DNS pointer action intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimaryIpDnsPtrIntent<'a> {
    /// Set PTR to this domain name.
    Set(PrimaryIpDnsPtr<'a>),
    /// Reset to provider default behavior.
    Reset,
}

/// Primary IP action endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimaryIpActionEndpoint {
    /// `GET /primary_ips/actions`.
    ListAll,
    /// `GET /primary_ips/actions/{id}`.
    Get(ActionId),
    /// `GET /primary_ips/{id}/actions`.
    ListForPrimaryIp(PrimaryIpId),
    /// `POST /primary_ips/{id}/actions/assign`.
    Assign(PrimaryIpId),
    /// `POST /primary_ips/{id}/actions/change_dns_ptr`.
    ChangeDnsPtr(PrimaryIpId),
    /// `POST /primary_ips/{id}/actions/change_protection`.
    ChangeProtection(PrimaryIpId),
    /// `POST /primary_ips/{id}/actions/unassign`.
    Unassign(PrimaryIpId),
}

impl PrimaryIpActionEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::ListAll | Self::Get(_) | Self::ListForPrimaryIp(_) => Method::Get,
            Self::Assign(_)
            | Self::ChangeDnsPtr(_)
            | Self::ChangeProtection(_)
            | Self::Unassign(_) => Method::Post,
        }
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::PrimaryIpActions
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, PrimaryIpRequestError> {
        match self {
            Self::ListAll => write_static_path(output, "/primary_ips/actions"),
            Self::Get(id) => {
                let id = PrimaryIpId::new(id.get()).ok_or(PrimaryIpRequestError::InvalidType)?;
                write_id_path(output, "/primary_ips/actions/", id, "")
            }
            Self::ListForPrimaryIp(id) => write_id_path(output, "/primary_ips/", id, "/actions"),
            Self::Assign(id) => write_id_path(output, "/primary_ips/", id, "/actions/assign"),
            Self::ChangeDnsPtr(id) => {
                write_id_path(output, "/primary_ips/", id, "/actions/change_dns_ptr")
            }
            Self::ChangeProtection(id) => {
                write_id_path(output, "/primary_ips/", id, "/actions/change_protection")
            }
            Self::Unassign(id) => write_id_path(output, "/primary_ips/", id, "/actions/unassign"),
        }
    }
}

/// Primary IP assign action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrimaryIpAssignRequest {
    assignee_id: PrimaryIpAssigneeId,
}

impl PrimaryIpAssignRequest {
    /// Creates an assign request.
    #[must_use]
    pub const fn new(assignee_id: PrimaryIpAssigneeId) -> Self {
        Self { assignee_id }
    }

    /// Returns the assignee ID.
    #[must_use]
    pub const fn assignee_id(self) -> PrimaryIpAssigneeId {
        self.assignee_id
    }
}

/// Primary IP DNS pointer action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrimaryIpChangeDnsPtrRequest<'a> {
    ip: PrimaryIpAddress<'a>,
    dns_ptr: PrimaryIpDnsPtrIntent<'a>,
}

impl<'a> PrimaryIpChangeDnsPtrRequest<'a> {
    /// Creates a DNS pointer request requiring explicit set or reset.
    #[must_use]
    pub const fn new(ip: PrimaryIpAddress<'a>, dns_ptr: PrimaryIpDnsPtrIntent<'a>) -> Self {
        Self { ip, dns_ptr }
    }

    /// Returns the explicit DNS pointer intent.
    #[must_use]
    pub const fn dns_ptr(self) -> PrimaryIpDnsPtrIntent<'a> {
        self.dns_ptr
    }

    /// Returns the IP address whose pointer changes.
    #[must_use]
    pub const fn ip(self) -> PrimaryIpAddress<'a> {
        self.ip
    }
}

/// Primary IP protection action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PrimaryIpProtectionRequest {
    delete: bool,
}

impl PrimaryIpProtectionRequest {
    /// Creates a protection request.
    #[must_use]
    pub const fn new(delete: bool) -> Self {
        Self { delete }
    }

    /// Returns the delete protection value.
    #[must_use]
    pub const fn delete(self) -> bool {
        self.delete
    }
}

const fn primary_ip_sort_value(
    field: PrimaryIpSortField,
    direction: SortDirection,
) -> &'static str {
    match (field, direction) {
        (PrimaryIpSortField::Id, SortDirection::Asc) => "id:asc",
        (PrimaryIpSortField::Id, SortDirection::Desc) => "id:desc",
        (PrimaryIpSortField::Created, SortDirection::Asc) => "created:asc",
        (PrimaryIpSortField::Created, SortDirection::Desc) => "created:desc",
    }
}

#[cfg(test)]
mod tests;
