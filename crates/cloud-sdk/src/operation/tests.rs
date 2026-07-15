use core::future::Future;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Context, Poll, Waker};

use super::{
    ContentTypePolicy, CostIntent, OperationImpact, OperationMetadata, OperationMetadataError,
    PreparationStorage, PrepareOperation, PreparedExecutionError, PreparedRequest, ProviderService,
    RequestSemantics, ResponseBodyPolicy, ResponsePolicy, ResponsePolicyError,
    ResponsePolicyValidationError, RetryEligibility,
};
use crate::transport::{
    AsyncTransport, BlockingTransport, BoundTransport, EndpointIdentity, EndpointIdentityError,
    EndpointScheme, MediaType, RequestTarget, ResponseContentType, ResponseStorageSanitizer,
    StatusCode, TransportRequest, TransportResponse,
};
use crate::{ApiFamily, Method, Provider};

static OK_STATUS: [StatusCode; 1] = [StatusCode::OK];
static JSON_MEDIA: [MediaType<'static>; 1] = [MediaType::JSON];

#[test]
fn operation_metadata_rejects_privilege_escalating_combinations() {
    assert_eq!(
        OperationMetadata::new(
            OperationImpact::ReadOnly,
            RequestSemantics::Idempotent,
            RetryEligibility::Never,
            CostIntent::NoKnownCost,
        ),
        Err(OperationMetadataError::ReadOnlyMustBeSafe)
    );
    for impact in [OperationImpact::Mutation, OperationImpact::Destructive] {
        assert_eq!(
            OperationMetadata::new(
                impact,
                RequestSemantics::Safe,
                RetryEligibility::Never,
                CostIntent::NoKnownCost,
            ),
            Err(OperationMetadataError::StateChangeCannotBeSafe)
        );
    }
    assert_eq!(
        OperationMetadata::new(
            OperationImpact::Mutation,
            RequestSemantics::NonIdempotent,
            RetryEligibility::ExplicitPolicy,
            CostIntent::MayIncurCost,
        ),
        Err(OperationMetadataError::NonIdempotentRetry)
    );

    let metadata = read_only_metadata();
    assert!(metadata.is_ok());
    if let Ok(metadata) = metadata {
        assert_eq!(metadata.impact(), OperationImpact::ReadOnly);
        assert_eq!(metadata.semantics(), RequestSemantics::Safe);
        assert_eq!(
            metadata.retry_eligibility(),
            RetryEligibility::ExplicitPolicy
        );
        assert_eq!(metadata.cost_intent(), CostIntent::NoKnownCost);
    }
}

#[test]
fn response_policy_requires_complete_coherent_configuration() {
    static DUPLICATE_STATUS: [StatusCode; 2] = [StatusCode::OK, StatusCode::OK];
    static ERROR_STATUS: [StatusCode; 1] = [StatusCode::TOO_MANY_REQUESTS];
    static DUPLICATE_MEDIA: [MediaType<'static>; 2] = [MediaType::JSON, MediaType::JSON];
    assert_eq!(
        ResponsePolicy::new(
            &[],
            ContentTypePolicy::Required(&JSON_MEDIA),
            ResponseBodyPolicy::Required,
            16,
        ),
        Err(ResponsePolicyValidationError::MissingSuccessStatus)
    );
    assert_eq!(
        ResponsePolicy::new(
            &OK_STATUS,
            ContentTypePolicy::Required(&[]),
            ResponseBodyPolicy::Required,
            16,
        ),
        Err(ResponsePolicyValidationError::MissingAcceptedMediaType)
    );
    assert_eq!(
        ResponsePolicy::new(
            &OK_STATUS,
            ContentTypePolicy::Required(&JSON_MEDIA),
            ResponseBodyPolicy::Required,
            0,
        ),
        Err(ResponsePolicyValidationError::RequiredBodyHasZeroLimit)
    );
    assert_eq!(
        ResponsePolicy::new(
            &OK_STATUS,
            ContentTypePolicy::Forbidden,
            ResponseBodyPolicy::Forbidden,
            1,
        ),
        Err(ResponsePolicyValidationError::ForbiddenBodyHasNonzeroLimit)
    );
    assert_eq!(
        ResponsePolicy::new(
            &OK_STATUS,
            ContentTypePolicy::Optional(&JSON_MEDIA),
            ResponseBodyPolicy::Forbidden,
            0,
        ),
        Err(ResponsePolicyValidationError::ForbiddenBodyAllowsContentType)
    );
    assert_eq!(
        ResponsePolicy::new(
            &DUPLICATE_STATUS,
            ContentTypePolicy::Required(&JSON_MEDIA),
            ResponseBodyPolicy::Required,
            16,
        ),
        Err(ResponsePolicyValidationError::DuplicateSuccessStatus)
    );
    assert_eq!(
        ResponsePolicy::new(
            &ERROR_STATUS,
            ContentTypePolicy::Required(&JSON_MEDIA),
            ResponseBodyPolicy::Required,
            16,
        ),
        Err(ResponsePolicyValidationError::NonSuccessStatus)
    );
    assert_eq!(
        ResponsePolicy::new(
            &OK_STATUS,
            ContentTypePolicy::Required(&DUPLICATE_MEDIA),
            ResponseBodyPolicy::Required,
            16,
        ),
        Err(ResponsePolicyValidationError::DuplicateAcceptedMediaType)
    );
}

#[test]
fn response_policy_classifies_every_rejection_before_decoding() {
    let required = json_response_policy(4);
    assert!(required.is_ok());
    let Ok(required) = required else { return };
    let json = ResponseContentType::new("application/json; charset=utf-8");
    assert!(json.is_ok());
    let Ok(json) = json else { return };

    let status = StatusCode::new(201).unwrap_or(StatusCode::OK);
    assert_eq!(
        required.validate(TransportResponse::new(status, b"{}")),
        Err(ResponsePolicyError::UnexpectedStatus)
    );
    assert_eq!(
        required.validate(TransportResponse::new(StatusCode::OK, b"12345")),
        Err(ResponsePolicyError::BodyTooLarge)
    );
    assert_eq!(
        required.validate(TransportResponse::new(StatusCode::OK, b"")),
        Err(ResponsePolicyError::MissingBody)
    );
    assert_eq!(
        required.validate(TransportResponse::new(StatusCode::OK, b"{}")),
        Err(ResponsePolicyError::MissingContentType)
    );
    let text = ResponseContentType::new("text/plain");
    assert!(text.is_ok());
    if let Ok(text) = text {
        assert_eq!(
            required
                .validate(TransportResponse::new(StatusCode::OK, b"{}").with_content_type(text)),
            Err(ResponsePolicyError::UnexpectedContentType)
        );
    }
    let checked =
        required.validate(TransportResponse::new(StatusCode::OK, b"{}").with_content_type(json));
    assert!(checked.is_ok_and(|response| response.body() == b"{}"));

    let forbidden = ResponsePolicy::new(
        &OK_STATUS,
        ContentTypePolicy::Forbidden,
        ResponseBodyPolicy::Forbidden,
        0,
    );
    assert!(forbidden.is_ok());
    if let Ok(forbidden) = forbidden {
        assert_eq!(
            forbidden.validate(TransportResponse::new(StatusCode::OK, b"x")),
            Err(ResponsePolicyError::ForbiddenBody)
        );
        assert_eq!(
            forbidden.validate(TransportResponse::new(StatusCode::OK, b"").with_content_type(json)),
            Err(ResponsePolicyError::ForbiddenContentType)
        );
    }
}

#[test]
fn prepared_blocking_execution_checks_endpoint_and_lends_only_policy_capacity() {
    let operation = ExampleOperation;
    let mut target = [0_u8; 32];
    let mut body = [0_u8; 8];
    let prepared = operation.prepare(PreparationStorage::new(&mut target, &mut body));
    assert!(prepared.is_ok());
    let Ok(prepared) = prepared else { return };

    let Ok(official) = official_endpoint() else {
        return;
    };
    let transport = RecordingTransport::new(official);
    let mut response_storage = [0xA5_u8; 64];
    let response = prepared.execute_blocking(&transport, &mut response_storage);
    assert!(response.is_ok_and(|response| response.body() == b"{}"));
    assert_eq!(transport.calls.load(Ordering::Acquire), 1);
    assert_eq!(transport.last_capacity.load(Ordering::Acquire), 16);
    assert_eq!(transport.sanitized_capacity.load(Ordering::Acquire), 64);
    assert_eq!(response_storage.get(2..), Some([0_u8; 62].as_slice()));

    let Ok(other) = other_endpoint() else { return };
    let mismatched = RecordingTransport::new(other);
    response_storage.fill(0xA5);
    let response = prepared.execute_blocking(&mismatched, &mut response_storage);
    assert_eq!(response, Err(PreparedExecutionError::EndpointMismatch));
    assert_eq!(mismatched.calls.load(Ordering::Acquire), 0);
    assert_eq!(mismatched.sanitized_capacity.load(Ordering::Acquire), 64);
    assert_eq!(response_storage, [0_u8; 64]);
}

#[test]
fn prepared_async_execution_uses_the_same_endpoint_and_response_policy() {
    let operation = ExampleOperation;
    let mut target = [0_u8; 32];
    let mut body = [0_u8; 8];
    let prepared = operation.prepare(PreparationStorage::new(&mut target, &mut body));
    assert!(prepared.is_ok());
    let Ok(prepared) = prepared else { return };
    let Ok(official) = official_endpoint() else {
        return;
    };
    let transport = RecordingTransport::new(official);
    let mut response_storage = [0xA5_u8; 64];
    {
        let future = prepared.execute_async(&transport, &mut response_storage);
        let mut future = core::pin::pin!(future);
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        let response = Future::poll(future.as_mut(), &mut context);
        assert!(matches!(response, Poll::Ready(Ok(_))));
        if let Poll::Ready(Ok(response)) = response {
            assert_eq!(response.body(), b"{}");
        }
    }
    assert_eq!(transport.calls.load(Ordering::Acquire), 1);
    assert_eq!(transport.last_capacity.load(Ordering::Acquire), 16);
    assert_eq!(transport.sanitized_capacity.load(Ordering::Acquire), 64);
    assert_eq!(response_storage.get(2..), Some([0_u8; 62].as_slice()));
}

#[test]
fn prepared_execution_errors_redact_transport_details() {
    use core::fmt::Write;

    let error = PreparedExecutionError::Transport("secret-transport-detail");
    let mut debug = DebugBuffer::new();
    assert!(write!(&mut debug, "{error:?}").is_ok());
    assert_eq!(debug.as_str(), "Transport([redacted])");
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExamplePrepareError {
    Buffer,
    Invalid,
}

struct ExampleOperation;

impl PrepareOperation for ExampleOperation {
    type Error = ExamplePrepareError;

    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error> {
        let (target_storage, body_storage) = storage.into_parts();
        let target = target_storage
            .get_mut(..8)
            .ok_or(ExamplePrepareError::Buffer)?;
        target.copy_from_slice(b"/servers");
        let target = core::str::from_utf8(target).map_err(|_| ExamplePrepareError::Invalid)?;
        let target = RequestTarget::new(target).map_err(|_| ExamplePrepareError::Invalid)?;
        let body = body_storage
            .get_mut(..2)
            .ok_or(ExamplePrepareError::Buffer)?;
        body.copy_from_slice(b"{}");
        let request = TransportRequest::new(Method::Post, target)
            .with_body(body)
            .with_content_type(crate::transport::ContentType::JSON);
        let metadata = OperationMetadata::new(
            OperationImpact::Mutation,
            RequestSemantics::Idempotent,
            RetryEligibility::ExplicitPolicy,
            CostIntent::MayIncurCost,
        )
        .map_err(|_| ExamplePrepareError::Invalid)?;
        let policy = json_response_policy(16).map_err(|_| ExamplePrepareError::Invalid)?;
        let endpoint = official_endpoint().map_err(|_| ExamplePrepareError::Invalid)?;
        Ok(PreparedRequest::new(
            request,
            ProviderService::new(Provider::Hetzner, ApiFamily::Cloud, endpoint),
            metadata,
            policy,
        ))
    }
}

struct RecordingTransport {
    endpoint: EndpointIdentity<'static>,
    calls: AtomicUsize,
    last_capacity: AtomicUsize,
    sanitized_capacity: AtomicUsize,
}

impl RecordingTransport {
    const fn new(endpoint: EndpointIdentity<'static>) -> Self {
        Self {
            endpoint,
            calls: AtomicUsize::new(0),
            last_capacity: AtomicUsize::new(0),
            sanitized_capacity: AtomicUsize::new(0),
        }
    }

    fn send_inner<'buffer>(
        &self,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, ()> {
        self.calls.fetch_add(1, Ordering::AcqRel);
        self.last_capacity
            .store(response_body.len(), Ordering::Release);
        let output = response_body.get_mut(..2).ok_or(())?;
        output.copy_from_slice(b"{}");
        let content_type = ResponseContentType::new("application/json").map_err(|_| ())?;
        Ok(TransportResponse::new(StatusCode::OK, output).with_content_type(content_type))
    }
}

impl BoundTransport for RecordingTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(self.endpoint)
    }
}

impl ResponseStorageSanitizer for RecordingTransport {
    fn sanitize_response_storage(&self, response_storage: &mut [u8]) {
        self.sanitized_capacity
            .store(response_storage.len(), Ordering::Release);
        response_storage.fill(0);
    }
}

impl BlockingTransport for RecordingTransport {
    type Error = ();

    fn send<'buffer>(
        &self,
        _request: TransportRequest<'_>,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, Self::Error> {
        self.send_inner(response_body)
    }
}

