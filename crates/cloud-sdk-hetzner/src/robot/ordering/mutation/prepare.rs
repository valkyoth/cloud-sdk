use cloud_sdk::Method;
use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::buffer::{SnapshotEncoder, encode_snapshot_bounded};
use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationImpact, OperationMetadata,
    PreparationStorage, PreparedRequest, ProviderService, RequestBodySensitivity, RequestIdPolicy,
    RequestSemantics, ResponseBodyPolicy, ResponsePolicy, RetryEligibility,
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

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const CREATED: &[StatusCode] = &[StatusCode::CREATED];
const HEADERS: [RequestHeader<'static>; 2] = [
    RequestHeader::accept(MediaType::JSON),
    RequestHeader::content_type(ContentType::FORM_URLENCODED),
];
const MAX_TARGET_BYTES: usize = 128;
/// Maximum accepted successful order response bytes.
pub const MAX_ROBOT_ORDER_MUTATION_RESPONSE_BYTES: usize = 1_048_576;

#[derive(Clone, Copy)]
pub(super) enum Kind<'a> {
    Standard(&'a RobotStandardOrderCreateRequest<'a>),
    Market(&'a RobotMarketOrderCreateRequest<'a>),
    Addon(&'a RobotAddonOrderCreateRequest<'a, 'a>),
}

pub(super) fn prepare<'storage>(
    kind: Kind<'_>,
    storage: PreparationStorage<'storage>,
) -> Result<PreparedRequest<'storage>, RobotOrderMutationRequestError> {
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
        official_robot_endpoint_policy().map_err(RobotOrderMutationRequestError::InvalidEndpoint)
    );
    let endpoint_identity = admitted!(
        official_robot_endpoint_identity().map_err(RobotOrderMutationRequestError::InvalidEndpoint)
    );
    let authentication = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(HETZNER_PROVIDER_ID),
        ScopeRequirement::Required(ROBOT_SERVICE_ID),
        ScopeRequirement::Required(endpoint_identity),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let metadata = admitted!(metadata());
    let response = admitted!(
        ResponsePolicy::new(
            CREATED,
            ContentTypePolicy::Required(JSON),
            ResponseBodyPolicy::Required,
            MAX_ROBOT_ORDER_MUTATION_RESPONSE_BYTES,
        )
        .map_err(RobotOrderMutationRequestError::InvalidResponsePolicy)
    );
    let content = admitted!(
        HeaderName::new("content-type").map_err(RobotOrderMutationRequestError::InvalidHeaders)
    );
    let raw = admitted!(
        RawResponsePolicy::new(
            MAX_ROBOT_ORDER_MUTATION_RESPONSE_BYTES,
            crate::robot::MAX_ROBOT_ERROR_BODY_BYTES,
            ResponseMediaPolicy::Required(JSON),
            ResponseMediaPolicy::Optional(JSON),
            &[content],
            MAX_INFORMATIONAL_RESPONSES,
        )
        .map_err(RobotOrderMutationRequestError::InvalidRawPolicy)
    );
    admitted!(
        PreparedRequest::validate_construction_policy(Method::Post, metadata, raw)
            .map_err(RobotOrderMutationRequestError::InvalidPreparedPolicy)
    );
    admitted!(validate_field_count(kind));
    let target_len = admitted!(encode_snapshot_bounded(
        kind,
        target_storage,
        MAX_TARGET_BYTES,
        RobotOrderMutationRequestError::Target,
        encode_target,
    ));
    let body_len = admitted!(encode_snapshot_bounded(
        kind,
        body_storage,
        crate::robot::MAX_ROBOT_FORM_BODY_BYTES,
        RobotOrderMutationRequestError::Form(crate::robot::RobotFormError::BodyTooLong),
        encode_form,
    ));
    let headers = admitted!(
        RequestHeaders::new(&HEADERS).map_err(RobotOrderMutationRequestError::InvalidHeaders)
    );
    let operation = admitted!(
        OperationId::new(operation_id(kind))
            .map_err(RobotOrderMutationRequestError::InvalidOperationId)
    );
    admitted!(validate_target(target_storage, target_len));

    // Stable Rust cannot conditionally recover the mutable target after a
    // successful branch returns a request borrowing it (rust-lang/rust#54663).
    // Guarded preparation owns cleanup if these prevalidated invariants drift.
    let target_text = target_storage
        .get(..target_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .ok_or(RobotOrderMutationRequestError::Target)?;
    let target =
        RequestTarget::new(target_text).map_err(RobotOrderMutationRequestError::InvalidTarget)?;
    let body = body_storage.get(..body_len).unwrap_or_default();
    let request = TransportRequest::new(Method::Post, target)
        .with_headers(headers)
        .with_body(body);
    let service = ProviderService::from_marker::<RobotService>(endpoint);
    let prepared = PreparedRequest::new(
        request,
        service,
        metadata,
        response,
        authentication,
        raw,
        RequestBodySensitivity::Sensitive,
    )
    .unwrap_or_else(|_| unreachable!("prevalidated Robot order policy changed during binding"));
    Ok(prepared.with_operation_id(operation).with_replayable_body())
}

