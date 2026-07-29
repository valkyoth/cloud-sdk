use std::string::String;
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{
    BlockingRawHttpExecutor, ContentType, DeliveryPhase, HeaderName, MediaType, RawResponsePolicy,
    RequestHeader, RequestHeaders, ResponseBuffer, ResponseMediaPolicy, StatusCode,
    TransportRequest,
};

use super::{custom_endpoint, test_timeouts};
#[cfg(feature = "blocking-rustls-fips")]
use crate::blocking::tests::fips_tls_policy;
use crate::blocking::{
    MAX_RAW_REQUEST_BODY_BYTES, RawBlockingClient, RawBlockingClientBuilder, RawHttpError,
    UserAgent,
};
use crate::test_server::{spawn, spawn_concurrent_pair, spawn_raw_response};

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
fn raw_blocking_sends_no_implicit_auth_or_json_accept() {
    let server = spawn(
        "200 OK",
        &[
            ("Content-Type", "application/json"),
            ("X-Ignored", "secret"),
        ],
        b"{}",
        Duration::ZERO,
    );
    let Ok(server) = server else { return };
    let Some(client) = build_raw_loopback(&server.endpoint) else {
        return;
    };
    let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
        return;
    };
    let Ok(content_type) = HeaderName::new("content-type") else {
        return;
    };
    let admitted = [content_type];
    let Some(policy) = json_policy(&admitted) else {
        return;
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
                    && value.headers().get("x-ignored").is_none()
            })
            .unwrap_or(false)
    );

    let recorded = server.request.recv_timeout(Duration::from_secs(2));
    assert!(recorded.is_ok());
    if let Ok(recorded) = recorded {
        let wire = String::from_utf8_lossy(&recorded.bytes).to_ascii_lowercase();
        assert!(!wire.contains("authorization:"));
        assert!(!wire.contains("accept: application/json"));
    }
}

#[test]
fn raw_blocking_uses_error_cap_and_response_started_phase() {
    let server = spawn(
        "400 Bad Request",
        &[("Content-Type", "application/json")],
        b"12345",
        Duration::ZERO,
    );
    let Ok(server) = server else { return };
    let Some(client) = build_raw_loopback(&server.endpoint) else {
        return;
    };
    let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
        return;
    };
    let Some(policy) = json_policy(&[]) else {
        return;
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
    let Ok(endpoint) = endpoint else { return };
    let Ok(user_agent) = UserAgent::new("cloud-sdk-raw-test/0.40") else {
        return;
    };
    let Some(timeouts) = test_timeouts() else {
        return;
    };
    let builder = RawBlockingClientBuilder::new(endpoint, user_agent, timeouts);
    #[cfg(feature = "blocking-rustls-fips")]
    let builder = {
        let Some(policy) = fips_tls_policy() else {
            return;
        };
        builder.with_fips_tls_policy(policy)
    };
    let Ok(client) = builder.build() else { return };
    let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
        return;
    };
    let Some(policy) = json_policy(&[]) else {
        return;
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
fn raw_blocking_rejects_oversized_request_body_before_network_access() {
    let Some(client) = build_raw_loopback("http://127.0.0.1:9/v1") else {
        return;
    };
    let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
        return;
    };
    let content_type = [RequestHeader::content_type(ContentType::JSON)];
    let Ok(headers) = RequestHeaders::new(&content_type) else {
        return;
    };
    let oversized = std::vec![0x5a; MAX_RAW_REQUEST_BODY_BYTES.saturating_add(1)];
    let Some(policy) = json_policy(&[]) else {
        return;
    };
    let mut body = [0xa5_u8; 16];
    let mut header_storage = [0xa5_u8; 128];
    let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
    assert!(matches!(
        client.execute(
            TransportRequest::new(Method::Post, target)
                .with_headers(headers)
                .with_body(&oversized),
            policy,
            response.writer(),
        ),
        Err(error)
            if error.phase() == DeliveryPhase::NotSent
                && error.error() == &RawHttpError::RequestBodyTooLarge
    ));
}

#[test]
fn raw_blocking_rejects_nested_runtime_without_sending() {
    let Some(client) = build_raw_loopback("http://127.0.0.1:9/v1") else {
        return;
    };
    let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
        return;
    };
    let Some(policy) = json_policy(&[]) else {
        return;
    };
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        return;
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
    ] {
        let server = spawn("200 OK", headers, b"{}", Duration::ZERO);
        let Ok(server) = server else { return };
        let Some(client) = build_raw_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
            return;
        };
        let Some(policy) = json_policy(&[]) else {
            return;
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
        return;
    };
    let Some(client) = build_raw_loopback(&server.endpoint) else {
        return;
    };
    let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
        return;
    };
    let Ok(policy) = RawResponsePolicy::new(
        16,
        4,
        ResponseMediaPolicy::Required(&[MediaType::JSON]),
        ResponseMediaPolicy::Optional(&[MediaType::JSON]),
        &[],
        1,
    ) else {
        return;
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
        return;
    };
    let Some(client) = build_raw_loopback(&server.endpoint) else {
        return;
    };
    let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
        return;
    };
    let Ok(policy) = RawResponsePolicy::new(
        16,
        4,
        ResponseMediaPolicy::Optional(&[MediaType::JSON]),
        ResponseMediaPolicy::Optional(&[MediaType::JSON]),
        &[],
        2,
    ) else {
        return;
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
        return;
    };
    let Ok(user_agent) = UserAgent::new("cloud-sdk-raw-test/0.40") else {
        return;
    };
    let Some(timeouts) = test_timeouts() else {
        return;
    };
    assert!(matches!(
        RawBlockingClientBuilder::new(endpoint, user_agent, timeouts).build(),
        Err(crate::blocking::BuildError::FipsTlsPolicyRequired)
    ));
}
