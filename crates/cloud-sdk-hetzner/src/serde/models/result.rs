//! Typed checked-decoder success results.

use alloc::vec::Vec;
use core::fmt;

use crate::pagination::PaginationMetadata;

use super::{
    ActionResult, CloudResource, DnsResource, FolderList, Location, LocationPage, Metrics, Pricing,
    Resource, SecurityResource, SensitiveText, StorageBox, StorageBoxPage, StorageBoxResource,
    StorageBoxSnapshot, StorageBoxSubaccount, StorageBoxType, StorageBoxTypePage, ZoneFile,
};

/// Typed successful result returned by the checked decoder.
// Results remain value-owned so allocation failure stays explicit in model parsers.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum HetznerSuccess {
    /// Operation succeeded without a response body.
    Empty,
    /// Source-complete paginated Cloud locations.
    Locations(LocationPage),
    /// One source-complete Cloud location.
    Location(Location),
    /// Source-complete paginated Console Storage Boxes.
    StorageBoxes(StorageBoxPage),
    /// One source-complete Console Storage Box.
    StorageBox(StorageBox),
    /// Source-complete paginated Console Storage Box types.
    StorageBoxTypes(StorageBoxTypePage),
    /// One source-complete Console Storage Box type.
    StorageBoxType(StorageBoxType),
    /// Source-complete Console Storage Box snapshots.
    StorageBoxSnapshots(Vec<StorageBoxSnapshot>),
    /// One source-complete Console Storage Box snapshot.
    StorageBoxSnapshot(StorageBoxSnapshot),
    /// Source-complete Console Storage Box subaccounts.
    StorageBoxSubaccounts(Vec<StorageBoxSubaccount>),
    /// One source-complete Console Storage Box subaccount.
    StorageBoxSubaccount(StorageBoxSubaccount),
    /// One validated action.
    Action(ActionResult),
    /// A bounded action list, optionally with pagination metadata.
    Actions {
        /// Validated actions.
        actions: Vec<ActionResult>,
        /// Pagination supplied by paginated endpoints.
        pagination: Option<PaginationMetadata>,
    },
    /// One provider resource.
    Resource(Resource),
    /// A bounded resource list, optionally with pagination metadata.
    Resources {
        /// Validated resources of one kind.
        resources: Vec<Resource>,
        /// Pagination supplied by paginated endpoints.
        pagination: Option<PaginationMetadata>,
    },
    /// One source-complete ordinary Cloud resource.
    CloudResource(CloudResource),
    /// Source-complete ordinary Cloud resources with optional pagination.
    CloudResources {
        /// Dedicated resource variants.
        resources: Vec<CloudResource>,
        /// Pagination supplied by paginated endpoints.
        pagination: Option<PaginationMetadata>,
    },
    /// One source-complete DNS resource.
    DnsResource(DnsResource),
    /// Source-complete DNS resources with optional pagination.
    DnsResources {
        /// Dedicated DNS resource variants.
        resources: Vec<DnsResource>,
        /// Pagination supplied by paginated endpoints.
        pagination: Option<PaginationMetadata>,
    },
    /// One source-complete security resource.
    SecurityResource(SecurityResource),
    /// Source-complete security resources with optional pagination.
    SecurityResources {
        /// Dedicated certificate or SSH-key variants.
        resources: Vec<SecurityResource>,
        /// Pagination supplied by paginated endpoints.
        pagination: Option<PaginationMetadata>,
    },
    /// A create/action result with optional resource, actions, and secrets.
    Composite(CompositeResult),
    /// Metrics response.
    Metrics(Metrics),
    /// Exported zonefile.
    ZoneFile(ZoneFile),
    /// Pricing summary.
    Pricing(Pricing),
    /// Storage Box folders.
    Folders(FolderList),
}

