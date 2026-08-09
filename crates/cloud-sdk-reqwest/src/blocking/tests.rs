use std::io::Cursor;
use std::string::String;
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{
    ContentType, RequestHeader, RequestHeaders, RequestTarget, StatusCode, TransportRequest,
};

use super::body::{ReadBodyError, read_bounded};
use super::{
    BearerToken, BlockingClientBuilder, CustomEndpointAcknowledgement, EndpointError,
    HttpsEndpoint, RequestTimeouts, TimeoutError, TransportError, UserAgent,
};
use crate::test_server::spawn;

mod authentication_policy;
mod basic;
mod endpoint_policy;
mod lifecycle;
mod method_domain;
mod raw_executor;
mod response_content_type;
mod support;

use support::{send_test, test_credential};

fn test_timeouts() -> Option<RequestTimeouts> {
    RequestTimeouts::new(Duration::from_secs(2), Duration::from_secs(1)).ok()
}

fn custom_endpoint(value: &str) -> Result<HttpsEndpoint, EndpointError> {
    HttpsEndpoint::new_custom(
        value,
        CustomEndpointAcknowledgement::trusted_operator_configuration(),
    )
}

fn build_loopback(endpoint: &str) -> Option<super::BlockingClient> {
    let endpoint = HttpsEndpoint::local_http(endpoint).ok()?;
    let token = BearerToken::new("test-token").ok()?;
    let user_agent = UserAgent::new("cloud-sdk-test/0.18").ok()?;
    let timeouts = test_timeouts()?;
    let credential = test_credential(token, &endpoint);
    let builder = BlockingClientBuilder::new(endpoint, credential, user_agent, timeouts);
    builder.build_for_loopback().ok()
}

#[test]
fn timeouts_are_explicit_nonzero_and_bounded() {
    assert_eq!(
        RequestTimeouts::new(Duration::ZERO, Duration::from_secs(1)),
        Err(TimeoutError::Zero)
    );
    assert_eq!(
        RequestTimeouts::new(Duration::from_secs(1), Duration::from_secs(2)),
        Err(TimeoutError::ExceedsTotal)
    );
    assert!(test_timeouts().is_some());
}

#[test]
fn bounded_reads_detect_overflow_without_panicking() {
    let mut exact_reader = Cursor::new(b"response".as_slice());
    let mut exact = [0_u8; 8];
    assert_eq!(read_bounded(&mut exact_reader, &mut exact), Ok(8));
    assert_eq!(exact, *b"response");

    let mut oversized_reader = Cursor::new(b"oversized".as_slice());
    let mut short = [0_u8; 4];
    assert!(matches!(
        read_bounded(&mut oversized_reader, &mut short),
        Err(ReadBodyError::TooLarge)
    ));
}

