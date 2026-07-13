//! Floating IP request domains.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::{ApiBaseUrl, EndpointPath};

use super::super::shared::{
    CloudLabels, CloudName, CloudQueryWriter, CloudRequestError, CloudResourceId, CloudText,
    static_path, write_id_path, write_static_path,
};

/// Floating IP identifier.
pub type FloatingIpId = CloudResourceId;
/// Floating IP server identifier.
pub type FloatingIpServerId = CloudResourceId;
/// Floating IP request name.
pub type FloatingIpName<'a> = CloudName<'a>;
/// Floating IP home location name.
pub type FloatingIpHomeLocation<'a> = CloudName<'a>;
/// Floating IP description value.
pub type FloatingIpDescription<'a> = CloudText<'a>;
/// Floating IP address text.
pub type FloatingIpAddress<'a> = CloudText<'a>;
/// Floating IP DNS pointer value.
pub type FloatingIpDnsPtr<'a> = CloudText<'a>;
/// Floating IP request labels.
pub type FloatingIpLabels<'a> = CloudLabels<'a>;
/// Floating IP request error.
pub type FloatingIpRequestError = CloudRequestError;

/// Floating IP address family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatingIpType {
    /// IPv4.
    Ipv4,
    /// IPv6.
    Ipv6,
}

/// Floating IP create placement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatingIpCreatePlacement<'a> {
    /// Create and assign to this server.
    Server(FloatingIpServerId),
    /// Create with this home location.
    HomeLocation(FloatingIpHomeLocation<'a>),
}

/// Floating IP sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatingIpSortField {
    /// Sort by ID.
    Id,
    /// Sort by creation timestamp.
    Created,
}

/// Floating IP CRUD endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatingIpEndpoint {
    /// `GET /floating_ips`.
    List,
    /// `POST /floating_ips`.
    Create,
    /// `GET /floating_ips/{id}`.
    Get(FloatingIpId),
    /// `PUT /floating_ips/{id}`.
    Update(FloatingIpId),
    /// `DELETE /floating_ips/{id}`.
    Delete(FloatingIpId),
}

impl FloatingIpEndpoint {
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
        EndpointGroup::FloatingIps
    }

    /// Returns a static endpoint path when no ID is required.
    pub fn static_path(self) -> Option<Result<EndpointPath<'static>, FloatingIpRequestError>> {
        match self {
            Self::List | Self::Create => Some(static_path("/floating_ips")),
            Self::Get(_) | Self::Update(_) | Self::Delete(_) => None,
        }
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, FloatingIpRequestError> {
        match self {
            Self::List | Self::Create => write_static_path(output, "/floating_ips"),
            Self::Get(id) | Self::Update(id) | Self::Delete(id) => {
                write_id_path(output, "/floating_ips/", id, "")
            }
        }
    }
}

/// Floating IP list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FloatingIpListRequest<'a> {
    name: Option<FloatingIpName<'a>>,
    label_selector: Option<LabelSelector<'a>>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(FloatingIpSortField, SortDirection)>,
}

impl<'a> FloatingIpListRequest<'a> {
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

    /// Sets exact name filtering.
    #[must_use]
    pub const fn with_name(mut self, name: FloatingIpName<'a>) -> Self {
        self.name = Some(name);
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
    pub const fn with_sort(mut self, field: FloatingIpSortField, direction: SortDirection) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, FloatingIpRequestError> {
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
            writer.pair("sort", floating_ip_sort_value(field, direction))?;
        }
        Ok(writer.len())
    }
}

impl Default for FloatingIpListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Floating IP create request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FloatingIpCreateRequest<'a> {
    ip_type: FloatingIpType,
    placement: FloatingIpCreatePlacement<'a>,
    name: Option<FloatingIpName<'a>>,
    description: Option<FloatingIpDescription<'a>>,
    labels: Option<FloatingIpLabels<'a>>,
}

impl<'a> FloatingIpCreateRequest<'a> {
    /// Creates a validated create request with explicit server or home location.
    pub fn try_new(
        ip_type: Option<FloatingIpType>,
        placement: Option<FloatingIpCreatePlacement<'a>>,
    ) -> Result<Self, FloatingIpRequestError> {
        Ok(Self {
            ip_type: ip_type.ok_or(FloatingIpRequestError::MissingRequiredField)?,
            placement: placement.ok_or(FloatingIpRequestError::MissingRequiredField)?,
            name: None,
            description: None,
            labels: None,
        })
    }

    /// Sets resource name.
    #[must_use]
    pub const fn with_name(mut self, name: FloatingIpName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets description.
    #[must_use]
    pub const fn with_description(mut self, description: FloatingIpDescription<'a>) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: FloatingIpLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the IP type.
    #[must_use]
    pub const fn ip_type(self) -> FloatingIpType {
        self.ip_type
    }

    /// Returns the explicit placement.
    #[must_use]
    pub const fn placement(self) -> FloatingIpCreatePlacement<'a> {
        self.placement
    }
}

/// Floating IP update request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FloatingIpUpdateRequest<'a> {
    id: FloatingIpId,
    name: Option<FloatingIpName<'a>>,
    description: Option<FloatingIpDescription<'a>>,
    labels: Option<FloatingIpLabels<'a>>,
}

