//! Load Balancer action endpoints and request bodies.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::cloud::ip::SubnetIpRange;
use crate::cloud::shared::write_id_path;
use crate::request::ApiBaseUrl;

use super::{
    LoadBalancerActionId, LoadBalancerAlgorithm, LoadBalancerId, LoadBalancerIp,
    LoadBalancerNetworkId, LoadBalancerPort, LoadBalancerRequestError, LoadBalancerService,
    LoadBalancerServiceUpdate, LoadBalancerType,
};

/// Load Balancer action endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadBalancerActionEndpoint {
    /// `GET /load_balancers/actions`.
    ListAll,
    /// `GET /load_balancers/actions/{id}`.
    Get(LoadBalancerActionId),
    /// `GET /load_balancers/{id}/actions`.
    ListForLoadBalancer(LoadBalancerId),
    /// Add a service.
    AddService(LoadBalancerId),
    /// Add a target.
    AddTarget(LoadBalancerId),
    /// Attach to a network.
    AttachToNetwork(LoadBalancerId),
    /// Change the balancing algorithm.
    ChangeAlgorithm(LoadBalancerId),
    /// Change reverse DNS.
    ChangeDnsPtr(LoadBalancerId),
    /// Change deletion protection.
    ChangeProtection(LoadBalancerId),
    /// Change Load Balancer type.
    ChangeType(LoadBalancerId),
    /// Delete a service.
    DeleteService(LoadBalancerId),
    /// Detach from a network.
    DetachFromNetwork(LoadBalancerId),
    /// Disable the public interface.
    DisablePublicInterface(LoadBalancerId),
    /// Enable the public interface.
    EnablePublicInterface(LoadBalancerId),
    /// Remove a target.
    RemoveTarget(LoadBalancerId),
    /// Update a service.
    UpdateService(LoadBalancerId),
}

impl LoadBalancerActionEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::ListAll | Self::Get(_) | Self::ListForLoadBalancer(_) => Method::Get,
            _ => Method::Post,
        }
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::LoadBalancerActions
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, LoadBalancerRequestError> {
        let len = match self {
            Self::ListAll => {
                crate::cloud::shared::write_static_path(output, "/load_balancers/actions")?
            }
            Self::Get(id) => write_id_path(output, "/load_balancers/actions/", id, "")?,
            Self::ListForLoadBalancer(id) => {
                write_id_path(output, "/load_balancers/", id, "/actions")?
            }
            Self::AddService(id) => action_path(output, id, "add_service")?,
            Self::AddTarget(id) => action_path(output, id, "add_target")?,
            Self::AttachToNetwork(id) => action_path(output, id, "attach_to_network")?,
            Self::ChangeAlgorithm(id) => action_path(output, id, "change_algorithm")?,
            Self::ChangeDnsPtr(id) => action_path(output, id, "change_dns_ptr")?,
            Self::ChangeProtection(id) => action_path(output, id, "change_protection")?,
            Self::ChangeType(id) => action_path(output, id, "change_type")?,
            Self::DeleteService(id) => action_path(output, id, "delete_service")?,
            Self::DetachFromNetwork(id) => action_path(output, id, "detach_from_network")?,
            Self::DisablePublicInterface(id) => {
                action_path(output, id, "disable_public_interface")?
            }
            Self::EnablePublicInterface(id) => action_path(output, id, "enable_public_interface")?,
            Self::RemoveTarget(id) => action_path(output, id, "remove_target")?,
            Self::UpdateService(id) => action_path(output, id, "update_service")?,
        };
        Ok(len)
    }
}

/// Add-service action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerAddServiceRequest<'a>(LoadBalancerService<'a>);

impl<'a> LoadBalancerAddServiceRequest<'a> {
    /// Creates an add-service body from a complete service.
    #[must_use]
    pub const fn new(service: LoadBalancerService<'a>) -> Self {
        Self(service)
    }
    /// Returns the service.
    #[must_use]
    pub const fn service(self) -> LoadBalancerService<'a> {
        self.0
    }
}

/// Update-service action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerUpdateServiceRequest<'a>(LoadBalancerServiceUpdate<'a>);

impl<'a> LoadBalancerUpdateServiceRequest<'a> {
    /// Creates an update-service body.
    #[must_use]
    pub const fn new(update: LoadBalancerServiceUpdate<'a>) -> Self {
        Self(update)
    }
    /// Returns the update.
    #[must_use]
    pub const fn update(self) -> LoadBalancerServiceUpdate<'a> {
        self.0
    }
}

/// Delete-service action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerDeleteServiceRequest(LoadBalancerPort);

impl LoadBalancerDeleteServiceRequest {
    /// Selects a service by its listening port.
    #[must_use]
    pub const fn new(listen_port: LoadBalancerPort) -> Self {
        Self(listen_port)
    }
    /// Returns the listening port.
    #[must_use]
    pub const fn listen_port(self) -> LoadBalancerPort {
        self.0
    }
}

/// Optional address selection when attaching to a network.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadBalancerNetworkAddress<'a> {
    /// Request a specific address.
    Ip(LoadBalancerIp<'a>),
    /// Request automatic assignment in a specific subnet.
    IpRange(SubnetIpRange<'a>),
    /// Request a specific address constrained to a subnet.
    IpInRange {
        /// Requested address.
        ip: LoadBalancerIp<'a>,
        /// Requested subnet.
        ip_range: SubnetIpRange<'a>,
    },
}

/// Attach-to-network action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerAttachNetworkRequest<'a> {
    network: LoadBalancerNetworkId,
    address: Option<LoadBalancerNetworkAddress<'a>>,
}

