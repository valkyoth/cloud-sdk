use cloud_sdk::Method;
use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::buffer::{write_query_pair, write_str, write_u64};
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
const MAX_SUCCESS_BYTES: usize = 8_388_608;
const ACCEPT: [RequestHeader<'static>; 1] = [RequestHeader::accept(MediaType::JSON)];
const FORM_HEADERS: [RequestHeader<'static>; 2] = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::FORM_URLENCODED),
];

#[derive(Clone, Copy)]
pub(super) enum Kind<'a> {
    List(Option<&'a RobotIpAddress>),
    Get(&'a RobotIpAddress),
    Update(&'a RobotIpAddress, RobotIpTrafficUpdate),
    GetMac(&'a RobotIpAddress),
    SetMac(&'a RobotIpAddress),
    DeleteMac(&'a RobotIpAddress),
}

macro_rules! prepare_operation {
    ($type:ty, $kind:ident, $($field:ident),+ $(,)?) => {
        impl PrepareOperation for $type {
            type Error = RobotIpRequestError;
            fn prepare<'storage>(
                &self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRequest<'storage>, Self::Error> {
                prepare(Kind::$kind($(&self.$field),+), storage)
            }
        }
    };
}

impl PrepareOperation for RobotIpListRequest {
    type Error = RobotIpRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::List(self.server_address.as_ref()), storage)
    }
}
prepare_operation!(RobotIpGetRequest, Get, address);
impl PrepareOperation for RobotIpUpdateRequest {
    type Error = RobotIpRequestError;
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        prepare(Kind::Update(&self.address, self.update), storage)
    }
}
prepare_operation!(RobotIpMacGetRequest, GetMac, address);
prepare_operation!(RobotIpMacSetRequest, SetMac, address);
prepare_operation!(RobotIpMacDeleteRequest, DeleteMac, address);

fn prepare<'storage>(
    kind: Kind<'_>,
    storage: PreparationStorage<'storage>,
) -> Result<PreparedRequest<'storage>, RobotIpRequestError> {
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
        admitted!(official_robot_endpoint_policy().map_err(RobotIpRequestError::InvalidEndpoint));
    let endpoint_identity =
        admitted!(official_robot_endpoint_identity().map_err(RobotIpRequestError::InvalidEndpoint));
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
    let response = admitted!(
        ResponsePolicy::new(
            OK,
            ContentTypePolicy::Required(JSON),
            ResponseBodyPolicy::Required,
            MAX_SUCCESS_BYTES,
        )
        .map_err(RobotIpRequestError::InvalidResponsePolicy)
    );
    let content =
        admitted!(HeaderName::new("content-type").map_err(RobotIpRequestError::InvalidHeaders));
    let raw = admitted!(
        RawResponsePolicy::new(
            MAX_SUCCESS_BYTES,
            crate::robot::MAX_ROBOT_ERROR_BODY_BYTES,
            ResponseMediaPolicy::Required(JSON),
            ResponseMediaPolicy::Optional(JSON),
            &[content],
            MAX_INFORMATIONAL_RESPONSES,
        )
        .map_err(RobotIpRequestError::InvalidRawPolicy)
    );
    let path_len = admitted!(write_target(kind, target_storage));
    let body_len = match kind {
        Kind::Update(_, update) => admitted!(write_update_form(update, body_storage)),
        _ => 0,
    };
    let headers = admitted!(
        RequestHeaders::new(if body_len == 0 {
            &ACCEPT
        } else {
            &FORM_HEADERS
        })
        .map_err(RobotIpRequestError::InvalidHeaders)
    );
    let operation_id =
        admitted!(OperationId::new(id(kind)).map_err(RobotIpRequestError::InvalidOperationId));
    let target_valid = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .and_then(|text| RequestTarget::new(text).ok())
        .is_some();
    if !target_valid {
        sanitize_bytes(target_storage);
        sanitize_bytes(body_storage);
        return Err(RobotIpRequestError::Path);
    }
    let Some(target_text) = target_storage
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
    else {
        unreachable!("validated Robot IP target lost UTF-8")
    };
    let Ok(target) = RequestTarget::new(target_text) else {
        unreachable!("validated Robot IP target became invalid")
    };
    let mut request = TransportRequest::new(method(kind), target).with_headers(headers);
    if body_len != 0 {
        let body = body_storage
            .get(..body_len)
            .ok_or(RobotIpRequestError::Path)?;
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
    .map_err(RobotIpRequestError::InvalidPreparedPolicy)?;
    Ok(prepared
        .with_operation_id(operation_id)
        .with_replayable_body())
}

fn write_target(kind: Kind<'_>, output: &mut [u8]) -> Result<usize, RobotIpRequestError> {
    let mut len = 0;
    write_str(output, &mut len, "/ip", RobotIpRequestError::Path)?;
    match kind {
        Kind::List(Some(server)) => server.with_text(|text| {
            write_str(output, &mut len, "?", RobotIpRequestError::Path)?;
            let mut first = true;
            write_query_pair(
                output,
                &mut len,
                &mut first,
                "server_ip",
                text,
                RobotIpRequestError::Path,
            )
        })?,
        Kind::List(None) => {}
        _ => {
            write_str(output, &mut len, "/", RobotIpRequestError::Path)?;
            address(kind)
                .with_text(|text| write_str(output, &mut len, text, RobotIpRequestError::Path))?;
            if matches!(kind, Kind::GetMac(_) | Kind::SetMac(_) | Kind::DeleteMac(_)) {
                write_str(output, &mut len, "/mac", RobotIpRequestError::Path)?;
            }
        }
    }
    Ok(len)
}

fn write_update_form(
    update: RobotIpTrafficUpdate,
    output: &mut [u8],
) -> Result<usize, RobotIpRequestError> {
    let warnings = if update.warnings == Some(true) {
        "true"
    } else {
        "false"
    };
    let hourly = DecimalText::new(update.hourly.unwrap_or(0));
    let daily = DecimalText::new(update.daily.unwrap_or(0));
    let monthly = DecimalText::new(update.monthly.unwrap_or(0));
    let placeholder = RobotFormField::sensitive("traffic_warnings", warnings)
        .map_err(RobotIpRequestError::Form)?;
    let mut fields = [placeholder; 4];
    let mut len = 0;
    macro_rules! push {
        ($name:literal, $value:expr) => {{
            let field =
                RobotFormField::sensitive($name, $value).map_err(RobotIpRequestError::Form)?;
            *fields.get_mut(len).ok_or(RobotIpRequestError::Path)? = field;
            len = len.checked_add(1).ok_or(RobotIpRequestError::Path)?;
        }};
    }
    if update.warnings.is_some() {
        push!("traffic_warnings", warnings);
    }
    if update.hourly.is_some() {
        push!("traffic_hourly", hourly.as_str());
    }
    if update.daily.is_some() {
        push!("traffic_daily", daily.as_str());
    }
    if update.monthly.is_some() {
        push!("traffic_monthly", monthly.as_str());
    }
    let selected = fields.get(..len).ok_or(RobotIpRequestError::Path)?;
    RobotForm::new(selected)
        .and_then(|form| form.write_prepared(output))
        .map_err(RobotIpRequestError::Form)
}

struct DecimalText {
    bytes: [u8; 20],
    len: usize,
}

impl DecimalText {
    fn new(value: u64) -> Self {
        let mut bytes = [0_u8; 20];
        let mut len = 0;
        write_u64(&mut bytes, &mut len, value, ())
            .unwrap_or_else(|()| unreachable!("u64 decimal text exceeded 20 bytes"));
        Self { bytes, len }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default())
            .unwrap_or_else(|_| unreachable!("decimal form text lost UTF-8"))
    }
}

