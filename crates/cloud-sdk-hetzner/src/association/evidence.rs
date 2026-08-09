//! Compiled evidence for the reviewed operation-binding manifest.

use super::policy::{HetznerOperation, OperationDescriptor};
use super::types::*;
use crate::identity::{CloudService, DnsService, SecurityService, StorageService};

/// Stable manifest label implemented by association marker types.
#[doc(hidden)]
pub trait EvidenceLabel {
    /// Exact value used by the reviewed typed-binding manifest.
    const LABEL: &'static str;
}

macro_rules! evidence_labels {
    ($($marker:ty => $label:literal),+ $(,)?) => {$(
        impl EvidenceLabel for $marker {
            const LABEL: &'static str = $label;
        }
    )+};
}

evidence_labels!(
    CloudService => "cloud",
    DnsService => "dns",
    SecurityService => "security",
    StorageService => "storage",
    CloudEndpointPolicy => "cloud-v1",
    StorageEndpointPolicy => "console-v1",
    BearerAuthentication => "bearer",
    BasicAuthentication => "basic",
    RequiredServiceScope => "required-service",
    QueryForbidden => "forbidden",
    OptionalQuery => "optional",
    RequiredQuery => "required",
    BodyForbidden => "forbidden",
    JsonBody => "json",
    AcceptJson => "accept-json",
    AcceptAndContentTypeJson => "accept-json+content-type-json",
    NoRequestMedia => "forbidden",
    JsonRequestMedia => "application-json",
    StatusOk => "200",
    StatusCreated => "201",
    StatusNoContent => "204",
    JsonSuccessBody => "required-json",
    EmptySuccessBody => "forbidden",
    JsonSuccessMedia => "application-json",
    ForbiddenSuccessMedia => "forbidden",
    JsonErrorBody => "required-json",
    JsonErrorMedia => "application-json",
    NoPagination => "none",
    NumberedPagination => "numbered",
    HetznerQuota => "hetzner",
    ExplicitRetry => "explicit",
    NeverRetry => "never",
    BufferedStreaming => "buffered",
    NoPermit => "none",
    MutationPermit => "mutation",
    DestructivePermit => "destructive",
    CostPermit => "cost",
    EmptyResponse => "empty",
    ActionResponse => "action",
    ActionsResponse => "actions",
    ActionsPageResponse => "actions-page",
    ResourceResponse => "resource",
    ResourceListResponse => "resource-list",
    ResourcePageResponse => "resource-page",
    CompositeResponse => "composite",
    MetricsResponse => "metrics",
    ZoneFileResponse => "zonefile",
    PricingResponse => "pricing",
    FoldersResponse => "folders",
);

/// Marker-derived evidence paired with one compiled operation descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[doc(hidden)]
pub struct OperationBindingEvidence {
    pub descriptor: OperationDescriptor,
    pub service: &'static str,
    pub endpoint_policy: &'static str,
    pub authentication: &'static str,
    pub authentication_scope: &'static str,
    pub query_policy: &'static str,
    pub body_policy: &'static str,
    pub request_headers: &'static str,
    pub request_media: &'static str,
    pub success_status: &'static str,
    pub success_shape: &'static str,
    pub success_body: &'static str,
    pub success_media: &'static str,
    pub error_body: &'static str,
    pub error_media: &'static str,
    pub pagination: &'static str,
    pub quota: &'static str,
    pub retry: &'static str,
    pub streaming: &'static str,
    pub permit_class: &'static str,
}

impl OperationBindingEvidence {
    /// Builds one evidence row entirely from an operation's associated types.
    pub const fn of<O: HetznerOperation>() -> Self
    where
        O::Service: EvidenceLabel,
        O::EndpointPolicy: EvidenceLabel,
        O::Authentication: EvidenceLabel,
        O::AuthenticationScope: EvidenceLabel,
        O::Query: EvidenceLabel,
        O::Body: EvidenceLabel,
        O::RequestHeaders: EvidenceLabel,
        O::RequestMedia: EvidenceLabel,
        O::SuccessStatus: EvidenceLabel,
        O::Success: EvidenceLabel,
        O::SuccessBody: EvidenceLabel,
        O::SuccessMedia: EvidenceLabel,
        O::ErrorBody: EvidenceLabel,
        O::ErrorMedia: EvidenceLabel,
        O::Pagination: EvidenceLabel,
        O::Quota: EvidenceLabel,
        O::Retry: EvidenceLabel,
        O::Streaming: EvidenceLabel,
        O::Permit: EvidenceLabel,
    {
        Self {
            descriptor: O::DESCRIPTOR,
            service: O::Service::LABEL,
            endpoint_policy: O::EndpointPolicy::LABEL,
            authentication: O::Authentication::LABEL,
            authentication_scope: O::AuthenticationScope::LABEL,
            query_policy: O::Query::LABEL,
            body_policy: O::Body::LABEL,
            request_headers: O::RequestHeaders::LABEL,
            request_media: O::RequestMedia::LABEL,
            success_status: O::SuccessStatus::LABEL,
            success_shape: O::Success::LABEL,
            success_body: O::SuccessBody::LABEL,
            success_media: O::SuccessMedia::LABEL,
            error_body: O::ErrorBody::LABEL,
            error_media: O::ErrorMedia::LABEL,
            pagination: O::Pagination::LABEL,
            quota: O::Quota::LABEL,
            retry: O::Retry::LABEL,
            streaming: O::Streaming::LABEL,
            permit_class: O::Permit::LABEL,
        }
    }
}
