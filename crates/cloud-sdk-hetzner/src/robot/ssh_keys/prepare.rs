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
use crate::robot::{
    RobotForm, RobotFormField, RobotSshKeyData, RobotSshKeyFingerprint, RobotSshKeyName,
};

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
const CREATED: &[StatusCode] = &[StatusCode::CREATED];
/// Maximum accepted success-body bytes for `GET /key`.
pub const MAX_ROBOT_SSH_KEY_LIST_RESPONSE_BYTES: usize = 2_097_152;
/// Maximum accepted success-body bytes for one SSH-key resource.
pub const MAX_ROBOT_SSH_KEY_ITEM_RESPONSE_BYTES: usize = 32_768;
const ACCEPT: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
const FORM_HEADERS: [RequestHeader<'static>; 2] = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::FORM_URLENCODED),
];

#[derive(Clone, Copy)]
pub(super) enum Kind<'a> {
    List,
    Create(&'a RobotSshKeyName, &'a RobotSshKeyData<'a>),
    Get(&'a RobotSshKeyFingerprint),
    Update(&'a RobotSshKeyFingerprint, &'a RobotSshKeyName),
    Delete(&'a RobotSshKeyFingerprint),
}

impl PrepareOperation for RobotSshKeyListRequest {
    type Error = RobotSshKeyRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::List, storage)
    }
}

impl PrepareOperation for RobotSshKeyCreateRequest<'_> {
    type Error = RobotSshKeyRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::Create(&self.name, &self.data), storage)
    }
}

macro_rules! prepare_operation {
    ($type:ty, $kind:ident, $($field:ident),+ $(,)?) => {
        impl PrepareOperation for $type {
            type Error = RobotSshKeyRequestError;
            fn prepare<'storage>(
                &self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRequest<'storage>, Self::Error> {
                prepare(Kind::$kind($(&self.$field),+), storage)
            }
        }
    };
}

prepare_operation!(RobotSshKeyGetRequest, Get, fingerprint);
prepare_operation!(RobotSshKeyUpdateRequest, Update, fingerprint, name);
prepare_operation!(RobotSshKeyDeleteRequest, Delete, fingerprint);

fn prepare<'storage>(
    kind: Kind<'_>,
    storage: PreparationStorage<'storage>,
) -> Result<PreparedRequest<'storage>, RobotSshKeyRequestError> {
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
        official_robot_endpoint_policy().map_err(RobotSshKeyRequestError::InvalidEndpoint)
    );
    let endpoint_identity = admitted!(
        official_robot_endpoint_identity().map_err(RobotSshKeyRequestError::InvalidEndpoint)
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
        .map_err(RobotSshKeyRequestError::InvalidResponsePolicy)
    );
    let content =
        admitted!(HeaderName::new("content-type").map_err(RobotSshKeyRequestError::InvalidHeaders));
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
        .map_err(RobotSshKeyRequestError::InvalidRawPolicy)
    );
    let path_len = admitted!(write_target(kind, target_storage));
    let body_len = admitted!(write_form(kind, body_storage));
    let headers = admitted!(
        RequestHeaders::new(if body_len == 0 {
            &ACCEPT
        } else {
            &FORM_HEADERS
        })
        .map_err(RobotSshKeyRequestError::InvalidHeaders)
    );
    let operation_id =
        admitted!(OperationId::new(id(kind)).map_err(RobotSshKeyRequestError::InvalidOperationId));
    let target_valid = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .and_then(|text| RequestTarget::new(text).ok())
        .is_some();
    if !target_valid {
        sanitize_bytes(target_storage);
        sanitize_bytes(body_storage);
        return Err(RobotSshKeyRequestError::Path);
    }
    let Some(target_text) = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
    else {
        unreachable!("validated Robot SSH-key target lost UTF-8")
    };
    let Ok(target) = RequestTarget::new(target_text) else {
        unreachable!("validated Robot SSH-key target became invalid")
    };
    let mut request = TransportRequest::new(method(kind), target).with_headers(headers);
    if body_len != 0 {
        let body = body_storage
            .get(..body_len)
            .ok_or(RobotSshKeyRequestError::Path)?;
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
    .map_err(RobotSshKeyRequestError::InvalidPreparedPolicy)
    .map(|prepared| {
        prepared
            .with_operation_id(operation_id)
            .with_replayable_body()
    })
}