impl<'a> FloatingIpUpdateRequest<'a> {
    /// Creates an update request.
    #[must_use]
    pub const fn new(id: FloatingIpId) -> Self {
        Self {
            id,
            name: None,
            description: None,
            labels: None,
        }
    }

    /// Sets replacement name.
    #[must_use]
    pub const fn with_name(mut self, name: FloatingIpName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets replacement description.
    #[must_use]
    pub const fn with_description(mut self, description: FloatingIpDescription<'a>) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets replacement labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: FloatingIpLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> FloatingIpEndpoint {
        FloatingIpEndpoint::Update(self.id)
    }
}

/// Explicit DNS pointer action intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatingIpDnsPtrIntent<'a> {
    /// Set PTR to this domain name.
    Set(FloatingIpDnsPtr<'a>),
    /// Reset IPv4 to default or remove the IPv6 record.
    Reset,
}

/// Floating IP action endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatingIpActionEndpoint {
    /// `GET /floating_ips/actions`.
    ListAll,
    /// `GET /floating_ips/actions/{id}`.
    Get(ActionId),
    /// `GET /floating_ips/{id}/actions`.
    ListForFloatingIp(FloatingIpId),
    /// `POST /floating_ips/{id}/actions/assign`.
    Assign(FloatingIpId),
    /// `POST /floating_ips/{id}/actions/change_dns_ptr`.
    ChangeDnsPtr(FloatingIpId),
    /// `POST /floating_ips/{id}/actions/change_protection`.
    ChangeProtection(FloatingIpId),
    /// `POST /floating_ips/{id}/actions/unassign`.
    Unassign(FloatingIpId),
}

impl FloatingIpActionEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::ListAll | Self::Get(_) | Self::ListForFloatingIp(_) => Method::Get,
            Self::Assign(_)
            | Self::ChangeDnsPtr(_)
            | Self::ChangeProtection(_)
            | Self::Unassign(_) => Method::Post,
        }
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::FloatingIpActions
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, FloatingIpRequestError> {
        match self {
            Self::ListAll => write_static_path(output, "/floating_ips/actions"),
            Self::Get(id) => {
                let id = FloatingIpId::new(id.get()).ok_or(FloatingIpRequestError::InvalidType)?;
                write_id_path(output, "/floating_ips/actions/", id, "")
            }
            Self::ListForFloatingIp(id) => write_id_path(output, "/floating_ips/", id, "/actions"),
            Self::Assign(id) => write_id_path(output, "/floating_ips/", id, "/actions/assign"),
            Self::ChangeDnsPtr(id) => {
                write_id_path(output, "/floating_ips/", id, "/actions/change_dns_ptr")
            }
            Self::ChangeProtection(id) => {
                write_id_path(output, "/floating_ips/", id, "/actions/change_protection")
            }
            Self::Unassign(id) => write_id_path(output, "/floating_ips/", id, "/actions/unassign"),
        }
    }
}

/// Floating IP assign action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FloatingIpAssignRequest {
    server: FloatingIpServerId,
}

impl FloatingIpAssignRequest {
    /// Creates an assign request.
    pub fn try_new(server: Option<FloatingIpServerId>) -> Result<Self, FloatingIpRequestError> {
        Ok(Self {
            server: server.ok_or(FloatingIpRequestError::MissingRequiredField)?,
        })
    }

    /// Returns the server ID.
    #[must_use]
    pub const fn server(self) -> FloatingIpServerId {
        self.server
    }
}

/// Floating IP DNS pointer action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FloatingIpChangeDnsPtrRequest<'a> {
    ip: FloatingIpAddress<'a>,
    dns_ptr: FloatingIpDnsPtrIntent<'a>,
}

impl<'a> FloatingIpChangeDnsPtrRequest<'a> {
    /// Creates a DNS pointer request requiring explicit set or reset.
    pub fn try_new(
        ip: FloatingIpAddress<'a>,
        dns_ptr: Option<FloatingIpDnsPtrIntent<'a>>,
    ) -> Result<Self, FloatingIpRequestError> {
        Ok(Self {
            ip,
            dns_ptr: dns_ptr.ok_or(FloatingIpRequestError::MissingDnsPtrIntent)?,
        })
    }

    /// Returns the IP address whose pointer changes.
    #[must_use]
    pub const fn ip(self) -> FloatingIpAddress<'a> {
        self.ip
    }

    /// Returns the explicit DNS pointer intent.
    #[must_use]
    pub const fn dns_ptr(self) -> FloatingIpDnsPtrIntent<'a> {
        self.dns_ptr
    }
}

/// Floating IP protection action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FloatingIpProtectionRequest {
    delete: bool,
}

impl FloatingIpProtectionRequest {
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

const fn floating_ip_sort_value(
    field: FloatingIpSortField,
    direction: SortDirection,
) -> &'static str {
    match (field, direction) {
        (FloatingIpSortField::Id, SortDirection::Asc) => "id:asc",
        (FloatingIpSortField::Id, SortDirection::Desc) => "id:desc",
        (FloatingIpSortField::Created, SortDirection::Asc) => "created:asc",
        (FloatingIpSortField::Created, SortDirection::Desc) => "created:desc",
    }
}

#[cfg(test)]
mod tests;
