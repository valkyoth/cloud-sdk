use cloud_sdk::Method;
use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::buffer::{SnapshotEncoder, encode_snapshot_bounded, measure_snapshot_bounded};
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

use super::{RobotTrafficRequest, RobotTrafficRequestError, RobotTrafficTarget};
use crate::endpoint::{official_robot_endpoint_identity, official_robot_endpoint_policy};
use crate::identity::{HETZNER_PROVIDER_ID, ROBOT_SERVICE_ID, RobotService};
use crate::robot::MAX_ROBOT_FORM_BODY_BYTES;

/// Maximum accepted success-body bytes for one traffic report.
pub const MAX_ROBOT_TRAFFIC_RESPONSE_BYTES: usize = 8_388_608;

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
const HEADERS: [RequestHeader<'static>; 2] = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::FORM_URLENCODED),
];

impl PrepareOperation for RobotTrafficRequest {
    type Error = RobotTrafficRequestError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        let (target_storage, body_storage) = storage.into_parts();
        sanitize_bytes(target_storage);
        sanitize_bytes(body_storage);
        prepare_inner(self, target_storage, body_storage)
    }
}

fn prepare_inner<'storage>(
    request: &RobotTrafficRequest,
    target_storage: &'storage mut [u8],
    body_storage: &'storage mut [u8],
) -> Result<PreparedRequest<'storage>, RobotTrafficRequestError> {
    let target_bytes = b"/traffic";
    let Some(target_output) = target_storage.get_mut(..target_bytes.len()) else {
        return Err(RobotTrafficRequestError::Storage);
    };
    target_output.copy_from_slice(target_bytes);
    let target_text =
        core::str::from_utf8(target_output).map_err(|_| RobotTrafficRequestError::Storage)?;
    let target =
        RequestTarget::new(target_text).map_err(RobotTrafficRequestError::InvalidTarget)?;
    let body_len = write_form(request, body_storage)?;
    let body = body_storage
        .get(..body_len)
        .ok_or(RobotTrafficRequestError::Storage)?;

    let endpoint =
        official_robot_endpoint_policy().map_err(RobotTrafficRequestError::InvalidEndpoint)?;
    let endpoint_identity =
        official_robot_endpoint_identity().map_err(RobotTrafficRequestError::InvalidEndpoint)?;
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(HETZNER_PROVIDER_ID),
        ScopeRequirement::Required(ROBOT_SERVICE_ID),
        ScopeRequirement::Required(endpoint_identity),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let service = ProviderService::from_marker::<RobotService>(endpoint);
    let metadata = OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Discard,
    )
    .map_err(RobotTrafficRequestError::InvalidMetadata)?;
    let response = ResponsePolicy::new(
        OK,
        ContentTypePolicy::Required(JSON),
        ResponseBodyPolicy::Required,
        MAX_ROBOT_TRAFFIC_RESPONSE_BYTES,
    )
    .map_err(RobotTrafficRequestError::InvalidResponsePolicy)?;
    let content =
        HeaderName::new("content-type").map_err(RobotTrafficRequestError::InvalidHeaders)?;
    let raw = RawResponsePolicy::new(
        MAX_ROBOT_TRAFFIC_RESPONSE_BYTES,
        crate::robot::MAX_ROBOT_ERROR_BODY_BYTES,
        ResponseMediaPolicy::Required(JSON),
        ResponseMediaPolicy::Optional(JSON),
        &[content],
        MAX_INFORMATIONAL_RESPONSES,
    )
    .map_err(RobotTrafficRequestError::InvalidRawPolicy)?;
    let headers =
        RequestHeaders::new(&HEADERS).map_err(RobotTrafficRequestError::InvalidHeaders)?;
    let operation_id = OperationId::new("robot_get_traffic")
        .map_err(RobotTrafficRequestError::InvalidOperationId)?;
    let transport = TransportRequest::new(Method::Post, target)
        .with_headers(headers)
        .with_body(body);
    PreparedRequest::new_read_only_post_query(
        transport,
        service,
        metadata,
        response,
        authentication,
        raw,
        RequestBodySensitivity::Sensitive,
    )
    .map_err(RobotTrafficRequestError::InvalidPreparedPolicy)
    .map(|prepared| {
        prepared
            .with_operation_id(operation_id)
            .with_replayable_body()
    })
}

fn write_form(
    request: &RobotTrafficRequest,
    output: &mut [u8],
) -> Result<usize, RobotTrafficRequestError> {
    let required = measure_snapshot_bounded(
        request,
        MAX_ROBOT_FORM_BODY_BYTES,
        RobotTrafficRequestError::Storage,
        encode_form,
    )?;
    if output.len() < required {
        return Err(RobotTrafficRequestError::Storage);
    }
    sanitize_bytes(output);
    encode_snapshot_bounded(
        request,
        output,
        MAX_ROBOT_FORM_BODY_BYTES,
        RobotTrafficRequestError::Storage,
        encode_form,
    )
}

fn encode_form(
    request: &RobotTrafficRequest,
    encoder: &mut SnapshotEncoder<'_, RobotTrafficRequestError>,
) -> Result<(), RobotTrafficRequestError> {
    let mut first = true;
    for target in &request.targets {
        target.with_text(|value| {
            field(
                encoder,
                &mut first,
                match target {
                    RobotTrafficTarget::Ip(_) => "ip[]",
                    RobotTrafficTarget::Subnet(_) => "subnet[]",
                },
                value,
            )
        })?;
    }
    request
        .interval
        .with_from(|value| field(encoder, &mut first, "from", value))?;
    request
        .interval
        .with_to(|value| field(encoder, &mut first, "to", value))?;
    field(
        encoder,
        &mut first,
        "type",
        request.interval.granularity().wire_name(),
    )?;
    if request.single_values {
        field(encoder, &mut first, "single_values", "true")?;
    }
    Ok(())
}

fn field(
    encoder: &mut SnapshotEncoder<'_, RobotTrafficRequestError>,
    first: &mut bool,
    name: &str,
    value: &str,
) -> Result<(), RobotTrafficRequestError> {
    if *first {
        *first = false;
    } else {
        encoder.byte(b'&')?;
    }
    encoder.form_component(name)?;
    encoder.byte(b'=')?;
    encoder.form_component(value)
}
