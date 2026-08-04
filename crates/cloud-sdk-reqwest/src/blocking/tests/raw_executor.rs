use std::string::String;
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{
    BlockingRawHttpExecutor, DeliveryPhase, HeaderName, MediaType, RawResponsePolicy,
    ResponseBuffer, ResponseMediaPolicy, ResponseMetadata, StatusCode, TransportRequest,
};

use super::{custom_endpoint, test_timeouts};
#[cfg(feature = "blocking-rustls-fips")]
use crate::blocking::tests::fips_tls_policy;
use crate::blocking::{RawBlockingClient, RawBlockingClientBuilder, RawHttpError, UserAgent};
use crate::test_server::{spawn, spawn_concurrent_pair, spawn_raw_response};

mod request_body;

fn build_raw_loopback(endpoint: &str) -> Option<RawBlockingClient> {
    let endpoint = crate::blocking::HttpsEndpoint::local_http(endpoint).ok()?;
    let user_agent = UserAgent::new("cloud-sdk-raw-test/0.40").ok()?;
    let builder = RawBlockingClientBuilder::new(endpoint, user_agent, test_timeouts()?);
    #[cfg(feature = "blocking-rustls-fips")]
    let builder = builder.with_fips_tls_policy(fips_tls_policy()?);
    builder.build_for_loopback().ok()
}

fn json_policy<'a>(admitted: &'a [HeaderName<'a>]) -> Option<RawResponsePolicy<'a>> {
    RawResponsePolicy::new(
        16,
        4,
        ResponseMediaPolicy::Required(&[MediaType::JSON]),
        ResponseMediaPolicy::Optional(&[MediaType::JSON]),
        admitted,
        2,
    )
    .ok()
}

#[test]
fn raw_blocking_precommitted_writer_fails_before_network_access() {
    let Some(client) = build_raw_loopback("http://127.0.0.1:1/v1") else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = cloud_sdk::transport::RequestTarget::new("/precommitted") else {
        unreachable!("security fixture construction failed");
    };
    let Some(policy) = json_policy(&[]) else {
        unreachable!("security fixture construction failed");
    };
    let mut body = [0xa5_u8; 8];
    let mut headers = [0xa5_u8; 128];
    let mut response = ResponseBuffer::new(&mut body, 8, &mut headers);
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
        client.execute(
            TransportRequest::new(Method::Get, target),
            policy,
            response.writer(),
        ),
        Err(cloud_sdk::transport::TransportFailure::not_sent(
            RawHttpError::ResponseAlreadyCommitted
        ))
    );
}

#[test]
fn raw_blocking_sends_no_implicit_auth_or_json_accept() {
    let server = spawn(
        "200 OK",
        &[
            ("Content-Type", "application/json"),
            ("X-Request-Id", "request-123"),
            ("RateLimit-Limit", "3600"),
            ("RateLimit-Remaining", "3599"),
            ("RateLimit-Reset", "1234567890"),
            ("X-Ignored", "secret"),
        ],
        b"{}",
        Duration::ZERO,
    );
    let Ok(server) = server else {
        unreachable!("security fixture construction failed")
    };
    let Some(client) = build_raw_loopback(&server.endpoint) else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
        unreachable!("security fixture construction failed");
    };
    let Ok(content_type) = HeaderName::new("content-type") else {
        unreachable!("security fixture construction failed");
    };
    let Ok(request_id) = HeaderName::new("x-request-id") else {
        unreachable!("security fixture construction failed");
    };
    let Ok(limit) = HeaderName::new("ratelimit-limit") else {
        unreachable!("security fixture construction failed");
    };
    let Ok(remaining) = HeaderName::new("ratelimit-remaining") else {
        unreachable!("security fixture construction failed");
    };
    let Ok(reset) = HeaderName::new("ratelimit-reset") else {
        unreachable!("security fixture construction failed");
    };
    let admitted = [content_type, request_id, limit, remaining, reset];
    let Some(policy) = json_policy(&admitted) else {
        unreachable!("security fixture construction failed");
    };
    let mut body = [0xa5_u8; 16];
    let mut header_storage = [0xa5_u8; 128];
    let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
    let result = client.execute(
        TransportRequest::new(Method::Get, target),
        policy,
        response.writer(),
    );
    assert!(result.is_ok());
    assert!(
        response
            .with_response(|value| {
                value.status() == StatusCode::OK
                    && value.body() == b"{}"
                    && value.headers().get("content-type").is_some()
                    && value.headers().get("x-request-id").is_some()
                    && value.headers().get("ratelimit-limit").is_some()
                    && value.headers().get("ratelimit-remaining").is_some()
                    && value.headers().get("ratelimit-reset").is_some()
                    && value.headers().get("x-ignored").is_none()
            })
            .unwrap_or(false)
    );

    let recorded = server.request.recv_timeout(Duration::from_secs(2));
    assert!(recorded.is_ok());
    let Ok(recorded) = recorded else {
        unreachable!("security fixture construction failed");
    };
    let wire = String::from_utf8_lossy(&recorded.bytes).to_ascii_lowercase();
    assert!(!wire.contains("authorization:"));
    assert!(!wire.contains("accept: application/json"));
}

