//! Firewall request domains.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::ApiBaseUrl;

use super::shared::{
    CloudLabels, CloudName, CloudQueryWriter, CloudRequestError, CloudResourceId, write_id_path,
    write_static_path,
};

pub mod actions;
pub mod rules;

/// Firewall identifier.
pub type FirewallId = CloudResourceId;
/// Server identifier accepted by Firewall resource selectors.
pub type FirewallServerId = CloudResourceId;
/// Firewall name.
pub type FirewallName<'a> = CloudName<'a>;
/// Firewall labels.
pub type FirewallLabels<'a> = CloudLabels<'a>;
/// Firewall request error.
pub type FirewallRequestError = CloudRequestError;

/// Firewall endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] =
    &[EndpointGroup::Firewalls, EndpointGroup::FirewallActions];

/// Firewall sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirewallSortField {
    /// Sort by ID.
    Id,
    /// Sort by name.
    Name,
    /// Sort by creation timestamp.
    Created,
}

/// A resource to which a Firewall is applied.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirewallResource<'a> {
    /// A concrete server.
    Server(FirewallServerId),
    /// Every server matching a label selector.
    LabelSelector(LabelSelector<'a>),
}

/// Firewall CRUD endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirewallEndpoint {
    /// `GET /firewalls`.
    List,
    /// `POST /firewalls`.
    Create,
    /// `GET /firewalls/{id}`.
    Get(FirewallId),
    /// `PUT /firewalls/{id}`.
    Update(FirewallId),
    /// `DELETE /firewalls/{id}`.
    Delete(FirewallId),
}

impl FirewallEndpoint {
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
        EndpointGroup::Firewalls
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, FirewallRequestError> {
        match self {
            Self::List | Self::Create => write_static_path(output, "/firewalls"),
            Self::Get(id) | Self::Update(id) | Self::Delete(id) => {
                write_id_path(output, "/firewalls/", id, "")
            }
        }
    }
}

/// Firewall list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirewallListRequest<'a> {
    name: Option<FirewallName<'a>>,
    label_selector: Option<LabelSelector<'a>>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(FirewallSortField, SortDirection)>,
}

impl<'a> FirewallListRequest<'a> {
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
    pub const fn with_name(mut self, name: FirewallName<'a>) -> Self {
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
    pub const fn with_sort(mut self, field: FirewallSortField, direction: SortDirection) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, FirewallRequestError> {
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
            writer.pair("sort", firewall_sort_value(field, direction))?;
        }
        Ok(writer.len())
    }
}

impl Default for FirewallListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Firewall create request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirewallCreateRequest<'a> {
    name: FirewallName<'a>,
    labels: Option<FirewallLabels<'a>>,
    rules: Option<rules::FirewallRuleSet<'a>>,
    apply_to: Option<&'a [FirewallResource<'a>]>,
}

impl<'a> FirewallCreateRequest<'a> {
    /// Creates a request with the required unique name.
    #[must_use]
    pub const fn new(name: FirewallName<'a>) -> Self {
        Self {
            name,
            labels: None,
            rules: None,
            apply_to: None,
        }
    }

    /// Sets labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: FirewallLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Sets the initial validated rules.
    #[must_use]
    pub const fn with_rules(mut self, rules: rules::FirewallRuleSet<'a>) -> Self {
        self.rules = Some(rules);
        self
    }

    /// Sets resources to apply the Firewall to.
    #[must_use]
    pub const fn with_resources(mut self, resources: &'a [FirewallResource<'a>]) -> Self {
        self.apply_to = Some(resources);
        self
    }

    /// Returns the required name.
    #[must_use]
    pub const fn name(self) -> FirewallName<'a> {
        self.name
    }

    /// Returns labels when supplied.
    #[must_use]
    pub const fn labels(self) -> Option<FirewallLabels<'a>> {
        self.labels
    }

    /// Returns the initial rules when supplied.
    #[must_use]
    pub const fn rules(self) -> Option<rules::FirewallRuleSet<'a>> {
        self.rules
    }

    /// Returns application targets when supplied.
    #[must_use]
    pub const fn resources(self) -> Option<&'a [FirewallResource<'a>]> {
        self.apply_to
    }
}

/// Firewall update request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirewallUpdateRequest<'a> {
    id: FirewallId,
    name: Option<FirewallName<'a>>,
    labels: Option<FirewallLabels<'a>>,
}

impl<'a> FirewallUpdateRequest<'a> {
    /// Creates an update request.
    #[must_use]
    pub const fn new(id: FirewallId) -> Self {
        Self {
            id,
            name: None,
            labels: None,
        }
    }

    /// Sets the replacement name.
    #[must_use]
    pub const fn with_name(mut self, name: FirewallName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets replacement labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: FirewallLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the update endpoint.
    #[must_use]
    pub const fn endpoint(self) -> FirewallEndpoint {
        FirewallEndpoint::Update(self.id)
    }

    /// Returns the replacement name when supplied.
    #[must_use]
    pub const fn name(self) -> Option<FirewallName<'a>> {
        self.name
    }

    /// Returns replacement labels when supplied.
    #[must_use]
    pub const fn labels(self) -> Option<FirewallLabels<'a>> {
        self.labels
    }
}

const fn firewall_sort_value(field: FirewallSortField, direction: SortDirection) -> &'static str {
    match (field, direction) {
        (FirewallSortField::Id, SortDirection::Asc) => "id:asc",
        (FirewallSortField::Id, SortDirection::Desc) => "id:desc",
        (FirewallSortField::Name, SortDirection::Asc) => "name:asc",
        (FirewallSortField::Name, SortDirection::Desc) => "name:desc",
        (FirewallSortField::Created, SortDirection::Asc) => "created:asc",
        (FirewallSortField::Created, SortDirection::Desc) => "created:desc",
    }
}

#[cfg(test)]
mod tests;
