//! Network action endpoints and request bodies.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::cloud::ip::{NetworkIpRange, SubnetIpRange};
use crate::request::ApiBaseUrl;

use super::super::shared::{CloudRequestError, CloudResourceId, write_id_path, write_static_path};
use super::resources::{NetworkId, NetworkRequestError, NetworkRoute, NetworkSubnet};

/// Network action endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkActionEndpoint {
    /// `GET /networks/actions`.
    ListAll,
    /// `GET /networks/actions/{id}`.
    Get(ActionId),
    /// `GET /networks/{id}/actions`.
    ListForNetwork(NetworkId),
    /// `POST /networks/{id}/actions/add_route`.
    AddRoute(NetworkId),
    /// `POST /networks/{id}/actions/add_subnet`.
    AddSubnet(NetworkId),
    /// `POST /networks/{id}/actions/change_ip_range`.
    ChangeIpRange(NetworkId),
    /// `POST /networks/{id}/actions/change_protection`.
    ChangeProtection(NetworkId),
    /// `POST /networks/{id}/actions/delete_route`.
    DeleteRoute(NetworkId),
    /// `POST /networks/{id}/actions/delete_subnet`.
    DeleteSubnet(NetworkId),
}

impl NetworkActionEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::ListAll | Self::Get(_) | Self::ListForNetwork(_) => Method::Get,
            Self::AddRoute(_)
            | Self::AddSubnet(_)
            | Self::ChangeIpRange(_)
            | Self::ChangeProtection(_)
            | Self::DeleteRoute(_)
            | Self::DeleteSubnet(_) => Method::Post,
        }
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::NetworkActions
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, NetworkRequestError> {
        match self {
            Self::ListAll => write_static_path(output, "/networks/actions"),
            Self::Get(id) => {
                let id = CloudResourceId::new(id.get()).ok_or(CloudRequestError::InvalidType)?;
                write_id_path(output, "/networks/actions/", id, "")
            }
            Self::ListForNetwork(id) => write_id_path(output, "/networks/", id, "/actions"),
            Self::AddRoute(id) => write_id_path(output, "/networks/", id, "/actions/add_route"),
            Self::AddSubnet(id) => write_id_path(output, "/networks/", id, "/actions/add_subnet"),
            Self::ChangeIpRange(id) => {
                write_id_path(output, "/networks/", id, "/actions/change_ip_range")
            }
            Self::ChangeProtection(id) => {
                write_id_path(output, "/networks/", id, "/actions/change_protection")
            }
            Self::DeleteRoute(id) => {
                write_id_path(output, "/networks/", id, "/actions/delete_route")
            }
            Self::DeleteSubnet(id) => {
                write_id_path(output, "/networks/", id, "/actions/delete_subnet")
            }
        }
    }
}

/// Add or delete route request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkRouteRequest<'a>(NetworkRoute<'a>);

impl<'a> NetworkRouteRequest<'a> {
    /// Creates a request with required route fields.
    #[must_use]
    pub const fn new(route: NetworkRoute<'a>) -> Self {
        Self(route)
    }

    /// Returns the route.
    #[must_use]
    pub const fn route(self) -> NetworkRoute<'a> {
        self.0
    }
}

/// Add-subnet request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkAddSubnetRequest<'a>(NetworkSubnet<'a>);

impl<'a> NetworkAddSubnetRequest<'a> {
    /// Creates a request with required type and network zone encoded by the subnet type.
    #[must_use]
    pub const fn new(subnet: NetworkSubnet<'a>) -> Self {
        Self(subnet)
    }

    /// Returns the subnet.
    #[must_use]
    pub const fn subnet(self) -> NetworkSubnet<'a> {
        self.0
    }
}

/// Delete-subnet request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkDeleteSubnetRequest<'a>(SubnetIpRange<'a>);

impl<'a> NetworkDeleteSubnetRequest<'a> {
    /// Creates a request with the required CIDR.
    #[must_use]
    pub const fn new(ip_range: SubnetIpRange<'a>) -> Self {
        Self(ip_range)
    }

    /// Returns the subnet range.
    #[must_use]
    pub const fn ip_range(self) -> SubnetIpRange<'a> {
        self.0
    }
}

/// Change-network-range request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkChangeIpRangeRequest<'a>(NetworkIpRange<'a>);

impl<'a> NetworkChangeIpRangeRequest<'a> {
    /// Creates a request with the required private range.
    #[must_use]
    pub const fn new(ip_range: NetworkIpRange<'a>) -> Self {
        Self(ip_range)
    }

    /// Returns the target range.
    #[must_use]
    pub const fn ip_range(self) -> NetworkIpRange<'a> {
        self.0
    }
}

/// Network delete-protection request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkProtectionRequest {
    delete: bool,
}

impl NetworkProtectionRequest {
    /// Creates an explicit protection request.
    #[must_use]
    pub const fn new(delete: bool) -> Self {
        Self { delete }
    }

    /// Returns the delete-protection setting.
    #[must_use]
    pub const fn delete(self) -> bool {
        self.delete
    }
}
