use alloc::vec;

use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationImpact, OperationMetadata,
    PreparedRequest, ProviderService, RequestIdPolicy, RequestSemantics, ResponseBodyPolicy,
    ResponsePolicy, RetryEligibility,
};
use cloud_sdk::transport::{
    EndpointIdentity, EndpointPolicy, EndpointScheme, HeaderName, HeaderSensitivity, MediaType,
    RawResponsePolicy, RequestTarget, ResponseBuffer, ResponseMediaPolicy, ResponseMetadata,
    StatusCode, TransportRequest,
};
use cloud_sdk::{Method, ServiceId};

use super::{
    CheckedHetznerResponse, HetznerDecodeError, decode_response as decode_checked_response,
    decode_response_at as decode_checked_response_at,
};
use crate::association::{HetznerOperation, Prepared};
use crate::identity::{
    CloudService, DNS_SERVICE_ID, DnsService, SECURITY_SERVICE_ID, STORAGE_SERVICE_ID,
    SecurityService, StorageService,
};
use cloud_sdk::rate_limit::WallClockTimestamp;

const JSON: &[MediaType<'static>] = &[MediaType::JSON];
const OK: &[StatusCode] = &[StatusCode::OK];
const CREATED: &[StatusCode] = &[StatusCode::CREATED];
const NO_CONTENT: &[StatusCode] = &[StatusCode::NO_CONTENT];

pub(super) fn prepared(
    operation: &'static str,
    service_id: ServiceId,
    status: StatusCode,
) -> PreparedRequest<'static> {
    let target = RequestTarget::new("/test");
    assert!(target.is_ok());
    let statuses = if status == StatusCode::OK {
        OK
    } else if status == StatusCode::CREATED {
        CREATED
    } else {
        NO_CONTENT
    };
    let empty = status == StatusCode::NO_CONTENT;
    let policy = ResponsePolicy::new(
        statuses,
        if empty {
            ContentTypePolicy::Forbidden
        } else {
            ContentTypePolicy::Required(JSON)
        },
        if empty {
            ResponseBodyPolicy::Forbidden
        } else {
            ResponseBodyPolicy::Required
        },
        if empty { 0 } else { 8_388_608 },
    );
    assert!(policy.is_ok());
    let metadata = OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        CostIntent::NoKnownCost,
        RequestIdPolicy::Protected,
    );
    assert!(metadata.is_ok());
    let endpoint = EndpointIdentity::new(
        EndpointScheme::Https,
        if service_id == STORAGE_SERVICE_ID {
            "api.hetzner.com"
        } else {
            "api.hetzner.cloud"
        },
        443,
        "/v1",
    );
    assert!(endpoint.is_ok());
    let operation_id = OperationId::new(operation);
    assert!(operation_id.is_ok());
    let endpoint = endpoint.unwrap_or_else(|_| unreachable!());
    let service = match service_id {
        STORAGE_SERVICE_ID => {
            ProviderService::from_marker::<StorageService>(EndpointPolicy::fixed(endpoint))
        }
        DNS_SERVICE_ID => {
            ProviderService::from_marker::<DnsService>(EndpointPolicy::fixed(endpoint))
        }
        SECURITY_SERVICE_ID => {
            ProviderService::from_marker::<SecurityService>(EndpointPolicy::fixed(endpoint))
        }
        _ => ProviderService::from_marker::<CloudService>(EndpointPolicy::fixed(endpoint)),
    };
    let authentication_policy = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(service.provider_id()),
        ScopeRequirement::Required(service.service_id()),
        ScopeRequirement::Required(endpoint),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let content_type = HeaderName::new("content-type").unwrap_or_else(|_| unreachable!());
    let request_id = HeaderName::new("x-request-id").unwrap_or_else(|_| unreachable!());
    let limit = HeaderName::new("ratelimit-limit").unwrap_or_else(|_| unreachable!());
    let remaining = HeaderName::new("ratelimit-remaining").unwrap_or_else(|_| unreachable!());
    let reset = HeaderName::new("ratelimit-reset").unwrap_or_else(|_| unreachable!());
    let retry_after = HeaderName::new("retry-after").unwrap_or_else(|_| unreachable!());
    let raw_policy = RawResponsePolicy::new(
        if empty { 0 } else { 8_388_608 },
        8_388_608,
        if empty {
            ResponseMediaPolicy::Forbidden
        } else {
            ResponseMediaPolicy::Required(JSON)
        },
        ResponseMediaPolicy::Required(JSON),
        &[
            content_type,
            request_id,
            limit,
            remaining,
            reset,
            retry_after,
        ],
        8,
    )
    .unwrap_or_else(|_| unreachable!());
    PreparedRequest::new(
        TransportRequest::new(Method::Get, target.unwrap_or_else(|_| unreachable!())),
        service,
        metadata.unwrap_or_else(|_| unreachable!()),
        policy.unwrap_or_else(|_| unreachable!()),
        authentication_policy,
        raw_policy,
        cloud_sdk::operation::RequestBodySensitivity::Public,
    )
    .unwrap_or_else(|_| unreachable!())
    .with_operation_id(operation_id.unwrap_or_else(|_| unreachable!()))
}

pub(super) struct TestResponse<'a> {
    status: StatusCode,
    body: &'a [u8],
    json: bool,
}