/// Validated multi-part success response.
pub struct CompositeResult {
    pub(in crate::serde) resource: Option<Resource>,
    pub(in crate::serde) cloud_resource: Option<CloudResource>,
    pub(in crate::serde) dns_resources: Vec<DnsResource>,
    pub(in crate::serde) security_resource: Option<SecurityResource>,
    pub(in crate::serde) storage_box_resource: Option<StorageBoxResource>,
    pub(in crate::serde) action: Option<ActionResult>,
    pub(in crate::serde) actions: Vec<ActionResult>,
    pub(in crate::serde) next_actions: Vec<ActionResult>,
    pub(in crate::serde) secrets: Vec<NamedSensitiveText>,
    pub(in crate::serde) null_secrets: Vec<&'static str>,
}

impl CompositeResult {
    /// Returns the created or changed resource when supplied.
    #[must_use]
    pub const fn resource(&self) -> Option<&Resource> {
        self.resource.as_ref()
    }

    /// Returns the source-complete ordinary Cloud resource when supplied.
    #[must_use]
    pub const fn cloud_resource(&self) -> Option<&CloudResource> {
        self.cloud_resource.as_ref()
    }

    /// Returns the source-complete security resource when supplied.
    #[must_use]
    pub const fn security_resource(&self) -> Option<&SecurityResource> {
        self.security_resource.as_ref()
    }

    /// Returns the source-locked Console resource when supplied.
    #[must_use]
    pub const fn storage_box_resource(&self) -> Option<&StorageBoxResource> {
        self.storage_box_resource.as_ref()
    }

    /// Returns the source-complete DNS resource when supplied.
    #[must_use]
    pub fn dns_resource(&self) -> Option<&DnsResource> {
        self.dns_resources.first()
    }

    /// Returns the singular action supplied by the operation.
    #[must_use]
    pub const fn action(&self) -> Option<&ActionResult> {
        self.action.as_ref()
    }

    /// Returns the source `actions` collection supplied by the operation.
    #[must_use]
    pub fn actions(&self) -> &[ActionResult] {
        &self.actions
    }

    /// Returns the source `next_actions` collection supplied by the operation.
    #[must_use]
    pub fn next_actions(&self) -> &[ActionResult] {
        &self.next_actions
    }

    /// Returns sensitive output fields held in protected owned storage.
    #[must_use]
    pub fn secrets(&self) -> &[NamedSensitiveText] {
        &self.secrets
    }

    /// Looks up a sensitive output while preserving absent, null, and text states.
    #[must_use]
    pub fn secret(&self, name: &str) -> Option<Option<&SensitiveText>> {
        if let Some(secret) = self.secrets.iter().find(|secret| secret.name() == name) {
            return Some(Some(secret.value()));
        }
        self.null_secrets.contains(&name).then_some(None)
    }
}

impl fmt::Debug for CompositeResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompositeResult")
            .field("resource", &self.resource)
            .field(
                "cloud_resource",
                &self.cloud_resource.as_ref().map(|_| "[redacted]"),
            )
            .field("dns_resource", &self.dns_resource().map(|_| "[redacted]"))
            .field(
                "storage_box_resource",
                &self.storage_box_resource.as_ref().map(|_| "[redacted]"),
            )
            .field("action", &self.action)
            .field("action_count", &self.actions.len())
            .field("next_action_count", &self.next_actions.len())
            .field("secrets", &"[redacted]")
            .finish()
    }
}

/// Named sensitive field returned by a provider operation.
#[derive(Eq, PartialEq)]
pub struct NamedSensitiveText {
    name: &'static str,
    value: SensitiveText,
}

impl NamedSensitiveText {
    pub(in crate::serde) fn new(name: &'static str, value: SensitiveText) -> Self {
        Self { name, value }
    }

    /// Returns the source-locked field name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Returns the sensitive field value through an explicit accessor.
    #[must_use]
    pub const fn value(&self) -> &SensitiveText {
        &self.value
    }
}

impl fmt::Debug for NamedSensitiveText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NamedSensitiveText")
            .field("name", &self.name)
            .field("value", &"[redacted]")
            .finish()
    }
}
