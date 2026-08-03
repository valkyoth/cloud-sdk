use std::string::String;
use std::time::Duration;
use std::vec::Vec;

use cloud_sdk::Method;
use cloud_sdk::authentication::drive_async_authenticated;
use cloud_sdk::rate_limit::RateLimit;
use cloud_sdk::transport::{
    AsyncExecutionError, ContentType, RequestHeader, RequestHeaders, RequestTarget, ResponseBuffer,
    ResponseStorageSanitizer, StatusCode, TransportRequest, TransportResponse,
};

use super::{
    AsyncClient, AsyncClientBuilder, BearerToken, HttpsEndpoint, RequestTimeouts, TransportError,
    UserAgent,
};
use crate::test_server::{spawn, spawn_split};

mod authentication_policy;
mod basic;
mod lifecycle;
mod raw_executor;
mod support;

use support::{authenticated, test_credential};

fn run_async_test(future: impl core::future::Future<Output = ()>) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build();
    assert!(runtime.is_ok());
    if let Ok(runtime) = runtime {
        runtime.block_on(future);
    }
}

fn test_timeouts() -> Option<RequestTimeouts> {
    RequestTimeouts::new(Duration::from_secs(2), Duration::from_secs(1)).ok()
}

#[test]
fn async_prepared_cleanup_contract_clears_the_complete_caller_buffer() {
    let client = build_loopback("http://127.0.0.1:1/v1");
    assert!(client.is_some());
    let Some(client) = client else { return };
    let mut output = [0xA5_u8; 64];
    client.sanitize_response_storage(&mut output);
    assert_eq!(output, [0_u8; 64]);
}

fn build_loopback(endpoint: &str) -> Option<AsyncClient> {
    let endpoint = HttpsEndpoint::local_http(endpoint).ok()?;
    let token = BearerToken::new("test-token").ok()?;
    let user_agent = UserAgent::new("cloud-sdk-test/0.18").ok()?;
    let timeouts = test_timeouts()?;
    let credential = test_credential(token, &endpoint);
    AsyncClientBuilder::new(endpoint, credential, user_agent, timeouts)
        .build_for_loopback()
        .ok()
}

struct CapturedResponse {
    status: StatusCode,
    body: Vec<u8>,
    content_type: Option<String>,
    rate_limit: Option<RateLimit>,
    rate_limit_remaining: Option<Vec<u8>>,
    content_type_header: Option<Vec<u8>>,
}

impl CapturedResponse {
    fn capture(response: TransportResponse<'_, '_>) -> Self {
        Self {
            status: response.status(),
            body: response.body().to_vec(),
            content_type: response
                .content_type()
                .ok()
                .flatten()
                .map(|content_type| String::from(content_type.as_str())),
            rate_limit: response.rate_limit(),
            rate_limit_remaining: response
                .headers()
                .get("ratelimit-remaining")
                .map(|header| header.value().to_vec()),
            content_type_header: response
                .headers()
                .get("content-type")
                .map(|header| header.value().to_vec()),
        }
    }

    const fn status(&self) -> StatusCode {
        self.status
    }

    fn body(&self) -> &[u8] {
        &self.body
    }

    fn content_type(&self) -> Option<&str> {
        self.content_type.as_deref()
    }

    const fn rate_limit(&self) -> Option<RateLimit> {
        self.rate_limit
    }

    fn rate_limit_remaining_header(&self) -> Option<&[u8]> {
        self.rate_limit_remaining.as_deref()
    }

    fn content_type_header(&self) -> Option<&[u8]> {
        self.content_type_header.as_deref()
    }
}

async fn send_test(
    client: &AsyncClient,
    request: TransportRequest<'_>,
    output: &mut [u8],
) -> Result<CapturedResponse, TransportError> {
    let capacity = output.len();
    let mut headers = [0_u8; 8192];
    let mut response = ResponseBuffer::new(output, capacity, &mut headers);
    drive_async_authenticated(client, authenticated(client, request), response.writer())
        .await
        .map_err(|failure| match failure {
            AsyncExecutionError::Transport(failure) => failure.into_error(),
            AsyncExecutionError::Response(_) => TransportError::ResponseCommitFailed,
        })?;
    response
        .with_response(CapturedResponse::capture)
        .map_err(|_| TransportError::ResponseCommitFailed)
}