impl AsyncTransport for RecordingTransport {
    type Error = ();

    async fn send<'transport, 'request, 'buffer>(
        &'transport self,
        _request: TransportRequest<'request>,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, Self::Error>
    where
        'request: 'transport,
        'buffer: 'transport,
    {
        self.send_inner(response_body)
    }
}

fn read_only_metadata() -> Result<OperationMetadata, OperationMetadataError> {
    OperationMetadata::new(
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        CostIntent::NoKnownCost,
    )
}

fn json_response_policy(
    max_body_bytes: usize,
) -> Result<ResponsePolicy, ResponsePolicyValidationError> {
    ResponsePolicy::new(
        &OK_STATUS,
        ContentTypePolicy::Required(&JSON_MEDIA),
        ResponseBodyPolicy::Required,
        max_body_bytes,
    )
}

fn official_endpoint() -> Result<EndpointIdentity<'static>, EndpointIdentityError> {
    EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1")
}

fn other_endpoint() -> Result<EndpointIdentity<'static>, EndpointIdentityError> {
    EndpointIdentity::new(EndpointScheme::Https, "example.invalid", 443, "/v1")
}

struct DebugBuffer {
    bytes: [u8; 64],
    len: usize,
}

impl DebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 64],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
    }
}

impl core::fmt::Write for DebugBuffer {
    fn write_str(&mut self, value: &str) -> core::fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(core::fmt::Error)?;
        let target = self.bytes.get_mut(self.len..end).ok_or(core::fmt::Error)?;
        target.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}
