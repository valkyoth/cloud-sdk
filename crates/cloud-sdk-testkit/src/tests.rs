use alloc::format;
use cloud_sdk::Method;
use cloud_sdk::transport::{
    BlockingTransport, CanonicalQuery, FormQuery, RequestPath, RequestQuery, RequestTarget,
    ResponseBuffer, ResponseMetadata, StatusCode, TransportRequest, TransportResponse, drive_async,
};
use core::future::Future;
use core::task::{Context, Poll, Waker};

use crate::{
    ActionFixture, ActionState, AdversarialKind, ExpectedRequest, FixtureBody, FixtureBodyError,
    FixtureKind, FixtureMetadataError, MockError, MockExchange, MockTransport, PaginationFixture,
    RateLimitFixture, RawFaultError, ResponseFixture, ResponseFixtureError, adversarial_corpus,
};

mod local_async;
mod prepared;
mod raw_fault;

#[test]
fn public_errors_implement_payload_free_core_error() {
    fn assert_error<E: core::error::Error>() {}

    assert_error::<FixtureBodyError>();
    assert_error::<FixtureMetadataError>();
    assert_error::<MockError>();
    assert_error::<RawFaultError>();
    assert_error::<ResponseFixtureError>();
    assert_eq!(
        format!("{}", FixtureBodyError::TooLarge),
        "fixture body exceeds the size limit"
    );
    assert_eq!(
        format!("{}", FixtureMetadataError::PaginationZero),
        "fixture pagination value must be nonzero"
    );
    assert_eq!(
        format!("{}", MockError::TargetMismatch),
        "mock request target differs from expectation"
    );
    assert_eq!(format!("{RawFaultError}"), "injected raw transport fault");
    assert_eq!(
        format!("{}", ResponseFixtureError::NonErrorStatus),
        "error fixture requires an HTTP error status"
    );
}

