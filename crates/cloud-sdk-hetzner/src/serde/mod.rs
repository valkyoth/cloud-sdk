//! Optional no_std Serde boundary.
//!
//! Current public request serialization is limited to checked RRSet body
//! wrappers, so endpoint selectors cannot leak into those JSON bodies and an
//! aggregate size policy is applied before serialization. Checked response
//! decoding binds every active operation to its source-locked status and
//! envelope shape, validates security-relevant fields, and returns typed
//! actions, source-complete ordinary Cloud resources, pagination, special
//! results, or API errors. Unknown future fields and enum strings are retained
//! explicitly after all source-known field, type, bound, and nullability checks.

mod binding;
mod checked;
mod incremental;
mod models;
mod pagination;
mod response;
mod rrsets;
mod strict_json;

pub use checked::{
    CheckedHetznerResponse, HetznerApiError, HetznerDecodeError,
    decode_associated_checked_response, decode_associated_response, decode_response,
    decode_response_at,
};
pub use incremental::{
    IncrementalJsonDecoder, IncrementalJsonError, IncrementalJsonEvent, IncrementalJsonLimits,
    IncrementalJsonLimitsError, IncrementalJsonProgress, IncrementalJsonVisitor, VisitControl,
};
pub use models::{
    AccessSettings, ActionResult, ActionResultError, ActionResultResource,
    AuthoritativeNameservers, Certificate, CertificateError, CertificateIssuanceState,
    CertificateKind, CertificateRenewalState, CertificateStatus, CertificateUse, CloudNumber,
    CloudObject, CloudResource, CloudResourceKind, CloudValue, CompositeResult, Deprecation,
    DnsRecord, DnsResource, DnsResourceKind, DnsRrset, DnsRrsetProtection, DnsRrsetType,
    DnsTsigAlgorithm, ExactDecimal, Firewall, FloatingIp, FolderList, HetznerSuccess, Image, Iso,
    Labels, LoadBalancer, LoadBalancerType, Location, LocationPage, MAX_ZONE_RECORD_COUNT,
    MetricPoint, MetricSeries, Metrics, Money, NamedSensitiveText, Network, PlacementGroup, Price,
    Pricing, PrimaryIp, PrimaryNameserver, Protection, Resource, ResourceIdentifier, ResourceKind,
    ResponseModelError, SecurityResource, SecurityResourceKind, SensitiveText, Server, ServerType,
    SnapshotPlan, SshKey, StorageBox, StorageBoxPage, StorageBoxStats, StorageBoxStatus,
    StorageBoxType, UtcTimestamp, Volume, Zone, ZoneDelegationStatus, ZoneFile, ZoneMode,
    ZoneProtection, ZoneRegistrar, ZoneStatus,
};
pub use pagination::PaginationEnvelope;
pub use response::{
    ActionEnvelope, ActionResource, ActionResponse, ApiErrorEnvelope, ApiErrorResponse,
    MAX_ACTION_RESPONSE_RESOURCES, MAX_API_ERROR_MESSAGE_BYTES, MAX_SERDE_RESPONSE_BYTES,
    ResponseBytes, ResponseSizeError,
};
pub use rrsets::{MAX_RRSET_JSON_BODY_BYTES, RrsetBodyError, RrsetRequestBody};

#[cfg(test)]
mod adversarial_tests;
#[cfg(test)]
mod checked_dns_tests;
#[cfg(test)]
mod checked_dns_zone_security_tests;
#[cfg(test)]
mod checked_fixtures;
#[cfg(test)]
mod checked_pagination_tests;
#[cfg(test)]
mod checked_security_resource_tests;
#[cfg(test)]
mod checked_security_tests;
#[cfg(test)]
mod checked_special_tests;
#[cfg(test)]
mod checked_ssh_key_algorithm_tests;
#[cfg(test)]
mod checked_test_support;
#[cfg(test)]
mod checked_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod vertical_tests;
