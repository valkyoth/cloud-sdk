use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationImpact, OperationMetadata, PreparedRequest,
    ProviderService, RequestIdPolicy, RequestSemantics, ResponseBodyPolicy, ResponsePolicy,
    RetryEligibility,
};
use cloud_sdk::transport::{
    EndpointIdentity, EndpointPolicy, EndpointScheme, HeaderName, MediaType, RawResponsePolicy,
    RequestHeader, RequestHeaders, RequestTarget, ResponseMediaPolicy, StatusCode,
    TransportRequest,
};
use cloud_sdk::{Method, ProviderId, ProviderMarker, ServiceId, ServiceMarker};
use cloud_sdk_testkit::{ExpectedRequest, FixtureBody, MockExchange, ResponseFixture};

pub const MAX_RESPONSE_BYTES: usize = 65_536;
pub static OK_STATUS: [StatusCode; 1] = [StatusCode::OK];
pub static JSON_MEDIA: [MediaType<'static>; 1] = [MediaType::JSON];

pub enum OvhcloudProbe {}

impl ProviderMarker for OvhcloudProbe {
    const ID: ProviderId = cloud_sdk::provider_id!("ovhcloud-probe");
}

pub enum ApiV2 {}

impl ServiceMarker for ApiV2 {
    type Provider = OvhcloudProbe;
    const ID: ServiceId = cloud_sdk::service_id!("api-v2");
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Operation {
    pub id: &'static str,
    pub template: &'static str,
    pub target: &'static str,
    pub paginated: bool,
    pub response: &'static [u8],
}

pub static OPERATIONS: [Operation; 10] = [
    operation("iam/permissionsgroup", "/iam/permissionsGroup", true, b"[]"),
    operation(
        "iam/permissionsgroup/by-permissionsgroupurn",
        "/iam/permissionsGroup/{permissionsGroupURN}",
        false,
        b"{}",
    ),
    operation("iam/policy", "/iam/policy", true, b"[]"),
    operation(
        "iam/policy/by-policyid",
        "/iam/policy/{policyId}",
        false,
        b"{}",
    ),
    operation("iam/resource", "/iam/resource", true, b"[]"),
    operation(
        "iam/resource/by-resourceurn",
        "/iam/resource/{resourceURN}",
        false,
        b"{}",
    ),
    operation("iam/resourcegroup", "/iam/resourceGroup", true, b"[]"),
    operation(
        "iam/resourcegroup/by-groupid",
        "/iam/resourceGroup/{groupId}",
        false,
        b"{}",
    ),
    operation(
        "notification/contactmean/by-contactmeanid/task",
        "/notification/contactMean/{contactMeanId}/task",
        true,
        b"[]",
    ),
    operation(
        "notification/contactmean/by-contactmeanid/task/by-taskid",
        "/notification/contactMean/{contactMeanId}/task/{taskId}",
        false,
        b"{}",
    ),
];

const fn operation(
    id: &'static str,
    template: &'static str,
    paginated: bool,
    response: &'static [u8],
) -> Operation {
    Operation {
        id,
        template,
        target: realized_target(id),
        paginated,
        response,
    }
}

const fn realized_target(id: &str) -> &'static str {
    match id.as_bytes() {
        b"iam/permissionsgroup" => "/iam/permissionsGroup",
        b"iam/permissionsgroup/by-permissionsgroupurn" => {
            "/iam/permissionsGroup/urn%3Aovh%3Aiam%3A%3Aexample%3Agroup%3Areaders"
        }
        b"iam/policy" => "/iam/policy",
        b"iam/policy/by-policyid" => "/iam/policy/11111111-1111-4111-8111-111111111111",
        b"iam/resource" => "/iam/resource",
        b"iam/resource/by-resourceurn" => {
            "/iam/resource/urn%3Aovh%3Aresource%3A%3Aexample%3Aproject%3Ademo"
        }
        b"iam/resourcegroup" => "/iam/resourceGroup",
        b"iam/resourcegroup/by-groupid" => {
            "/iam/resourceGroup/22222222-2222-4222-8222-222222222222"
        }
        b"notification/contactmean/by-contactmeanid/task" => {
            "/notification/contactMean/33333333-3333-4333-8333-333333333333/task"
        }
        b"notification/contactmean/by-contactmeanid/task/by-taskid" => {
            "/notification/contactMean/33333333-3333-4333-8333-333333333333/task/44444444-4444-4444-8444-444444444444"
        }
        _ => "",
    }
}

pub fn endpoint() -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, "eu.api.ovh.com", 443, "/v2")
        .unwrap_or_else(|_| unreachable!("source-locked endpoint must remain valid"))
}

