//! Validated owned success-response models.

mod actions;
mod resources;
mod special;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::pagination::PaginationMetadata;

pub use actions::{ActionResult, ActionResultError, ActionResultResource};
pub use resources::{Resource, ResourceIdentifier, ResourceKind};
pub use special::{
    FolderList, MetricPoint, MetricSeries, Metrics, Pricing, SensitiveText, ZoneFile,
};

pub(crate) use actions::{parse_action, parse_actions};
pub(crate) use resources::{parse_pagination, parse_resource, parse_resources};
pub(crate) use special::{parse_folders, parse_metrics, parse_pricing, parse_zonefile};

/// Failure while validating a parsed success-response model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseModelError {
    /// A source-required field is absent.
    MissingField,
    /// A field has the wrong JSON type.
    WrongType,
    /// A resource or action identifier is invalid.
    InvalidIdentifier,
    /// A bounded text field is empty, too long, or contains control bytes.
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
);

/// Typed successful result returned by the checked decoder.
#[derive(Clone, Debug, PartialEq)]
pub enum HetznerSuccess {
    /// Operation succeeded without a response body.
    Empty,
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
#[derive(Clone, PartialEq)]
pub struct CompositeResult {
    pub(super) resource: Option<Resource>,
    pub(super) actions: Vec<ActionResult>,
    pub(super) secrets: Vec<NamedSensitiveText>,
}

impl CompositeResult {
    /// Returns the created or changed resource when supplied.
    #[must_use]
    pub const fn resource(&self) -> Option<&Resource> {
        self.resource.as_ref()
    }

    /// Returns actions supplied by the operation.
    #[must_use]
    pub fn actions(&self) -> &[ActionResult] {
        &self.actions
    }

    /// Returns sensitive output fields. Callers must clear the response buffer.
    #[must_use]
    pub fn secrets(&self) -> &[NamedSensitiveText] {
        &self.secrets
    }
}

impl fmt::Debug for CompositeResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompositeResult")
            .field("resource", &self.resource)
            .field("actions", &self.actions)
            .field("secrets", &"[redacted]")
            .finish()
    }
}

/// Named sensitive field returned by a provider operation.
#[derive(Clone, Eq, PartialEq)]
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

pub(super) fn object(
    value: &serde_json::Value,
) -> Result<&serde_json::Map<String, serde_json::Value>, ResponseModelError> {
    value.as_object().ok_or(ResponseModelError::WrongType)
}

pub(super) fn required<'a>(
    object: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<&'a serde_json::Value, ResponseModelError> {
    object.get(key).ok_or(ResponseModelError::MissingField)
}

pub(super) fn checked_text(value: &str, max: usize) -> Result<String, ResponseModelError> {
    if value.is_empty() || value.len() > max || value.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(ResponseModelError::InvalidText);
    }
    Ok(String::from(value))
}
