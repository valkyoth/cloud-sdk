//! Validated owned success-response models.

mod actions;
mod certificate;
pub(crate) mod cloud_constraints;
mod cloud_resources;
mod cloud_schema;
mod cloud_value;
mod dns;
mod location;
mod metrics;
mod resources;
mod result;
mod scalars;
mod security;
mod special;
mod ssh_key;
pub(crate) mod ssh_wire;
mod storage_box;
mod wipe_string;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use cloud_sdk_sanitization::sanitize_string;

use crate::serde::strict_json::{Map, Value};

use wipe_string::WipeString;

pub use actions::{ActionResult, ActionResultError, ActionResultResource};
pub use certificate::{
    Certificate, CertificateError, CertificateIssuanceState, CertificateKind,
    CertificateRenewalState, CertificateStatus, CertificateUse,
};
pub use cloud_resources::{
    CloudResource, CloudResourceKind, Firewall, FloatingIp, Image, Iso, LoadBalancer,
    LoadBalancerType, Network, PlacementGroup, PrimaryIp, Server, ServerType, Volume,
};
pub use cloud_value::{CloudNumber, CloudObject, CloudValue};
pub use dns::{
    AuthoritativeNameservers, DnsRecord, DnsResource, DnsResourceKind, DnsRrset,
    DnsRrsetProtection, DnsRrsetType, DnsTsigAlgorithm, MAX_ZONE_RECORD_COUNT, PrimaryNameserver,
    Zone, ZoneDelegationStatus, ZoneMode, ZoneProtection, ZoneRegistrar, ZoneStatus,
};
pub use location::{Location, LocationPage};
pub use metrics::{MetricPoint, MetricSeries, Metrics};
pub use resources::{Resource, ResourceIdentifier, ResourceKind};
pub use result::{CompositeResult, HetznerSuccess, NamedSensitiveText};
pub use scalars::{ExactDecimal, UtcTimestamp};
pub use security::{SecurityResource, SecurityResourceKind};
pub use special::{FolderList, Pricing, SensitiveText, ZoneFile};
pub use ssh_key::SshKey;
pub use storage_box::{
    AccessSettings, Deprecation, Money, Price, Protection, SnapshotPlan, StorageBox,
    StorageBoxPage, StorageBoxResource, StorageBoxSnapshot, StorageBoxSnapshotReference,
    StorageBoxSnapshotStats, StorageBoxStats, StorageBoxStatus, StorageBoxSubaccount,
    StorageBoxSubaccountAccessSettings, StorageBoxSubaccountReference, StorageBoxType,
    StorageBoxTypePage,
};

pub(crate) use actions::{parse_action, parse_actions};
pub(crate) use certificate::parse_certificate;
pub(crate) use cloud_resources::{
    is_cloud_resource_root, parse_cloud_resource, parse_cloud_resources,
};
pub(crate) use dns::{is_dns_resource_root, parse_dns_resource, parse_dns_resources};
pub(crate) use location::{parse_location, parse_location_page};
pub(crate) use metrics::parse_metrics;
pub(crate) use resources::{parse_pagination, parse_resource, parse_resources};
pub(crate) use scalars::valid_utc_timestamp;
pub(crate) use security::{
    is_security_resource_root, parse_security_resource, parse_security_resources,
};
pub(crate) use special::{parse_folders, parse_pricing, parse_zonefile};
pub(crate) use ssh_key::parse_ssh_key;
pub(crate) use storage_box::{
    parse_storage_box, parse_storage_box_composite_resource, parse_storage_box_page,
    parse_storage_box_snapshot, parse_storage_box_snapshots, parse_storage_box_subaccount,
    parse_storage_box_subaccounts, parse_storage_box_type, parse_storage_box_type_page,
};

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
    /// A typed response contains a different resource identity than requested.
    ResponseIdentityMismatch,
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
    Self::ResponseIdentityMismatch => "Hetzner response identity does not match its request",
);

/// Fallibly constructed, deterministic provider labels.
#[derive(Clone, Eq, PartialEq)]
pub struct Labels(Vec<(String, String)>);

impl Labels {
    fn parse(value: &Value, maximum: usize) -> Result<Self, ResponseModelError> {
        let fields = object(value)?;
        if fields.len() > maximum {
            return Err(ResponseModelError::TooManyItems);
        }
        let mut labels = Self(Vec::new());
        labels
            .0
            .try_reserve_exact(fields.len())
            .map_err(|_| ResponseModelError::Allocation)?;
        for (key, value) in fields.iter() {
            let key = WipeString::new(checked_text(key.as_str(), 128)?);
            let value = WipeString::new(
                value
                    .try_with_str(|value| {
                        if value.is_empty() {
                            Ok(String::new())
                        } else {
                            checked_text(value, 1_024)
                        }
                    })
                    .map_err(|_| ResponseModelError::InvalidText)?
                    .ok_or(ResponseModelError::WrongType)??,
            );
            labels.0.push((key.into_inner(), value.into_inner()));
        }
        labels
            .0
            .sort_unstable_by(|left, right| left.0.cmp(&right.0));
        Ok(labels)
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

impl fmt::Debug for Labels {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Labels")
            .field("count", &self.0.len())
            .field("values", &"[redacted]")
            .finish()
    }
}

impl Drop for Labels {
    fn drop(&mut self) {
        for (key, value) in &mut self.0 {
            sanitize_string(key);
            sanitize_string(value);
        }
    }
}

pub(super) fn parse_labels(value: &Value, maximum: usize) -> Result<Labels, ResponseModelError> {
    Labels::parse(value, maximum)
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

pub(super) fn valid_error_code(value: &str, max: usize) -> bool {
    !value.is_empty()
        && value.len() <= max
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

pub(super) fn is_unsafe_display_character(character: char) -> bool {
    crate::display::is_unsafe_display_character(character)
}

#[cfg(test)]
mod model_tests {
    use super::{ResponseModelError, checked_text, valid_error_code};

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

    #[test]
    fn error_codes_use_a_bounded_ascii_machine_identifier_grammar() {
        for valid in ["forbidden", "future-code.v2", "ERROR_42"] {
            assert!(valid_error_code(valid, 128));
        }
        for invalid in [
            "",
            "has space",
            "line\u{2028}break",
            "soft\u{00ad}hyphen",
            "direction\u{206a}control",
        ] {
            assert!(!valid_error_code(invalid, 128));
        }
        assert!(!valid_error_code(&"a".repeat(129), 128));
    }
}
