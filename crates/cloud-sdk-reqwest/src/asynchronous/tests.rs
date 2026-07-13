use std::string::String;
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{AsyncTransport, ContentType, RequestTarget, TransportRequest};

use super::{
    AsyncClient, AsyncClientBuilder, BearerToken, HttpsEndpoint, RequestTimeouts, TransportError,
    UserAgent,
};
use crate::test_server::{spawn, spawn_split};

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

fn build_loopback(endpoint: &str) -> Option<AsyncClient> {
    let endpoint = HttpsEndpoint::local_http(endpoint).ok()?;
    let token = BearerToken::new("test-token").ok()?;
    let user_agent = UserAgent::new("cloud-sdk-test/0.18").ok()?;
    let timeouts = test_timeouts()?;
    AsyncClientBuilder::new(endpoint, token, user_agent, timeouts)
        .build_for_loopback()
        .ok()
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
        let Some(mut client) = build_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = RequestTarget::new("/servers?name=test%20server") else {
            return;
        };
        let request = TransportRequest::new(Method::Post, target)
            .with_body(br#"{"name":"server"}"#)
            .with_content_type(ContentType::JSON);
        let mut output = [0xa5_u8; 32];
        let response = AsyncTransport::send(&mut client, request, &mut output).await;
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
            assert!(wire.contains("content-type: application/json\r\n"));
            assert!(wire.ends_with(r#"{"name":"server"}"#));
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
        let Some(mut client) = build_loopback(&redirect.endpoint) else {
            return;
        };
        let Ok(target) = RequestTarget::new("/servers") else {
            return;
        };
        let mut output = [0_u8; 16];
        let response = AsyncTransport::send(
            &mut client,
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
        let Some(mut client) = build_loopback(&oversized.endpoint) else {
            return;
        };
        let mut short = [0xa5_u8; 4];
        let result = AsyncTransport::send(
            &mut client,
            TransportRequest::new(Method::Get, target),
            &mut short,
        )
        .await;
        assert!(matches!(result, Err(TransportError::ResponseTooLarge)));
        assert_eq!(short, [0_u8; 4]);
    });
}

#[test]
fn async_response_propagates_validated_rate_limit_headers() {
    run_async_test(async {
        let server = spawn(
            "200 OK",
            &[
                ("RateLimit-Limit", "3600"),
                ("RateLimit-Remaining", "3599"),
                ("RateLimit-Reset", "42"),
            ],
            b"{}",
            Duration::ZERO,
        );
        let Ok(server) = server else { return };
        let Some(mut client) = build_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = RequestTarget::new("/servers") else {
            return;
        };
        let mut output = [0_u8; 8];
        let response = AsyncTransport::send(
            &mut client,
            TransportRequest::new(Method::Get, target),
            &mut output,
        )
        .await;
        assert!(response.is_ok());
        let Some(rate_limit) = response.ok().and_then(|value| value.rate_limit()) else {
            return;
        };
        assert_eq!(rate_limit.limit(), 3600);
        assert_eq!(rate_limit.remaining(), 3599);
        assert_eq!(rate_limit.reset_epoch_seconds(), 42);
    });
}

#[test]
fn missing_content_type_fails_before_network_access() {
    run_async_test(async {
        let Some(mut client) = build_loopback("http://127.0.0.1:9/v1") else {
            return;
        };
        let Ok(target) = RequestTarget::new("/servers") else {
            return;
        };
        let mut output = [0xa5_u8; 8];
        let result = AsyncTransport::send(
            &mut client,
            TransportRequest::new(Method::Post, target).with_body(b"{}"),
            &mut output,
        )
        .await;
        assert!(matches!(result, Err(TransportError::MissingContentType)));
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
        let client =
            AsyncClientBuilder::new(endpoint, token, user_agent, timeouts).build_for_loopback();
        let Ok(mut client) = client else { return };
        let Ok(target) = RequestTarget::new("/slow") else {
            return;
        };
        let mut output = [0xa5_u8; 8];
        let result = AsyncTransport::send(
            &mut client,
            TransportRequest::new(Method::Get, target),
            &mut output,
        )
        .await;
        assert!(matches!(result, Err(TransportError::TimedOut)));
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
        let Some(mut client) = build_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = RequestTarget::new("/slow") else {
            return;
        };
        let mut output = [0xa5_u8; 32];
        let future = AsyncTransport::send(
            &mut client,
            TransportRequest::new(Method::Get, target),
            &mut output,
        );
        let result = tokio::time::timeout(Duration::from_millis(100), future).await;
        assert!(result.is_err(), "unexpected early completion: {result:?}");
        assert_eq!(output, [0_u8; 32]);
    });
}

#[test]
fn async_client_debug_redacts_endpoint_and_token() {
    let endpoint = HttpsEndpoint::new("https://api.example.test/v1");
    let token = BearerToken::new("secret-token");
    let user_agent = UserAgent::new("cloud-sdk-test/0.18");
    let timeouts = test_timeouts();
    let (Ok(endpoint), Ok(token), Ok(user_agent), Some(timeouts)) =
        (endpoint, token, user_agent, timeouts)
    else {
        return;
    };
    let builder = AsyncClientBuilder::new(endpoint, token, user_agent, timeouts);
    let debug = std::format!("{builder:?}");
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("secret-token"));
    assert!(!debug.contains("api.example.test"));
}
