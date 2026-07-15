use std::format;
use std::io::Cursor;
#[cfg(feature = "blocking-rustls-fips")]
use std::println;
use std::string::String;
use std::time::Duration;
#[cfg(feature = "blocking-rustls-fips")]
use std::vec;

#[cfg(feature = "blocking-rustls-fips")]
use rustls::RootCertStore;
#[cfg(feature = "blocking-rustls-fips")]
use rustls::pki_types::pem::PemObject;
#[cfg(feature = "blocking-rustls-fips")]
use rustls::pki_types::{CertificateDer, CertificateRevocationListDer};

use cloud_sdk::Method;
use cloud_sdk::transport::{
    BlockingTransport, ContentType, RequestTarget, StatusCode, TransportRequest,
};

use super::body::{ReadBodyError, read_bounded};
use super::{
    BearerToken, BearerTokenError, BlockingClientBuilder, EndpointError, HttpsEndpoint,
    RequestTimeouts, TimeoutError, TransportError, UserAgent,
};
#[cfg(feature = "blocking-rustls-fips")]
use super::{BuildError, FipsTlsPolicy};
use crate::test_server::spawn;

mod lifecycle;
mod response_content_type;

fn test_timeouts() -> Option<RequestTimeouts> {
    RequestTimeouts::new(Duration::from_secs(2), Duration::from_secs(1)).ok()
}

fn build_loopback(endpoint: &str) -> Option<super::BlockingClient> {
    let endpoint = HttpsEndpoint::local_http(endpoint).ok()?;
    let token = BearerToken::new("test-token").ok()?;
    let user_agent = UserAgent::new("cloud-sdk-test/0.18").ok()?;
    let timeouts = test_timeouts()?;
    let builder = BlockingClientBuilder::new(endpoint, token, user_agent, timeouts);
    #[cfg(feature = "blocking-rustls-fips")]
    let builder = builder.with_fips_tls_policy(fips_tls_policy()?);
    builder.build_for_loopback().ok()
}

#[cfg(feature = "blocking-rustls-fips")]
fn fips_roots() -> Option<RootCertStore> {
    let certificate =
        CertificateDer::from_pem_slice(include_bytes!("../../testdata/fips_root.pem")).ok()?;
    let mut roots = RootCertStore::empty();
    roots.add(certificate).ok()?;
    Some(roots)
}

#[cfg(feature = "blocking-rustls-fips")]
fn fips_tls_policy() -> Option<FipsTlsPolicy> {
    let crl =
        CertificateRevocationListDer::from_pem_slice(include_bytes!("../../testdata/fips.crl.pem"))
            .ok()?;
    FipsTlsPolicy::new(fips_roots()?, vec![crl]).ok()
}

#[test]
fn bearer_tokens_are_bounded_validated_and_redacted() {
    assert!(matches!(BearerToken::new(""), Err(BearerTokenError::Empty)));
    assert!(matches!(
        BearerToken::new("token with space"),
        Err(BearerTokenError::InvalidByte)
    ));
    assert!(matches!(
        BearerToken::new("token=bad"),
        Err(BearerTokenError::InvalidByte)
    ));
    let token = BearerToken::new("token-value==");
    assert!(token.is_ok());
    if let Ok(token) = token {
        assert_eq!(token.owned_bytes(), b"Bearer token-value==");
        let debug = format!("{token:?}");
        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains("token-value"));
        let header = token.header_value();
        assert!(header.is_ok());
        if let Ok(header) = header {
            assert!(header.is_sensitive());
        }
    }
}

