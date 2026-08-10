//! Canonical retry-fingerprint wire encoding.

use super::writer::{Writer, canonical_host_len};
use super::{
    DOMAIN, FingerprintBuildError, FingerprintScope, MAX_FINGERPRINT_SCOPE_BYTES,
    map_infallible_error,
};
use crate::operation::PreparedRequest;
use crate::transport::{EndpointIdentity, EndpointScheme};

pub(super) fn encoded_len(
    prepared: &PreparedRequest<'_>,
    endpoint: EndpointIdentity<'_>,
    scope: FingerprintScope<'_>,
) -> Result<usize, FingerprintBuildError<core::convert::Infallible>> {
    let operation = prepared
        .operation_id()
        .ok_or(FingerprintBuildError::MissingOperationId)?;
    let scope = scope_bytes(scope)?;
    let request = prepared.transport_request();
    let query = request.target().query_bytes().unwrap_or_default();
    let mut len = DOMAIN.len();
    for value in [
        prepared.service().provider_id().as_str().as_bytes(),
        prepared.service().service_id().as_str().as_bytes(),
        operation.as_str().as_bytes(),
        request.method().as_str().as_bytes(),
        endpoint.base_path().as_bytes(),
        request.target().path().as_str().as_bytes(),
        query,
        request.body(),
    ] {
        len = field_len(len, value.len())?;
    }
    len = field_len(len, 1)?;
    len = field_len(len, canonical_host_len(endpoint.canonical_host()))?;
    len = field_len(len, 2)?;
    len = field_len(len, 1)?;
    len = field_len(len, 2)?;
    len = field_len(len, 1)?;
    len = field_len(len, scope.len())?;
    len = field_len(len, 1)?;
    for header in request.headers().as_slice() {
        len = field_len(len, header.name().as_str().len())?;
        len = field_len(len, header.value().as_str().len())?;
        len = field_len(len, 1)?;
    }
    Ok(len)
}

fn field_len<E>(current: usize, value_len: usize) -> Result<usize, FingerprintBuildError<E>> {
    current
        .checked_add(9)
        .and_then(|value| value.checked_add(value_len))
        .ok_or(FingerprintBuildError::LengthOverflow)
}

fn scope_bytes(
    scope: FingerprintScope<'_>,
) -> Result<&[u8], FingerprintBuildError<core::convert::Infallible>> {
    let bytes = match scope {
        FingerprintScope::Absent => &[][..],
        FingerprintScope::Value(bytes) => bytes,
    };
    if bytes.len() > MAX_FINGERPRINT_SCOPE_BYTES {
        return Err(FingerprintBuildError::ScopeTooLong);
    }
    Ok(bytes)
}

pub(super) fn encode<E>(
    prepared: &PreparedRequest<'_>,
    endpoint: EndpointIdentity<'_>,
    scope: FingerprintScope<'_>,
    writer: &mut Writer<'_>,
) -> Result<(), FingerprintBuildError<E>> {
    let operation = prepared
        .operation_id()
        .ok_or(FingerprintBuildError::MissingOperationId)?;
    let scope_present = matches!(scope, FingerprintScope::Value(_));
    let scope = scope_bytes(scope).map_err(map_infallible_error)?;
    let request = prepared.transport_request();
    writer.raw(DOMAIN)?;
    writer.field(1, prepared.service().provider_id().as_str().as_bytes())?;
    writer.field(2, prepared.service().service_id().as_str().as_bytes())?;
    writer.field(3, operation.as_str().as_bytes())?;
    writer.field(4, request.method().as_str().as_bytes())?;
    writer.field(
        5,
        &[match endpoint.scheme() {
            EndpointScheme::Http => 0,
            EndpointScheme::Https => 1,
        }],
    )?;
    writer.canonical_host_field(6, endpoint.canonical_host())?;
    writer.field(7, &endpoint.effective_port().to_be_bytes())?;
    writer.field(8, endpoint.base_path().as_bytes())?;
    writer.field(9, request.target().path().as_str().as_bytes())?;
    let query = request.target().query_bytes();
    writer.field(10, &[u8::from(query.is_some())])?;
    writer.field(11, query.unwrap_or_default())?;
    let count = u16::try_from(request.headers().as_slice().len())
        .map_err(|_| FingerprintBuildError::LengthOverflow)?;
    writer.field(12, &count.to_be_bytes())?;
    for header in request.headers().as_slice() {
        writer.lowercase_field(13, header.name().as_str().as_bytes())?;
        writer.field(14, header.value().as_str().as_bytes())?;
        writer.field(18, &[u8::from(header.sensitivity().is_sensitive())])?;
    }
    writer.field(15, request.body())?;
    writer.field(16, &[u8::from(scope_present)])?;
    writer.field(17, scope)?;
    writer.field(
        19,
        &[u8::from(prepared.body_sensitivity().requires_digest())],
    )?;
    Ok(())
}
