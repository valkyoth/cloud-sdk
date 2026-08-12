use cloud_sdk::Method;
use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::buffer::write_str;
use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationImpact, OperationMetadata,
    PreparationStorage, PrepareOperation, PreparedRequest, ProviderService, RequestBodySensitivity,
    RequestIdPolicy, RequestSemantics, ResponseBodyPolicy, ResponsePolicy, RetryEligibility,
};
use cloud_sdk::transport::{
    ContentType, HeaderName, MAX_INFORMATIONAL_RESPONSES, MediaType, RawResponsePolicy,
    RequestHeader, RequestHeaders, RequestTarget, ResponseMediaPolicy, StatusCode,
    TransportRequest,
};
use cloud_sdk_sanitization::sanitize_bytes;

use super::request::*;
use crate::endpoint::{official_robot_endpoint_identity, official_robot_endpoint_policy};
use crate::identity::{HETZNER_PROVIDER_ID, ROBOT_SERVICE_ID, RobotService};
use crate::robot::{RobotForm, RobotFormField, RobotIpAddress};

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
/// Maximum accepted success-body bytes for `GET /failover`.
pub const MAX_ROBOT_FAILOVER_LIST_RESPONSE_BYTES: usize = 2_097_152;
/// Maximum accepted success-body bytes for one failover resource.
pub const MAX_ROBOT_FAILOVER_ITEM_RESPONSE_BYTES: usize = 16_384;
const ACCEPT: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
const FORM_HEADERS: [RequestHeader<'static>; 2] = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::FORM_URLENCODED),
];

#[derive(Clone, Copy)]
pub(super) enum Kind<'a> {
    List,
    Get(&'a RobotIpAddress),
    Reroute(&'a RobotIpAddress, &'a RobotIpAddress),
    DeleteRoute(&'a RobotIpAddress),
}

impl PrepareOperation for RobotFailoverListRequest {
    type Error = RobotFailoverRequestError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::List, storage)
    }
}

macro_rules! prepare_route {
    ($type:ty, $kind:ident) => {
        impl PrepareOperation for $type {
            type Error = RobotFailoverRequestError;

            fn prepare<'storage>(
                &self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRequest<'storage>, Self::Error> {
                prepare(Kind::$kind(&self.route), storage)
            }
        }
    };
}

prepare_route!(RobotFailoverGetRequest, Get);
prepare_route!(RobotFailoverDeleteRouteRequest, DeleteRoute);

impl PrepareOperation for RobotFailoverRerouteRequest {
    type Error = RobotFailoverRequestError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::Reroute(&self.route, &self.active_server), storage)
    }
}

fn prepare<'storage>(
    kind: Kind<'_>,
    storage: PreparationStorage<'storage>,
) -> Result<PreparedRequest<'storage>, RobotFailoverRequestError> {
    let (target_storage, body_storage) = storage.into_parts();
    sanitize_bytes(target_storage);
    sanitize_bytes(body_storage);
    macro_rules! admitted {
        ($result:expr) => {
            match $result {
                Ok(value) => value,
                Err(error) => {
                    sanitize_bytes(target_storage);
                    sanitize_bytes(body_storage);
                    return Err(error);
                }
            }
        };
    }

    let endpoint = admitted!(
        official_robot_endpoint_policy().map_err(RobotFailoverRequestError::InvalidEndpoint)
    );
    let endpoint_identity = admitted!(
        official_robot_endpoint_identity().map_err(RobotFailoverRequestError::InvalidEndpoint)
    );
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(HETZNER_PROVIDER_ID),
        ScopeRequirement::Required(ROBOT_SERVICE_ID),
        ScopeRequirement::Required(endpoint_identity),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let service = ProviderService::from_marker::<RobotService>(endpoint);
    let metadata = admitted!(metadata(kind));
    let maximum = maximum_response_bytes(kind);
    let response = admitted!(
        ResponsePolicy::new(
            OK,
            ContentTypePolicy::Required(JSON),
            ResponseBodyPolicy::Required,
            maximum,
        )
        .map_err(RobotFailoverRequestError::InvalidResponsePolicy)
    );
    let content = admitted!(
        HeaderName::new("content-type").map_err(RobotFailoverRequestError::InvalidHeaders)
    );
    let raw = admitted!(
        RawResponsePolicy::new(
            maximum,
            crate::robot::MAX_ROBOT_ERROR_BODY_BYTES,
            ResponseMediaPolicy::Required(JSON),
            ResponseMediaPolicy::Optional(JSON),
            &[content],
            MAX_INFORMATIONAL_RESPONSES,
        )
        .map_err(RobotFailoverRequestError::InvalidRawPolicy)
    );
    let path_len = admitted!(write_target(kind, target_storage));
    let body_len = match kind {
        Kind::Reroute(_, destination) => admitted!(write_reroute_form(destination, body_storage)),
        Kind::List | Kind::Get(_) | Kind::DeleteRoute(_) => 0,
    };
    let headers = admitted!(
        RequestHeaders::new(if body_len == 0 {
            &ACCEPT
        } else {
            &FORM_HEADERS
        })
        .map_err(RobotFailoverRequestError::InvalidHeaders)
    );
    let operation_id = admitted!(
        OperationId::new(id(kind)).map_err(RobotFailoverRequestError::InvalidOperationId)
    );
    let target_valid = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .and_then(|text| RequestTarget::new(text).ok())
        .is_some();
    if !target_valid {
        sanitize_bytes(target_storage);
        sanitize_bytes(body_storage);
        return Err(RobotFailoverRequestError::Path);
    }
    let Some(target_text) = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
    else {
        unreachable!("validated Robot failover target lost UTF-8")
    };
    let Ok(target) = RequestTarget::new(target_text) else {
        unreachable!("validated Robot failover target became invalid")
    };
    let mut request = TransportRequest::new(method(kind), target).with_headers(headers);
    if body_len != 0 {
        let body = body_storage
            .get(..body_len)
            .ok_or(RobotFailoverRequestError::Path)?;
        request = request.with_body(body);
    }
    PreparedRequest::new(
        request,
        service,
        metadata,
        response,
        authentication,
        raw,
        if body_len == 0 {
            RequestBodySensitivity::Public
        } else {
            RequestBodySensitivity::Sensitive
        },
    )
    .map_err(RobotFailoverRequestError::InvalidPreparedPolicy)
    .map(|prepared| {
        prepared
            .with_operation_id(operation_id)
            .with_replayable_body()
    })
}

