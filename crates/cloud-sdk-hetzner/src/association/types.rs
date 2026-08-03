//! Zero-sized policy types used by operation associations.

use cloud_sdk::Method;
use cloud_sdk::operation::{CostIntent, OperationImpact, RequestSemantics};
use cloud_sdk::transport::{MAX_RAW_RESPONSE_BODY_BYTES, StatusCode};

use super::policy::{
    AuthenticationClass, BodyPolicy, MAX_ASSOCIATED_JSON_BYTES, PaginationPolicy, PermitClass,
    QueryPolicy, ResponseShape, RetryPolicy,
};
use crate::request::ApiBaseUrl;

macro_rules! marker {
    ($($name:ident),+ $(,)?) => {$(
        #[doc = concat!("Compile-time `", stringify!($name), "` association marker.")]
        #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name;
    )+};
}

marker!(
    CloudEndpointPolicy,
    StorageEndpointPolicy,
    BearerAuthentication,
    BasicAuthentication,
    RequiredServiceScope,
    QueryForbidden,
    OptionalQuery,
    RequiredQuery,
    BodyForbidden,
    JsonBody,
    AcceptJson,
    AcceptAndContentTypeJson,
    NoRequestMedia,
    JsonRequestMedia,
    StatusOk,
    StatusCreated,
    StatusNoContent,
    JsonSuccessBody,
    EmptySuccessBody,
    JsonSuccessMedia,
    ForbiddenSuccessMedia,
    JsonErrorBody,
    JsonErrorMedia,
    JsonResponseCaps,
    NoPagination,
    NumberedPagination,
    HetznerQuota,
    ExplicitRetry,
    NeverRetry,
    BufferedStreaming,
    NoPermit,
    MutationPermit,
    DestructivePermit,
    CostPermit,
    EmptyResponse,
    ActionResponse,
    ActionsResponse,
    ActionsPageResponse,
    ResourceResponse,
    ResourceListResponse,
    ResourcePageResponse,
    CompositeResponse,
    MetricsResponse,
    ZoneFileResponse,
    PricingResponse,
    FoldersResponse,
    HetznerErrorResponse,
    GetMethod,
    PostMethod,
    PutMethod,
    DeleteMethod,
);

pub(crate) trait EndpointAssociation {
    const BASE: ApiBaseUrl;
}
impl EndpointAssociation for CloudEndpointPolicy {
    const BASE: ApiBaseUrl = ApiBaseUrl::CloudV1;
}
impl EndpointAssociation for StorageEndpointPolicy {
    const BASE: ApiBaseUrl = ApiBaseUrl::HetznerV1;
}

pub(crate) trait AuthenticationAssociation {
    const CLASS: AuthenticationClass;
}
impl AuthenticationAssociation for BearerAuthentication {
    const CLASS: AuthenticationClass = AuthenticationClass::Bearer;
}
impl AuthenticationAssociation for BasicAuthentication {
    const CLASS: AuthenticationClass = AuthenticationClass::Basic;
}

pub(crate) trait QueryAssociation {
    const POLICY: QueryPolicy;
}
impl QueryAssociation for QueryForbidden {
    const POLICY: QueryPolicy = QueryPolicy::Forbidden;
}
impl QueryAssociation for OptionalQuery {
    const POLICY: QueryPolicy = QueryPolicy::Optional;
}
impl QueryAssociation for RequiredQuery {
    const POLICY: QueryPolicy = QueryPolicy::Required;
}

pub(crate) trait BodyAssociation {
    type Headers;
    type Media;
    const POLICY: BodyPolicy;
}
impl BodyAssociation for BodyForbidden {
    type Headers = AcceptJson;
    type Media = NoRequestMedia;
    const POLICY: BodyPolicy = BodyPolicy::Forbidden;
}
impl BodyAssociation for JsonBody {
    type Headers = AcceptAndContentTypeJson;
    type Media = JsonRequestMedia;
    const POLICY: BodyPolicy = BodyPolicy::RequiredJson;
}