#[test]
fn endpoints_reject_authority_and_normalization_ambiguity() {
    let redacted = HttpsEndpoint::new_custom("https://api.example.test/v1");
    assert!(redacted.is_ok());
    if let Ok(redacted) = redacted {
        let debug = format!("{redacted:?}");
        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains("api.example.test"));
    }
    assert!(matches!(
        HttpsEndpoint::new_custom("http://api.example.test/v1"),
        Err(EndpointError::HttpsRequired)
    ));
    assert!(matches!(
        HttpsEndpoint::new_custom("https://user@api.example.test/v1"),
        Err(EndpointError::CredentialsForbidden)
    ));
    assert!(matches!(
        HttpsEndpoint::new_custom("https://api.example.test/v1?token=x"),
        Err(EndpointError::QueryForbidden)
    ));
    assert!(matches!(
        HttpsEndpoint::new_custom("https://api.example.test/v1/"),
        Err(EndpointError::TrailingSlash)
    ));
    for endpoint in [
        "https://api.example.test/v1/../admin",
        "https://api.example.test/%76%31",
        "https://api.example.test/v1//admin",
    ] {
        assert!(matches!(
            HttpsEndpoint::new_custom(endpoint),
            Err(EndpointError::IdentityRejected)
        ));
    }

    let endpoint = HttpsEndpoint::new_custom("https://api.example.test/v1");
    let safe = RequestTarget::new("/servers?name=test%20server");
    if let (Ok(endpoint), Ok(safe)) = (endpoint, safe) {
        let url = endpoint.compose(safe);
        assert_eq!(
            url.as_ref().map(reqwest::Url::as_str),
            Ok("https://api.example.test/v1/servers?name=test%20server")
        );
        for target in ["/%2e%2e/admin", "/x%2fy", "/x%5cevil", "/x%25%32%66"] {
            let target = RequestTarget::new(target);
            assert!(target.is_ok());
            if let Ok(target) = target {
                assert_eq!(
                    endpoint.compose(target),
                    Err(EndpointError::InvalidTargetEncoding)
                );
            }
        }
        let parent = RequestTarget::new("/servers/../admin");
        if let Ok(parent) = parent {
            assert_eq!(
                endpoint.compose(parent),
                Err(EndpointError::TargetNormalized)
            );
        }
    }
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
    assert!(server.is_ok());
    let Ok(server) = server else { return };
    let client = build_loopback(&server.endpoint);
    assert!(client.is_some());
    let Some(client) = client else { return };
    let target = RequestTarget::new("/servers?name=test%20server");
    assert!(target.is_ok());
    let Ok(target) = target else { return };
    let request = TransportRequest::new(Method::Post, target)
        .with_body(br#"{"name":"server"}"#)
        .with_content_type(ContentType::JSON);
    let mut output = [0xa5_u8; 32];
    let response = client.send(request, &mut output);
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
}

#[test]
fn redirects_are_returned_and_oversized_bodies_are_cleared() {
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
    let response = client.send(TransportRequest::new(Method::Get, target), &mut output);
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
    assert!(matches!(
        client.send(TransportRequest::new(Method::Get, target), &mut short),
        Err(TransportError::ResponseTooLarge)
    ));
    assert_eq!(short, [0_u8; 4]);
}

#[test]
fn response_propagates_validated_rate_limit_headers() {
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
    let response = client.send(TransportRequest::new(Method::Get, target), &mut output);
    assert!(response.is_ok());
    let Ok(response) = response else { return };
    let Some(content_type) = response.content_type() else {
        return;
    };
    assert_eq!(content_type.as_str(), "application/json; charset=utf-8");
    let Some(rate_limit) = response.rate_limit() else {
        return;
    };
    assert_eq!(rate_limit.limit(), 3600);
    assert_eq!(rate_limit.remaining(), 3599);
    assert_eq!(rate_limit.reset_epoch_seconds(), 42);
}

