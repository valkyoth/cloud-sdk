use cloud_sdk::Method;
use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::buffer::{write_str, write_u64};
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

use super::form::write_form;
use super::request::*;
use super::{RobotVSwitchId, RobotVSwitchName, RobotVSwitchServers, RobotVlanId};
use crate::endpoint::{official_robot_endpoint_identity, official_robot_endpoint_policy};
use crate::identity::{HETZNER_PROVIDER_ID, ROBOT_SERVICE_ID, RobotService};
use crate::robot::RobotCancellationSchedule;

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
const CREATED: &[StatusCode] = &[StatusCode::CREATED];
/// Maximum accepted success-body bytes for `GET /vswitch`.
pub const MAX_ROBOT_VSWITCH_LIST_RESPONSE_BYTES: usize = 1_048_576;
/// Maximum accepted success-body bytes for one vSwitch resource.
pub const MAX_ROBOT_VSWITCH_ITEM_RESPONSE_BYTES: usize = 1_048_576;
const ACCEPT: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
const FORM_HEADERS: [RequestHeader<'static>; 2] = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::FORM_URLENCODED),
];

#[derive(Clone, Copy)]
pub(super) enum Kind<'a> {
    List,
    Create(&'a RobotVSwitchName, RobotVlanId),
    Get(RobotVSwitchId),
    Update(RobotVSwitchId, &'a RobotVSwitchUpdateIntent),
    Cancel(RobotVSwitchId, &'a RobotCancellationSchedule),
    AddServers(RobotVSwitchId, RobotVSwitchServers<'a>),
    RemoveServers(RobotVSwitchId, RobotVSwitchServers<'a>),
}

impl PrepareOperation for RobotVSwitchListRequest {
    type Error = RobotVSwitchRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::List, storage)
    }
}

impl PrepareOperation for RobotVSwitchCreateRequest {
    type Error = RobotVSwitchRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::Create(&self.name, self.vlan), storage)
    }
}

impl PrepareOperation for RobotVSwitchGetRequest {
    type Error = RobotVSwitchRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::Get(self.id), storage)
    }
}

impl PrepareOperation for RobotVSwitchUpdateRequest {
    type Error = RobotVSwitchRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::Update(self.id, &self.intent), storage)
    }
}

impl PrepareOperation for RobotVSwitchCancelRequest {
    type Error = RobotVSwitchRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::Cancel(self.id, &self.schedule), storage)
    }
}

impl PrepareOperation for RobotVSwitchAddServersRequest<'_> {
    type Error = RobotVSwitchRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::AddServers(self.id, self.servers), storage)
    }
}

impl PrepareOperation for RobotVSwitchRemoveServersRequest<'_> {
    type Error = RobotVSwitchRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::RemoveServers(self.id, self.servers), storage)
    }
}

fn prepare<'storage>(
    kind: Kind<'_>,
    storage: PreparationStorage<'storage>,
) -> Result<PreparedRequest<'storage>, RobotVSwitchRequestError> {
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
        official_robot_endpoint_policy().map_err(RobotVSwitchRequestError::InvalidEndpoint)
    );
    let endpoint_identity = admitted!(
        official_robot_endpoint_identity().map_err(RobotVSwitchRequestError::InvalidEndpoint)
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
    let empty_success = empty_success(kind);
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
        .map_err(RobotVSwitchRequestError::InvalidResponsePolicy)
    );
    let content = admitted!(
        HeaderName::new("content-type").map_err(RobotVSwitchRequestError::InvalidHeaders)
    );
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
        .map_err(RobotVSwitchRequestError::InvalidRawPolicy)
    );
    admitted!(
        PreparedRequest::validate_construction_policy(method(kind), metadata, raw)
            .map_err(RobotVSwitchRequestError::InvalidPreparedPolicy)
    );
    let path_len = admitted!(write_target(kind, target_storage));
    let body_len = admitted!(write_form(kind, body_storage));
    let headers = admitted!(
        RequestHeaders::new(if body_len == 0 {
            &ACCEPT
        } else {
            &FORM_HEADERS
        })
        .map_err(RobotVSwitchRequestError::InvalidHeaders)
    );
    let operation_id =
        admitted!(OperationId::new(id(kind)).map_err(RobotVSwitchRequestError::InvalidOperationId));
    let target_valid = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .and_then(|text| RequestTarget::new(text).ok())
        .is_some();
    if !target_valid || body_len > body_storage.len() {
        sanitize_bytes(target_storage);
        sanitize_bytes(body_storage);
        return Err(RobotVSwitchRequestError::Path);
    }
    let Some(target_text) = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
    else {
        unreachable!("validated Robot vSwitch target lost UTF-8")
    };
    let Ok(target) = RequestTarget::new(target_text) else {
        unreachable!("validated Robot vSwitch target became invalid")
    };
    let mut request = TransportRequest::new(method(kind), target).with_headers(headers);
    if body_len != 0 {
        let body = body_storage
            .get(..body_len)
            .unwrap_or_else(|| unreachable!("validated Robot vSwitch form exceeded storage"));
        request = request.with_body(body);
    }
    let prepared = PreparedRequest::new(
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
    .unwrap_or_else(|_| unreachable!("prevalidated Robot vSwitch policy changed during binding"));
    Ok(prepared
        .with_operation_id(operation_id)
        .with_replayable_body())
}

