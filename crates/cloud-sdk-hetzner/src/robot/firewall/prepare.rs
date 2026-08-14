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
use super::types::RobotFirewallTemplateId;
use super::value::RobotFirewallTemplateConfig;
use crate::endpoint::{official_robot_endpoint_identity, official_robot_endpoint_policy};
use crate::identity::{HETZNER_PROVIDER_ID, ROBOT_SERVICE_ID, RobotService};
use crate::robot::RobotServerNumber;

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
const CREATED: &[StatusCode] = &[StatusCode::CREATED];
/// Maximum accepted body bytes for one firewall or template resource.
pub const MAX_ROBOT_FIREWALL_ITEM_RESPONSE_BYTES: usize = 262_144;
/// Maximum accepted body bytes for the template inventory.
pub const MAX_ROBOT_FIREWALL_TEMPLATE_LIST_RESPONSE_BYTES: usize = 2_097_152;
const ACCEPT: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
const FORM_HEADERS: [RequestHeader<'static>; 2] = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::FORM_URLENCODED),
];

#[derive(Clone, Copy)]
pub(super) enum Kind<'a> {
    Get(&'a RobotServerNumber),
    Replace(&'a RobotServerNumber, RobotFirewallReplaceIntent<'a>),
    Delete(&'a RobotServerNumber),
    TemplateList,
    TemplateCreate(RobotFirewallTemplateConfig<'a>),
    TemplateGet(RobotFirewallTemplateId),
    TemplateUpdate(RobotFirewallTemplateId, RobotFirewallTemplateConfig<'a>),
    TemplateDelete(RobotFirewallTemplateId),
}

macro_rules! prepare_server_operation {
    ($type:ty, $kind:ident) => {
        impl PrepareOperation for $type {
            type Error = RobotFirewallRequestError;
            fn prepare<'storage>(
                &self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRequest<'storage>, Self::Error> {
                prepare(Kind::$kind(&self.server), storage)
            }
        }
    };
}

prepare_server_operation!(RobotFirewallGetRequest, Get);
prepare_server_operation!(RobotFirewallDeleteRequest, Delete);

impl PrepareOperation for RobotFirewallReplaceRequest<'_> {
    type Error = RobotFirewallRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::Replace(&self.server, self.intent), storage)
    }
}

impl PrepareOperation for RobotFirewallTemplateListRequest {
    type Error = RobotFirewallRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::TemplateList, storage)
    }
}

impl PrepareOperation for RobotFirewallTemplateCreateRequest<'_> {
    type Error = RobotFirewallRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::TemplateCreate(self.config), storage)
    }
}

impl PrepareOperation for RobotFirewallTemplateGetRequest {
    type Error = RobotFirewallRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::TemplateGet(self.template_id), storage)
    }
}

impl PrepareOperation for RobotFirewallTemplateUpdateRequest<'_> {
    type Error = RobotFirewallRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::TemplateUpdate(self.template_id, self.config), storage)
    }
}

impl PrepareOperation for RobotFirewallTemplateDeleteRequest {
    type Error = RobotFirewallRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::TemplateDelete(self.template_id), storage)
    }
}

fn prepare<'storage>(
    kind: Kind<'_>,
    storage: PreparationStorage<'storage>,
) -> Result<PreparedRequest<'storage>, RobotFirewallRequestError> {
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
        official_robot_endpoint_policy().map_err(RobotFirewallRequestError::InvalidEndpoint)
    );
    let endpoint_identity = admitted!(
        official_robot_endpoint_identity().map_err(RobotFirewallRequestError::InvalidEndpoint)
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
    let empty_success = matches!(kind, Kind::TemplateDelete(_));
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
        .map_err(RobotFirewallRequestError::InvalidResponsePolicy)
    );
    let content = admitted!(
        HeaderName::new("content-type").map_err(RobotFirewallRequestError::InvalidHeaders)
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
        .map_err(RobotFirewallRequestError::InvalidRawPolicy)
    );
    admitted!(
        PreparedRequest::validate_construction_policy(method(kind), metadata, raw)
            .map_err(RobotFirewallRequestError::InvalidPreparedPolicy)
    );
    let path_len = admitted!(write_target(kind, target_storage));
    let body_len = admitted!(write_form(kind, body_storage));
    let headers = admitted!(
        RequestHeaders::new(if body_len == 0 {
            &ACCEPT
        } else {
            &FORM_HEADERS
        })
        .map_err(RobotFirewallRequestError::InvalidHeaders)
    );
    let operation_id = admitted!(
        OperationId::new(id(kind)).map_err(RobotFirewallRequestError::InvalidOperationId)
    );
    if body_len > body_storage.len() {
        sanitize_bytes(target_storage);
        sanitize_bytes(body_storage);
        return Err(RobotFirewallRequestError::Path);
    }
    let target_valid = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .and_then(|text| RequestTarget::new(text).ok())
        .is_some();
    if !target_valid {
        sanitize_bytes(target_storage);
        sanitize_bytes(body_storage);
        return Err(RobotFirewallRequestError::Path);
    }
    let Some(target_text) = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
    else {
        unreachable!("validated Robot firewall target lost UTF-8")
    };
    let Ok(target) = RequestTarget::new(target_text) else {
        unreachable!("validated Robot firewall target became invalid")
    };
    let mut request = TransportRequest::new(method(kind), target).with_headers(headers);
    if body_len != 0 {
        let body = body_storage
            .get(..body_len)
            .unwrap_or_else(|| unreachable!("validated Robot firewall form exceeded storage"));
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
    .unwrap_or_else(|_| unreachable!("prevalidated Robot firewall policy changed during binding"));
    Ok(prepared
        .with_operation_id(operation_id)
        .with_replayable_body())
}