fn write_target(kind: Kind<'_>, output: &mut [u8]) -> Result<usize, RobotSshKeyRequestError> {
    let mut len = 0;
    write_str(output, &mut len, "/key", RobotSshKeyRequestError::Path)?;
    let fingerprint = match kind {
        Kind::Get(value) | Kind::Update(value, _) | Kind::Delete(value) => Some(value),
        Kind::List | Kind::Create(_, _) => None,
    };
    if let Some(fingerprint) = fingerprint {
        write_str(output, &mut len, "/", RobotSshKeyRequestError::Path)?;
        fingerprint
            .with_text(|text| write_str(output, &mut len, text, RobotSshKeyRequestError::Path))?;
    }
    Ok(len)
}

fn write_form(kind: Kind<'_>, output: &mut [u8]) -> Result<usize, RobotSshKeyRequestError> {
    match kind {
        Kind::Create(name, data) => name.with_text(|name| {
            data.with_text(|data| {
                let fields = [
                    RobotFormField::sensitive("name", name)
                        .map_err(RobotSshKeyRequestError::Form)?,
                    RobotFormField::sensitive("data", data)
                        .map_err(RobotSshKeyRequestError::Form)?,
                ];
                RobotForm::new(&fields)
                    .and_then(|form| form.write_prepared(output))
                    .map_err(RobotSshKeyRequestError::Form)
            })
        }),
        Kind::Update(_, name) => name.with_text(|name| {
            let field =
                RobotFormField::sensitive("name", name).map_err(RobotSshKeyRequestError::Form)?;
            RobotForm::new(&[field])
                .and_then(|form| form.write_prepared(output))
                .map_err(RobotSshKeyRequestError::Form)
        }),
        Kind::List | Kind::Get(_) | Kind::Delete(_) => Ok(0),
    }
}

const fn method(kind: Kind<'_>) -> Method {
    match kind {
        Kind::List | Kind::Get(_) => Method::Get,
        Kind::Create(_, _) | Kind::Update(_, _) => Method::Post,
        Kind::Delete(_) => Method::Delete,
    }
}

const fn id(kind: Kind<'_>) -> &'static str {
    match kind {
        Kind::List => "robot_list_ssh_keys",
        Kind::Create(_, _) => "robot_create_ssh_key",
        Kind::Get(_) => "robot_get_ssh_key",
        Kind::Update(_, _) => "robot_update_ssh_key",
        Kind::Delete(_) => "robot_delete_ssh_key",
    }
}

const fn success_statuses(kind: Kind<'_>) -> &'static [StatusCode] {
    match kind {
        Kind::Create(_, _) => CREATED,
        Kind::List | Kind::Get(_) | Kind::Update(_, _) | Kind::Delete(_) => OK,
    }
}

const fn maximum_response_bytes(kind: Kind<'_>) -> usize {
    match kind {
        Kind::List => MAX_ROBOT_SSH_KEY_LIST_RESPONSE_BYTES,
        Kind::Delete(_) => 0,
        Kind::Create(_, _) | Kind::Get(_) | Kind::Update(_, _) => {
            MAX_ROBOT_SSH_KEY_ITEM_RESPONSE_BYTES
        }
    }
}

fn metadata(kind: Kind<'_>) -> Result<OperationMetadata, RobotSshKeyRequestError> {
    let (impact, semantics, retry) = match kind {
        Kind::List | Kind::Get(_) => (
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
        ),
        Kind::Create(_, _) | Kind::Update(_, _) => (
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
    .map_err(RobotSshKeyRequestError::InvalidMetadata)
}
