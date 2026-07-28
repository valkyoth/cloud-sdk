use alloc::vec;

use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationId, OperationImpact, OperationMetadata,
    PreparedRequest, ProviderService, RequestIdPolicy, RequestSemantics, ResponseBodyPolicy,
    ResponsePolicy, RetryEligibility,
};
use cloud_sdk::transport::{
    EndpointIdentity, EndpointPolicy, EndpointScheme, MediaType, RequestTarget, ResponseBuffer,
    ResponseContentType, ResponseMetadata, StatusCode, TransportRequest,
};
use cloud_sdk::{Method, ServiceId};

use super::{
    CheckedHetznerResponse, HetznerDecodeError, decode_response as decode_checked_response,
};
use crate::identity::{CloudService, STORAGE_SERVICE_ID, StorageService};

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
    PreparedRequest::new(
        TransportRequest::new(Method::Get, target.unwrap_or_else(|_| unreachable!())),
        if service_id == STORAGE_SERVICE_ID {
            ProviderService::from_marker::<StorageService>(EndpointPolicy::fixed(
                endpoint.unwrap_or_else(|_| unreachable!()),
            ))
        } else {
            ProviderService::from_marker::<CloudService>(EndpointPolicy::fixed(
                endpoint.unwrap_or_else(|_| unreachable!()),
            ))
        },
        metadata.unwrap_or_else(|_| unreachable!()),
        policy.unwrap_or_else(|_| unreachable!()),
    )
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
    let mut storage = vec![0_u8; fixture.body.len()];
    let capacity = storage.len();
    let mut response = ResponseBuffer::new(&mut storage, capacity);
    response
        .writer()
        .body_mut()
        .map_err(HetznerDecodeError::ResponseWriter)?
        .copy_from_slice(fixture.body);
    let metadata = if fixture.json {
        ResponseMetadata::EMPTY.with_content_type(json_content_type())
    } else {
        ResponseMetadata::EMPTY
    };
    response
        .writer()
        .commit(fixture.status, fixture.body.len(), metadata)
        .map_err(HetznerDecodeError::ResponseWriter)?;
    decode_checked_response(prepared, response)
}

fn json_content_type() -> ResponseContentType {
    let content_type = ResponseContentType::new("application/json; charset=utf-8");
    assert!(content_type.is_ok());
    content_type.unwrap_or_else(|_| unreachable!())
}