pub(super) const fn response(status: StatusCode, body: &[u8]) -> TestResponse<'_> {
    TestResponse {
        status,
        body,
        json: true,
    }
}

pub(super) const fn empty_response(status: StatusCode) -> TestResponse<'static> {
    TestResponse {
        status,
        body: b"",
        json: false,
    }
}

pub(super) fn decode_response(
    prepared: PreparedRequest<'_>,
    fixture: TestResponse<'_>,
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    decode_response_with_metadata(prepared, fixture, None, &[], None)
}

pub(super) fn decode_typed_response<O: HetznerOperation>(
    prepared: Prepared<'_, O>,
    fixture: TestResponse<'_>,
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    decode_typed_response_mode(prepared, fixture, false)
}

pub(super) fn decode_typed_checked_response<O: HetznerOperation>(
    prepared: Prepared<'_, O>,
    fixture: TestResponse<'_>,
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    decode_typed_response_mode(prepared, fixture, true)
}

fn decode_typed_response_mode<O: HetznerOperation>(
    prepared: Prepared<'_, O>,
    fixture: TestResponse<'_>,
    validate_first: bool,
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    let mut storage = vec![0_u8; fixture.body.len()];
    let mut header_storage = [0_u8; 8192];
    let capacity = storage.len();
    let mut response = ResponseBuffer::new(&mut storage, capacity, &mut header_storage);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .map_err(HetznerDecodeError::ResponseWriter)?;
    if fixture.json {
        attempt
            .headers_mut()
            .map_err(HetznerDecodeError::ResponseWriter)?
            .try_push(
                "content-type",
                b"application/json; charset=utf-8",
                HeaderSensitivity::Public,
            )
            .map_err(|_| HetznerDecodeError::MalformedPayload)?;
    }
    attempt
        .body_mut()
        .map_err(HetznerDecodeError::ResponseWriter)?
        .copy_from_slice(fixture.body);
    attempt
        .commit(fixture.status, fixture.body.len(), ResponseMetadata::EMPTY)
        .map_err(HetznerDecodeError::ResponseWriter)?;
    drop(attempt);
    if validate_first {
        let checked = prepared
            .validate_response(response)
            .map_err(HetznerDecodeError::ResponsePolicy)?;
        super::decode_associated_checked_response(checked)
    } else {
        super::decode_associated_response(prepared, response)
    }
}

pub(super) fn assert_decode_error(
    result: Result<CheckedHetznerResponse, HetznerDecodeError>,
    expected: HetznerDecodeError,
) {
    let Err(actual) = result else {
        unreachable!("response fixture unexpectedly decoded successfully")
    };
    assert_eq!(actual, expected);
}

pub(super) fn decode_response_with_request_id(
    prepared: PreparedRequest<'_>,
    fixture: TestResponse<'_>,
    request_id: &[u8],
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    decode_response_with_metadata(prepared, fixture, Some(request_id), &[], None)
}

pub(super) fn decode_response_with_headers(
    prepared: PreparedRequest<'_>,
    fixture: TestResponse<'_>,
    headers: &[(&str, &[u8])],
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    decode_response_with_metadata(prepared, fixture, None, headers, None)
}

pub(super) fn decode_response_with_headers_at(
    prepared: PreparedRequest<'_>,
    fixture: TestResponse<'_>,
    headers: &[(&str, &[u8])],
    now: WallClockTimestamp,
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    decode_response_with_metadata(prepared, fixture, None, headers, Some(now))
}

fn decode_response_with_metadata(
    prepared: PreparedRequest<'_>,
    fixture: TestResponse<'_>,
    request_id: Option<&[u8]>,
    headers: &[(&str, &[u8])],
    now: Option<WallClockTimestamp>,
) -> Result<CheckedHetznerResponse, HetznerDecodeError> {
    let mut storage = vec![0_u8; fixture.body.len()];
    let mut header_storage = [0_u8; 8192];
    let capacity = storage.len();
    let mut response = ResponseBuffer::new(&mut storage, capacity, &mut header_storage);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .map_err(HetznerDecodeError::ResponseWriter)?;
    if let Some(request_id) = request_id {
        attempt
            .headers_mut()
            .map_err(HetznerDecodeError::ResponseWriter)?
            .try_push("x-request-id", request_id, HeaderSensitivity::Sensitive)
            .map_err(|_| HetznerDecodeError::MalformedPayload)?;
    }
    for (name, value) in headers {
        attempt
            .headers_mut()
            .map_err(HetznerDecodeError::ResponseWriter)?
            .try_push(name, value, HeaderSensitivity::Public)
            .map_err(|_| HetznerDecodeError::MalformedPayload)?;
    }
    if fixture.json {
        attempt
            .headers_mut()
            .map_err(HetznerDecodeError::ResponseWriter)?
            .try_push(
                "content-type",
                b"application/json; charset=utf-8",
                HeaderSensitivity::Public,
            )
            .map_err(|_| HetznerDecodeError::MalformedPayload)?;
    }
    attempt
        .body_mut()
        .map_err(HetznerDecodeError::ResponseWriter)?
        .copy_from_slice(fixture.body);
    attempt
        .commit(fixture.status, fixture.body.len(), ResponseMetadata::EMPTY)
        .map_err(HetznerDecodeError::ResponseWriter)?;
    drop(attempt);
    match now {
        Some(now) => decode_checked_response_at(prepared, response, now),
        None => decode_checked_response(prepared, response),
    }
}
