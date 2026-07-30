use std::string::String;
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::authentication::{
    AuthenticatedRequest, AuthenticationScopePolicy, BlockingAuthenticatedTransport,
    ScopeRequirement,
};
use cloud_sdk::transport::{BoundTransport, RequestTarget, ResponseBuffer, TransportRequest};

use super::super::{
    BasicCredential, BasicCredentialScope, BasicPassword, BasicUsername, BlockingBasicClient,
    BlockingBasicClientBuilder, BuildError, HttpsEndpoint, TransportError, UserAgent,
};
#[cfg(feature = "blocking-rustls-fips")]
use super::fips_tls_policy;
use super::test_timeouts;
use crate::test_server::spawn;

fn credential(endpoint: &HttpsEndpoint) -> Option<BasicCredential> {
    BasicCredential::new(
        BasicUsername::new("Aladdin").ok()?,
        BasicPassword::new("open sesame").ok()?,
        BasicCredentialScope::new(
            cloud_sdk::provider_id!("hetzner"),
            cloud_sdk::service_id!("robot"),
            endpoint.clone(),
        ),
    )
    .ok()
}

fn build_loopback(endpoint: &str) -> Option<BlockingBasicClient> {
    let endpoint = HttpsEndpoint::local_http(endpoint).ok()?;
    let credential = credential(&endpoint)?;
    let builder = BlockingBasicClientBuilder::new(
        endpoint,
        credential,
        UserAgent::new("cloud-sdk-basic-test/0.42").ok()?,
        test_timeouts()?,
    );
    #[cfg(feature = "blocking-rustls-fips")]
    let builder = builder.with_fips_tls_policy(fips_tls_policy()?);
    builder.build_for_loopback().ok()
}

fn request<'a>(
    client: &'a BlockingBasicClient,
    target: RequestTarget<'a>,
) -> AuthenticatedRequest<'a, 'a> {
    let endpoint = client
        .endpoint_identity()
        .unwrap_or_else(|_| unreachable!());
    let policy = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(cloud_sdk::provider_id!("hetzner")),
        ScopeRequirement::Required(cloud_sdk::service_id!("robot")),
        ScopeRequirement::Required(endpoint),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    AuthenticatedRequest::new(TransportRequest::new(Method::Get, target), policy)
}

#[test]
fn blocking_basic_client_sends_exact_authorization_and_target() {
    let Ok(server) = spawn(
        "200 OK",
        &[("Content-Type", "application/json")],
        b"{}",
        Duration::ZERO,
    ) else {
        return;
    };
    let Some(client) = build_loopback(&server.endpoint) else {
        return;
    };
    let Ok(target) = RequestTarget::new("/server/321") else {
        return;
    };
    let mut body = [0xa5_u8; 8];
    let mut headers = [0xa5_u8; 512];
    let mut response = ResponseBuffer::new(&mut body, 8, &mut headers);
    let result = BlockingAuthenticatedTransport::send_authenticated(
        &client,
        request(&client, target),
        response.writer(),
    );
    assert_eq!(result, Ok(()));

    let recorded = server.request.recv_timeout(Duration::from_secs(2));
    assert!(recorded.is_ok());
    if let Ok(recorded) = recorded {
        let wire = String::from_utf8_lossy(&recorded.bytes).to_ascii_lowercase();
        assert!(wire.starts_with("get /v1/server/321 http/1.1\r\n"));
        assert!(wire.contains("authorization: basic qwxhzgrpbjpvcgvuihnlc2ftzq==\r\n"));
    }
}

#[test]
fn blocking_basic_builder_rejects_a_different_credential_endpoint() {
    let Ok(configured) = HttpsEndpoint::local_http("http://127.0.0.1:3000/v1") else {
        return;
    };
    let Ok(credential_endpoint) = HttpsEndpoint::local_http("http://127.0.0.1:3001/v1") else {
        return;
    };
    let Some(credential) = credential(&credential_endpoint) else {
        return;
    };
    let Some(timeouts) = test_timeouts() else {
        return;
    };
    let Ok(user_agent) = UserAgent::new("cloud-sdk-basic-test/0.42") else {
        return;
    };
    assert!(matches!(
        BlockingBasicClientBuilder::new(configured, credential, user_agent, timeouts)
            .build_for_loopback(),
        Err(BuildError::CredentialEndpointMismatch)
    ));
}

#[test]
fn blocking_basic_client_rejects_incomplete_scope_before_network() {
    let Some(client) = build_loopback("http://127.0.0.1:1/v1") else {
        return;
    };
    let Ok(endpoint) = client.endpoint_identity() else {
        return;
    };
    let Ok(target) = RequestTarget::new("/server") else {
        return;
    };
    let policy = AuthenticationScopePolicy::new(
        ScopeRequirement::Optional(cloud_sdk::provider_id!("hetzner")),
        ScopeRequirement::Required(cloud_sdk::service_id!("robot")),
        ScopeRequirement::Required(endpoint),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let authenticated =
        AuthenticatedRequest::new(TransportRequest::new(Method::Get, target), policy);
    let mut body = [0_u8; 8];
    let mut headers = [0_u8; 512];
    let mut response = ResponseBuffer::new(&mut body, 8, &mut headers);
    assert_eq!(
        BlockingAuthenticatedTransport::send_authenticated(
            &client,
            authenticated,
            response.writer(),
        ),
        Err(TransportError::AuthenticationScopeRejected)
    );
    drop(response);
    assert_eq!(body, [0_u8; 8]);
    assert_eq!(headers, [0_u8; 512]);
}
