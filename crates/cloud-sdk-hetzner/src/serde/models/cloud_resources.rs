//! Dedicated ordinary Hetzner Cloud resource models.

use alloc::vec::Vec;
use core::fmt;

use crate::serde::strict_json::Value;

use super::cloud_schema::validate_model;
use super::{CloudObject, ResponseModelError};

macro_rules! cloud_model {
    ($name:ident, $model:literal) => {
        #[doc = concat!("Source-complete `", $model, "` response model.")]
        #[derive(PartialEq)]
        pub struct $name {
            id: u64,
            fields: CloudObject,
        }

        impl $name {
            /// Fallibly copies this resource and its complete field tree.
            pub fn try_clone(&self) -> Result<Self, ResponseModelError> {
                Ok(Self {
                    id: self.id,
                    fields: self.fields.try_clone()?,
                })
            }

            /// Returns every known and future source field in stable order.
            #[must_use]
            pub const fn fields(&self) -> &CloudObject {
                &self.fields
            }

            /// Returns the positive provider identifier.
            #[must_use]
            pub const fn id(&self) -> u64 {
                self.id
            }

            /// Returns the provider name when this model carries one and it is non-null.
            #[must_use]
            pub fn name(&self) -> Option<&str> {
                self.fields.text("name")
            }

            fn parse(value: &Value) -> Result<Self, ResponseModelError> {
                validate_model($model, value)?;
                let fields = CloudObject::from_value(value)?;
                let id = fields
                    .u64("id")
                    .ok_or(ResponseModelError::InvalidIdentifier)?;
                if id == 0 {
                    return Err(ResponseModelError::InvalidIdentifier);
                }
                Ok(Self { id, fields })
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct(stringify!($name))
                    .field("id", &"[redacted]")
                    .field("fields", &"[redacted]")
                    .finish()
            }
        }
    };
}

cloud_model!(Firewall, "firewall");
cloud_model!(FloatingIp, "floating_ip");
cloud_model!(Image, "image");
cloud_model!(Iso, "iso");
cloud_model!(LoadBalancer, "load_balancer");
cloud_model!(LoadBalancerType, "load_balancer_type");
cloud_model!(Network, "network");
cloud_model!(NetworkMember, "network_member");
cloud_model!(PlacementGroup, "placement_group");
cloud_model!(PrimaryIp, "primary_ip");
cloud_model!(Server, "server");
cloud_model!(ServerType, "server_type");
cloud_model!(Volume, "volume");

/// Ordinary Cloud resource family with a source-complete model in v0.63.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum CloudResourceKind {
    /// Firewall.
    Firewall,
    /// Floating IP.
    FloatingIp,
    /// Image.
    Image,
    /// ISO.
    Iso,
    /// Load balancer.
    LoadBalancer,
    /// Load-balancer type.
    LoadBalancerType,
    /// Network.
    Network,
    /// Placement group.
    PlacementGroup,
    /// Primary IP.
    PrimaryIp,
    /// Server.
    Server,
    /// Server type.
    ServerType,
    /// Volume.
    Volume,
    /// Resource attached to a Network (appended to preserve existing discriminants).
    NetworkMember,
}

/// Dedicated source-complete ordinary Cloud resource.
#[derive(PartialEq)]
#[non_exhaustive]
pub enum CloudResource {
    /// Firewall.
    Firewall(Firewall),
    /// Floating IP.
    FloatingIp(FloatingIp),
    /// Image.
    Image(Image),
    /// ISO.
    Iso(Iso),
    /// Load balancer.
    LoadBalancer(LoadBalancer),
    /// Load-balancer type.
    LoadBalancerType(LoadBalancerType),
    /// Network.
    Network(Network),
    /// Resource attached to a Network.
    NetworkMember(NetworkMember),
    /// Placement group.
    PlacementGroup(PlacementGroup),
    /// Primary IP.
    PrimaryIp(PrimaryIp),
    /// Server.
    Server(Server),
    /// Server type.
    ServerType(ServerType),
    /// Volume.
    Volume(Volume),
}

impl CloudResource {
    /// Fallibly copies this resource and its complete field tree.
    pub fn try_clone(&self) -> Result<Self, ResponseModelError> {
        match self {
            Self::Firewall(value) => value.try_clone().map(Self::Firewall),
            Self::FloatingIp(value) => value.try_clone().map(Self::FloatingIp),
            Self::Image(value) => value.try_clone().map(Self::Image),
            Self::Iso(value) => value.try_clone().map(Self::Iso),
            Self::LoadBalancer(value) => value.try_clone().map(Self::LoadBalancer),
            Self::LoadBalancerType(value) => value.try_clone().map(Self::LoadBalancerType),
            Self::Network(value) => value.try_clone().map(Self::Network),
            Self::NetworkMember(value) => value.try_clone().map(Self::NetworkMember),
            Self::PlacementGroup(value) => value.try_clone().map(Self::PlacementGroup),
            Self::PrimaryIp(value) => value.try_clone().map(Self::PrimaryIp),
            Self::Server(value) => value.try_clone().map(Self::Server),
            Self::ServerType(value) => value.try_clone().map(Self::ServerType),
            Self::Volume(value) => value.try_clone().map(Self::Volume),
        }
    }

