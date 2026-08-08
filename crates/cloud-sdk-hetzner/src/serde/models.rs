//! Validated owned success-response models.

mod actions;
mod certificate;
mod cloud_constraints;
mod cloud_resources;
mod cloud_schema;
mod cloud_value;
mod location;
mod metrics;
mod resources;
mod scalars;
mod special;
mod storage_box;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::pagination::PaginationMetadata;
use crate::serde::strict_json::{Map, Value};

pub use actions::{ActionResult, ActionResultError, ActionResultResource};
pub use certificate::{
    Certificate, CertificateError, CertificateKind, CertificateStatus, CertificateUse,
};
pub use cloud_resources::{
    CloudResource, CloudResourceKind, Firewall, FloatingIp, Image, Iso, LoadBalancer,
    LoadBalancerType, Network, PlacementGroup, PrimaryIp, Server, ServerType, Volume,
};
pub use cloud_value::{CloudNumber, CloudObject, CloudValue};
pub use location::{Location, LocationPage};
pub use metrics::{MetricPoint, MetricSeries, Metrics};
pub use resources::{Resource, ResourceIdentifier, ResourceKind};
pub use scalars::{ExactDecimal, UtcTimestamp};
pub use special::{FolderList, Pricing, SensitiveText, ZoneFile};
pub use storage_box::{
    AccessSettings, Deprecation, Money, Price, Protection, SnapshotPlan, StorageBox,
    StorageBoxPage, StorageBoxStats, StorageBoxStatus, StorageBoxType,
};

pub(crate) use actions::{parse_action, parse_actions};
pub(crate) use certificate::parse_certificate;
pub(crate) use cloud_resources::{
    is_cloud_resource_root, parse_cloud_resource, parse_cloud_resources,
};
pub(crate) use location::{parse_location, parse_location_page};
pub(crate) use metrics::parse_metrics;
pub(crate) use resources::{parse_pagination, parse_resource, parse_resources};
pub(crate) use scalars::valid_utc_timestamp;
pub(crate) use special::{parse_folders, parse_pricing, parse_zonefile};
pub(crate) use storage_box::parse_storage_box_page;

/// Failure while validating a parsed success-response model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseModelError {
    /// A source-required field is absent.
    MissingField,
    /// A field has the wrong JSON type.
    WrongType,
    /// A resource or action identifier is invalid.
    InvalidIdentifier,
    /// A bounded text field is empty, too long, or contains unsafe controls.
    InvalidText,
    /// A provider enum contains an unknown value.
    UnknownEnumValue,
    /// A list exceeds its model-specific bound.
    TooManyItems,
    /// Pagination metadata is missing or incoherent.
    InvalidPagination,
    /// The envelope does not match the operation's source-locked response shape.
    EnvelopeMismatch,
    /// A numeric value is outside its source-locked range.
    InvalidNumber,
    /// Memory required for a bounded response model could not be reserved.
    Allocation,
    /// The committed source-derived model schema is malformed or incomplete.
    SchemaMismatch,
}

impl_static_error!(ResponseModelError,
    Self::MissingField => "Hetzner response is missing a required field",
    Self::WrongType => "Hetzner response field has the wrong type",
    Self::InvalidIdentifier => "Hetzner response identifier is invalid",
    Self::InvalidText => "Hetzner response text is invalid",
    Self::UnknownEnumValue => "Hetzner response contains an unknown enum value",
    Self::TooManyItems => "Hetzner response list exceeds its model limit",
    Self::InvalidPagination => "Hetzner response pagination is invalid",
    Self::EnvelopeMismatch => "Hetzner response does not match the operation envelope",
    Self::InvalidNumber => "Hetzner response number is invalid",
    Self::Allocation => "Hetzner response model allocation failed",
    Self::SchemaMismatch => "Hetzner source-derived model schema is invalid",
);

/// Fallibly constructed, deterministic provider labels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Labels(Vec<(String, String)>);

