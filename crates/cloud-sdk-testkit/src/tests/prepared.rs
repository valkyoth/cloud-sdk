use core::future::Future;
use core::task::{Context, Poll, Waker};

use cloud_sdk::operation::{
    ContentTypePolicy, CostIntent, OperationImpact, OperationMetadata, PreparedExecutionError,
    PreparedRequest, ProviderService, RequestSemantics, ResponseBodyPolicy, ResponsePolicy,
    ResponsePolicyError, RetryEligibility,
};
use cloud_sdk::transport::{
    ContentType, EndpointIdentity, EndpointIdentityError, EndpointScheme, MediaType, RequestTarget,
    StatusCode, TransportRequest,
};
use cloud_sdk::{ApiFamily, Method, Provider};

use crate::{
    ExpectedRequest, FixtureBody, MockError, MockExchange, MockTransport, PreparedRequestRecord,
    ResponseFixture,
};

static OK_STATUS: [StatusCode; 1] = [StatusCode::OK];
static JSON_MEDIA: [MediaType<'static>; 1] = [MediaType::JSON];

#[test]
fn prepared_records_capture_policy_and_redact_request_values() {
    let prepared = prepared_request(16);
    assert!(prepared.is_ok());
    let Ok(prepared) = prepared else { return };
    let record = PreparedRequestRecord::capture(prepared);
    assert_eq!(record.method(), Method::Post);
    assert_eq!(record.target_len(), 8);
    assert_eq!(record.body_len(), 2);
    assert!(record.has_request_content_type());
    assert_eq!(record.service().provider(), Provider::Hetzner);
    assert_eq!(record.service().family(), ApiFamily::Cloud);
    assert_eq!(record.metadata().impact(), OperationImpact::Mutation);
    assert_eq!(
        record.metadata().retry_eligibility(),
        RetryEligibility::ExplicitPolicy
    );
    assert_eq!(record.response_policy().max_body_bytes(), 16);

    let debug = alloc::format!("{record:?}");
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("/servers"));
    assert!(!debug.contains("{}"));
}

#[test]
fn bound_mock_executes_prepared_requests_for_blocking_and_async_contracts() {
    let prepared = prepared_request(16);
    let exchange = successful_exchange();
    let endpoint = official_endpoint();
    assert!(prepared.is_ok() && exchange.is_ok() && endpoint.is_ok());
    let (Ok(prepared), Ok(exchange), Ok(endpoint)) = (prepared, exchange, endpoint) else {
        return;
    };
    let exchanges = [exchange, exchange];
    let mock = MockTransport::new(&exchanges).with_endpoint(endpoint);

    let mut blocking_output = [0_u8; 32];
    let blocking = prepared.execute_blocking(&mock, &mut blocking_output);
    assert!(blocking.is_ok_and(|response| response.body() == b"{}"));

    let mut async_output = [0_u8; 32];
    let future = prepared.execute_async(&mock, &mut async_output);
    let mut future = core::pin::pin!(future);
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    let asynchronous = Future::poll(future.as_mut(), &mut context);
    assert!(matches!(asynchronous, Poll::Ready(Ok(_))));
    assert!(mock.is_complete());
}

#[test]
fn mock_models_endpoint_status_content_type_and_empty_body_failures() {
    let prepared = prepared_request(16);
    let expected = expected_request();
    let endpoint = official_endpoint();
    let other = other_endpoint();
    assert!(prepared.is_ok() && expected.is_ok() && endpoint.is_ok() && other.is_ok());
    let (Ok(prepared), Ok(expected), Ok(endpoint), Ok(other)) =
        (prepared, expected, endpoint, other)
    else {
        return;
    };
    let Ok(json_body) = FixtureBody::new(b"{}") else {
        return;
    };
    let Ok(empty_body) = FixtureBody::new(b"") else {
        return;
    };

    let success = ResponseFixture::success(json_body).with_content_type("application/json");
    let exchanges = [MockExchange::new(expected, success)];
    let wrong_endpoint = MockTransport::new(&exchanges).with_endpoint(other);
    let mut output = [0_u8; 16];
    assert_eq!(
        prepared.execute_blocking(&wrong_endpoint, &mut output),
        Err(PreparedExecutionError::EndpointMismatch)
    );
    assert_eq!(wrong_endpoint.remaining(), 1);

    let error = ResponseFixture::error(StatusCode::TOO_MANY_REQUESTS, json_body);
    assert!(error.is_ok());
    if let Ok(error) = error {
        let exchanges = [MockExchange::new(
            expected,
            error.with_content_type("application/json"),
        )];
        let mock = MockTransport::new(&exchanges).with_endpoint(endpoint);
        assert_eq!(
            prepared.execute_blocking(&mock, &mut output),
            Err(PreparedExecutionError::ResponsePolicy(
                ResponsePolicyError::UnexpectedStatus
            ))
        );
    }

    for (fixture, expected_error) in [
        (
            ResponseFixture::success(json_body),
            ResponsePolicyError::MissingContentType,
        ),
        (
            ResponseFixture::success(json_body).with_content_type("text/plain"),
            ResponsePolicyError::UnexpectedContentType,
        ),
        (
            ResponseFixture::success(empty_body).with_content_type("application/json"),
            ResponsePolicyError::MissingBody,
        ),
    ] {
        let exchanges = [MockExchange::new(expected, fixture)];
        let mock = MockTransport::new(&exchanges).with_endpoint(endpoint);
        assert_eq!(
            prepared.execute_blocking(&mock, &mut output),
            Err(PreparedExecutionError::ResponsePolicy(expected_error))
        );
    }
}

