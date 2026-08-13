use cloud_sdk::Method;
use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::buffer::{write_query_pair, write_str};
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
use crate::robot::{RobotForm, RobotFormField, RobotIpAddress, RobotRdnsName};

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
const CREATED: &[StatusCode] = &[StatusCode::CREATED];
const OK_OR_CREATED: &[StatusCode] = &[StatusCode::OK, StatusCode::CREATED];
/// Maximum accepted success-body bytes for `GET /rdns`.
pub const MAX_ROBOT_RDNS_LIST_RESPONSE_BYTES: usize = 2_097_152;
/// Maximum accepted success-body bytes for one reverse-DNS resource.
pub const MAX_ROBOT_RDNS_ITEM_RESPONSE_BYTES: usize = 16_384;
const ACCEPT: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
const FORM_HEADERS: [RequestHeader<'static>; 2] = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::FORM_URLENCODED),
];

#[derive(Clone, Copy)]
pub(super) enum Kind<'a> {
    List(Option<&'a RobotIpAddress>),
    Get(&'a RobotIpAddress),
    Set(&'a RobotIpAddress, &'a RobotRdnsName),
    Update(&'a RobotIpAddress, &'a RobotRdnsName),
    Delete(&'a RobotIpAddress),
}

impl PrepareOperation for RobotRdnsListRequest {
    type Error = RobotRdnsRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::List(self.server_address.as_ref()), storage)
    }
}

macro_rules! prepare_operation {
    ($type:ty, $kind:ident, $($field:ident),+ $(,)?) => {
        impl PrepareOperation for $type {
            type Error = RobotRdnsRequestError;
            fn prepare<'storage>(
                &self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRequest<'storage>, Self::Error> {
                prepare(Kind::$kind($(&self.$field),+), storage)
            }
        }
    };
}

prepare_operation!(RobotRdnsGetRequest, Get, address);
prepare_operation!(RobotRdnsSetRequest, Set, address, ptr);
prepare_operation!(RobotRdnsUpdateRequest, Update, address, ptr);
prepare_operation!(RobotRdnsDeleteRequest, Delete, address);

