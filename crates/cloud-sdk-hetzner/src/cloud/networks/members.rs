//! Resources attached to a Network.
//!
//! Combine this endpoint with `SourceLockedQuery` and
//! `SourceQueryOperation::LIST_NETWORK_MEMBERS` for repeated type, subnet,
//! status and sort filters and numbered pagination.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::cloud::shared::write_id_path;
use crate::request::ApiBaseUrl;

use super::{NetworkId, NetworkRequestError};

/// Read-only endpoint listing resources attached to one Network.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NetworkMembersEndpoint(NetworkId);

impl NetworkMembersEndpoint {
    /// Selects the Network whose members will be returned.
    #[must_use]
    pub const fn new(id: NetworkId) -> Self {
        Self(id)
    }

    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        Method::Get
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::Networks
    }

    /// Returns the fixed base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes a validated path into caller-owned storage.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, NetworkRequestError> {
        write_id_path(output, "/networks/", self.0, "/members")
    }
}
