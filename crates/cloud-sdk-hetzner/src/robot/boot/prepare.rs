use alloc::vec::Vec;

use cloud_sdk::Method;
use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::buffer::{write_byte, write_str};
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
use crate::robot::{RobotForm, RobotFormField, RobotServerNumber};

/// Maximum success-body bytes accepted for one boot response.
pub const MAX_ROBOT_BOOT_RESPONSE_BYTES: usize = 1_048_576;
const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
const ACCEPT: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
const FORM_HEADERS: [RequestHeader<'static>; 2] = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::FORM_URLENCODED),
];

#[derive(Clone, Copy)]
pub(super) enum Kind<'a> {
    Boot(&'a RobotServerNumber),
    RescueGet(&'a RobotServerNumber),
    RescueActivate(&'a RobotRescueActivateRequest<'a>),
    RescueDeactivate(&'a RobotServerNumber),
    RescueLast(&'a RobotServerNumber),
    LinuxGet(&'a RobotServerNumber),
    LinuxActivate(&'a RobotLinuxActivateRequest<'a>),
    LinuxDeactivate(&'a RobotServerNumber),
    LinuxLast(&'a RobotServerNumber),
    VncGet(&'a RobotServerNumber),
    VncActivate(&'a RobotVncActivateRequest<'a>),
    VncDeactivate(&'a RobotServerNumber),
    WindowsGet(&'a RobotServerNumber),
    WindowsActivate(&'a RobotWindowsActivateRequest<'a>),
    WindowsDeactivate(&'a RobotServerNumber),
}

macro_rules! prepare_number {
    ($type:ty, $variant:ident) => {
        impl PrepareOperation for $type {
            type Error = RobotBootRequestError;
            fn prepare<'storage>(
                &self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRequest<'storage>, Self::Error> {
                prepare(Kind::$variant(&self.number), storage)
            }
        }
    };
}

prepare_number!(RobotBootGetRequest, Boot);
prepare_number!(RobotRescueGetRequest, RescueGet);
prepare_number!(RobotRescueDeactivateRequest, RescueDeactivate);
prepare_number!(RobotRescueLastRequest, RescueLast);
prepare_number!(RobotLinuxGetRequest, LinuxGet);
prepare_number!(RobotLinuxDeactivateRequest, LinuxDeactivate);
prepare_number!(RobotLinuxLastRequest, LinuxLast);
prepare_number!(RobotVncGetRequest, VncGet);
prepare_number!(RobotVncDeactivateRequest, VncDeactivate);
prepare_number!(RobotWindowsGetRequest, WindowsGet);
prepare_number!(RobotWindowsDeactivateRequest, WindowsDeactivate);

macro_rules! prepare_activation {
    ($type:ty, $variant:ident) => {
        impl PrepareOperation for $type {
            type Error = RobotBootRequestError;
            fn prepare<'storage>(
                &self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRequest<'storage>, Self::Error> {
                prepare(Kind::$variant(self), storage)
            }
        }
    };
}

prepare_activation!(RobotRescueActivateRequest<'_>, RescueActivate);
prepare_activation!(RobotLinuxActivateRequest<'_>, LinuxActivate);
prepare_activation!(RobotVncActivateRequest<'_>, VncActivate);
prepare_activation!(RobotWindowsActivateRequest<'_>, WindowsActivate);

pub(super) fn prepare<'storage>(
    kind: Kind<'_>,
    storage: PreparationStorage<'storage>,
) -> Result<PreparedRequest<'storage>, RobotBootRequestError> {
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
        admitted!(official_robot_endpoint_policy().map_err(RobotBootRequestError::InvalidEndpoint));
    let identity = admitted!(
        official_robot_endpoint_identity().map_err(RobotBootRequestError::InvalidEndpoint)
    );
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(HETZNER_PROVIDER_ID),
        ScopeRequirement::Required(ROBOT_SERVICE_ID),
        ScopeRequirement::Required(identity),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let service = ProviderService::from_marker::<RobotService>(endpoint);
    let metadata = admitted!(metadata(kind));
    let response = admitted!(
        ResponsePolicy::new(
            OK,
            ContentTypePolicy::Required(JSON),
            ResponseBodyPolicy::Required,
            MAX_ROBOT_BOOT_RESPONSE_BYTES,
        )
        .map_err(RobotBootRequestError::InvalidResponsePolicy)
    );
    let content =
        admitted!(HeaderName::new("content-type").map_err(RobotBootRequestError::InvalidHeaders));
    let raw = admitted!(
        RawResponsePolicy::new(
            MAX_ROBOT_BOOT_RESPONSE_BYTES,
            crate::robot::MAX_ROBOT_ERROR_BODY_BYTES,
            ResponseMediaPolicy::Required(JSON),
            ResponseMediaPolicy::Optional(JSON),
            &[content],
            MAX_INFORMATIONAL_RESPONSES,
        )
        .map_err(RobotBootRequestError::InvalidRawPolicy)
    );
    let target_len = admitted!(write_target(kind, target_storage));
    let body_len = admitted!(write_form(kind, body_storage));
    let headers = admitted!(
        RequestHeaders::new(if body_len == 0 {
            &ACCEPT
        } else {
            &FORM_HEADERS
        })
        .map_err(RobotBootRequestError::InvalidHeaders)
    );
    let operation_id = admitted!(
        OperationId::new(operation_id(kind)).map_err(RobotBootRequestError::InvalidOperationId)
    );
    let target_valid = target_storage
        .get(..target_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .and_then(|text| RequestTarget::new(text).ok())
        .is_some();
    if !target_valid {
        sanitize_bytes(target_storage);
        sanitize_bytes(body_storage);
        return Err(RobotBootRequestError::Path);
    }
    let Some(target_text) = target_storage
        .get(..target_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
    else {
        unreachable!("validated Robot boot target lost UTF-8")
    };
    let Ok(target) = RequestTarget::new(target_text) else {
        unreachable!("validated Robot boot target became invalid")
    };
    let mut request = TransportRequest::new(method(kind), target).with_headers(headers);
    if body_len != 0 {
        let Some(body) = body_storage.get(..body_len) else {
            unreachable!("validated Robot boot form length exceeded storage")
        };
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
    .map_err(RobotBootRequestError::InvalidPreparedPolicy)
    .map(|prepared| {
        prepared
            .with_operation_id(operation_id)
            .with_replayable_body()
    })
}

fn write_target(kind: Kind<'_>, output: &mut [u8]) -> Result<usize, RobotBootRequestError> {
    let mut len = 0;
    write_str(output, &mut len, "/boot/", RobotBootRequestError::Path)?;
    number(kind).with_decimal_bytes(|digits| {
        for digit in digits {
            write_byte(output, &mut len, *digit, RobotBootRequestError::Path)?;
        }
        Ok(())
    })?;
    write_str(output, &mut len, suffix(kind), RobotBootRequestError::Path)?;
    Ok(len)
}

fn write_form(kind: Kind<'_>, output: &mut [u8]) -> Result<usize, RobotBootRequestError> {
    let mut fields = Vec::new();
    let count = match kind {
        Kind::RescueActivate(request) => request
            .keys
            .len()
            .checked_add(2)
            .and_then(|count| count.checked_add(usize::from(request.keyboard.is_some())))
            .ok_or(RobotBootRequestError::Allocation)?,
        Kind::LinuxActivate(request) => request
            .keys
            .len()
            .checked_add(2)
            .ok_or(RobotBootRequestError::Allocation)?,
        Kind::VncActivate(_) | Kind::WindowsActivate(_) => 2,
        _ => return Ok(0),
    };
    fields
        .try_reserve_exact(count)
        .map_err(|_| RobotBootRequestError::Allocation)?;
    match kind {
        Kind::RescueActivate(request) => {
            push(&mut fields, "os", request.os.as_str())?;
            for key in request.keys {
                push(&mut fields, "authorized_key[]", key.as_str())?;
            }
            if let Some(keyboard) = request.keyboard {
                push(&mut fields, "keyboard", keyboard.as_str())?;
            }
        }
        Kind::LinuxActivate(request) => {
            push(&mut fields, "dist", request.distribution.as_str())?;
            push(&mut fields, "lang", request.language.as_str())?;
            for key in request.keys {
                push(&mut fields, "authorized_key[]", key.as_str())?;
            }
        }
        Kind::VncActivate(request) => {
            push(&mut fields, "dist", request.distribution.as_str())?;
            push(&mut fields, "lang", request.language.as_str())?;
        }
        Kind::WindowsActivate(request) => {
            push(&mut fields, "lang", request.language.as_str())?;
            push(&mut fields, "os", request.operating_system.as_str())?;
        }
        _ => unreachable!("non-activation returned before form construction"),
    }
    RobotForm::new(&fields)
        .and_then(|form| form.write_prepared(output))
        .map_err(RobotBootRequestError::Form)
}

fn push<'a>(
    fields: &mut Vec<RobotFormField<'a>>,
    name: &'a str,
    value: &'a str,
) -> Result<(), RobotBootRequestError> {
    fields.push(RobotFormField::sensitive(name, value).map_err(RobotBootRequestError::Form)?);
    Ok(())
}

pub(super) const fn number(kind: Kind<'_>) -> &RobotServerNumber {
    match kind {
        Kind::Boot(number)
        | Kind::RescueGet(number)
        | Kind::RescueDeactivate(number)
        | Kind::RescueLast(number)
        | Kind::LinuxGet(number)
        | Kind::LinuxDeactivate(number)
        | Kind::LinuxLast(number)
        | Kind::VncGet(number)
        | Kind::VncDeactivate(number)
        | Kind::WindowsGet(number)
        | Kind::WindowsDeactivate(number) => number,
        Kind::RescueActivate(request) => &request.number,
        Kind::LinuxActivate(request) => &request.number,
        Kind::VncActivate(request) => &request.number,
        Kind::WindowsActivate(request) => &request.number,
    }
}

const fn suffix(kind: Kind<'_>) -> &'static str {
    match kind {
        Kind::Boot(_) => "",
        Kind::RescueGet(_) | Kind::RescueActivate(_) | Kind::RescueDeactivate(_) => "/rescue",
        Kind::RescueLast(_) => "/rescue/last",
        Kind::LinuxGet(_) | Kind::LinuxActivate(_) | Kind::LinuxDeactivate(_) => "/linux",
        Kind::LinuxLast(_) => "/linux/last",
        Kind::VncGet(_) | Kind::VncActivate(_) | Kind::VncDeactivate(_) => "/vnc",
        Kind::WindowsGet(_) | Kind::WindowsActivate(_) | Kind::WindowsDeactivate(_) => "/windows",
    }
}

pub(super) const fn method(kind: Kind<'_>) -> Method {
    match kind {
        Kind::RescueActivate(_)
        | Kind::LinuxActivate(_)
        | Kind::VncActivate(_)
        | Kind::WindowsActivate(_) => Method::Post,
        Kind::RescueDeactivate(_)
        | Kind::LinuxDeactivate(_)
        | Kind::VncDeactivate(_)
        | Kind::WindowsDeactivate(_) => Method::Delete,
        _ => Method::Get,
    }
}

pub(super) const fn operation_id(kind: Kind<'_>) -> &'static str {
    match kind {
        Kind::Boot(_) => "robot_get_boot",
        Kind::RescueGet(_) => "robot_get_rescue",
        Kind::RescueActivate(_) => "robot_activate_rescue",
        Kind::RescueDeactivate(_) => "robot_deactivate_rescue",
        Kind::RescueLast(_) => "robot_get_last_rescue",
        Kind::LinuxGet(_) => "robot_get_linux",
        Kind::LinuxActivate(_) => "robot_activate_linux",
        Kind::LinuxDeactivate(_) => "robot_deactivate_linux",
        Kind::LinuxLast(_) => "robot_get_last_linux",
        Kind::VncGet(_) => "robot_get_vnc",
        Kind::VncActivate(_) => "robot_activate_vnc",
        Kind::VncDeactivate(_) => "robot_deactivate_vnc",
        Kind::WindowsGet(_) => "robot_get_windows",
        Kind::WindowsActivate(_) => "robot_activate_windows",
        Kind::WindowsDeactivate(_) => "robot_deactivate_windows",
    }
}

fn metadata(kind: Kind<'_>) -> Result<OperationMetadata, RobotBootRequestError> {
    let (impact, semantics, retry) = match kind {
        Kind::Boot(_)
        | Kind::RescueGet(_)
        | Kind::RescueLast(_)
        | Kind::LinuxGet(_)
        | Kind::LinuxLast(_)
        | Kind::VncGet(_)
        | Kind::WindowsGet(_) => (
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
        ),
        Kind::RescueActivate(_)
        | Kind::RescueDeactivate(_)
        | Kind::LinuxDeactivate(_)
        | Kind::VncDeactivate(_)
        | Kind::WindowsDeactivate(_) => (
            OperationImpact::Mutation,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
        Kind::LinuxActivate(_) | Kind::VncActivate(_) | Kind::WindowsActivate(_) => (
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
    .map_err(RobotBootRequestError::InvalidMetadata)
}