#[test]
fn fixture_builders_cover_success_pagination_action_rate_limit_and_error() {
    let body = FixtureBody::new(br#"{"ok":true}"#);
    let pagination = PaginationFixture::new(2, 50, 75, 2);
    let action = ActionFixture::new(ActionState::Running, 25);
    let rate_limit = RateLimitFixture::new(3600, 0, 42);
    if let (Ok(body), Ok(pagination), Ok(action), Ok(rate_limit)) =
        (body, pagination, action, rate_limit)
    {
        assert_eq!(ResponseFixture::success(body).kind(), FixtureKind::Success);
        assert_eq!(
            ResponseFixture::paginated(body, pagination).pagination(),
            Some(pagination)
        );
        assert_eq!(
            ResponseFixture::action(body, action).action_metadata(),
            Some(action)
        );
        assert_eq!(
            ResponseFixture::rate_limited(body, rate_limit).rate_limit(),
            Some(rate_limit)
        );
        assert!(matches!(
            ResponseFixture::error(StatusCode::OK, body),
            Err(ResponseFixtureError::NonErrorStatus)
        ));
        let error =
            StatusCode::new(503).and_then(|status| ResponseFixture::error(status, body).ok());
        assert!(error.is_some());
    }
}

#[test]
fn metadata_rejects_incoherent_values() {
    assert_eq!(
        PaginationFixture::new(0, 50, 0, 1),
        Err(FixtureMetadataError::PaginationZero)
    );
    assert_eq!(
        PaginationFixture::new(3, 50, 100, 2),
        Err(FixtureMetadataError::PageAfterLast)
    );
    assert_eq!(
        PaginationFixture::new(1, 50, 100, 1),
        Err(FixtureMetadataError::InvalidLastPage)
    );
    assert_eq!(
        ActionFixture::new(ActionState::Success, 101),
        Err(FixtureMetadataError::InvalidProgress)
    );
    assert_eq!(
        RateLimitFixture::new(10, 11, 42),
        Err(FixtureMetadataError::RemainingExceedsLimit)
    );
}

#[test]
fn fixture_body_write_is_atomic_and_repeated_body_is_compact() {
    let body = FixtureBody::new(b"response");
    if let Ok(body) = body {
        let mut short = [0xa5_u8; 4];
        let original = short;
        assert_eq!(
            body.write_to(&mut short),
            Err(FixtureBodyError::OutputTooSmall)
        );
        assert_eq!(short, original);
    }

    let repeated = FixtureBody::repeated(b'x', 8);
    if let Ok(repeated) = repeated {
        let mut output = [0_u8; 8];
        assert_eq!(repeated.write_to(&mut output), Ok(8));
        assert_eq!(output, [b'x'; 8]);
    }
}

#[test]
fn mock_transport_is_ordered_fail_closed_and_non_consuming_on_mismatch() {
    let target = RequestTarget::new("/servers?page=1");
    let body = FixtureBody::new(br#"{"servers":[]}"#);
    if let (Ok(target), Ok(body)) = (target, body) {
        let exchange = MockExchange::new(
            ExpectedRequest::new(Method::Get, target),
            ResponseFixture::success(body),
        );
        let exchanges = [exchange];
        let transport = MockTransport::new(&exchanges);
        let wrong = TransportRequest::new(Method::Delete, target);
        let mut output = [0xa5_u8; 32];
        assert!(matches!(
            send_blocking(&transport, wrong, &mut output),
            Err(MockError::MethodMismatch)
        ));
        assert_eq!(transport.remaining(), 1);

        let request = TransportRequest::new(Method::Get, target);
        {
            let response = inspect_blocking(&transport, request, &mut output, |response| {
                response.body() == br#"{"servers":[]}"#
            });
            assert!(response.is_ok());
            assert_eq!(response, Ok(true));
        }
        assert!(transport.is_complete());
        assert!(matches!(
            send_blocking(&transport, request, &mut output),
            Err(MockError::Exhausted)
        ));
    }
}

#[test]
fn mock_rejects_precommitted_writer_without_consuming_exchange() {
    let (Ok(target), Ok(body)) = (
        RequestTarget::new("/servers"),
        FixtureBody::new(br#"{"servers":[]}"#),
    ) else {
        return;
    };
    let exchanges = [MockExchange::new(
        ExpectedRequest::new(Method::Get, target),
        ResponseFixture::success(body),
    )];
    let transport = MockTransport::new(&exchanges);
    let mut output = [0xa5_u8; 32];
    let mut headers = [0xa5_u8; 8192];
    let mut response = ResponseBuffer::new(&mut output, 32, &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!());
    assert!(
        attempt
            .commit(StatusCode::OK, 0, ResponseMetadata::EMPTY)
            .is_ok()
    );
    drop(attempt);
    assert_eq!(
        BlockingTransport::send(
            &transport,
            TransportRequest::new(Method::Get, target),
            response.writer(),
        ),
        Err(MockError::ResponseWriterRejected)
    );
    assert_eq!(transport.remaining(), 1);
}

#[test]
fn mock_transport_distinguishes_query_presence_and_dialect() {
    let path = RequestPath::new("/resources");
    let canonical = CanonicalQuery::new("name=test");
    let form = FormQuery::new("name=test");
    assert!(path.is_ok() && canonical.is_ok() && form.is_ok());
    let (Ok(path), Ok(canonical), Ok(form)) = (path, canonical, form) else {
        return;
    };
    let mut canonical_output = [0_u8; 32];
    let mut form_output = [0_u8; 32];
    let canonical_target = RequestTarget::assemble(
        path,
        RequestQuery::Canonical(canonical),
        &mut canonical_output,
    );
    let form_target = RequestTarget::assemble(path, RequestQuery::Form(form), &mut form_output);
    let body = FixtureBody::new(b"ok");
    assert!(canonical_target.is_ok() && form_target.is_ok() && body.is_ok());
    let (Ok(canonical_target), Ok(form_target), Ok(body)) = (canonical_target, form_target, body)
    else {
        return;
    };
    let exchanges = [MockExchange::new(
        ExpectedRequest::new(Method::Get, canonical_target),
        ResponseFixture::success(body),
    )];
    let transport = MockTransport::new(&exchanges);
    let mut response = [0_u8; 2];

    assert!(matches!(
        send_blocking(
            &transport,
            TransportRequest::new(Method::Get, form_target),
            &mut response,
        ),
        Err(MockError::TargetMismatch)
    ));
    assert_eq!(transport.remaining(), 1);
    assert!(
        send_blocking(
            &transport,
            TransportRequest::new(Method::Get, canonical_target),
            &mut response,
        )
        .is_ok()
    );
}

#[test]
fn mock_transport_matches_extension_methods_without_aliasing() {
    let target = RequestTarget::new("/cache");
    let method = Method::extension("PURGE");
    let body = FixtureBody::new(b"");
    if let (Ok(target), Ok(method), Ok(body)) = (target, method, body) {
        let exchange = MockExchange::new(
            ExpectedRequest::new(method, target),
            ResponseFixture::success(body),
        );
        let exchanges = [exchange];
        let transport = MockTransport::new(&exchanges);
        let mut output = [0_u8; 1];
        assert!(
            send_blocking(
                &transport,
                TransportRequest::new(method, target),
                &mut output,
            )
            .is_ok()
        );
    }
}

#[test]
fn mock_transport_does_not_consume_exchange_when_response_buffer_is_small() {
    let target = RequestTarget::new("/actions/1");
    let body = FixtureBody::new(b"response");
    if let (Ok(target), Ok(body)) = (target, body) {
        let exchanges = [MockExchange::new(
            ExpectedRequest::new(Method::Get, target),
            ResponseFixture::success(body),
        )];
        let transport = MockTransport::new(&exchanges);
        let request = TransportRequest::new(Method::Get, target);
        let mut short = [0xa5_u8; 4];
        assert!(matches!(
            send_blocking(&transport, request, &mut short),
            Err(MockError::ResponseBufferTooSmall)
        ));
        assert_eq!(short, [0_u8; 4]);
        assert_eq!(transport.remaining(), 1);
    }
}

#[test]
fn mock_transport_propagates_rate_limit_metadata() {
    let target = RequestTarget::new("/servers?page=1");
    let body = FixtureBody::new(br#"{"servers":[]}"#);
    let rate_limit = RateLimitFixture::new(3600, 3599, 42);
    if let (Ok(target), Ok(body), Ok(rate_limit)) = (target, body, rate_limit) {
        let response = ResponseFixture::paginated(
            body,
            PaginationFixture::new(1, 25, 0, 1).unwrap_or_else(|_| unreachable!()),
        )
        .with_rate_limit(rate_limit);
        let exchanges = [MockExchange::new(
            ExpectedRequest::new(Method::Get, target),
            response,
        )];
        let transport = MockTransport::new(&exchanges);
        let mut output = [0_u8; 32];
        let result = inspect_blocking(
            &transport,
            TransportRequest::new(Method::Get, target),
            &mut output,
            |response| response.rate_limit(),
        );
        assert!(result.is_ok());
        let Some(metadata) = result.ok().flatten() else {
            return;
        };
        assert_eq!(metadata.limit(), 3600);
        assert_eq!(metadata.remaining(), 3599);
        assert_eq!(metadata.reset_epoch_seconds(), 42);
    }
}

#[test]
fn async_mock_transport_matches_blocking_behavior_without_an_executor() {
    let target = RequestTarget::new("/servers/42");
    let body = FixtureBody::new(br#"{"id":42}"#);
    if let (Ok(target), Ok(body)) = (target, body) {
        let exchanges = [MockExchange::new(
            ExpectedRequest::new(Method::Get, target),
            ResponseFixture::success(body),
        )];
        let transport = MockTransport::new(&exchanges);
        let mut output = [0_u8; 32];
        let mut headers = [0_u8; 8192];
        let mut response_buffer = ResponseBuffer::new(&mut output, 32, &mut headers);
        {
            let future = drive_async(
                &transport,
                TransportRequest::new(Method::Get, target),
                response_buffer.writer(),
            );
            let mut future = core::pin::pin!(future);
            let waker = Waker::noop();
            let mut context = Context::from_waker(waker);
            let response = Future::poll(future.as_mut(), &mut context);
            assert!(matches!(response, Poll::Ready(Ok(_))));
        }
        assert!(
            response_buffer
                .with_response(|response| response.body() == br#"{"id":42}"#)
                .is_ok_and(core::convert::identity)
        );
        assert!(transport.is_complete());
    }
}

#[test]
fn dropping_unpolled_async_mock_does_not_consume_or_write() {
    let target = RequestTarget::new("/actions/7");
    let body = FixtureBody::new(b"response");
    if let (Ok(target), Ok(body)) = (target, body) {
        let exchanges = [MockExchange::new(
            ExpectedRequest::new(Method::Get, target),
            ResponseFixture::success(body),
        )];
        let transport = MockTransport::new(&exchanges);
        let mut output = [0xa5_u8; 16];
        let mut headers = [0xa5_u8; 8192];
        {
            let mut response_buffer = ResponseBuffer::new(&mut output, 16, &mut headers);
            let future = drive_async(
                &transport,
                TransportRequest::new(Method::Get, target),
                response_buffer.writer(),
            );
            drop(future);
        }
        assert_eq!(output, [0_u8; 16]);
        assert_eq!(transport.remaining(), 1);
    }
}

#[test]
fn mock_transport_distinguishes_target_and_body_mismatches_without_leaking_debug() {
    let expected_target = RequestTarget::new("/servers");
    let wrong_target = RequestTarget::new("/servers/secret");
    let response_body = FixtureBody::new(b"response-secret");
    if let (Ok(expected_target), Ok(wrong_target), Ok(response_body)) =
        (expected_target, wrong_target, response_body)
    {
        let exchange = MockExchange::new(
            ExpectedRequest::new(Method::Post, expected_target).with_body(b"expected-secret"),
            ResponseFixture::success(response_body),
        );
        let debug = format!("{exchange:?}");
        let exchanges = [exchange];
        let transport = MockTransport::new(&exchanges);
        let mut output = [0_u8; 32];

        assert!(matches!(
            send_blocking(
                &transport,
                TransportRequest::new(Method::Post, wrong_target).with_body(b"expected-secret"),
                &mut output,
            ),
            Err(MockError::TargetMismatch)
        ));
        assert!(matches!(
            send_blocking(
                &transport,
                TransportRequest::new(Method::Post, expected_target).with_body(b"wrong-secret"),
                &mut output,
            ),
            Err(MockError::BodyMismatch)
        ));
        assert_eq!(transport.remaining(), 1);

        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains("secret"));
    }
}

fn send_blocking(
    transport: &MockTransport<'_>,
    request: TransportRequest<'_>,
    storage: &mut [u8],
) -> Result<(), MockError> {
    inspect_blocking(transport, request, storage, |_| ())
}

fn inspect_blocking<R>(
    transport: &MockTransport<'_>,
    request: TransportRequest<'_>,
    storage: &mut [u8],
    inspect: impl for<'response> FnOnce(TransportResponse<'response, '_>) -> R,
) -> Result<R, MockError> {
    let capacity = storage.len();
    let mut headers = [0_u8; 8192];
    let mut response = ResponseBuffer::new(storage, capacity, &mut headers);
    BlockingTransport::send(transport, request, response.writer())?;
    response
        .with_response(inspect)
        .map_err(|_| MockError::ResponseWriterRejected)
}

#[test]
fn adversarial_corpus_is_complete_and_oversized_case_is_compact() {
    let corpus = adversarial_corpus();
    assert!(corpus.is_ok());
    let Ok(corpus) = corpus else { return };
    let expected = [
        AdversarialKind::MalformedJson,
        AdversarialKind::UnknownFields,
        AdversarialKind::MissingRequiredFields,
        AdversarialKind::OversizedResponse,
        AdversarialKind::InvalidPagination,
        AdversarialKind::InvalidActionState,
    ];
    for (fixture, kind) in corpus.iter().zip(expected) {
        assert_eq!(fixture.kind(), kind);
    }
    let oversized = corpus
        .iter()
        .find(|fixture| fixture.kind() == AdversarialKind::OversizedResponse);
    assert!(oversized.is_some());
    if let Some(oversized) = oversized {
        assert_eq!(oversized.body().as_bytes(), None);
        assert_eq!(oversized.body().len(), 8_388_609);
    }
}