impl Labels {
    fn parse(value: &Value, maximum: usize) -> Result<Self, ResponseModelError> {
        let fields = object(value)?;
        if fields.len() > maximum {
            return Err(ResponseModelError::TooManyItems);
        }
        let mut labels = Vec::new();
        labels
            .try_reserve_exact(fields.len())
            .map_err(|_| ResponseModelError::Allocation)?;
        for (key, value) in fields.iter() {
            let key = checked_text(key.as_str(), 128)?;
            let value = value
                .try_with_str(|value| {
                    if value.is_empty() {
                        Ok(String::new())
                    } else {
                        checked_text(value, 1_024)
                    }
                })
                .map_err(|_| ResponseModelError::InvalidText)?
                .ok_or(ResponseModelError::WrongType)??;
            labels.push((key, value));
        }
        labels.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        Ok(Self(labels))
    }

    /// Returns the value for one exact label key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.0
            .binary_search_by(|(candidate, _)| candidate.as_str().cmp(key))
            .ok()
            .and_then(|index| self.0.get(index))
            .map(|(_, value)| value)
    }

    /// Returns the number of labels.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Reports whether no labels are present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates over labels in stable key order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
    }
}

pub(super) fn parse_labels(value: &Value, maximum: usize) -> Result<Labels, ResponseModelError> {
    Labels::parse(value, maximum)
}

/// Typed successful result returned by the checked decoder.
#[derive(Debug, PartialEq)]
pub enum HetznerSuccess {
    /// Operation succeeded without a response body.
    Empty,
    /// Source-complete paginated Cloud locations.
    Locations(LocationPage),
    /// One source-complete Cloud location.
    Location(Location),
    /// Source-complete certificate output with protected PEM material.
    Certificate(Certificate),
    /// Source-complete paginated Console Storage Boxes.
    StorageBoxes(StorageBoxPage),
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
#[derive(PartialEq)]
pub struct CompositeResult {
    pub(super) resource: Option<Resource>,
    pub(super) cloud_resource: Option<CloudResource>,
    pub(super) action: Option<ActionResult>,
    pub(super) actions: Vec<ActionResult>,
    pub(super) next_actions: Vec<ActionResult>,
    pub(super) secrets: Vec<NamedSensitiveText>,
    pub(super) null_secrets: Vec<&'static str>,
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
    pub(super) fn new(name: &'static str, value: SensitiveText) -> Self {
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

pub(super) fn object(value: &Value) -> Result<&Map, ResponseModelError> {
    value.as_object().ok_or(ResponseModelError::WrongType)
}

pub(super) fn required<'a>(object: &'a Map, key: &str) -> Result<&'a Value, ResponseModelError> {
    object.get(key).ok_or(ResponseModelError::MissingField)
}

pub(super) fn value_text(value: &Value, max: usize) -> Result<String, ResponseModelError> {
    value
        .try_with_str(|value| checked_text(value, max))
        .map_err(|_| ResponseModelError::InvalidText)?
        .ok_or(ResponseModelError::WrongType)?
}

pub(super) fn checked_text(value: &str, max: usize) -> Result<String, ResponseModelError> {
    validate_text(value, max)?;
    let mut output = String::new();
    output
        .try_reserve_exact(value.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    output.push_str(value);
    Ok(output)
}

pub(super) fn validate_text(value: &str, max: usize) -> Result<(), ResponseModelError> {
    if value.is_empty() || value.len() > max || value.chars().any(is_unsafe_display_character) {
        return Err(ResponseModelError::InvalidText);
    }
    Ok(())
}

fn is_unsafe_display_character(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{061c}'
                | '\u{200b}'..='\u{200f}'
                | '\u{202a}'..='\u{202e}'
                | '\u{2060}'..='\u{2069}'
                | '\u{feff}'
        )
}

#[cfg(test)]
mod model_tests {
    use super::{ResponseModelError, checked_text};

    #[test]
    fn checked_text_rejects_unicode_controls_and_invisible_formatting() {
        for value in [
            "line\u{0085}break",
            "right\u{202e}left",
            "zero\u{200b}width",
            "isolate\u{2066}text",
            "mark\u{061c}text",
            "bom\u{feff}text",
        ] {
            assert_eq!(
                checked_text(value, 64),
                Err(ResponseModelError::InvalidText)
            );
        }
        assert_eq!(
            checked_text("visible text", 64).as_deref(),
            Ok("visible text")
        );
    }
}