#[test]
fn raw_blocking_uses_error_cap_and_response_started_phase() {
    let server = spawn(
        "400 Bad Request",
        &[("Content-Type", "application/json")],
        b"12345",
        Duration::ZERO,
    );
    let Ok(server) = server else {
        unreachable!("security fixture construction failed")
    };
    let Some(client) = build_raw_loopback(&server.endpoint) else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
        unreachable!("security fixture construction failed");
    };
    let Some(policy) = json_policy(&[]) else {
        unreachable!("security fixture construction failed");
    };
    let mut body = [0xa5_u8; 16];
    let mut header_storage = [0xa5_u8; 128];
    let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
    let failure = client.execute(
        TransportRequest::new(Method::Get, target),
        policy,
        response.writer(),
    );
    assert!(matches!(
        failure,
        Err(error)
            if error.phase() == DeliveryPhase::ResponseStarted
                && error.error() == &RawHttpError::ResponseTooLarge
    ));
}

#[test]
fn raw_blocking_connect_failure_is_not_sent() {
    let endpoint = custom_endpoint("https://127.0.0.1:9/v1");
    let Ok(endpoint) = endpoint else {
        unreachable!("security fixture construction failed")
    };
    let Ok(user_agent) = UserAgent::new("cloud-sdk-raw-test/0.40") else {
        unreachable!("security fixture construction failed");
    };
    let Some(timeouts) = test_timeouts() else {
        unreachable!("security fixture construction failed");
    };
    let builder = RawBlockingClientBuilder::new(endpoint, user_agent, timeouts);
    #[cfg(feature = "blocking-rustls-fips")]
    let builder = {
        let Some(policy) = fips_tls_policy() else {
            unreachable!("security fixture construction failed");
        };
        builder.with_fips_tls_policy(policy)
    };
    let Ok(client) = builder.build() else {
        unreachable!("security fixture construction failed")
    };
    let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
        unreachable!("security fixture construction failed");
    };
    let Some(policy) = json_policy(&[]) else {
        unreachable!("security fixture construction failed");
    };
    let mut body = [0xa5_u8; 16];
    let mut header_storage = [0xa5_u8; 128];
    let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
    let failure = client.execute(
        TransportRequest::new(Method::Get, target),
        policy,
        response.writer(),
    );
    assert!(matches!(
        failure,
        Err(error) if error.phase() == DeliveryPhase::NotSent
    ));
}

#[test]
fn raw_blocking_rejects_nested_runtime_without_sending() {
    let Some(client) = build_raw_loopback("http://127.0.0.1:9/v1") else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
        unreachable!("security fixture construction failed");
    };
    let Some(policy) = json_policy(&[]) else {
        unreachable!("security fixture construction failed");
    };
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        unreachable!("security fixture construction failed");
    };
    runtime.block_on(async {
        let mut body = [0xa5_u8; 16];
        let mut header_storage = [0xa5_u8; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
        assert!(matches!(
            client.execute(
                TransportRequest::new(Method::Get, target),
                policy,
                response.writer(),
            ),
            Err(error)
                if error.phase() == DeliveryPhase::NotSent
                    && error.error() == &RawHttpError::BlockingRuntimeContext
        ));
    });
}

