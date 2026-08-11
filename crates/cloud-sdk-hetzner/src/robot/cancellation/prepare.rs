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
use super::{RobotCancellationSchedule, RobotIpAddress, RobotSubnetAddress};
use crate::endpoint::{official_robot_endpoint_identity, official_robot_endpoint_policy};
use crate::identity::{HETZNER_PROVIDER_ID, ROBOT_SERVICE_ID, RobotService};
use crate::robot::server::RobotServerNumber;
use crate::robot::{RobotForm, RobotFormField};

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
const MAX_SUCCESS_BYTES: usize = 65_536;
const ACCEPT: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
const FORM_HEADERS: [RequestHeader<'static>; 2] = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::FORM_URLENCODED),
];

#[derive(Clone, Copy)]
pub(super) enum Target<'a> {
    Server(&'a RobotServerNumber),
    Ip(&'a RobotIpAddress),
    Subnet(&'a RobotSubnetAddress),
}

#[derive(Clone, Copy)]
pub(super) enum Kind<'a> {
    Get(Target<'a>),
    CreateServer(&'a RobotServerNumber),
    CreateIp(&'a RobotIpAddress),
    CreateSubnet(&'a RobotSubnetAddress),
    Delete(Target<'a>),
}

impl PrepareOperation for RobotServerCancellationGetRequest {
    type Error = RobotCancellationRequestError;
    fn prepare<'s>(
        &self,
        storage: PreparationStorage<'s>,
    ) -> Result<PreparedRequest<'s>, Self::Error> {
        prepare(Kind::Get(Target::Server(&self.number)), None, None, storage)
    }
}
impl PrepareOperation for RobotServerCancellationCreateRequest<'_> {
    type Error = RobotCancellationRequestError;
    fn prepare<'s>(
        &self,
        storage: PreparationStorage<'s>,
    ) -> Result<PreparedRequest<'s>, Self::Error> {
        prepare(
            Kind::CreateServer(&self.number),
            Some(&self.schedule),
            Some(self),
            storage,
        )
    }
}
impl PrepareOperation for RobotServerCancellationDeleteRequest {
    type Error = RobotCancellationRequestError;
    fn prepare<'s>(
        &self,
        storage: PreparationStorage<'s>,
    ) -> Result<PreparedRequest<'s>, Self::Error> {
        prepare(
            Kind::Delete(Target::Server(&self.number)),
            None,
            None,
            storage,
        )
    }
}

macro_rules! address_prepare {
    ($get:ident, $create:ident, $delete:ident, $field:ident, $target:ident, $kind:ident) => {
        impl PrepareOperation for $get {
            type Error = RobotCancellationRequestError;
            fn prepare<'s>(
                &self,
                storage: PreparationStorage<'s>,
            ) -> Result<PreparedRequest<'s>, Self::Error> {
                prepare(
                    Kind::Get(Target::$target(&self.$field)),
                    None,
                    None,
                    storage,
                )
            }
        }
        impl PrepareOperation for $create {
            type Error = RobotCancellationRequestError;
            fn prepare<'s>(
                &self,
                storage: PreparationStorage<'s>,
            ) -> Result<PreparedRequest<'s>, Self::Error> {
                prepare(
                    Kind::$kind(&self.$field),
                    Some(&self.schedule),
                    None,
                    storage,
                )
            }
        }
        impl PrepareOperation for $delete {
            type Error = RobotCancellationRequestError;
            fn prepare<'s>(
                &self,
                storage: PreparationStorage<'s>,
            ) -> Result<PreparedRequest<'s>, Self::Error> {
                prepare(
                    Kind::Delete(Target::$target(&self.$field)),
                    None,
                    None,
                    storage,
                )
            }
        }
    };
}

address_prepare!(
    RobotIpCancellationGetRequest,
    RobotIpCancellationCreateRequest,
    RobotIpCancellationDeleteRequest,
    ip,
    Ip,
    CreateIp
);
address_prepare!(
    RobotSubnetCancellationGetRequest,
    RobotSubnetCancellationCreateRequest,
    RobotSubnetCancellationDeleteRequest,
    subnet,
    Subnet,
    CreateSubnet
);

