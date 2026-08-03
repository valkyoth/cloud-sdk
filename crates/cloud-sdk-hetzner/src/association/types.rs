//! Zero-sized policy types used by operation associations.

use cloud_sdk::Method;
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
    const METHOD: Method;
}
impl MethodAssociation for GetMethod {
    const METHOD: Method = Method::Get;
}
impl MethodAssociation for PutMethod {
    const METHOD: Method = Method::Put;
}
impl MethodAssociation for PostMethod {
    const METHOD: Method = Method::Post;
}
impl MethodAssociation for DeleteMethod {
    const METHOD: Method = Method::Delete;
}

pub(crate) trait RetryAssociation {
    const POLICY: RetryPolicy;
}
impl RetryAssociation for ExplicitRetry {
    const POLICY: RetryPolicy = RetryPolicy::Explicit;
}
impl RetryAssociation for NeverRetry {
    const POLICY: RetryPolicy = RetryPolicy::Never;
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

const _: () = assert!(MAX_ASSOCIATED_JSON_BYTES <= MAX_RAW_RESPONSE_BODY_BYTES);