fn write_target(kind: Kind<'_>, output: &mut [u8]) -> Result<usize, RobotFailoverRequestError> {
    let mut len = 0;
    write_str(
        output,
        &mut len,
        "/failover",
        RobotFailoverRequestError::Path,
    )?;
    if let Some(route) = route(kind) {
        write_str(output, &mut len, "/", RobotFailoverRequestError::Path)?;
        route
            .with_text(|text| write_str(output, &mut len, text, RobotFailoverRequestError::Path))?;
    }
    Ok(len)
}

fn write_reroute_form(
    destination: &RobotIpAddress,
    output: &mut [u8],
) -> Result<usize, RobotFailoverRequestError> {
    destination.with_text(|text| {
        let field = RobotFormField::sensitive("active_server_ip", text)
            .map_err(RobotFailoverRequestError::Form)?;
        RobotForm::new(&[field])
            .and_then(|form| form.write_prepared(output))
            .map_err(RobotFailoverRequestError::Form)
    })
}

const fn route(kind: Kind<'_>) -> Option<&RobotIpAddress> {
    match kind {
        Kind::List => None,
        Kind::Get(route) | Kind::Reroute(route, _) | Kind::DeleteRoute(route) => Some(route),
    }
}

const fn method(kind: Kind<'_>) -> Method {
    match kind {
        Kind::List | Kind::Get(_) => Method::Get,
        Kind::Reroute(_, _) => Method::Post,
        Kind::DeleteRoute(_) => Method::Delete,
    }
}

const fn id(kind: Kind<'_>) -> &'static str {
    match kind {
        Kind::List => "robot_list_failovers",
        Kind::Get(_) => "robot_get_failover",
        Kind::Reroute(_, _) => "robot_reroute_failover",
        Kind::DeleteRoute(_) => "robot_delete_failover_route",
    }
}

const fn maximum_response_bytes(kind: Kind<'_>) -> usize {
    if matches!(kind, Kind::List) {
        MAX_ROBOT_FAILOVER_LIST_RESPONSE_BYTES
    } else {
        MAX_ROBOT_FAILOVER_ITEM_RESPONSE_BYTES
    }
}

fn metadata(kind: Kind<'_>) -> Result<OperationMetadata, RobotFailoverRequestError> {
    let (impact, semantics, retry) = match kind {
        Kind::List | Kind::Get(_) => (
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
        ),
        Kind::Reroute(_, _) => (
            OperationImpact::Mutation,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
        Kind::DeleteRoute(_) => (
            OperationImpact::Destructive,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
    };
    OperationMetadata::new(
        impact,
        semantics,
        retry,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .map_err(RobotFailoverRequestError::InvalidMetadata)
}