fn prepare<'s>(
    kind: Kind<'_>,
    schedule: Option<&RobotCancellationSchedule>,
    server: Option<&RobotServerCancellationCreateRequest<'_>>,
    storage: PreparationStorage<'s>,
) -> Result<PreparedRequest<'s>, RobotCancellationRequestError> {
    let (target_storage, body_storage) = storage.into_parts();
    sanitize_bytes(target_storage);
    sanitize_bytes(body_storage);
    prepare_inner(kind, schedule, server, target_storage, body_storage)
}

fn prepare_inner<'s>(
    kind: Kind<'_>,
    schedule: Option<&RobotCancellationSchedule>,
    server: Option<&RobotServerCancellationCreateRequest<'_>>,
    target_storage: &'s mut [u8],
    body_storage: &'s mut [u8],
) -> Result<PreparedRequest<'s>, RobotCancellationRequestError> {
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
        official_robot_endpoint_policy().map_err(RobotCancellationRequestError::InvalidEndpoint)?;
    let service = ProviderService::from_marker::<RobotService>(endpoint);
    let endpoint_identity = official_robot_endpoint_identity()
        .map_err(RobotCancellationRequestError::InvalidEndpoint)?;
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(HETZNER_PROVIDER_ID),
        ScopeRequirement::Required(ROBOT_SERVICE_ID),
        ScopeRequirement::Required(endpoint_identity),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let metadata = metadata(kind)?;
    let empty_success = matches!(kind, Kind::Delete(Target::Server(_)));
    let response = ResponsePolicy::new(
        OK,
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
        if empty_success { 0 } else { MAX_SUCCESS_BYTES },
    )
    .map_err(RobotCancellationRequestError::InvalidResponsePolicy)?;
    let content =
        HeaderName::new("content-type").map_err(RobotCancellationRequestError::InvalidHeaders)?;
    let raw = RawResponsePolicy::new(
        if empty_success { 0 } else { MAX_SUCCESS_BYTES },
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
    .map_err(RobotCancellationRequestError::InvalidRawPolicy)?;
    let path_len = admitted!(write_path(kind, target_storage));
    let target_valid = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .and_then(|text| RequestTarget::new(text).ok())
        .is_some();
    if !target_valid {
        sanitize_bytes(target_storage);
        sanitize_bytes(body_storage);
        return Err(RobotCancellationRequestError::Path);
    }
    let body_len = match schedule {
        None => 0,
        Some(value) => admitted!(write_form(value, server, body_storage)),
    };
    let headers = admitted!(
        RequestHeaders::new(if body_len == 0 {
            &ACCEPT
        } else {
            &FORM_HEADERS
        })
        .map_err(RobotCancellationRequestError::InvalidHeaders)
    );
    let Some(target_text) = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
    else {
        unreachable!("validated Robot cancellation target lost UTF-8")
    };
    let Ok(target) = RequestTarget::new(target_text) else {
        unreachable!("validated Robot cancellation target became invalid")
    };
    let mut request = TransportRequest::new(method(kind), target).with_headers(headers);
    if body_len != 0 {
        let body = body_storage
            .get(..body_len)
            .ok_or(RobotCancellationRequestError::Path)?;
        request = request.with_body(body);
    }
    let operation_id =
        OperationId::new(id(kind)).map_err(RobotCancellationRequestError::InvalidOperationId)?;
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
    .map_err(RobotCancellationRequestError::InvalidPreparedPolicy)?;
    Ok(prepared
        .with_operation_id(operation_id)
        .with_replayable_body())
}

fn write_form(
    schedule: &RobotCancellationSchedule,
    server: Option<&RobotServerCancellationCreateRequest<'_>>,
    output: &mut [u8],
) -> Result<usize, RobotCancellationRequestError> {
    let mut write = |date: &str| {
        let mut fields = [RobotFormField::sensitive("cancellation_date", date)
            .map_err(RobotCancellationRequestError::Form)?; 3];
        let mut len = 1;
        if let Some(request) = server {
            if let Some(reason) = request.reason {
                let field = RobotFormField::sensitive("cancellation_reason", reason.as_str())
                    .map_err(RobotCancellationRequestError::Form)?;
                *fields
                    .get_mut(len)
                    .ok_or(RobotCancellationRequestError::Path)? = field;
                len = len
                    .checked_add(1)
                    .ok_or(RobotCancellationRequestError::Path)?;
            }
            if request.reservation != RobotLocationReservationIntent::Omit {
                let value = if request.reservation == RobotLocationReservationIntent::Reserve {
                    "true"
                } else {
                    "false"
                };
                let field = RobotFormField::sensitive("reserve_location", value)
                    .map_err(RobotCancellationRequestError::Form)?;
                *fields
                    .get_mut(len)
                    .ok_or(RobotCancellationRequestError::Path)? = field;
                len = len
                    .checked_add(1)
                    .ok_or(RobotCancellationRequestError::Path)?;
            }
        }
        let selected = fields
            .get(..len)
            .ok_or(RobotCancellationRequestError::Path)?;
        RobotForm::new(selected)
            .and_then(|form| form.write_prepared(output))
            .map_err(RobotCancellationRequestError::Form)
    };
    match schedule {
        RobotCancellationSchedule::Immediate => write("now"),
        RobotCancellationSchedule::On(date) => date.with_text(write),
    }
}

