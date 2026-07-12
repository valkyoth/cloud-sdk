use alloc::format;
use cloud_sdk::Method;
use cloud_sdk::transport::{BlockingTransport, RequestTarget, StatusCode, TransportRequest};

use crate::{
    ActionFixture, ActionState, AdversarialKind, ExpectedRequest, FixtureBody, FixtureBodyError,
    FixtureKind, FixtureMetadataError, MockError, MockExchange, MockTransport, PaginationFixture,
    RateLimitFixture, ResponseFixture, ResponseFixtureError, adversarial_corpus,
};

#[test]
fn fixture_builders_cover_success_pagination_action_rate_limit_and_error() {
    let body = FixtureBody::new(br#"{"ok":true}"#);
    let pagination = PaginationFixture::new(2, 50, 75, 2);
    let action = ActionFixture::new(ActionState::Running, 25);
    let rate_limit = RateLimitFixture::new(3600, 0, Some(42));
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
        assert_eq!(
            ResponseFixture::error(StatusCode::OK, body),
            Err(ResponseFixtureError::NonErrorStatus)
        );
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
        RateLimitFixture::new(10, 11, None),
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
        let mut transport = MockTransport::new(&exchanges);
        let wrong = TransportRequest::new(Method::Delete, target);
        let mut output = [0xa5_u8; 32];
        assert_eq!(
            transport.send(wrong, &mut output),
            Err(MockError::MethodMismatch)
        );
        assert_eq!(transport.remaining(), 1);

        let request = TransportRequest::new(Method::Get, target);
        let response = transport.send(request, &mut output);
        assert_eq!(response.map(|value| value.body_len()), Ok(14));
        assert!(transport.is_complete());
        assert_eq!(
            transport.send(request, &mut output),
            Err(MockError::Exhausted)
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
        let mut transport = MockTransport::new(&exchanges);
        let request = TransportRequest::new(Method::Get, target);
        let mut short = [0xa5_u8; 4];
        let original = short;
        assert_eq!(
            transport.send(request, &mut short),
            Err(MockError::ResponseBufferTooSmall)
        );
        assert_eq!(short, original);
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
        let exchanges = [exchange];
        let mut transport = MockTransport::new(&exchanges);
        let mut output = [0_u8; 32];

        assert_eq!(
            transport.send(
                TransportRequest::new(Method::Post, wrong_target).with_body(b"expected-secret"),
                &mut output,
            ),
            Err(MockError::TargetMismatch)
        );
        assert_eq!(
            transport.send(
                TransportRequest::new(Method::Post, expected_target).with_body(b"wrong-secret"),
                &mut output,
            ),
            Err(MockError::BodyMismatch)
        );
        assert_eq!(transport.remaining(), 1);

        let debug = format!("{exchange:?}");
        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains("secret"));
    }
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