impl<'a> LoadBalancerAttachNetworkRequest<'a> {
    /// Creates an attachment with automatic address assignment.
    #[must_use]
    pub const fn new(network: LoadBalancerNetworkId) -> Self {
        Self {
            network,
            address: None,
        }
    }

    /// Sets one coherent address-selection mode.
    #[must_use]
    pub const fn with_address(mut self, address: LoadBalancerNetworkAddress<'a>) -> Self {
        self.address = Some(address);
        self
    }

    /// Returns the network ID.
    #[must_use]
    pub const fn network(self) -> LoadBalancerNetworkId {
        self.network
    }
    /// Returns address selection when supplied.
    #[must_use]
    pub const fn address(self) -> Option<LoadBalancerNetworkAddress<'a>> {
        self.address
    }
}

/// Detach-from-network action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerDetachNetworkRequest(LoadBalancerNetworkId);

impl LoadBalancerDetachNetworkRequest {
    /// Creates a detachment request.
    #[must_use]
    pub const fn new(network: LoadBalancerNetworkId) -> Self {
        Self(network)
    }
    /// Returns the network ID.
    #[must_use]
    pub const fn network(self) -> LoadBalancerNetworkId {
        self.0
    }
}

/// Reverse-DNS pointer value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerDnsPtr<'a>(&'a str);

impl<'a> LoadBalancerDnsPtr<'a> {
    /// Validates a conservative ASCII DNS hostname.
    pub fn new(value: &'a str) -> Result<Self, LoadBalancerRequestError> {
        if value.is_empty()
            || value.len() > 253
            || value.starts_with('.')
            || value.ends_with('.')
            || value.split('.').any(invalid_dns_label)
        {
            return Err(LoadBalancerRequestError::InvalidText);
        }
        Ok(Self(value))
    }

    /// Returns the pointer value.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// Explicit reverse-DNS mutation intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadBalancerDnsPtrIntent<'a> {
    /// Set a pointer value.
    Set(LoadBalancerDnsPtr<'a>),
    /// Emit JSON null to reset/remove the pointer.
    Reset,
}

/// Change-DNS-pointer action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerChangeDnsPtrRequest<'a> {
    ip: LoadBalancerIp<'a>,
    dns_ptr: LoadBalancerDnsPtrIntent<'a>,
}

impl<'a> LoadBalancerChangeDnsPtrRequest<'a> {
    /// Creates a request requiring explicit set or reset intent.
    pub fn try_new(
        ip: LoadBalancerIp<'a>,
        dns_ptr: Option<LoadBalancerDnsPtrIntent<'a>>,
    ) -> Result<Self, LoadBalancerRequestError> {
        Ok(Self {
            ip,
            dns_ptr: dns_ptr.ok_or(LoadBalancerRequestError::MissingDnsPtrIntent)?,
        })
    }
    /// Returns the address.
    #[must_use]
    pub const fn ip(self) -> LoadBalancerIp<'a> {
        self.ip
    }
    /// Returns explicit pointer intent.
    #[must_use]
    pub const fn dns_ptr(self) -> LoadBalancerDnsPtrIntent<'a> {
        self.dns_ptr
    }
}

/// Change-protection action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerProtectionRequest(bool);

impl LoadBalancerProtectionRequest {
    /// Creates an explicit delete-protection request.
    #[must_use]
    pub const fn new(delete: bool) -> Self {
        Self(delete)
    }
    /// Returns delete protection.
    #[must_use]
    pub const fn delete(self) -> bool {
        self.0
    }
}

/// Change-type action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerChangeTypeRequest<'a>(LoadBalancerType<'a>);

impl<'a> LoadBalancerChangeTypeRequest<'a> {
    /// Creates a type-change request.
    #[must_use]
    pub const fn new(load_balancer_type: LoadBalancerType<'a>) -> Self {
        Self(load_balancer_type)
    }
    /// Returns the type reference.
    #[must_use]
    pub const fn load_balancer_type(self) -> LoadBalancerType<'a> {
        self.0
    }
}

/// Change-algorithm action body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerChangeAlgorithmRequest(LoadBalancerAlgorithm);

impl LoadBalancerChangeAlgorithmRequest {
    /// Creates an algorithm-change request.
    #[must_use]
    pub const fn new(algorithm: LoadBalancerAlgorithm) -> Self {
        Self(algorithm)
    }
    /// Returns the algorithm.
    #[must_use]
    pub const fn algorithm(self) -> LoadBalancerAlgorithm {
        self.0
    }
}

fn action_path(
    output: &mut [u8],
    id: LoadBalancerId,
    action: &str,
) -> Result<usize, crate::cloud::shared::CloudRequestError> {
    let suffix = match action {
        "add_service" => "/actions/add_service",
        "add_target" => "/actions/add_target",
        "attach_to_network" => "/actions/attach_to_network",
        "change_algorithm" => "/actions/change_algorithm",
        "change_dns_ptr" => "/actions/change_dns_ptr",
        "change_protection" => "/actions/change_protection",
        "change_type" => "/actions/change_type",
        "delete_service" => "/actions/delete_service",
        "detach_from_network" => "/actions/detach_from_network",
        "disable_public_interface" => "/actions/disable_public_interface",
        "enable_public_interface" => "/actions/enable_public_interface",
        "remove_target" => "/actions/remove_target",
        "update_service" => "/actions/update_service",
        _ => return Err(crate::cloud::shared::CloudRequestError::InvalidType),
    };
    write_id_path(output, "/load_balancers/", id, suffix)
}

fn invalid_dns_label(label: &str) -> bool {
    label.is_empty()
        || label.len() > 63
        || label.starts_with('-')
        || label.ends_with('-')
        || label
            .bytes()
            .any(|byte| !(byte.is_ascii_alphanumeric() || byte == b'-'))
}
