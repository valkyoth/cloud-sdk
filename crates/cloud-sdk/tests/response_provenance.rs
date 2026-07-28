//! Adversarial response provenance and cleanup contract tests.

use core::future::Future;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Context, Poll, Waker};

use cloud_sdk::Method;
use cloud_sdk::operation::{
    ContentTypePolicy, RequestIdPolicy, ResponseBodyPolicy, ResponsePolicy, ResponsePolicyError,
};
use cloud_sdk::transport::{
    AsyncTransport, BlockingTransport, HeaderSensitivity, MediaType, RequestTarget, ResponseBuffer,
    ResponseMetadata, ResponseStorageSanitizer, ResponseWriter, ResponseWriterError, StatusCode,
    TransportRequest,
};

static OK: [StatusCode; 1] = [StatusCode::OK];
static JSON: [MediaType<'static>; 1] = [MediaType::JSON];

#[test]
fn writer_rejects_forged_lengths_duplicate_commits_and_post_commit_writes() {
    let sanitizer = CountingSanitizer::new();
    let mut storage = [0xa5_u8; 16];
    let mut headers = [0xa5_u8; 8192];
    {
        let mut response =
            ResponseBuffer::with_additive_sanitizer(&mut storage, 4, &mut headers, &sanitizer);
        assert_eq!(response.writer().body_capacity(), 4);
        assert_eq!(
            response
                .writer()
                .commit(StatusCode::OK, 5, ResponseMetadata::EMPTY),
            Err(ResponseWriterError::InitializedLengthTooLarge)
        );
        assert!(!response.writer().is_committed());
        assert!(response.writer().body_mut().is_ok());
        assert_eq!(
            response
                .writer()
                .commit(StatusCode::OK, 0, ResponseMetadata::EMPTY),
            Ok(())
        );
        assert_eq!(
            response.writer().body_mut().map(|body| body.len()),
            Err(ResponseWriterError::AlreadyCommitted)
        );
        assert_eq!(
            response
                .writer()
                .commit(StatusCode::OK, 0, ResponseMetadata::EMPTY),
            Err(ResponseWriterError::AlreadyCommitted)
        );
    }
    assert_eq!(storage, [0_u8; 16]);
    assert_eq!(headers, [0_u8; 8192]);
    assert_eq!(sanitizer.calls(), 2);
}

#[test]
fn uncommitted_response_fails_closed_and_clears_complete_storage() {
    let sanitizer = CountingSanitizer::new();
    let mut storage = [0xa5_u8; 16];
    let mut headers = [0xa5_u8; 8192];
    let policy = json_policy(8);
    assert!(policy.is_ok());
    let Ok(policy) = policy else { return };
    let result = policy.validate(
        ResponseBuffer::with_additive_sanitizer(&mut storage, 8, &mut headers, &sanitizer),
        RequestIdPolicy::Discard,
    );
    assert!(matches!(
        result,
        Err(ResponsePolicyError::UncommittedResponse)
    ));
    drop(result);
    assert_eq!(storage, [0_u8; 16]);
    assert_eq!(headers, [0_u8; 8192]);
    assert_eq!(sanitizer.calls(), 2);
}

#[test]
fn owned_decode_clears_before_return_and_borrow_is_guard_scoped() -> Result<(), &'static str> {
    let sanitizer = CountingSanitizer::new();
    let mut storage = [0xa5_u8; 16];
    let mut headers = [0xa5_u8; 8192];
    let mut response =
        ResponseBuffer::with_additive_sanitizer(&mut storage, 8, &mut headers, &sanitizer);
    let output = response
        .writer()
        .body_mut()
        .map_err(|_| "response body was unavailable")?;
    output
        .get_mut(..2)
        .ok_or("response body was too small")?
        .copy_from_slice(b"{}");
    response
        .writer()
        .headers_mut()
        .map_err(|_| "response headers were unavailable")?
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .map_err(|_| "content type was invalid")?;
    response
        .writer()
        .commit(StatusCode::OK, 2, ResponseMetadata::EMPTY)
        .map_err(|_| "response commitment failed")?;
    let policy = json_policy(8).map_err(|_| "response policy was invalid")?;
    let checked = policy
        .validate(response, RequestIdPolicy::Protected)
        .map_err(|_| "response policy rejected fixture")?;
    assert!(checked.with_borrowed(|view| view.body() == b"{}"));
    let decoded = checked.decode_owned(|view| Ok::<usize, &'static str>(view.body().len()))?;
    assert_eq!(decoded, 2);
    assert_eq!(sanitizer.calls(), 2);
    assert_eq!(storage, [0_u8; 16]);
    assert_eq!(headers, [0_u8; 8192]);
    Ok(())
}

#[test]
fn blocking_and_async_transports_share_the_same_sealed_writer_contract() {
    let transport = ExampleTransport {
        sanitizer: CountingSanitizer::new(),
    };
    let request = test_request();
    assert!(request.is_some());
    let Some(request) = request else { return };

    let mut blocking_storage = [0xa5_u8; 8];
    let mut blocking_headers = [0xa5_u8; 8192];
    {
        let mut response = ResponseBuffer::with_additive_sanitizer(
            &mut blocking_storage,
            8,
            &mut blocking_headers,
            &transport,
        );
        assert!(BlockingTransport::send(&transport, request, response.writer()).is_ok());
        assert!(
            response
                .with_response(|view| view.body() == b"{}")
                .is_ok_and(core::convert::identity)
        );
    }
    assert_eq!(blocking_storage, [0_u8; 8]);
    assert_eq!(blocking_headers, [0_u8; 8192]);

    let mut async_storage = [0xa5_u8; 8];
    let mut async_headers = [0xa5_u8; 8192];
    {
        let mut response = ResponseBuffer::with_additive_sanitizer(
            &mut async_storage,
            8,
            &mut async_headers,
            &transport,
        );
        {
            let future = AsyncTransport::send(&transport, request, response.writer());
            let mut future = core::pin::pin!(future);
            let mut context = Context::from_waker(Waker::noop());
            assert!(matches!(
                Future::poll(future.as_mut(), &mut context),
                Poll::Ready(Ok(()))
            ));
        }
        assert!(
            response
                .with_response(|view| view.body() == b"{}")
                .is_ok_and(core::convert::identity)
        );
    }
    assert_eq!(async_storage, [0_u8; 8]);
    assert_eq!(async_headers, [0_u8; 8192]);
    assert_eq!(transport.sanitizer.calls(), 4);
}

fn json_policy(
    max_body_bytes: usize,
) -> Result<ResponsePolicy, cloud_sdk::operation::ResponsePolicyValidationError> {
    ResponsePolicy::new(
        &OK,
        ContentTypePolicy::Required(&JSON),
        ResponseBodyPolicy::Required,
        max_body_bytes,
    )
}

fn test_request() -> Option<TransportRequest<'static>> {
    RequestTarget::new("/test")
        .ok()
        .map(|target| TransportRequest::new(Method::Get, target))
}