#[test]
fn incomplete_rate_limit_headers_fail_closed() {
    let server = spawn(
        "200 OK",
        &[("RateLimit-Limit", "3600")],
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
    assert!(matches!(
        client.send(TransportRequest::new(Method::Get, target), &mut output),
        Err(TransportError::InvalidRateLimitHeaders)
    ));
    assert_eq!(output, [0_u8; 8]);
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
    let Ok(server) = server else { return };
    let Some(client) = build_loopback(&server.endpoint) else {
        return;
    };
    let Ok(target) = RequestTarget::new("/servers") else {
        return;
    };
    let mut output = [0xa5_u8; 8];
    assert!(matches!(
        client.send(TransportRequest::new(Method::Get, target), &mut output),
        Err(TransportError::InvalidRateLimitHeaders)
    ));
    assert_eq!(output, [0_u8; 8]);
}

#[test]
fn nonempty_body_requires_content_type_before_network_access() {
    let Some(client) = build_loopback("http://127.0.0.1:9/v1") else {
        return;
    };
    let Ok(target) = RequestTarget::new("/servers") else {
        return;
    };
    let mut output = [0xa5_u8; 8];
    assert!(matches!(
        client.send(
            TransportRequest::new(Method::Post, target).with_body(b"{}"),
            &mut output,
        ),
        Err(TransportError::MissingContentType)
    ));
    assert_eq!(output, [0_u8; 8]);
}

#[test]
fn response_timeout_is_payload_free_and_clears_output() {
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
        BlockingClientBuilder::new(endpoint, token, user_agent, timeouts).build_for_loopback();
    let Ok(client) = client else { return };
    let Ok(target) = RequestTarget::new("/slow") else {
        return;
    };
    let mut output = [0xa5_u8; 8];
    assert!(matches!(
        client.send(TransportRequest::new(Method::Get, target), &mut output),
        Err(TransportError::TimedOut)
    ));
    assert_eq!(output, [0_u8; 8]);
}

#[test]
fn status_constant_remains_compatible_with_transport_response() {
    assert_eq!(StatusCode::OK.get(), 200);
}

#[cfg(feature = "blocking-rustls-fips")]
#[test]
fn fips_provider_and_complete_client_configuration_report_fips() {
    let Some(policy) = fips_tls_policy() else {
        return;
    };
    assert_eq!(super::config::test_fips_configuration(&policy), Ok(true));
}

#[cfg(feature = "blocking-rustls-fips")]
#[test]
fn non_fips_provider_and_complete_configuration_fail_closed() {
    let Some(policy) = fips_tls_policy() else {
        return;
    };
    assert_eq!(super::config::test_non_fips_rejection(&policy), Ok(true));
}

#[cfg(feature = "blocking-rustls-fips")]
#[test]
fn fips_policy_rejects_missing_roots_crls_and_malformed_crls() {
    let crl = CertificateRevocationListDer::from(vec![0xff]);
    assert!(matches!(
        FipsTlsPolicy::new(RootCertStore::empty(), vec![crl.clone()]),
        Err(BuildError::FipsTrustRootsRequired)
    ));
    let Some(roots) = fips_roots() else { return };
    assert!(matches!(
        FipsTlsPolicy::new(roots, vec![]),
        Err(BuildError::FipsCertificateRevocationListsRequired)
    ));
    let Some(roots) = fips_roots() else { return };
    let Ok(policy) = FipsTlsPolicy::new(roots, vec![crl]) else {
        return;
    };
    assert_eq!(
        super::config::test_fips_configuration(&policy),
        Err(BuildError::FipsRevocationVerifierFailed)
    );
}

#[cfg(feature = "blocking-rustls-fips")]
#[test]
fn fips_client_builder_requires_an_explicit_tls_policy() {
    let endpoint = HttpsEndpoint::new_custom("https://api.example.test");
    let token = BearerToken::new("test-token");
    let user_agent = UserAgent::new("cloud-sdk-test/0.23");
    let timeouts = test_timeouts();
    let (Ok(endpoint), Ok(token), Ok(user_agent), Some(timeouts)) =
        (endpoint, token, user_agent, timeouts)
    else {
        return;
    };
    assert!(matches!(
        BlockingClientBuilder::new(endpoint, token, user_agent, timeouts).build(),
        Err(BuildError::FipsTlsPolicyRequired)
    ));
}

#[cfg(feature = "blocking-rustls-fips")]
#[test]
fn preinstalled_non_fips_global_provider_does_not_influence_fips_client() {
    const CHILD: &str = "CLOUD_SDK_FIPS_GLOBAL_PROVIDER_CHILD";
    const CHILD_MARKER: &str = "cloud-sdk FIPS global-provider child ran";
    if std::env::var_os(CHILD).is_some() {
        let Some(policy) = fips_tls_policy() else {
            return;
        };
        assert!(super::config::test_non_fips_global_independence(&policy));
        println!("{CHILD_MARKER}");
        return;
    }

    let executable = std::env::current_exe();
    assert!(executable.is_ok());
    let Ok(executable) = executable else { return };
    let output = std::process::Command::new(executable)
        .args([
            "--exact",
            "blocking::tests::preinstalled_non_fips_global_provider_does_not_influence_fips_client",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .output();
    assert!(output.is_ok());
    let Ok(output) = output else { return };
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "isolated FIPS test failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains(CHILD_MARKER),
        "isolated FIPS test did not run"
    );
}