fn address(kind: Kind<'_>) -> &RobotIpAddress {
    match kind {
        Kind::Get(value)
        | Kind::Update(value, _)
        | Kind::GetMac(value)
        | Kind::SetMac(value)
        | Kind::DeleteMac(value) => value,
        Kind::List(_) => unreachable!("list has no path address"),
    }
}

const fn method(kind: Kind<'_>) -> Method {
    match kind {
        Kind::List(_) | Kind::Get(_) | Kind::GetMac(_) => Method::Get,
        Kind::Update(_, _) => Method::Post,
        Kind::SetMac(_) => Method::Put,
        Kind::DeleteMac(_) => Method::Delete,
    }
}

const fn id(kind: Kind<'_>) -> &'static str {
    match kind {
        Kind::List(_) => "robot_list_ips",
        Kind::Get(_) => "robot_get_ip",
        Kind::Update(_, _) => "robot_update_ip",
        Kind::GetMac(_) => "robot_get_ip_mac",
        Kind::SetMac(_) => "robot_set_ip_mac",
        Kind::DeleteMac(_) => "robot_delete_ip_mac",
    }
}

fn metadata(kind: Kind<'_>) -> Result<OperationMetadata, RobotIpRequestError> {
    let (impact, semantics, retry) = match kind {
        Kind::List(_) | Kind::Get(_) | Kind::GetMac(_) => (
            OperationImpact::ReadOnly,
            RequestSemantics::Safe,
            RetryEligibility::ExplicitPolicy,
        ),
        Kind::Update(_, _) => (
            OperationImpact::Mutation,
            RequestSemantics::Idempotent,
            RetryEligibility::ExplicitPolicy,
        ),
        Kind::SetMac(_) => (
            OperationImpact::Mutation,
            RequestSemantics::NonIdempotent,
            RetryEligibility::Never,
        ),
        Kind::DeleteMac(_) => (
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
    .map_err(RobotIpRequestError::InvalidMetadata)
}