struct CountingSanitizer {
    calls: AtomicUsize,
}

impl CountingSanitizer {
    const fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::Acquire)
    }
}

impl ResponseStorageSanitizer for CountingSanitizer {
    fn sanitize_response_storage(&self, _response_storage: &mut [u8]) {
        self.calls.fetch_add(1, Ordering::AcqRel);
    }
}

struct ExampleTransport {
    sanitizer: CountingSanitizer,
}

impl ExampleTransport {
    fn send_inner(response: &mut ResponseWriter<'_>) -> Result<(), ResponseWriterError> {
        let output = response
            .body_mut()?
            .get_mut(..2)
            .ok_or(ResponseWriterError::InitializedLengthTooLarge)?;
        output.copy_from_slice(b"{}");
        response
            .headers_mut()?
            .try_push(
                "content-type",
                b"application/json",
                HeaderSensitivity::Public,
            )
            .map_err(|_| ResponseWriterError::InitializedLengthTooLarge)?;
        response.commit(StatusCode::OK, 2, ResponseMetadata::EMPTY)
    }
}

impl ResponseStorageSanitizer for ExampleTransport {
    fn sanitize_response_storage(&self, response_storage: &mut [u8]) {
        self.sanitizer.sanitize_response_storage(response_storage);
    }
}

impl BlockingTransport for ExampleTransport {
    type Error = ResponseWriterError;

    fn send(
        &self,
        _request: TransportRequest<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        Self::send_inner(response)
    }
}

impl AsyncTransport for ExampleTransport {
    type Error = ResponseWriterError;

    async fn send<'transport, 'request, 'writer>(
        &'transport self,
        _request: TransportRequest<'request>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
    {
        Self::send_inner(response)
    }
}