fn write_target(kind: Kind<'_>, output: &mut [u8]) -> Result<usize, RobotFirewallRequestError> {
    let mut len = 0;
    match kind {
        Kind::Get(server) | Kind::Replace(server, _) | Kind::Delete(server) => {
            write_str(
                output,
                &mut len,
                "/firewall/",
                RobotFirewallRequestError::Path,
            )?;
            server.with_number(|server| {
                write_u64(output, &mut len, server, RobotFirewallRequestError::Path)
            })?;
        }
        Kind::TemplateList | Kind::TemplateCreate(_) => {
            write_str(
                output,
                &mut len,
                "/firewall/template",
                RobotFirewallRequestError::Path,
            )?;
        }
        Kind::TemplateGet(template)
        | Kind::TemplateUpdate(template, _)
        | Kind::TemplateDelete(template) => {
            write_str(
                output,
                &mut len,
                "/firewall/template/",
                RobotFirewallRequestError::Path,
            )?;
            write_u64(
                output,
                &mut len,
                template.get(),
                RobotFirewallRequestError::Path,
            )?;
        }
    }
    Ok(len)
}

const fn method(kind: Kind<'_>) -> Method {
    match kind {
        Kind::Get(_) | Kind::TemplateList | Kind::TemplateGet(_) => Method::Get,
        Kind::Replace(_, _) | Kind::TemplateCreate(_) | Kind::TemplateUpdate(_, _) => Method::Post,
        Kind::Delete(_) | Kind::TemplateDelete(_) => Method::Delete,
    }
}

const fn id(kind: Kind<'_>) -> &'static str {
    match kind {
        Kind::Get(_) => "robot_get_firewall",
        Kind::Replace(_, _) => "robot_update_firewall",
        Kind::Delete(_) => "robot_delete_firewall",
        Kind::TemplateList => "robot_list_firewall_templates",
        Kind::TemplateCreate(_) => "robot_create_firewall_template",
        Kind::TemplateGet(_) => "robot_get_firewall_template",
        Kind::TemplateUpdate(_, _) => "robot_update_firewall_template",
        Kind::TemplateDelete(_) => "robot_delete_firewall_template",
    }
}

const fn success_statuses(kind: Kind<'_>) -> &'static [StatusCode] {
    if matches!(kind, Kind::TemplateCreate(_)) {
        CREATED
    } else {
        OK
    }
}

const fn maximum_response_bytes(kind: Kind<'_>) -> usize {
    match kind {
        Kind::TemplateList => MAX_ROBOT_FIREWALL_TEMPLATE_LIST_RESPONSE_BYTES,
        Kind::TemplateDelete(_) => 0,
        _ => MAX_ROBOT_FIREWALL_ITEM_RESPONSE_BYTES,
    }
}

fn metadata(kind: Kind<'_>) -> Result<OperationMetadata, RobotFirewallRequestError> {
    let (impact, semantics, retry) = match kind {
        Kind::Get(_) | Kind::TemplateList | Kind::TemplateGet(_) => (
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
        ),
        Kind::Replace(_, _) | Kind::TemplateCreate(_) | Kind::TemplateUpdate(_, _) => (
            OperationImpact::Mutation,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
        Kind::Delete(_) | Kind::TemplateDelete(_) => (
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
    .map_err(RobotFirewallRequestError::InvalidMetadata)
}