fn write_path(kind: Kind<'_>, output: &mut [u8]) -> Result<usize, RobotCancellationRequestError> {
    let target = target(kind);
    let mut len = 0;
    match target {
        Target::Server(number) => {
            write_str(
                output,
                &mut len,
                "/server/",
                RobotCancellationRequestError::Path,
            )?;
            number.with_decimal_bytes(|value| write_bytes(output, &mut len, value))?;
        }
        Target::Ip(ip) => {
            write_str(
                output,
                &mut len,
                "/ip/",
                RobotCancellationRequestError::Path,
            )?;
            ip.with_text(|value| {
                write_str(output, &mut len, value, RobotCancellationRequestError::Path)
            })?;
        }
        Target::Subnet(subnet) => {
            write_str(
                output,
                &mut len,
                "/subnet/",
                RobotCancellationRequestError::Path,
            )?;
            subnet.with_text(|value| {
                write_str(output, &mut len, value, RobotCancellationRequestError::Path)
            })?;
        }
    }
    write_str(
        output,
        &mut len,
        "/cancellation",
        RobotCancellationRequestError::Path,
    )?;
    Ok(len)
}

fn write_bytes(
    output: &mut [u8],
    len: &mut usize,
    value: &[u8],
) -> Result<(), RobotCancellationRequestError> {
    for byte in value {
        cloud_sdk::buffer::write_byte(output, len, *byte, RobotCancellationRequestError::Path)?;
    }
    Ok(())
}

const fn target(kind: Kind<'_>) -> Target<'_> {
    match kind {
        Kind::Get(value) | Kind::Delete(value) => value,
        Kind::CreateServer(value) => Target::Server(value),
        Kind::CreateIp(value) => Target::Ip(value),
        Kind::CreateSubnet(value) => Target::Subnet(value),
    }
}
const fn method(kind: Kind<'_>) -> Method {
    match kind {
        Kind::Get(_) => Method::Get,
        Kind::Delete(_) => Method::Delete,
        _ => Method::Post,
    }
}
const fn id(kind: Kind<'_>) -> &'static str {
    match kind {
        Kind::Get(Target::Server(_)) => "robot_get_server_cancellation",
        Kind::CreateServer(_) => "robot_create_server_cancellation",
        Kind::Delete(Target::Server(_)) => "robot_delete_server_cancellation",
        Kind::Get(Target::Ip(_)) => "robot_get_ip_cancellation",
        Kind::CreateIp(_) => "robot_create_ip_cancellation",
        Kind::Delete(Target::Ip(_)) => "robot_delete_ip_cancellation",
        Kind::Get(Target::Subnet(_)) => "robot_get_subnet_cancellation",
        Kind::CreateSubnet(_) => "robot_create_subnet_cancellation",
        Kind::Delete(Target::Subnet(_)) => "robot_delete_subnet_cancellation",
    }
}
fn metadata(kind: Kind<'_>) -> Result<OperationMetadata, RobotCancellationRequestError> {
    let (impact, semantics, retry) = match kind {
        Kind::Get(_) => (
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
        ),
        Kind::CreateServer(_) | Kind::CreateIp(_) | Kind::CreateSubnet(_) => (
            OperationImpact::Destructive,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
        Kind::Delete(_) => (
            OperationImpact::Destructive,
            RequestSemantics::Idempotent,
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
    .map_err(RobotCancellationRequestError::InvalidMetadata)
}
