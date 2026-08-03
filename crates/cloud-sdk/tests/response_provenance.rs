//! Adversarial response provenance and cleanup contract tests.

use core::future::{Future, pending};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Context, Poll, Waker};

use cloud_sdk::Method;
use cloud_sdk::operation::{
    ContentTypePolicy, RequestIdPolicy, ResponseBodyPolicy, ResponsePolicy, ResponsePolicyError,
};
use cloud_sdk::transport::{
    AsyncResponseStaging, AsyncTransport, BlockingTransport, HeaderSensitivity, MediaType,
    RequestTarget, ResponseBuffer, ResponseCompletion, ResponseMetadata, ResponseStorageSanitizer,
    ResponseWriter, ResponseWriterError, StatusCode, TransportRequest, drive_async,
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
        let mut attempt = response
            .writer()
            .begin_attempt()
            .unwrap_or_else(|_| unreachable!());
        assert_eq!(
            attempt.commit(StatusCode::OK, 5, ResponseMetadata::EMPTY),
            Err(ResponseWriterError::InitializedLengthTooLarge)
        );
        assert!(attempt.body_mut().is_ok());
        assert_eq!(
            attempt.commit(StatusCode::OK, 0, ResponseMetadata::EMPTY),
            Ok(())
        );
        assert_eq!(
            attempt.body_mut().map(|body| body.len()),
            Err(ResponseWriterError::AlreadyCommitted)
        );
        assert_eq!(
            attempt.commit(StatusCode::OK, 0, ResponseMetadata::EMPTY),
            Err(ResponseWriterError::AlreadyCommitted)
        );
        drop(attempt);
        assert!(response.writer().is_committed());
        assert_eq!(
            response.writer().begin_attempt().map(|_| ()),
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
fn response_attempt_clears_failed_state_before_reuse() -> Result<(), &'static str> {
    let mut storage = [0xa5_u8; 16];
    let mut headers = [0xa5_u8; 8192];
    let mut response = ResponseBuffer::new(&mut storage, 8, &mut headers);
    {
        let mut attempt = response
            .writer()
            .begin_attempt()
            .map_err(|_| "attempt rejected")?;
        attempt
            .body_mut()
            .map_err(|_| "attempt body unavailable")?
            .fill(0x42);
        attempt
            .headers_mut()
            .map_err(|_| "attempt headers unavailable")?
            .try_push("x-request-id", b"partial", HeaderSensitivity::Sensitive)
            .map_err(|_| "attempt header rejected")?;
    }
    let mut attempt = response
        .writer()
        .begin_attempt()
        .map_err(|_| "second attempt rejected")?;
    assert!(
        attempt
            .body_mut()
            .is_ok_and(|body| body.iter().all(|byte| *byte == 0))
    );
    assert!(attempt.headers().is_empty());
    attempt
        .commit(StatusCode::OK, 0, ResponseMetadata::EMPTY)
        .map_err(|_| "second attempt commit failed")?;
    drop(attempt);
    assert!(
        response
            .with_response(|view| view.body().is_empty() && view.headers().is_empty())
            .is_ok_and(core::convert::identity)
    );
    Ok(())
}

#[test]
fn cancelling_response_attempt_clears_reusable_state() {
    let mut storage = [0xa5_u8; 16];
    let mut headers = [0xa5_u8; 8192];
    let mut response = ResponseBuffer::new(&mut storage, 8, &mut headers);
    {
        let future = pending_dirty_attempt(response.writer());
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Pending
        ));
    }
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!());
    assert!(
        attempt
            .body_mut()
            .is_ok_and(|body| body.iter().all(|byte| *byte == 0))
    );
    assert!(attempt.headers().is_empty());
}

async fn pending_dirty_attempt(response: &mut ResponseWriter<'_>) {
    let Ok(mut attempt) = response.begin_attempt() else {
        return;
    };
    if let Ok(body) = attempt.body_mut() {
        body.fill(0x5a);
    }
    if let Ok(headers) = attempt.headers_mut() {
        let _ = headers.try_push("x-request-id", b"partial", HeaderSensitivity::Sensitive);
    }
    pending::<()>().await;
}

#[test]
fn owned_decode_clears_before_return_and_borrow_is_guard_scoped() -> Result<(), &'static str> {
    let sanitizer = CountingSanitizer::new();
    let mut storage = [0xa5_u8; 16];
    let mut headers = [0xa5_u8; 8192];
    let mut response =
        ResponseBuffer::with_additive_sanitizer(&mut storage, 8, &mut headers, &sanitizer);
    {
        let mut attempt = response
            .writer()
            .begin_attempt()
            .map_err(|_| "response attempt was unavailable")?;
        attempt
            .body_mut()
            .map_err(|_| "response body was unavailable")?
            .get_mut(..2)
            .ok_or("response body was too small")?
            .copy_from_slice(b"{}");
        attempt
            .headers_mut()
            .map_err(|_| "response headers were unavailable")?
            .try_push(
                "content-type",
                b"application/json",
                HeaderSensitivity::Public,
            )
            .map_err(|_| "content type was invalid")?;
        attempt
            .commit(StatusCode::OK, 2, ResponseMetadata::EMPTY)
            .map_err(|_| "response commitment failed")?;
    }
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
fn blocking_and_async_transports_share_sealed_response_provenance() {
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
            let future = drive_async(&transport, request, response.writer());
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
        let mut attempt = response.begin_attempt()?;
        let output = attempt
            .body_mut()?
            .get_mut(..2)
            .ok_or(ResponseWriterError::InitializedLengthTooLarge)?;
        output.copy_from_slice(b"{}");
        attempt
            .headers_mut()?
            .try_push(
                "content-type",
                b"application/json",
                HeaderSensitivity::Public,
            )
            .map_err(|_| ResponseWriterError::InitializedLengthTooLarge)?;
        attempt.commit(StatusCode::OK, 2, ResponseMetadata::EMPTY)
    }

    fn stage_inner(
        response: &mut AsyncResponseStaging<'_, '_>,
    ) -> Result<ResponseCompletion, ResponseWriterError> {
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
        Ok(ResponseCompletion::new(
            StatusCode::OK,
            2,
            ResponseMetadata::EMPTY,
        ))
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

    async fn send<'transport, 'request, 'writer, 'buffer>(
        &'transport self,
        _request: TransportRequest<'request>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'buffer: 'writer,
    {
        Self::stage_inner(&mut response)
    }
}