    /// Returns the exact resource family.
    #[must_use]
    pub const fn kind(&self) -> CloudResourceKind {
        match self {
            Self::Firewall(_) => CloudResourceKind::Firewall,
            Self::FloatingIp(_) => CloudResourceKind::FloatingIp,
            Self::Image(_) => CloudResourceKind::Image,
            Self::Iso(_) => CloudResourceKind::Iso,
            Self::LoadBalancer(_) => CloudResourceKind::LoadBalancer,
            Self::LoadBalancerType(_) => CloudResourceKind::LoadBalancerType,
            Self::Network(_) => CloudResourceKind::Network,
            Self::NetworkMember(_) => CloudResourceKind::NetworkMember,
            Self::PlacementGroup(_) => CloudResourceKind::PlacementGroup,
            Self::PrimaryIp(_) => CloudResourceKind::PrimaryIp,
            Self::Server(_) => CloudResourceKind::Server,
            Self::ServerType(_) => CloudResourceKind::ServerType,
            Self::Volume(_) => CloudResourceKind::Volume,
        }
    }

    /// Returns every retained field without erasing the resource family.
    #[must_use]
    pub const fn fields(&self) -> &CloudObject {
        match self {
            Self::Firewall(value) => value.fields(),
            Self::FloatingIp(value) => value.fields(),
            Self::Image(value) => value.fields(),
            Self::Iso(value) => value.fields(),
            Self::LoadBalancer(value) => value.fields(),
            Self::LoadBalancerType(value) => value.fields(),
            Self::Network(value) => value.fields(),
            Self::NetworkMember(value) => value.fields(),
            Self::PlacementGroup(value) => value.fields(),
            Self::PrimaryIp(value) => value.fields(),
            Self::Server(value) => value.fields(),
            Self::ServerType(value) => value.fields(),
            Self::Volume(value) => value.fields(),
        }
    }

    /// Returns the positive provider identifier.
    #[must_use]
    pub fn id(&self) -> u64 {
        match self {
            Self::Firewall(value) => value.id(),
            Self::FloatingIp(value) => value.id(),
            Self::Image(value) => value.id(),
            Self::Iso(value) => value.id(),
            Self::LoadBalancer(value) => value.id(),
            Self::LoadBalancerType(value) => value.id(),
            Self::Network(value) => value.id(),
            Self::NetworkMember(value) => value.id(),
            Self::PlacementGroup(value) => value.id(),
            Self::PrimaryIp(value) => value.id(),
            Self::Server(value) => value.id(),
            Self::ServerType(value) => value.id(),
            Self::Volume(value) => value.id(),
        }
    }

    /// Returns the provider name when present and non-null.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.fields().text("name")
    }
}

impl fmt::Debug for CloudResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CloudResource")
            .field("kind", &self.kind())
            .field("id", &"[redacted]")
            .field("fields", &"[redacted]")
            .finish()
    }
}

pub(crate) fn is_cloud_resource_root(root: &str) -> bool {
    model_for_root(root).is_some()
}

pub(crate) fn parse_cloud_resource(
    root: &str,
    value: &Value,
) -> Result<CloudResource, ResponseModelError> {
    match model_for_root(root).ok_or(ResponseModelError::EnvelopeMismatch)? {
        "firewall" => Firewall::parse(value).map(CloudResource::Firewall),
        "floating_ip" => FloatingIp::parse(value).map(CloudResource::FloatingIp),
        "image" => Image::parse(value).map(CloudResource::Image),
        "iso" => Iso::parse(value).map(CloudResource::Iso),
        "load_balancer" => LoadBalancer::parse(value).map(CloudResource::LoadBalancer),
        "load_balancer_type" => LoadBalancerType::parse(value).map(CloudResource::LoadBalancerType),
        "network" => Network::parse(value).map(CloudResource::Network),
        "network_member" => NetworkMember::parse(value).map(CloudResource::NetworkMember),
        "placement_group" => PlacementGroup::parse(value).map(CloudResource::PlacementGroup),
        "primary_ip" => PrimaryIp::parse(value).map(CloudResource::PrimaryIp),
        "server" => Server::parse(value).map(CloudResource::Server),
        "server_type" => ServerType::parse(value).map(CloudResource::ServerType),
        "volume" => Volume::parse(value).map(CloudResource::Volume),
        _ => Err(ResponseModelError::EnvelopeMismatch),
    }
}

pub(crate) fn parse_cloud_resources(
    root: &str,
    value: &Value,
) -> Result<Vec<CloudResource>, ResponseModelError> {
    let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
    if values.len() > 1_024 {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut resources = Vec::new();
    resources
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        resources.push(parse_cloud_resource(root, value)?);
    }
    Ok(resources)
}

fn model_for_root(root: &str) -> Option<&'static str> {
    match root {
        "firewall" | "firewalls" => Some("firewall"),
        "floating_ip" | "floating_ips" => Some("floating_ip"),
        "image" | "images" => Some("image"),
        "iso" | "isos" => Some("iso"),
        "load_balancer" | "load_balancers" => Some("load_balancer"),
        "load_balancer_type" | "load_balancer_types" => Some("load_balancer_type"),
        "network" | "networks" => Some("network"),
        "members" => Some("network_member"),
        "placement_group" | "placement_groups" => Some("placement_group"),
        "primary_ip" | "primary_ips" => Some("primary_ip"),
        "server" | "servers" => Some("server"),
        "server_type" | "server_types" => Some("server_type"),
        "volume" | "volumes" => Some("volume"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "cloud_resource_tests.rs"]
mod tests;
