//! Canonical plan-confirm wire encoding.

use super::{DOMAIN_V1, DOMAIN_V2, PlanConfirmation, PlanFingerprintBuildError, validate};
use crate::buffer::SnapshotEncoder;
use crate::operation::{PermitIdempotencyKey, PermitScope, ReplayPolicy};
use crate::transport::{CanonicalHost, EndpointIdentity, EndpointScheme};

pub(super) fn encode<E: Copy>(
    plan: &PlanConfirmation<'_, '_>,
    writer: &mut SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
) -> Result<(), PlanFingerprintBuildError<E>> {
    encode_inner(plan, writer, false)
}

pub(super) fn encode_with_authorization_evidence<E: Copy>(
    plan: &PlanConfirmation<'_, '_>,
    writer: &mut SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
) -> Result<(), PlanFingerprintBuildError<E>> {
    encode_inner(plan, writer, true)
}

fn encode_inner<E: Copy>(
    plan: &PlanConfirmation<'_, '_>,
    writer: &mut SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    authorization_evidence_supplied: bool,
) -> Result<(), PlanFingerprintBuildError<E>> {
    let scope = validate(plan, authorization_evidence_supplied)?;
    let prepared = plan.prepared;
    let request = prepared.transport_request();
    let authorization_evidence_required = prepared.authorization_evidence_required();
    writer.bytes(if authorization_evidence_required {
        DOMAIN_V2
    } else {
        DOMAIN_V1
    })?;
    field(
        writer,
        1,
        prepared.service().provider_id().as_str().as_bytes(),
    )?;
    field(
        writer,
        2,
        prepared.service().service_id().as_str().as_bytes(),
    )?;
    field(
        writer,
        3,
        prepared
            .operation_id()
            .map(|id| id.as_str())
            .unwrap_or_default()
            .as_bytes(),
    )?;
    field(writer, 4, request.method().as_str().as_bytes())?;
    endpoint_fields(writer, plan.endpoint)?;
    field(writer, 10, request.target().path().as_str().as_bytes())?;
    optional_field(writer, 11, request.target().query_bytes())?;
    let headers = request.headers().as_slice();
    field(
        writer,
        12,
        &u64::try_from(headers.len())
            .map_err(|_| PlanFingerprintBuildError::InputTooLarge)?
            .to_be_bytes(),
    )?;
    for header in headers {
        lowercase_field(writer, 13, header.name().as_str().as_bytes())?;
        field(writer, 14, header.value().as_str().as_bytes())?;
        field(writer, 15, &[u8::from(header.sensitivity().is_sensitive())])?;
    }
    field(writer, 16, request.body())?;
    optional_field(
        writer,
        17,
        plan.account
            .bytes()
            .map_err(PlanFingerprintBuildError::Context)?,
    )?;
    optional_field(
        writer,
        18,
        plan.tenant
            .bytes()
            .map_err(PlanFingerprintBuildError::Context)?,
    )?;
    field(writer, 19, plan.context.bytes())?;
    field(writer, 20, &[scope_byte(scope)])?;
    field(
        writer,
        21,
        &plan.validity.issued_at().as_seconds().to_be_bytes(),
    )?;
    field(
        writer,
        22,
        &plan.validity.expires_at().as_seconds().to_be_bytes(),
    )?;
    field(writer, 23, &[replay_byte(plan.replay)])?;
    field(writer, 24, &plan.attempts.get().to_be_bytes())?;
    optional_field(
        writer,
        25,
        plan.idempotency.map(PermitIdempotencyKey::bytes),
    )?;
    cost_fields(writer, plan.cost)?;
    field(
        writer,
        30,
        &[u8::from(prepared.body_sensitivity().requires_digest())],
    )?;
    if authorization_evidence_required {
        field(writer, 31, &[1])?;
    }
    Ok(())
}

fn endpoint_fields<E: Copy>(
    writer: &mut SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    endpoint: EndpointIdentity<'_>,
) -> Result<(), PlanFingerprintBuildError<E>> {
    field(
        writer,
        5,
        &[match endpoint.scheme() {
            EndpointScheme::Http => 0,
            EndpointScheme::Https => 1,
        }],
    )?;
    writer.byte(6)?;
    match endpoint.canonical_host() {
        CanonicalHost::Dns(value) => {
            writer.byte(0)?;
            sized_bytes(writer, value.as_bytes())?;
        }
        CanonicalHost::Ipv4(value) => {
            writer.byte(1)?;
            sized_bytes(writer, &value)?;
        }
        CanonicalHost::Ipv6(value) => {
            writer.byte(2)?;
            sized_bytes(writer, &value)?;
        }
    }
    field(writer, 7, &endpoint.effective_port().to_be_bytes())?;
    field(writer, 8, endpoint.base_path().as_bytes())?;
    field(writer, 9, &[])
}

fn cost_fields<E: Copy>(
    writer: &mut SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    cost: Option<super::PlanCost>,
) -> Result<(), PlanFingerprintBuildError<E>> {
    let Some(cost) = cost else {
        for tag in 26..=29 {
            field(writer, tag, &[])?;
        }
        return Ok(());
    };
    let (currency, scale, observed, ceiling) = cost.fields();
    field(writer, 26, &currency.as_bytes())?;
    field(writer, 27, &[scale])?;
    field(writer, 28, &observed.to_be_bytes())?;
    field(writer, 29, &ceiling.to_be_bytes())
}

fn field<E: Copy>(
    writer: &mut SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    tag: u8,
    bytes: &[u8],
) -> Result<(), PlanFingerprintBuildError<E>> {
    writer.byte(tag)?;
    sized_bytes(writer, bytes)
}

fn sized_bytes<E: Copy>(
    writer: &mut SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    bytes: &[u8],
) -> Result<(), PlanFingerprintBuildError<E>> {
    writer.bytes(
        &u64::try_from(bytes.len())
            .map_err(|_| PlanFingerprintBuildError::InputTooLarge)?
            .to_be_bytes(),
    )?;
    writer.bytes(bytes)
}

fn optional_field<E: Copy>(
    writer: &mut SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    tag: u8,
    bytes: Option<&[u8]>,
) -> Result<(), PlanFingerprintBuildError<E>> {
    field(writer, tag, &[u8::from(bytes.is_some())])?;
    field(writer, tag, bytes.unwrap_or_default())
}

fn lowercase_field<E: Copy>(
    writer: &mut SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    tag: u8,
    bytes: &[u8],
) -> Result<(), PlanFingerprintBuildError<E>> {
    writer.byte(tag)?;
    writer.bytes(
        &u64::try_from(bytes.len())
            .map_err(|_| PlanFingerprintBuildError::InputTooLarge)?
            .to_be_bytes(),
    )?;
    for byte in bytes {
        writer.byte(byte.to_ascii_lowercase())?;
    }
    Ok(())
}

const fn scope_byte(scope: PermitScope) -> u8 {
    match scope {
        PermitScope::Mutation => 1,
        PermitScope::Destructive => 2,
        PermitScope::Cost => 3,
    }
}

const fn replay_byte(replay: ReplayPolicy) -> u8 {
    match replay {
        ReplayPolicy::SingleAttempt => 1,
        ReplayPolicy::RecoverNotSent => 2,
        ReplayPolicy::ReconcileThenRetry => 3,
    }
}