pub fn request_headers(paginated: bool) -> ([RequestHeader<'static>; 1], usize) {
    let page_size = RequestHeader::new("x-pagination-size", "100")
        .unwrap_or_else(|_| unreachable!("source-locked header must remain valid"));
    ([page_size], usize::from(paginated))
}

pub fn prepared<'a>(operation: Operation, entries: &'a [RequestHeader<'a>]) -> PreparedRequest<'a> {
    let target = RequestTarget::new(operation.target)
        .unwrap_or_else(|_| unreachable!("source-locked target must remain valid"));
    let headers = RequestHeaders::new(entries)
        .unwrap_or_else(|_| unreachable!("source-locked headers must remain valid"));
    let request = TransportRequest::new(Method::Get, target).with_headers(headers);
    let metadata = OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::Never,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .unwrap_or_else(|_| unreachable!("read-only metadata must remain valid"));
    let response_policy = ResponsePolicy::new(
        &OK_STATUS,
        ContentTypePolicy::Required(&JSON_MEDIA),
        ResponseBodyPolicy::Required,
        MAX_RESPONSE_BYTES,
    )
    .unwrap_or_else(|_| unreachable!("response policy must remain valid"));
    let identity = endpoint();
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(OvhcloudProbe::ID),
        ScopeRequirement::Required(ApiV2::ID),
        ScopeRequirement::Required(identity),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let content_type = HeaderName::new("content-type")
        .unwrap_or_else(|_| unreachable!("source-locked header must remain valid"));
    let next_cursor = HeaderName::new("x-pagination-cursor-next")
        .unwrap_or_else(|_| unreachable!("source-locked header must remain valid"));
    let retained = [content_type, next_cursor];
    let raw_policy = RawResponsePolicy::new(
        MAX_RESPONSE_BYTES,
        MAX_RESPONSE_BYTES,
        ResponseMediaPolicy::Required(&JSON_MEDIA),
        ResponseMediaPolicy::Required(&JSON_MEDIA),
        &retained,
        8,
    )
    .unwrap_or_else(|_| unreachable!("raw response policy must remain valid"));
    PreparedRequest::new(
        request,
        ProviderService::from_marker::<ApiV2>(EndpointPolicy::fixed(identity)),
        metadata,
        response_policy,
        authentication,
        raw_policy,
    )
    .unwrap_or_else(|_| unreachable!("prepared probe request must remain valid"))
}

#[allow(dead_code)]
pub fn exchange<'a>(operation: Operation, entries: &'a [RequestHeader<'a>]) -> MockExchange<'a> {
    let target = RequestTarget::new(operation.target)
        .unwrap_or_else(|_| unreachable!("source-locked target must remain valid"));
    let headers = RequestHeaders::new(entries)
        .unwrap_or_else(|_| unreachable!("source-locked headers must remain valid"));
    let body = FixtureBody::new(operation.response)
        .unwrap_or_else(|_| unreachable!("bounded response fixture must remain valid"));
    MockExchange::new(
        ExpectedRequest::new(Method::Get, target).with_headers(headers),
        ResponseFixture::success(body).with_content_type("application/json"),
    )
}
