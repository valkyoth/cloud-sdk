//! Resource identity and list models.

use alloc::string::String;
use alloc::vec::Vec;

use serde_json::Value;

use super::{ResponseModelError, checked_text, object, required};
use crate::pagination::{Page, PaginationMetadata, PerPage};

const MAX_RESOURCES: usize = 1024;
const MAX_RESOURCE_TEXT_BYTES: usize = 1024;

/// Hetzner resource family represented by a response.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResourceKind {
    /// Certificate.
    Certificate,
    /// Firewall.
    Firewall,
    /// Floating IP.
    FloatingIp,
    /// Image.
    Image,
    /// ISO image.
    Iso,
    /// Load balancer.
    LoadBalancer,
    /// Load-balancer type.
    LoadBalancerType,
    /// Location.
    Location,
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
    /// SSH key.
    SshKey,
    /// Volume.
    Volume,
    /// DNS zone.
    Zone,
    /// DNS RRSet.
    Rrset,
    /// Storage Box.
    StorageBox,
    /// Storage Box subaccount.
    StorageBoxSubaccount,
    /// Storage Box snapshot.
    StorageBoxSnapshot,
    /// Storage Box type.
    StorageBoxType,
}

impl ResourceKind {
    pub(super) fn from_root(root: &str) -> Option<Self> {
        match root {
            "certificate" | "certificates" => Some(Self::Certificate),
            "firewall" | "firewalls" => Some(Self::Firewall),
            "floating_ip" | "floating_ips" => Some(Self::FloatingIp),
            "image" | "images" => Some(Self::Image),
            "iso" | "isos" => Some(Self::Iso),
            "load_balancer" | "load_balancers" => Some(Self::LoadBalancer),
            "load_balancer_type" | "load_balancer_types" => Some(Self::LoadBalancerType),
            "location" | "locations" => Some(Self::Location),
            "network" | "networks" => Some(Self::Network),
            "placement_group" | "placement_groups" => Some(Self::PlacementGroup),
            "primary_ip" | "primary_ips" => Some(Self::PrimaryIp),
            "server" | "servers" => Some(Self::Server),
            "server_type" | "server_types" => Some(Self::ServerType),
            "ssh_key" | "ssh_keys" => Some(Self::SshKey),
            "volume" | "volumes" => Some(Self::Volume),
            "zone" | "zones" => Some(Self::Zone),
            "rrset" | "rrsets" => Some(Self::Rrset),
            "storage_box" | "storage_boxes" => Some(Self::StorageBox),
            "subaccount" | "subaccounts" => Some(Self::StorageBoxSubaccount),
            "snapshot" | "snapshots" => Some(Self::StorageBoxSnapshot),
            "storage_box_type" | "storage_box_types" => Some(Self::StorageBoxType),
            _ => None,
        }
    }
}

/// Validated provider resource identifier.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResourceIdentifier {
    /// Positive integer identifier.
    Integer(u64),
    /// Bounded textual identifier, used by resources such as RRSets.
    Text(String),
}

/// Validated resource identity and common state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resource {
    kind: ResourceKind,
    id: ResourceIdentifier,
    name: Option<String>,
    status: Option<String>,
}

impl Resource {
    /// Returns the resource family.
    #[must_use]
    pub const fn kind(&self) -> ResourceKind {
        self.kind
    }

    /// Returns the validated resource identifier.
    #[must_use]
    pub const fn id(&self) -> &ResourceIdentifier {
        &self.id
    }

    /// Returns the validated resource name when supplied by the schema.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the validated source-known status when supplied.
    #[must_use]
    pub fn status(&self) -> Option<&str> {
        self.status.as_deref()
    }
}

pub(crate) fn parse_resource(root: &str, value: &Value) -> Result<Resource, ResponseModelError> {
    let kind = ResourceKind::from_root(root).ok_or(ResponseModelError::EnvelopeMismatch)?;
    let fields = object(value)?;
    let id = parse_id(kind, required(fields, "id")?)?;
    let name = fields
        .get("name")
        .map(|value| {
            value
                .as_str()
                .ok_or(ResponseModelError::WrongType)
                .and_then(|text| checked_text(text, MAX_RESOURCE_TEXT_BYTES))
        })
        .transpose()?;
    let status = fields
        .get("status")
        .map(|value| parse_status(kind, value))
        .transpose()?;
    Ok(Resource {
        kind,
        id,
        name,
        status,
    })
}