fn validate_field_count(kind: Kind<'_>) -> Result<(), RobotOrderMutationRequestError> {
    let count = match kind {
        Kind::Standard(request) => request
            .plan
            .addons()
            .iter()
            .try_fold(4_u64, |total, addon| total.checked_add(addon.quantity())),
        Kind::Market(_) => Some(3),
        Kind::Addon(request) => Some(match request.parameters {
            RobotAddonOrderParameters::Ip { .. } => 3,
            RobotAddonOrderParameters::Subnet {
                gateway: Some(_), ..
            } => 4,
            RobotAddonOrderParameters::Subnet { gateway: None, .. } => 3,
            RobotAddonOrderParameters::Other => 2,
        }),
    };
    if count.is_some_and(|value| value <= crate::robot::MAX_ROBOT_FORM_FIELDS as u64) {
        Ok(())
    } else {
        Err(RobotOrderMutationRequestError::Form(
            crate::robot::RobotFormError::TooManyFields,
        ))
    }
}

fn validate_target(
    target_storage: &[u8],
    target_len: usize,
) -> Result<(), RobotOrderMutationRequestError> {
    let target_text = target_storage
        .get(..target_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .ok_or(RobotOrderMutationRequestError::Target)?;
    RequestTarget::new(target_text)
        .map(|_| ())
        .map_err(RobotOrderMutationRequestError::InvalidTarget)
}

fn encode_target(
    kind: Kind<'_>,
    encoder: &mut SnapshotEncoder<'_, RobotOrderMutationRequestError>,
) -> Result<(), RobotOrderMutationRequestError> {
    encoder.string(match kind {
        Kind::Standard(_) => "/order/server/transaction",
        Kind::Market(_) => "/order/server_market/transaction",
        Kind::Addon(_) => "/order/server_addon/transaction",
    })
}

fn encode_form(
    kind: Kind<'_>,
    encoder: &mut SnapshotEncoder<'_, RobotOrderMutationRequestError>,
) -> Result<(), RobotOrderMutationRequestError> {
    let mut first = true;
    match kind {
        Kind::Standard(request) => {
            let plan = request.plan;
            plan.product()
                .id()
                .with_text(|value| pair(encoder, &mut first, "product_id", value))?;
            plan.distribution()
                .with_text(|value| pair(encoder, &mut first, "dist", value))?;
            plan.language()
                .with_text(|value| pair(encoder, &mut first, "lang", value))?;
            plan.price()
                .location()
                .with_text(|value| pair(encoder, &mut first, "location", value))?;
            for addon in plan.addons() {
                for _ in 0..addon.quantity() {
                    addon
                        .addon()
                        .id()
                        .with_text(|value| pair(encoder, &mut first, "addon[]", value))?;
                }
            }
            Ok(())
        }
        Kind::Market(request) => {
            pair_u64(
                encoder,
                &mut first,
                "product_id",
                request.plan.product().id().get(),
            )?;
            request
                .plan
                .distribution()
                .with_text(|value| pair(encoder, &mut first, "dist", value))?;
            request
                .plan
                .language()
                .with_text(|value| pair(encoder, &mut first, "lang", value))
        }
        Kind::Addon(request) => {
            request.plan.server().with_decimal_bytes(|value| {
                pair_bytes(encoder, &mut first, "server_number", value)
            })?;
            request
                .plan
                .product()
                .id()
                .with_text(|value| pair(encoder, &mut first, "product_id", value))?;
            match request.parameters {
                RobotAddonOrderParameters::Ip { reason } => {
                    pair(encoder, &mut first, "reason", reason.as_str())
                }
                RobotAddonOrderParameters::Subnet { reason, gateway } => {
                    pair(encoder, &mut first, "reason", reason.as_str())?;
                    if let Some(gateway) = gateway {
                        pair_ipv4(encoder, &mut first, "gateway", gateway)?;
                    }
                    Ok(())
                }
                RobotAddonOrderParameters::Other => Ok(()),
            }
        }
    }
}

fn prefix(
    encoder: &mut SnapshotEncoder<'_, RobotOrderMutationRequestError>,
    first: &mut bool,
    name: &str,
) -> Result<(), RobotOrderMutationRequestError> {
    if *first {
        *first = false;
    } else {
        encoder.byte(b'&')?;
    }
    encoder.form_component(name)?;
    encoder.byte(b'=')
}

fn pair(
    encoder: &mut SnapshotEncoder<'_, RobotOrderMutationRequestError>,
    first: &mut bool,
    name: &str,
    value: &str,
) -> Result<(), RobotOrderMutationRequestError> {
    prefix(encoder, first, name)?;
    encoder.form_component(value)
}

fn pair_bytes(
    encoder: &mut SnapshotEncoder<'_, RobotOrderMutationRequestError>,
    first: &mut bool,
    name: &str,
    value: &[u8],
) -> Result<(), RobotOrderMutationRequestError> {
    prefix(encoder, first, name)?;
    encoder.bytes(value)
}

fn pair_u64(
    encoder: &mut SnapshotEncoder<'_, RobotOrderMutationRequestError>,
    first: &mut bool,
    name: &str,
    value: u64,
) -> Result<(), RobotOrderMutationRequestError> {
    prefix(encoder, first, name)?;
    encoder.u64(value)
}

fn pair_ipv4(
    encoder: &mut SnapshotEncoder<'_, RobotOrderMutationRequestError>,
    first: &mut bool,
    name: &str,
    value: core::net::Ipv4Addr,
) -> Result<(), RobotOrderMutationRequestError> {
    prefix(encoder, first, name)?;
    let octets = value.octets();
    for (index, octet) in octets.iter().enumerate() {
        if index != 0 {
            encoder.byte(b'.')?;
        }
        encoder.u64(u64::from(*octet))?;
    }
    Ok(())
}

const fn operation_id(kind: Kind<'_>) -> &'static str {
    match kind {
        Kind::Standard(_) => "robot_create_server_transaction",
        Kind::Market(_) => "robot_create_server_market_transaction",
        Kind::Addon(_) => "robot_create_server_addon_transaction",
    }
}

fn metadata() -> Result<OperationMetadata, RobotOrderMutationRequestError> {
    OperationMetadata::new(
        OperationImpact::Mutation,
        RequestSemantics::NonIdempotent,
        RetryEligibility::Never,
        CostIntent::MayIncurCost,
        RequestIdPolicy::Discard,
    )
    .map_err(RobotOrderMutationRequestError::InvalidMetadata)
}