#[test]
fn raw_blocking_rejects_duplicate_and_trailer_heads() {
    for (headers, expected) in [
        (
            &[
                ("Content-Type", "application/json"),
                ("Content-Type", "application/json"),
            ][..],
            RawHttpError::DuplicateResponseHeader,
        ),
        (
            &[
                ("Content-Type", "application/json"),
                ("Trailer", "x-checksum"),
            ][..],
            RawHttpError::ResponseTrailersRejected,
        ),
        (
            &[
                ("Content-Type", "application/json"),
                ("RateLimit-Remaining", "2"),
                ("RateLimit-Remaining", "1"),
            ][..],
            RawHttpError::DuplicateResponseHeader,
        ),
    ] {
        let server = spawn("200 OK", headers, b"{}", Duration::ZERO);
        let Ok(server) = server else {
            unreachable!("security fixture construction failed")
        };
        let Some(client) = build_raw_loopback(&server.endpoint) else {
            unreachable!("security fixture construction failed");
        };
        let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
            unreachable!("security fixture construction failed");
        };
        let Some(policy) = json_policy(&[]) else {
            unreachable!("security fixture construction failed");
        };
        let mut body = [0xa5_u8; 16];
        let mut header_storage = [0xa5_u8; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
        assert!(matches!(
            client.execute(
                TransportRequest::new(Method::Get, target),
                policy,
                response.writer(),
            ),
            Err(error) if error.error() == &expected
        ));
    }
}

#[test]
fn raw_blocking_enforces_informational_limits_at_the_wire() {
    let wire = b"HTTP/1.1 103 Early Hints\r\nLink: </a>\r\n\r\n\
HTTP/1.1 103 Early Hints\r\nLink: </b>\r\n\r\n\
HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\
Connection: close\r\n\r\n{}";
    let Ok(server) = spawn_raw_response(wire) else {
        unreachable!("security fixture construction failed");
    };
    let Some(client) = build_raw_loopback(&server.endpoint) else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
        unreachable!("security fixture construction failed");
    };
    let Ok(policy) = RawResponsePolicy::new(
        16,
        4,
        ResponseMediaPolicy::Required(&[MediaType::JSON]),
        ResponseMediaPolicy::Optional(&[MediaType::JSON]),
        &[],
        1,
    ) else {
        unreachable!("security fixture construction failed");
    };
    let mut body = [0xa5_u8; 16];
    let mut header_storage = [0xa5_u8; 128];
    let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
    assert!(matches!(
        client.execute(
            TransportRequest::new(Method::Get, target),
            policy,
            response.writer(),
        ),
        Err(error)
            if error.phase() == DeliveryPhase::ResponseStarted
                && error.error() == &RawHttpError::TooManyInformationalResponses
    ));
}

#[test]
fn raw_blocking_client_is_clone_send_sync_and_concurrent() {
    fn assert_traits<T: Clone + Send + Sync>() {}
    assert_traits::<RawBlockingClient>();

    let Ok(server) = spawn_concurrent_pair("200 OK", b"{}") else {
        unreachable!("security fixture construction failed");
    };
    let Some(client) = build_raw_loopback(&server.endpoint) else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
        unreachable!("security fixture construction failed");
    };
    let Ok(policy) = RawResponsePolicy::new(
        16,
        4,
        ResponseMediaPolicy::Optional(&[MediaType::JSON]),
        ResponseMediaPolicy::Optional(&[MediaType::JSON]),
        &[],
        2,
    ) else {
        unreachable!("security fixture construction failed");
    };
    std::thread::scope(|scope| {
        let first = client.clone();
        let second = client.clone();
        let first = scope.spawn(move || execute_small(&first, target, policy));
        let second = scope.spawn(move || execute_small(&second, target, policy));
        assert!(first.join().is_ok_and(core::convert::identity));
        assert!(second.join().is_ok_and(core::convert::identity));
    });
}

fn execute_small(
    client: &RawBlockingClient,
    target: cloud_sdk::transport::RequestTarget<'_>,
    policy: RawResponsePolicy<'_>,
) -> bool {
    let mut body = [0xa5_u8; 16];
    let mut header_storage = [0xa5_u8; 128];
    let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
    client
        .execute(
            TransportRequest::new(Method::Get, target),
            policy,
            response.writer(),
        )
        .is_ok()
}

#[cfg(feature = "blocking-rustls-fips")]
#[test]
fn raw_fips_builder_requires_the_same_explicit_tls_policy() {
    let Ok(endpoint) = custom_endpoint("https://example.com/v1") else {
        unreachable!("security fixture construction failed");
    };
    let Ok(user_agent) = UserAgent::new("cloud-sdk-raw-test/0.40") else {
        unreachable!("security fixture construction failed");
    };
    let Some(timeouts) = test_timeouts() else {
        unreachable!("security fixture construction failed");
    };
    assert!(matches!(
        RawBlockingClientBuilder::new(endpoint, user_agent, timeouts).build(),
        Err(crate::blocking::BuildError::FipsTlsPolicyRequired)
    ));
}