pub(crate) fn parse_resources(
    root: &str,
    value: &Value,
) -> Result<Vec<Resource>, ResponseModelError> {
    let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
    if values.len() > MAX_RESOURCES {
        return Err(ResponseModelError::TooManyItems);
    }
    values
        .iter()
        .map(|value| parse_resource(root, value))
        .collect()
}

pub(crate) fn parse_pagination(value: &Value) -> Result<PaginationMetadata, ResponseModelError> {
    let meta = object(value)?;
    let pagination = object(required(meta, "pagination")?)?;
    let page = required_u64(pagination, "page")
        .and_then(|value| Page::new(value).map_err(|_| ResponseModelError::InvalidPagination))?;
    let per_page = required_u64(pagination, "per_page")
        .and_then(|value| u16::try_from(value).map_err(|_| ResponseModelError::InvalidPagination))
        .and_then(|value| PerPage::new(value).map_err(|_| ResponseModelError::InvalidPagination))?;
    let previous = optional_page(pagination, "previous_page")?;
    let next = optional_page(pagination, "next_page")?;
    let last = optional_page(pagination, "last_page")?;
    let total = required_nullable_u64(pagination, "total_entries")?;
    PaginationMetadata::new(page, per_page, previous, next, last, total)
        .map_err(|_| ResponseModelError::InvalidPagination)
}

fn parse_id(kind: ResourceKind, value: &Value) -> Result<ResourceIdentifier, ResponseModelError> {
    if kind == ResourceKind::Rrset {
        let value = value.as_str().ok_or(ResponseModelError::WrongType)?;
        return checked_text(value, MAX_RESOURCE_TEXT_BYTES)
            .map(ResourceIdentifier::Text)
            .map_err(|_| ResponseModelError::InvalidIdentifier);
    }
    if let Some(value) = value.as_u64() {
        return (value != 0)
            .then_some(ResourceIdentifier::Integer(value))
            .ok_or(ResponseModelError::InvalidIdentifier);
    }
    Err(ResponseModelError::WrongType)
}

fn parse_status(kind: ResourceKind, value: &Value) -> Result<String, ResponseModelError> {
    let value = value.as_str().ok_or(ResponseModelError::WrongType)?;
    let known = match kind {
        ResourceKind::Server => matches!(
            value,
            "running"
                | "initializing"
                | "starting"
                | "stopping"
                | "off"
                | "deleting"
                | "migrating"
                | "rebuilding"
                | "unknown"
        ),
        ResourceKind::Image => matches!(value, "available" | "creating" | "unavailable"),
        ResourceKind::Volume => matches!(value, "available" | "creating"),
        ResourceKind::Zone => matches!(value, "ok" | "updating" | "error"),
        ResourceKind::StorageBox => matches!(value, "active" | "initializing" | "locked"),
        _ => true,
    };
    if !known {
        return Err(ResponseModelError::UnknownEnumValue);
    }
    checked_text(value, MAX_RESOURCE_TEXT_BYTES)
}

fn required_u64(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<u64, ResponseModelError> {
    required(object, key)?
        .as_u64()
        .ok_or(ResponseModelError::InvalidPagination)
}

fn required_nullable_u64(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<u64>, ResponseModelError> {
    let value = required(object, key)?;
    if value.is_null() {
        Ok(None)
    } else {
        value
            .as_u64()
            .map(Some)
            .ok_or(ResponseModelError::InvalidPagination)
    }
}

fn optional_page(
    object: &serde_json::Map<String, Value>,
    key: &str,
) -> Result<Option<Page>, ResponseModelError> {
    required_nullable_u64(object, key)?
        .map(|value| Page::new(value).map_err(|_| ResponseModelError::InvalidPagination))
        .transpose()
}
