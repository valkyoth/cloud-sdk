//! Firewall action endpoints and request bodies.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::request::ApiBaseUrl;

use super::super::shared::{CloudRequestError, CloudResourceId, write_id_path, write_static_path};
use super::rules::FirewallRuleSet;
use super::{FirewallId, FirewallRequestError, FirewallResource};

/// Firewall action endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FirewallActionEndpoint {
    /// `GET /firewalls/actions`.
    ListAll,
    /// `GET /firewalls/actions/{id}`.
    Get(ActionId),
    /// `GET /firewalls/{id}/actions`.
    ListForFirewall(FirewallId),
    /// `POST /firewalls/{id}/actions/apply_to_resources`.
    ApplyToResources(FirewallId),
    /// `POST /firewalls/{id}/actions/remove_from_resources`.
    RemoveFromResources(FirewallId),
    /// `POST /firewalls/{id}/actions/set_rules`.
    SetRules(FirewallId),
}

impl FirewallActionEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::ListAll | Self::Get(_) | Self::ListForFirewall(_) => Method::Get,
            Self::ApplyToResources(_) | Self::RemoveFromResources(_) | Self::SetRules(_) => {
                Method::Post
            }
        }
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::FirewallActions
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, FirewallRequestError> {
        match self {
            Self::ListAll => write_static_path(output, "/firewalls/actions"),
            Self::Get(id) => {
                let id = CloudResourceId::new(id.get()).ok_or(CloudRequestError::InvalidType)?;
                write_id_path(output, "/firewalls/actions/", id, "")
            }
            Self::ListForFirewall(id) => write_id_path(output, "/firewalls/", id, "/actions"),
            Self::ApplyToResources(id) => {
                write_id_path(output, "/firewalls/", id, "/actions/apply_to_resources")
            }
            Self::RemoveFromResources(id) => {
                write_id_path(output, "/firewalls/", id, "/actions/remove_from_resources")
            }
            Self::SetRules(id) => write_id_path(output, "/firewalls/", id, "/actions/set_rules"),
        }
    }
}

/// Required resource list for apply/remove actions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirewallResourcesRequest<'a> {
    resources: &'a [FirewallResource<'a>],
    intent: FirewallResourceIntent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FirewallResourceIntent {
    Apply,
    Remove,
}

impl<'a> FirewallResourcesRequest<'a> {
    /// Creates an apply-to-resources request.
    ///
    /// An explicitly empty list is retained for exact API intent. Use
    /// [`Self::remove`] for the destructive remove operation.
    #[must_use]
    pub const fn new(resources: &'a [FirewallResource<'a>]) -> Self {
        Self::apply(resources)
    }

    /// Creates an apply-to-resources request.
    #[must_use]
    pub const fn apply(resources: &'a [FirewallResource<'a>]) -> Self {
        Self {
            resources,
            intent: FirewallResourceIntent::Apply,
        }
    }

    /// Creates a remove-from-resources request.
    #[must_use]
    pub const fn remove(resources: &'a [FirewallResource<'a>]) -> Self {
        Self {
            resources,
            intent: FirewallResourceIntent::Remove,
        }
    }

    /// Returns the resources.
    #[must_use]
    pub const fn resources(self) -> &'a [FirewallResource<'a>] {
        self.resources
    }

    pub(crate) const fn intent(self) -> FirewallResourceIntent {
        self.intent
    }
}

/// Required replacement rules for the set-rules action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FirewallSetRulesRequest<'a> {
    rules: FirewallRuleSet<'a>,
}

impl<'a> FirewallSetRulesRequest<'a> {
    /// Creates an explicit replacement request, including an empty ruleset.
    #[must_use]
    pub const fn new(rules: FirewallRuleSet<'a>) -> Self {
        Self { rules }
    }

    /// Returns the replacement rules.
    #[must_use]
    pub const fn rules(self) -> FirewallRuleSet<'a> {
        self.rules
    }
}