fn prepare<'storage>(
    kind: Kind<'_>,
    storage: PreparationStorage<'storage>,
) -> Result<PreparedRequest<'storage>, RobotRdnsRequestError> {
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

    let endpoint =
        admitted!(official_robot_endpoint_policy().map_err(RobotRdnsRequestError::InvalidEndpoint));
    let endpoint_identity = admitted!(
        official_robot_endpoint_identity().map_err(RobotRdnsRequestError::InvalidEndpoint)
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
    let empty_success = matches!(kind, Kind::Delete(_));
    let maximum = maximum_response_bytes(kind);
    let response = admitted!(
        ResponsePolicy::new(
            success_statuses(kind),
            if empty_success {
                ContentTypePolicy::Forbidden
            } else {
                ContentTypePolicy::Required(JSON)
            },
            if empty_success {
                ResponseBodyPolicy::Forbidden
            } else {
                ResponseBodyPolicy::Required
            },
            maximum,
        )
        .map_err(RobotRdnsRequestError::InvalidResponsePolicy)
    );
    let content =
        admitted!(HeaderName::new("content-type").map_err(RobotRdnsRequestError::InvalidHeaders));
    let raw = admitted!(
        RawResponsePolicy::new(
            maximum,
            crate::robot::MAX_ROBOT_ERROR_BODY_BYTES,
            if empty_success {
                ResponseMediaPolicy::Forbidden
            } else {
                ResponseMediaPolicy::Required(JSON)
            },
            ResponseMediaPolicy::Optional(JSON),
            &[content],
            MAX_INFORMATIONAL_RESPONSES,
        )
        .map_err(RobotRdnsRequestError::InvalidRawPolicy)
    );
    let path_len = admitted!(write_target(kind, target_storage));
    let body_len = match kind {
        Kind::Set(_, ptr) | Kind::Update(_, ptr) => {
            admitted!(write_ptr_form(ptr, body_storage))
        }
        Kind::List(_) | Kind::Get(_) | Kind::Delete(_) => 0,
    };
    let headers = admitted!(
        RequestHeaders::new(if body_len == 0 {
            &ACCEPT
        } else {
            &FORM_HEADERS
        })
        .map_err(RobotRdnsRequestError::InvalidHeaders)
    );
    let operation_id =
        admitted!(OperationId::new(id(kind)).map_err(RobotRdnsRequestError::InvalidOperationId));
    let target_valid = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .and_then(|text| RequestTarget::new(text).ok())
        .is_some();
    if !target_valid {
        sanitize_bytes(target_storage);
        sanitize_bytes(body_storage);
        return Err(RobotRdnsRequestError::Path);
    }
    let Some(target_text) = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
    else {
        unreachable!("validated Robot reverse-DNS target lost UTF-8")
    };
    let Ok(target) = RequestTarget::new(target_text) else {
        unreachable!("validated Robot reverse-DNS target became invalid")
    };
    let mut request = TransportRequest::new(method(kind), target).with_headers(headers);
    if body_len != 0 {
        let body = body_storage
            .get(..body_len)
            .ok_or(RobotRdnsRequestError::Path)?;
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
    .map_err(RobotRdnsRequestError::InvalidPreparedPolicy)
    .map(|prepared| {
        prepared
            .with_operation_id(operation_id)
            .with_replayable_body()
    })
}

fn write_target(kind: Kind<'_>, output: &mut [u8]) -> Result<usize, RobotRdnsRequestError> {
    let mut len = 0;
    write_str(output, &mut len, "/rdns", RobotRdnsRequestError::Path)?;
    match kind {
        Kind::List(Some(server)) => server.with_text(|text| {
            write_str(output, &mut len, "?", RobotRdnsRequestError::Path)?;
            let mut first = true;
            write_query_pair(
                output,
                &mut len,
                &mut first,
                "server_ip",
                text,
                RobotRdnsRequestError::Path,
            )
        })?,
        Kind::List(None) => {}
        Kind::Get(address)
        | Kind::Set(address, _)
        | Kind::Update(address, _)
        | Kind::Delete(address) => {
            write_str(output, &mut len, "/", RobotRdnsRequestError::Path)?;
            address
                .with_text(|text| write_str(output, &mut len, text, RobotRdnsRequestError::Path))?;
        }
    }
    Ok(len)
}

fn write_ptr_form(ptr: &RobotRdnsName, output: &mut [u8]) -> Result<usize, RobotRdnsRequestError> {
    ptr.with_text(|text| {
        let field = RobotFormField::sensitive("ptr", text).map_err(RobotRdnsRequestError::Form)?;
        RobotForm::new(&[field])
            .and_then(|form| form.write_prepared(output))
            .map_err(RobotRdnsRequestError::Form)
    })
}

const fn method(kind: Kind<'_>) -> Method {
    match kind {
        Kind::List(_) | Kind::Get(_) => Method::Get,
        Kind::Set(_, _) => Method::Put,
        Kind::Update(_, _) => Method::Post,
        Kind::Delete(_) => Method::Delete,
    }
}

const fn id(kind: Kind<'_>) -> &'static str {
    match kind {
        Kind::List(_) => "robot_list_rdns",
        Kind::Get(_) => "robot_get_rdns",
        Kind::Set(_, _) => "robot_set_rdns",
        Kind::Update(_, _) => "robot_update_rdns",
        Kind::Delete(_) => "robot_delete_rdns",
    }
}

const fn success_statuses(kind: Kind<'_>) -> &'static [StatusCode] {
    match kind {
        Kind::Set(_, _) => CREATED,
        Kind::Update(_, _) => OK_OR_CREATED,
        Kind::List(_) | Kind::Get(_) | Kind::Delete(_) => OK,
    }
}

const fn maximum_response_bytes(kind: Kind<'_>) -> usize {
    match kind {
        Kind::List(_) => MAX_ROBOT_RDNS_LIST_RESPONSE_BYTES,
        Kind::Delete(_) => 0,
        Kind::Get(_) | Kind::Set(_, _) | Kind::Update(_, _) => MAX_ROBOT_RDNS_ITEM_RESPONSE_BYTES,
    }
}

fn metadata(kind: Kind<'_>) -> Result<OperationMetadata, RobotRdnsRequestError> {
    let (impact, semantics, retry) = match kind {
        Kind::List(_) | Kind::Get(_) => (
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
        ),
        Kind::Set(_, _) => (
            OperationImpact::Mutation,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
        Kind::Update(_, _) => (
            OperationImpact::Mutation,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
        Kind::Delete(_) => (
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
    .map_err(RobotRdnsRequestError::InvalidMetadata)
}