fn write_target(kind: Kind<'_>, output: &mut [u8]) -> Result<usize, RobotVSwitchRequestError> {
    let mut len = 0;
    write_str(output, &mut len, "/vswitch", RobotVSwitchRequestError::Path)?;
    if let Some(vswitch) = vswitch_id(kind) {
        write_str(output, &mut len, "/", RobotVSwitchRequestError::Path)?;
        write_u64(
            output,
            &mut len,
            vswitch.get(),
            RobotVSwitchRequestError::Path,
        )?;
        if matches!(kind, Kind::AddServers(_, _) | Kind::RemoveServers(_, _)) {
            write_str(output, &mut len, "/server", RobotVSwitchRequestError::Path)?;
        }
    }
    Ok(len)
}

const fn vswitch_id(kind: Kind<'_>) -> Option<RobotVSwitchId> {
    match kind {
        Kind::List | Kind::Create(_, _) => None,
        Kind::Get(id)
        | Kind::Update(id, _)
        | Kind::Cancel(id, _)
        | Kind::AddServers(id, _)
        | Kind::RemoveServers(id, _) => Some(id),
    }
}

const fn method(kind: Kind<'_>) -> Method {
    match kind {
        Kind::List | Kind::Get(_) => Method::Get,
        Kind::Create(_, _) | Kind::Update(_, _) | Kind::AddServers(_, _) => Method::Post,
        Kind::Cancel(_, _) | Kind::RemoveServers(_, _) => Method::Delete,
    }
}

const fn id(kind: Kind<'_>) -> &'static str {
    match kind {
        Kind::List => "robot_list_vswitches",
        Kind::Create(_, _) => "robot_create_vswitch",
        Kind::Get(_) => "robot_get_vswitch",
        Kind::Update(_, _) => "robot_update_vswitch",
        Kind::Cancel(_, _) => "robot_cancel_vswitch",
        Kind::AddServers(_, _) => "robot_add_vswitch_servers",
        Kind::RemoveServers(_, _) => "robot_remove_vswitch_servers",
    }
}

const fn success_statuses(kind: Kind<'_>) -> &'static [StatusCode] {
    if matches!(kind, Kind::Create(_, _)) {
        CREATED
    } else {
        OK
    }
}

const fn empty_success(kind: Kind<'_>) -> bool {
    matches!(
        kind,
        Kind::Update(_, _)
            | Kind::Cancel(_, _)
            | Kind::AddServers(_, _)
            | Kind::RemoveServers(_, _)
    )
}

const fn maximum_response_bytes(kind: Kind<'_>) -> usize {
    match kind {
        Kind::List => MAX_ROBOT_VSWITCH_LIST_RESPONSE_BYTES,
        Kind::Create(_, _) | Kind::Get(_) => MAX_ROBOT_VSWITCH_ITEM_RESPONSE_BYTES,
        Kind::Update(_, _)
        | Kind::Cancel(_, _)
        | Kind::AddServers(_, _)
        | Kind::RemoveServers(_, _) => 0,
    }
}

fn metadata(kind: Kind<'_>) -> Result<OperationMetadata, RobotVSwitchRequestError> {
    let (impact, semantics, retry) = match kind {
        Kind::List | Kind::Get(_) => (
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
        ),
        Kind::Create(_, _) | Kind::Update(_, _) | Kind::AddServers(_, _) => (
            OperationImpact::Mutation,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
        Kind::Cancel(_, _) | Kind::RemoveServers(_, _) => (
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
    .map_err(RobotVSwitchRequestError::InvalidMetadata)
}
