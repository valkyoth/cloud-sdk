use super::{
    ContentType, ContentTypeError, RequestPathError, RequestTarget, RequestTargetError,
    ResponseBuffer, ResponseContentType, ResponseMetadata, ResponseStorageSanitizer,
    ResponseWriter, StatusCode, TransportRequest,
};
use crate::Method;
use crate::rate_limit::RateLimit;
use core::cell::Cell;
use core::fmt::Write;
use core::future::Future;
use core::task::{Context, Poll, Waker};

use super::{AsyncTransport, BlockingTransport};

#[test]
fn request_targets_are_origin_form_and_bounded() {
    let target = RequestTarget::new("/servers?page=2");
    assert_eq!(target.map(RequestTarget::as_str), Ok("/servers?page=2"));
    assert_eq!(
        RequestTarget::new("https://example.invalid/servers"),
        Err(RequestTargetError::Path(RequestPathError::NotOriginForm))
    );
    assert_eq!(
        RequestTarget::new("//evil.example/steal"),
        Err(RequestTargetError::Path(RequestPathError::NotOriginForm))
    );
    assert_eq!(
        RequestTarget::new("///evil.example/steal"),
        Err(RequestTargetError::Path(RequestPathError::NotOriginForm))
    );
    assert_eq!(
        RequestTarget::new("/servers#fragment"),
        Err(RequestTargetError::Path(RequestPathError::InvalidByte))
    );
    assert_eq!(
        RequestTarget::new("/\\evil"),
        Err(RequestTargetError::Path(RequestPathError::InvalidByte))
    );
    assert_eq!(
        RequestTarget::new(""),
        Err(RequestTargetError::Path(RequestPathError::Empty))
    );
    assert_eq!(
        RequestTarget::new("/servers bad"),
        Err(RequestTargetError::Path(RequestPathError::InvalidByte))
    );
    let mut accepted = [b'a'; super::MAX_REQUEST_TARGET_BYTES];
    if let Some(first) = accepted.first_mut() {
        *first = b'/';
    }
    let accepted = core::str::from_utf8(&accepted);
    assert!(accepted.is_ok());
    if let Ok(accepted) = accepted {
        assert!(RequestTarget::new(accepted).is_ok());
    }
    let mut rejected = [b'a'; super::MAX_REQUEST_TARGET_BYTES + 1];
    if let Some(first) = rejected.first_mut() {
        *first = b'/';
    }
    let rejected = core::str::from_utf8(&rejected);
    assert!(rejected.is_ok());
    if let Ok(rejected) = rejected {
        assert_eq!(
            RequestTarget::new(rejected),
            Err(RequestTargetError::TooLong)
        );
    }
}

#[test]
fn transport_request_debug_redacts_target_and_body() {
    let target = RequestTarget::new("/servers?token=secret");
    if let Ok(target) = target {
        let content_type = ContentType::new("application/x-private; token=secret-content");
        assert!(content_type.is_ok());
        let request = TransportRequest::new(Method::Post, target).with_body(b"secret-body");
        if let Ok(content_type) = content_type {
            let entries = [super::RequestHeader::content_type(content_type)];
            let headers = super::RequestHeaders::new(&entries);
            assert!(headers.is_ok());
            if let Ok(headers) = headers {
                let request = request.with_headers(headers);
                let mut debug = DebugBuffer::new();
                assert!(write!(&mut debug, "{request:?}").is_ok());
                let debug = debug.as_str();
                assert!(debug.contains("[redacted]"));
                assert!(!debug.contains("secret"));
                assert!(!debug.contains("application/x-private"));
            }
        }
    }
}

#[test]
fn content_types_are_bounded_and_header_safe() {
    assert_eq!(
        ContentType::new("application/json").map(ContentType::as_str),
        Ok("application/json")
    );
    assert_eq!(
        ContentType::new("text/plain; charset=utf-8").map(ContentType::as_str),
        Ok("text/plain; charset=utf-8")
    );
    assert_eq!(ContentType::new(""), Err(ContentTypeError::Empty));
    assert_eq!(
        ContentType::new("application"),
        Err(ContentTypeError::Invalid)
    );
    assert_eq!(
        ContentType::new("application/json\r\nx-evil: true"),
        Err(ContentTypeError::Invalid)
    );
    let oversized = [b'a'; super::MAX_CONTENT_TYPE_BYTES + 1];
    let oversized = core::str::from_utf8(&oversized);
    assert!(oversized.is_ok());
    if let Ok(oversized) = oversized {
        assert_eq!(ContentType::new(oversized), Err(ContentTypeError::TooLong));
    }
}

#[test]
fn transport_requests_preserve_explicit_headers() {
    let target = RequestTarget::new("/servers");
    if let Ok(target) = target {
        let entries = [super::RequestHeader::content_type(ContentType::JSON)];
        let headers = super::RequestHeaders::new(&entries);
        assert!(headers.is_ok());
        let request = TransportRequest::new(Method::Post, target).with_body(b"{}");
        if let Ok(headers) = headers {
            let request = request.with_headers(headers);
            assert_eq!(
                request
                    .headers()
                    .get("content-type")
                    .map(|header| header.value().as_str()),
                Some("application/json")
            );
        }
    }
}

#[test]
fn status_codes_are_bounded_and_classified() {
    assert_eq!(StatusCode::new(99), None);
    assert!(StatusCode::new(204).is_some_and(StatusCode::is_success));
    assert!(StatusCode::new(429).is_some_and(StatusCode::is_error));
    assert_eq!(StatusCode::new(600), None);
}

