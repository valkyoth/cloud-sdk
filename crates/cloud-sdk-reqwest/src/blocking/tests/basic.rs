use std::string::String;
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::operation::{PreparedExecutionError, PreparedRequest, ProviderService};
use cloud_sdk::transport::{BoundTransport, EndpointPolicy, RequestTarget, TransportRequest};

use super::super::{
    BasicCredential, BasicCredentialScope, BasicPassword, BasicUsername, BlockingBasicClient,
    BlockingBasicClientBuilder, BuildError, HttpsEndpoint, TransportError, UserAgent,
};
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
    builder.build_for_loopback().ok()
}

fn request<'a>(client: &'a BlockingBasicClient, target: RequestTarget<'a>) -> PreparedRequest<'a> {
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
    super::support::prepared_with_policy(
        TransportRequest::new(Method::Get, target),
        ProviderService::new(
            cloud_sdk::provider_id!("hetzner"),
            cloud_sdk::service_id!("robot"),
            EndpointPolicy::fixed(endpoint),
        ),
        policy,
    )
}

#[test]
fn blocking_basic_client_sends_exact_authorization_and_target() {
    let Ok(server) = spawn(
        "200 OK",
        &[("Content-Type", "application/json")],
        b"{}",
        Duration::ZERO,
    ) else {
        unreachable!("security fixture construction failed");
    };
    let Some(client) = build_loopback(&server.endpoint) else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = RequestTarget::new("/server/321") else {
        unreachable!("security fixture construction failed");
    };
    let mut body = [0xa5_u8; 8];
    let mut headers = [0xa5_u8; 512];
    let result = request(&client, target).execute_blocking(&client, &mut body, &mut headers);
    assert!(result.is_ok());

    let recorded = server.request.recv_timeout(Duration::from_secs(2));
    assert!(recorded.is_ok());
    let Ok(recorded) = recorded else {
        unreachable!("security fixture construction failed");
    };
    let wire = String::from_utf8_lossy(&recorded.bytes).to_ascii_lowercase();
    assert!(wire.starts_with("get /v1/server/321 http/1.1\r\n"));
    assert!(wire.contains("authorization: basic qwxhzgrpbjpvcgvuihnlc2ftzq==\r\n"));
}

#[test]
fn blocking_basic_builder_rejects_a_different_credential_endpoint() {
    let Ok(configured) = HttpsEndpoint::local_http("http://127.0.0.1:3000/v1") else {
        unreachable!("security fixture construction failed");
    };
    let Ok(credential_endpoint) = HttpsEndpoint::local_http("http://127.0.0.1:3001/v1") else {
        unreachable!("security fixture construction failed");
    };
    let Some(credential) = credential(&credential_endpoint) else {
        unreachable!("security fixture construction failed");
    };
    let Some(timeouts) = test_timeouts() else {
        unreachable!("security fixture construction failed");
    };
    let Ok(user_agent) = UserAgent::new("cloud-sdk-basic-test/0.42") else {
        unreachable!("security fixture construction failed");
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
        unreachable!("security fixture construction failed");
    };
    let Ok(endpoint) = client.endpoint_identity() else {
        unreachable!("security fixture construction failed");
    };
    let Ok(target) = RequestTarget::new("/server") else {
        unreachable!("security fixture construction failed");
    };
    let policy = AuthenticationScopePolicy::new(
        ScopeRequirement::Optional(cloud_sdk::provider_id!("hetzner")),
        ScopeRequirement::Required(cloud_sdk::service_id!("robot")),
        ScopeRequirement::Required(endpoint),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let request = super::support::prepared_with_policy(
        TransportRequest::new(Method::Get, target),
        ProviderService::new(
            cloud_sdk::provider_id!("hetzner"),
            cloud_sdk::service_id!("robot"),
            EndpointPolicy::fixed(endpoint),
        ),
        policy,
    );
    let mut body = [0_u8; 8];
    let mut headers = [0_u8; 512];
    assert!(matches!(
        request.execute_blocking(&client, &mut body, &mut headers),
        Err(PreparedExecutionError::Transport(failure))
            if failure == cloud_sdk::transport::TransportFailure::not_sent(
                TransportError::AuthenticationScopeRejected
            )
    ));
    assert_eq!(body, [0_u8; 8]);
    assert_eq!(headers, [0_u8; 512]);
}