#[test]
fn blocking_client_sends_exact_headers_target_and_body_once() {
    let server = spawn(
        "503 Service Unavailable",
        &[],
        b"retry-later",
        Duration::ZERO,
    );
    let Ok(server) = server else {
        unreachable!("security fixture construction failed")
    };
    let client = build_loopback(&server.endpoint);
    assert!(client.is_some());
    let Some(client) = client else {
        unreachable!("security fixture construction failed")
    };
    let target = RequestTarget::new("/servers?name=test%20server");
    assert!(target.is_ok());
    let Ok(target) = target else {
        unreachable!("security fixture construction failed")
    };
    let sensitive = RequestHeader::sensitive("x-test-secret", "redacted-value");
    assert!(sensitive.is_ok());
    let Ok(sensitive) = sensitive else {
        unreachable!("security fixture construction failed")
    };
    let entries = [
        RequestHeader::accept(cloud_sdk::transport::MediaType::JSON),
        RequestHeader::content_type(ContentType::JSON),
        sensitive,
    ];
    let headers = RequestHeaders::new(&entries);
    assert!(headers.is_ok());
    let Ok(headers) = headers else {
        unreachable!("security fixture construction failed")
    };
    let request = TransportRequest::new(Method::Post, target)
        .with_body(br#"{"name":"server"}"#)
        .with_headers(headers);
    let mut output = [0xa5_u8; 32];
    let response = send_test(&client, request, &mut output);
    assert!(matches!(
        response,
        Err(TransportError::ResponseCommitFailed)
    ));

    let recorded = server.request.recv_timeout(Duration::from_secs(2));
    assert!(recorded.is_ok());
    let Ok(recorded) = recorded else {
        unreachable!("security fixture construction failed");
    };
    let wire = String::from_utf8_lossy(&recorded.bytes).to_ascii_lowercase();
    assert!(wire.starts_with("post /v1/servers?name=test%20server http/1.1\r\n"));
    assert!(wire.contains("authorization: bearer test-token\r\n"));
    assert!(wire.contains("user-agent: cloud-sdk-test/0.18\r\n"));
    assert!(wire.contains("accept: application/json\r\n"));
    assert!(wire.contains("content-type: application/json\r\n"));
    assert!(wire.contains("x-test-secret: redacted-value\r\n"));
    assert!(wire.ends_with(r#"{"name":"server"}"#));
}

#[test]
fn redirects_are_not_followed_or_admitted_and_oversized_bodies_are_cleared() {
    let redirect = spawn(
        "302 Found",
        &[("Location", "https://evil.example/steal")],
        b"redirect",
        Duration::ZERO,
    );
    let Ok(redirect) = redirect else {
        unreachable!("security fixture construction failed")
    };
    let Some(client) = build_loopback(&redirect.endpoint) else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = RequestTarget::new("/servers") else {
        unreachable!("security fixture construction failed");
    };
    let mut output = [0_u8; 16];
    let response = send_test(
        &client,
        TransportRequest::new(Method::Get, target),
        &mut output,
    );
    assert!(matches!(
        response,
        Err(TransportError::ResponseCommitFailed)
    ));
    assert_eq!(output, [0_u8; 16]);

    let oversized = spawn("200 OK", &[], b"oversized", Duration::ZERO);
    let Ok(oversized) = oversized else {
        unreachable!("security fixture construction failed")
    };
    let Some(client) = build_loopback(&oversized.endpoint) else {
        unreachable!("security fixture construction failed");
    };
    let mut short = [0xa5_u8; 4];
    assert!(matches!(
        send_test(
            &client,
            TransportRequest::new(Method::Get, target),
            &mut short,
        ),
        Err(TransportError::RawHttp(
            super::RawHttpError::ResponseTooLarge
        ))
    ));
    assert_eq!(short, [0_u8; 4]);
}

#[test]
fn checked_response_exposes_content_type_without_transport_rate_limit_decoding() {
    let server = spawn(
        "200 OK",
        &[
            ("Content-Type", "application/json; charset=utf-8"),
            ("RateLimit-Limit", "3600"),
            ("RateLimit-Remaining", "3599"),
            ("RateLimit-Reset", "42"),
        ],
        b"{}",
        Duration::ZERO,
    );
    let Ok(server) = server else {
        unreachable!("security fixture construction failed")
    };
    let Some(client) = build_loopback(&server.endpoint) else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = RequestTarget::new("/servers") else {
        unreachable!("security fixture construction failed");
    };
    let mut output = [0_u8; 8];
    let response = send_test(
        &client,
        TransportRequest::new(Method::Get, target),
        &mut output,
    );
    assert!(response.is_ok());
    let Ok(response) = response else {
        unreachable!("security fixture construction failed")
    };
    let Some(content_type) = response.content_type() else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(content_type, "application/json; charset=utf-8");
    assert_eq!(response.rate_limit(), None);
}

#[test]
fn incomplete_rate_limit_headers_are_retained_for_provider_decoding() {
    let server = spawn(
        "200 OK",
        &[("RateLimit-Limit", "3600")],
        b"secret",
        Duration::ZERO,
    );
    let Ok(server) = server else {
        unreachable!("security fixture construction failed")
    };
    let Some(client) = build_loopback(&server.endpoint) else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = RequestTarget::new("/servers") else {
        unreachable!("security fixture construction failed");
    };
    let mut output = [0xa5_u8; 8];
    let response = send_test(
        &client,
        TransportRequest::new(Method::Get, target),
        &mut output,
    );
    assert!(response.is_ok());
}

#[test]
fn duplicate_rate_limit_headers_fail_closed() {
    let server = spawn(
        "200 OK",
        &[
            ("RateLimit-Limit", "3600"),
            ("RateLimit-Limit", "7200"),
            ("RateLimit-Remaining", "3599"),
            ("RateLimit-Reset", "42"),
        ],
        b"secret",
        Duration::ZERO,
    );
    let Ok(server) = server else {
        unreachable!("security fixture construction failed")
    };
    let Some(client) = build_loopback(&server.endpoint) else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = RequestTarget::new("/servers") else {
        unreachable!("security fixture construction failed");
    };
    let mut output = [0xa5_u8; 8];
    assert!(matches!(
        send_test(
            &client,
            TransportRequest::new(Method::Get, target),
            &mut output,
        ),
        Err(TransportError::RawHttp(
            super::RawHttpError::DuplicateResponseHeader
        ))
    ));
    assert_eq!(output, [0_u8; 8]);
}

#[test]
fn nonempty_body_requires_content_type_before_network_access() {
    let Some(client) = build_loopback("http://127.0.0.1:9/v1") else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = RequestTarget::new("/servers") else {
        unreachable!("security fixture construction failed");
    };
    let mut output = [0xa5_u8; 8];
    assert!(matches!(
        send_test(
            &client,
            TransportRequest::new(Method::Post, target).with_body(b"{}"),
            &mut output,
        ),
        Err(TransportError::RawHttp(
            super::RawHttpError::MissingContentType
        ))
    ));
    assert_eq!(output, [0_u8; 8]);
}

#[test]
fn response_timeout_is_payload_free_and_clears_output() {
    let server = spawn("200 OK", &[], b"late", Duration::from_millis(100));
    let Ok(server) = server else {
        unreachable!("security fixture construction failed")
    };
    let endpoint = HttpsEndpoint::local_http(&server.endpoint);
    let token = BearerToken::new("test-token");
    let user_agent = UserAgent::new("cloud-sdk-test/0.18");
    let timeouts = RequestTimeouts::new(Duration::from_millis(40), Duration::from_millis(20));
    let (Ok(endpoint), Ok(token), Ok(user_agent), Ok(timeouts)) =
        (endpoint, token, user_agent, timeouts)
    else {
        unreachable!("security fixture construction failed");
    };
    let credential = test_credential(token, &endpoint);
    let builder = BlockingClientBuilder::new(endpoint, credential, user_agent, timeouts);
    let client = builder.build_for_loopback();
    let Ok(client) = client else {
        unreachable!("security fixture construction failed")
    };
    let Ok(target) = RequestTarget::new("/slow") else {
        unreachable!("security fixture construction failed");
    };
    let mut output = [0xa5_u8; 8];
    assert!(matches!(
        send_test(
            &client,
            TransportRequest::new(Method::Get, target),
            &mut output,
        ),
        Err(TransportError::RawHttp(super::RawHttpError::TimedOut))
    ));
    assert_eq!(output, [0_u8; 8]);
}

#[test]
fn status_constant_remains_compatible_with_transport_response() {
    assert_eq!(StatusCode::OK.get(), 200);
}