#[test]
fn transport_response_borrows_body_propagates_metadata_and_redacts_debug() {
    let output = b"secret-response-trailing-capacity";
    let body = output.get(..15).unwrap_or_default();
    let rate_limit = RateLimit::new(3600, 3599, 42).ok();
    let mut storage = *output;
    let mut header_storage = [0_u8; 8192];
    let mut buffer = ResponseBuffer::with_additive_sanitizer(
        &mut storage,
        15,
        &mut header_storage,
        &FillSanitizer,
    );
    assert!(
        buffer
            .writer()
            .headers_mut()
            .and_then(|headers| {
                headers
                    .try_push(
                        "content-type",
                        b"application/private; token=secret-content",
                        super::HeaderSensitivity::Sensitive,
                    )
                    .map_err(|_| super::ResponseWriterError::InitializedLengthTooLarge)
            })
            .is_ok()
    );
    assert!(
        buffer
            .writer()
            .body_mut()
            .map(|initialized| initialized.copy_from_slice(body))
            .is_ok()
    );
    let mut metadata = ResponseMetadata::EMPTY;
    if let Some(value) = rate_limit {
        metadata = metadata.with_rate_limit(value);
    }
    assert!(
        buffer
            .writer()
            .commit(StatusCode::OK, body.len(), metadata)
            .is_ok()
    );
    let mut debug = DebugBuffer::new();
    assert!(
        buffer
            .with_response(|response| {
                assert_eq!(response.status(), StatusCode::OK);
                assert_eq!(response.body(), b"secret-response");
                assert_eq!(response.rate_limit(), rate_limit);
                assert_eq!(
                    response.content_type().map(ResponseContentType::as_str),
                    Some("application/private; token=secret-content")
                );
                write!(&mut debug, "{response:?}")
            })
            .is_ok_and(|result| result.is_ok())
    );
    let debug = debug.as_str();
    assert!(debug.contains("body_len: 15"));
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("secret-response"));
    assert!(!debug.contains("secret-content"));
}

struct SequentialBlockingTransport {
    calls: Cell<u8>,
}

impl BlockingTransport for SequentialBlockingTransport {
    type Error = ();

    fn send(
        &self,
        _request: TransportRequest<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        self.calls.set(self.calls.get().saturating_add(1));
        let output = response
            .body_mut()
            .map_err(|_| ())?
            .get_mut(..2)
            .ok_or(())?;
        output.copy_from_slice(b"ok");
        response
            .commit(StatusCode::OK, 2, ResponseMetadata::EMPTY)
            .map_err(|_| ())
    }
}

struct SequentialAsyncTransport {
    _not_sync: Cell<()>,
}

impl AsyncTransport for SequentialAsyncTransport {
    type Error = ();

    // Avoid capturing the deliberately non-Sync receiver in the Send future.
    #[allow(clippy::manual_async_fn)]
    fn send<'transport, 'request, 'writer>(
        &'transport self,
        _request: TransportRequest<'request>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'writer
    where
        'transport: 'writer,
        'request: 'writer,
    {
        async move {
            let output = response
                .body_mut()
                .map_err(|_| ())?
                .get_mut(..2)
                .ok_or(())?;
            output.copy_from_slice(b"ok");
            response
                .commit(StatusCode::OK, 2, ResponseMetadata::EMPTY)
                .map_err(|_| ())
        }
    }
}

#[test]
fn non_sync_transports_remain_usable_sequentially() {
    let Ok(target) = RequestTarget::new("/sequential") else {
        return;
    };
    let request = TransportRequest::new(Method::Get, target);
    let blocking = SequentialBlockingTransport {
        calls: Cell::new(0),
    };
    let mut blocking_output = [0_u8; 2];
    let mut blocking_headers = [0_u8; 8192];
    let mut blocking_response = ResponseBuffer::new(&mut blocking_output, 2, &mut blocking_headers);
    assert!(blocking.send(request, blocking_response.writer()).is_ok());
    assert!(
        blocking_response
            .with_response(|response| response.body() == b"ok")
            .is_ok_and(core::convert::identity)
    );
    assert_eq!(blocking.calls.get(), 1);

    let asynchronous = SequentialAsyncTransport {
        _not_sync: Cell::new(()),
    };
    let mut async_output = [0_u8; 2];
    let mut async_headers = [0_u8; 8192];
    let mut async_response = ResponseBuffer::new(&mut async_output, 2, &mut async_headers);
    {
        let future = AsyncTransport::send(&asynchronous, request, async_response.writer());
        let mut future = core::pin::pin!(future);
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        let response = Future::poll(future.as_mut(), &mut context);
        assert!(matches!(response, Poll::Ready(Ok(_))));
    }
    assert!(
        async_response
            .with_response(|response| response.body() == b"ok")
            .is_ok_and(core::convert::identity)
    );
}

struct FillSanitizer;

impl ResponseStorageSanitizer for FillSanitizer {
    fn sanitize_response_storage(&self, _response_storage: &mut [u8]) {}
}

struct DebugBuffer {
    bytes: [u8; 512],
    len: usize,
}

impl DebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 512],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
    }
}

impl Write for DebugBuffer {
    fn write_str(&mut self, value: &str) -> core::fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(core::fmt::Error)?;
        let target = self.bytes.get_mut(self.len..end).ok_or(core::fmt::Error)?;
        target.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}