#[test]
fn mock_models_oversized_responses_and_retry_classification_mistakes() {
    let prepared = prepared_request(2);
    let expected = expected_request();
    let endpoint = official_endpoint();
    assert!(prepared.is_ok() && expected.is_ok() && endpoint.is_ok());
    let (Ok(prepared), Ok(expected), Ok(endpoint)) = (prepared, expected, endpoint) else {
        return;
    };
    let Ok(oversized_body) = FixtureBody::new(b"123") else {
        return;
    };
    let fixture = ResponseFixture::success(oversized_body).with_content_type("application/json");
    let exchanges = [MockExchange::new(expected, fixture)];
    let mock = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut output = [0_u8; 64];
    assert_eq!(
        prepared.execute_blocking(&mock, &mut output),
        Err(PreparedExecutionError::Transport(
            MockError::ResponseBufferTooSmall
        ))
    );
    assert_eq!(mock.remaining(), 1);

    let record = PreparedRequestRecord::capture(prepared);
    assert_ne!(record.metadata().impact(), OperationImpact::ReadOnly);
    assert_ne!(record.metadata().semantics(), RequestSemantics::Safe);
    assert_eq!(
        record.metadata().retry_eligibility(),
        RetryEligibility::ExplicitPolicy
    );
}

fn prepared_request(max_body_bytes: usize) -> Result<PreparedRequest<'static>, ()> {
    let target = RequestTarget::new("/servers").map_err(|_| ())?;
    let request = TransportRequest::new(Method::Post, target)
        .with_body(b"{}")
        .with_content_type(ContentType::JSON);
    let metadata = OperationMetadata::new(
        OperationImpact::Mutation,
        RequestSemantics::Idempotent,
        RetryEligibility::ExplicitPolicy,
        CostIntent::MayIncurCost,
    )
    .map_err(|_| ())?;
    let response_policy = ResponsePolicy::new(
        &OK_STATUS,
        ContentTypePolicy::Required(&JSON_MEDIA),
        ResponseBodyPolicy::Required,
        max_body_bytes,
    )
    .map_err(|_| ())?;
    let endpoint = official_endpoint().map_err(|_| ())?;
    Ok(PreparedRequest::new(
        request,
        ProviderService::new(Provider::Hetzner, ApiFamily::Cloud, endpoint),
        metadata,
        response_policy,
    ))
}

fn expected_request() -> Result<ExpectedRequest<'static>, ()> {
    let target = RequestTarget::new("/servers").map_err(|_| ())?;
    Ok(ExpectedRequest::new(Method::Post, target)
        .with_body(b"{}")
        .with_content_type(ContentType::JSON))
}

fn successful_exchange() -> Result<MockExchange<'static>, ()> {
    let body = FixtureBody::new(b"{}").map_err(|_| ())?;
    Ok(MockExchange::new(
        expected_request()?,
        ResponseFixture::success(body).with_content_type("application/json; charset=utf-8"),
    ))
}

fn official_endpoint() -> Result<EndpointIdentity<'static>, EndpointIdentityError> {
    EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1")
}

fn other_endpoint() -> Result<EndpointIdentity<'static>, EndpointIdentityError> {
    EndpointIdentity::new(EndpointScheme::Https, "example.invalid", 443, "/v1")
}
