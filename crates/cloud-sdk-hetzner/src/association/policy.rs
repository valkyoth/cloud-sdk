//! Sealed association traits and policy marker types.

use cloud_sdk::operation::OperationId;
use cloud_sdk::transport::StatusCode;
use cloud_sdk::{Method, ServiceId, ServiceMarker};

use crate::identity::Hetzner;
use crate::request::ApiBaseUrl;

/// Maximum success or error JSON body admitted by prepared Hetzner requests.
pub const MAX_ASSOCIATED_JSON_BYTES: usize = 8_388_608;

/// Authentication mechanism required by an associated operation.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AuthenticationClass {
    /// Hetzner Cloud, DNS, security, and Storage management bearer token.
    Bearer,
    /// Reserved for future source-reviewed Basic authentication operations.
    Basic,
}

/// Query presence admitted by an operation.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum QueryPolicy {
    /// No query is admitted.
    Forbidden,
    /// The query may be omitted.
    Optional,
    /// A query is required.
    Required,
}

/// Request-body shape admitted by an operation.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BodyPolicy {
    /// No request body or content type is admitted.
    Forbidden,
    /// One JSON request body and JSON content type are required.
    RequiredJson,
}

/// Pagination strategy associated with an operation.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PaginationPolicy {
    /// The operation is not source-locked as paginated.
    None,
    /// Hetzner one-based numbered pagination is used.
    Numbered,
}

/// Retry strategy admitted by operation semantics.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RetryPolicy {
    /// Automatic repetition is forbidden.
    Never,
    /// A caller-owned explicit policy may retry.
    Explicit,
}

/// Required execution authorization class.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PermitClass {
    /// A read-only operation needs no mutation permit.
    None,
    /// A state-changing operation requires mutation intent.
    Mutation,
    /// A destructive operation requires destructive intent.
    Destructive,
    /// A potentially billed operation requires explicit cost intent.
    Cost,
}

/// Source-locked successful response family.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ResponseShape {
    /// No success body.
    Empty,
    /// One action envelope.
    Action,
    /// Multiple actions without pagination metadata.
    Actions,
    /// Paginated actions.
    ActionsPage,
    /// One resource envelope.
    Resource,
    /// Multiple resources without pagination metadata.
    ResourceList,
    /// Paginated resources.
    ResourcePage,
    /// A multi-resource response.
    Composite,
    /// Metrics data.
    Metrics,
    /// A zonefile payload.
    ZoneFile,
    /// Pricing data.
    Pricing,
    /// Storage folder data.
    Folders,
}

/// Complete inspectable association for one operation marker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperationDescriptor {
    operation_id: OperationId,
    service_id: ServiceId,
    api_base_url: ApiBaseUrl,
    authentication: AuthenticationClass,
    method: Method,
    query: QueryPolicy,
    body: BodyPolicy,
    success_status: StatusCode,
    response_shape: ResponseShape,
    success_body_bytes: usize,
    error_body_bytes: usize,
    pagination: PaginationPolicy,
    retry: RetryPolicy,
    permit: PermitClass,
}

impl OperationDescriptor {
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        operation_id: OperationId,
        service_id: ServiceId,
        api_base_url: ApiBaseUrl,
        authentication: AuthenticationClass,
        method: Method,
        query: QueryPolicy,
        body: BodyPolicy,
        success_status: StatusCode,
        response_shape: ResponseShape,
        pagination: PaginationPolicy,
        retry: RetryPolicy,
        permit: PermitClass,
    ) -> Self {
        let success_body_bytes = if matches!(response_shape, ResponseShape::Empty) {
            0
        } else {
            MAX_ASSOCIATED_JSON_BYTES
        };
        Self {
            operation_id,
            service_id,
            api_base_url,
            authentication,
            method,
            query,
            body,
            success_status,
            response_shape,
            success_body_bytes,
            error_body_bytes: MAX_ASSOCIATED_JSON_BYTES,
            pagination,
            retry,
            permit,
        }
    }

    /// Returns the provider operation identifier.
    #[must_use]
    pub const fn operation_id(self) -> OperationId {
        self.operation_id
    }
    /// Returns the provider-owned service identifier.
    #[must_use]
    pub const fn service_id(self) -> ServiceId {
        self.service_id
    }
    /// Returns the exact official API authority and base-path family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        self.api_base_url
    }
    /// Returns the required authentication mechanism.
    #[must_use]
    pub const fn authentication(self) -> AuthenticationClass {
        self.authentication
    }
    /// Returns the exact HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        self.method
    }
    /// Returns the query-presence policy.
    #[must_use]
    pub const fn query_policy(self) -> QueryPolicy {
        self.query
    }
    /// Returns the request-body and request-media policy.
    #[must_use]
    pub const fn body_policy(self) -> BodyPolicy {
        self.body
    }
    /// Returns the sole admitted success status.
    #[must_use]
    pub const fn success_status(self) -> StatusCode {
        self.success_status
    }
    /// Returns the successful response model family.
    #[must_use]
    pub const fn response_shape(self) -> ResponseShape {
        self.response_shape
    }
    /// Returns the successful response-body cap.
    #[must_use]
    pub const fn success_body_bytes(self) -> usize {
        self.success_body_bytes
    }
    /// Returns the provider-error response-body cap.
    #[must_use]
    pub const fn error_body_bytes(self) -> usize {
        self.error_body_bytes
    }
    /// Returns the pagination strategy.
    #[must_use]
    pub const fn pagination(self) -> PaginationPolicy {
        self.pagination
    }
    /// Returns the retry strategy.
    #[must_use]
    pub const fn retry(self) -> RetryPolicy {
        self.retry
    }
    /// Returns the required execution permit class.
    #[must_use]
    pub const fn permit(self) -> PermitClass {
        self.permit
    }
}

mod private {
    pub trait Sealed {}
}

/// Sealed operation association implemented by all active Hetzner operations.
pub trait HetznerOperation: private::Sealed + 'static {
    /// Provider-owned service marker.
    type Service: ServiceMarker<Provider = Hetzner>;
    /// Fixed endpoint-policy marker.
    type EndpointPolicy;
    /// Authentication-class marker.
    type Authentication;
    /// Authentication-scope marker.
    type AuthenticationScope;
    /// Query-presence marker.
    type Query;
    /// Request-body marker.
    type Body;
    /// Request-header marker.
    type RequestHeaders;
    /// Request-media marker.
    type RequestMedia;
    /// Success-status marker.
    type SuccessStatus;
    /// Success-body marker.
    type SuccessBody;
    /// Success-media marker.
    type SuccessMedia;
    /// Provider-error body marker.
    type ErrorBody;
    /// Provider-error media marker.
    type ErrorMedia;
    /// Response-cap marker.
    type ResponseCaps;
    /// Pagination marker.
    type Pagination;
    /// Quota marker.
    type Quota;
    /// Retry marker.
    type Retry;
    /// Streaming-mode marker.
    type Streaming;
    /// Successful provider response family marker.
    type Success;
    /// Provider error response marker.
    type Error;
    /// Required permit marker.
    type Permit;

    /// Complete source-locked operation association.
    const DESCRIPTOR: OperationDescriptor;
}

/// Alias emphasizing that [`HetznerOperation`] is an operation association.
pub trait OperationAssociation: HetznerOperation {}
impl<T: HetznerOperation> OperationAssociation for T {}

/// Sealed association for operations that require no execution permit.
pub trait ReadOnlyOperation: HetznerOperation {}

pub(crate) use private::Sealed;