#[test]
fn async_client_sends_exact_headers_target_and_body_once() {
    run_async_test(async {
        let server = spawn(
            "503 Service Unavailable",
            &[],
            b"retry-later",
            Duration::ZERO,
        );
        let Ok(server) = server else { return };
        let Some(client) = build_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = RequestTarget::new("/servers?name=test%20server") else {
            return;
        };
        let Ok(sensitive) = RequestHeader::sensitive("x-test-secret", "redacted-value") else {
            return;
        };
        let entries = [
            RequestHeader::accept(cloud_sdk::transport::MediaType::JSON),
            RequestHeader::content_type(ContentType::JSON),
            sensitive,
        ];
        let Ok(headers) = RequestHeaders::new(&entries) else {
            return;
        };
        let request = TransportRequest::new(Method::Post, target)
            .with_body(br#"{"name":"server"}"#)
            .with_headers(headers);
        let mut output = [0xa5_u8; 32];
        let response = send_test(&client, request, &mut output).await;
        assert!(response.is_ok());
        if let Ok(response) = response {
            assert_eq!(response.status().get(), 503);
            assert_eq!(response.body(), b"retry-later");
        }

        let recorded = server.request.recv_timeout(Duration::from_secs(2));
        assert!(recorded.is_ok());
        if let Ok(recorded) = recorded {
            let wire = String::from_utf8_lossy(&recorded.bytes).to_ascii_lowercase();
            assert!(wire.starts_with("post /v1/servers?name=test%20server http/1.1\r\n"));
            assert!(wire.contains("authorization: bearer test-token\r\n"));
            assert!(wire.contains("user-agent: cloud-sdk-test/0.18\r\n"));
            assert!(wire.contains("accept: application/json\r\n"));
            assert!(wire.contains("content-type: application/json\r\n"));
            assert!(wire.contains("x-test-secret: redacted-value\r\n"));
            assert!(wire.contains("content-type: application/json\r\n"));
            assert!(wire.ends_with(r#"{"name":"server"}"#));
        }
    });
}

#[test]
fn async_client_sends_complete_method_domain_exactly() {
    run_async_test(async {
        for method in [
            Method::Patch,
            Method::Head,
            Method::Options,
            Method::extension("PURGE").unwrap_or_else(|_| unreachable!()),
        ] {
            let server = spawn("200 OK", &[], b"", Duration::ZERO).ok();
            assert!(server.is_some(), "method-domain loopback server must start");
            let Some(server) = server else {
                unreachable!("successful server assertion guarantees a loopback server")
            };
            let client = build_loopback(&server.endpoint);
            assert!(client.is_some(), "method-domain loopback client must build");
            let Some(client) = client else {
                unreachable!("successful client assertion guarantees a loopback client")
            };
            let target = RequestTarget::new("/method-check");
            assert!(target.is_ok(), "static method-domain target must be valid");
            let Ok(target) = target else {
                unreachable!("successful target assertion guarantees a request target")
            };
            let mut output = [0_u8; 1];
            let response =
                send_test(&client, TransportRequest::new(method, target), &mut output).await;
            assert!(response.is_ok());

            let recorded = server.request.recv_timeout(Duration::from_secs(2));
            assert!(recorded.is_ok());
            let Ok(recorded) = recorded else {
                unreachable!("successful receive assertion guarantees a recorded request")
            };
            let wire = String::from_utf8_lossy(&recorded.bytes);
            assert!(wire.starts_with(method.as_str()));
            assert!(wire[method.as_str().len()..].starts_with(" /v1/method-check HTTP/1.1\r\n"));
        }
    });
}

#[test]
fn async_redirect_is_not_followed_and_oversized_body_is_rejected() {
    run_async_test(async {
        let redirect = spawn(
            "302 Found",
            &[("Location", "https://evil.example/steal")],
            b"redirect",
            Duration::ZERO,
        );
        let Ok(redirect) = redirect else { return };
        let Some(client) = build_loopback(&redirect.endpoint) else {
            return;
        };
        let Ok(target) = RequestTarget::new("/servers") else {
            return;
        };
        let mut output = [0_u8; 16];
        let response = send_test(
            &client,
            TransportRequest::new(Method::Get, target),
            &mut output,
        )
        .await;
        assert!(response.is_ok());
        if let Ok(response) = response {
            assert_eq!(response.status().get(), 302);
            assert_eq!(response.body(), b"redirect");
        }

        let oversized = spawn("200 OK", &[], b"oversized", Duration::ZERO);
        let Ok(oversized) = oversized else { return };
        let Some(client) = build_loopback(&oversized.endpoint) else {
            return;
        };
        let mut short = [0xa5_u8; 4];
        let result = send_test(
            &client,
            TransportRequest::new(Method::Get, target),
            &mut short,
        )
        .await;
        assert!(matches!(
            result,
            Err(TransportError::RawHttp(
                super::RawHttpError::ResponseTooLarge
            ))
        ));
        assert_eq!(short, [0_u8; 4]);
    });
}

#[test]
fn async_response_retains_admitted_rate_limit_headers_without_transport_decoding() {
    run_async_test(async {
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
        let Ok(server) = server else { return };
        let Some(client) = build_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = RequestTarget::new("/servers") else {
            return;
        };
        let mut output = [0_u8; 8];
        let response = send_test(
            &client,
            TransportRequest::new(Method::Get, target),
            &mut output,
        )
        .await;
        assert!(response.is_ok());
        let Ok(response) = response else { return };
        let Some(content_type) = response.content_type() else {
            return;
        };
        assert_eq!(content_type, "application/json; charset=utf-8");
        assert_eq!(response.rate_limit(), None);
        assert_eq!(
            response.rate_limit_remaining_header(),
            Some(b"3599".as_slice())
        );
        assert_eq!(
            response.content_type_header(),
            Some(b"application/json; charset=utf-8".as_slice())
        );
    });
}

#[test]
fn async_malformed_or_duplicate_response_content_type_fails_closed() {
    run_async_test(async {
        let Ok(target) = RequestTarget::new("/servers") else {
            return;
        };
        for (headers, expected) in [
            (
                &[("Content-Type", "application/json; charset")][..],
                TransportError::RawHttp(super::RawHttpError::InvalidResponseContentType),
            ),
            (
                &[
                    ("Content-Type", "application/json"),
                    ("Content-Type", "text/plain"),
                ][..],
                TransportError::RawHttp(super::RawHttpError::DuplicateResponseHeader),
            ),
        ] {
            let server = spawn("200 OK", headers, b"secret", Duration::ZERO);
            let Ok(server) = server else { return };
            let Some(client) = build_loopback(&server.endpoint) else {
                return;
            };
            let mut output = [0xa5_u8; 8];
            assert!(matches!(
                send_test(
                    &client,
                    TransportRequest::new(Method::Get, target),
                    &mut output,
                )
                .await,
                Err(error) if error == expected
            ));
            assert_eq!(output, [0_u8; 8]);
        }
    });
}

#[test]
fn async_duplicate_rate_limit_headers_fail_closed() {
    run_async_test(async {
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
        let Ok(server) = server else { return };
        let Some(client) = build_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = RequestTarget::new("/servers") else {
            return;
        };
        let mut output = [0xa5_u8; 8];
        let result = send_test(
            &client,
            TransportRequest::new(Method::Get, target),
            &mut output,
        )
        .await;
        assert!(matches!(
            result,
            Err(TransportError::RawHttp(
                super::RawHttpError::DuplicateResponseHeader
            ))
        ));
        assert_eq!(output, [0_u8; 8]);
    });
}

#[test]
fn internal_timeout_is_payload_free_and_clears_output() {
    run_async_test(async {
        let server = spawn("200 OK", &[], b"late", Duration::from_millis(100));
        let Ok(server) = server else { return };
        let endpoint = HttpsEndpoint::local_http(&server.endpoint);
        let token = BearerToken::new("test-token");
        let user_agent = UserAgent::new("cloud-sdk-test/0.18");
        let timeouts = RequestTimeouts::new(Duration::from_millis(40), Duration::from_millis(20));
        let (Ok(endpoint), Ok(token), Ok(user_agent), Ok(timeouts)) =
            (endpoint, token, user_agent, timeouts)
        else {
            return;
        };
        let credential = test_credential(token, &endpoint);
        let client = AsyncClientBuilder::new(endpoint, credential, user_agent, timeouts)
            .build_for_loopback();
        let Ok(client) = client else { return };
        let Ok(target) = RequestTarget::new("/slow") else {
            return;
        };
        let mut output = [0xa5_u8; 8];
        let result = send_test(
            &client,
            TransportRequest::new(Method::Get, target),
            &mut output,
        )
        .await;
        assert!(matches!(
            result,
            Err(TransportError::RawHttp(super::RawHttpError::TimedOut))
        ));
        assert_eq!(output, [0_u8; 8]);
    });
}

#[test]
fn caller_cancellation_after_partial_body_never_exposes_response() {
    run_async_test(async {
        let server = spawn_split(
            "200 OK",
            b"secret-prefix",
            b"-tail",
            Duration::from_millis(500),
        );
        let Ok(server) = server else { return };
        let Some(client) = build_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = RequestTarget::new("/slow") else {
            return;
        };
        let mut output = [0xa5_u8; 32];
        let mut headers = [0xa5_u8; 8192];
        {
            let mut response = ResponseBuffer::new(&mut output, 32, &mut headers);
            let future = drive_async_authenticated(
                &client,
                authenticated(&client, TransportRequest::new(Method::Get, target)),
                response.writer(),
            );
            let result = tokio::time::timeout(Duration::from_millis(100), future).await;
            assert!(result.is_err(), "unexpected early completion: {result:?}");
        }
        assert_eq!(output, [0_u8; 32]);
        assert_eq!(headers, [0_u8; 8192]);
    });
}