pub(crate) trait MethodAssociation {
    type Retry;
    const METHOD: Method;
    const RETRY: RetryPolicy;
}
impl MethodAssociation for GetMethod {
    type Retry = ExplicitRetry;
    const METHOD: Method = Method::Get;
    const RETRY: RetryPolicy = RetryPolicy::Explicit;
}
impl MethodAssociation for PutMethod {
    type Retry = ExplicitRetry;
    const METHOD: Method = Method::Put;
    const RETRY: RetryPolicy = RetryPolicy::Explicit;
}
impl MethodAssociation for PostMethod {
    type Retry = NeverRetry;
    const METHOD: Method = Method::Post;
    const RETRY: RetryPolicy = RetryPolicy::Never;
}
impl MethodAssociation for DeleteMethod {
    type Retry = NeverRetry;
    const METHOD: Method = Method::Delete;
    const RETRY: RetryPolicy = RetryPolicy::Never;
}

pub(crate) trait StatusAssociation {
    const STATUS: StatusCode;
}
impl StatusAssociation for StatusOk {
    const STATUS: StatusCode = StatusCode::OK;
}
impl StatusAssociation for StatusCreated {
    const STATUS: StatusCode = StatusCode::CREATED;
}
impl StatusAssociation for StatusNoContent {
    const STATUS: StatusCode = StatusCode::NO_CONTENT;
}

pub(crate) trait ResponseAssociation {
    type Body;
    type Media;
    const SHAPE: ResponseShape;
}

macro_rules! json_response {
    ($marker:ident, $shape:ident) => {
        impl ResponseAssociation for $marker {
            type Body = JsonSuccessBody;
            type Media = JsonSuccessMedia;
            const SHAPE: ResponseShape = ResponseShape::$shape;
        }
    };
}
impl ResponseAssociation for EmptyResponse {
    type Body = EmptySuccessBody;
    type Media = ForbiddenSuccessMedia;
    const SHAPE: ResponseShape = ResponseShape::Empty;
}
json_response!(ActionResponse, Action);
json_response!(ActionsResponse, Actions);
json_response!(ActionsPageResponse, ActionsPage);
json_response!(ResourceResponse, Resource);
json_response!(ResourceListResponse, ResourceList);
json_response!(ResourcePageResponse, ResourcePage);
json_response!(CompositeResponse, Composite);
json_response!(MetricsResponse, Metrics);
json_response!(ZoneFileResponse, ZoneFile);
json_response!(PricingResponse, Pricing);
json_response!(FoldersResponse, Folders);

pub(crate) trait PaginationAssociation {
    const POLICY: PaginationPolicy;
}
impl PaginationAssociation for NoPagination {
    const POLICY: PaginationPolicy = PaginationPolicy::None;
}
impl PaginationAssociation for NumberedPagination {
    const POLICY: PaginationPolicy = PaginationPolicy::Numbered;
}

pub(crate) trait PermitAssociation {
    const CLASS: PermitClass;
}
impl PermitAssociation for NoPermit {
    const CLASS: PermitClass = PermitClass::None;
}
impl PermitAssociation for MutationPermit {
    const CLASS: PermitClass = PermitClass::Mutation;
}
impl PermitAssociation for DestructivePermit {
    const CLASS: PermitClass = PermitClass::Destructive;
}
impl PermitAssociation for CostPermit {
    const CLASS: PermitClass = PermitClass::Cost;
}

pub(crate) const fn metadata_permit(impact: OperationImpact, cost: CostIntent) -> PermitClass {
    if matches!(cost, CostIntent::MayIncurCost) {
        PermitClass::Cost
    } else {
        match impact {
            OperationImpact::ReadOnly => PermitClass::None,
            OperationImpact::Mutation => PermitClass::Mutation,
            OperationImpact::Destructive => PermitClass::Destructive,
        }
    }
}

pub(crate) const fn metadata_retry(
    semantics: RequestSemantics,
    retry: cloud_sdk::operation::RetryEligibility,
) -> RetryPolicy {
    if matches!(
        semantics,
        RequestSemantics::Safe | RequestSemantics::Idempotent
    ) && matches!(
        retry,
        cloud_sdk::operation::RetryEligibility::ExplicitPolicy
    ) {
        RetryPolicy::Explicit
    } else {
        RetryPolicy::Never
    }
}

const _: () = assert!(MAX_ASSOCIATED_JSON_BYTES <= MAX_RAW_RESPONSE_BODY_BYTES);
