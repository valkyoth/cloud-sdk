use std::string::String;
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, AuthenticationScopePolicy, ScopeRequirement,
};
use cloud_sdk::transport::{BoundTransport, RequestTarget, ResponseBuffer, TransportRequest};

use super::super::{
    AsyncBasicClient, AsyncBasicClientBuilder, BasicCredential, BasicCredentialScope,
    BasicPassword, BasicUsername, BuildError, HttpsEndpoint, UserAgent,
};
use super::{run_async_test, test_timeouts};
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

fn build_loopback(endpoint: &str) -> Option<AsyncBasicClient> {
    let endpoint = HttpsEndpoint::local_http(endpoint).ok()?;
    let credential = credential(&endpoint)?;
    AsyncBasicClientBuilder::new(
        endpoint,
        credential,
        UserAgent::new("cloud-sdk-basic-test/0.42").ok()?,
        test_timeouts()?,
    )
    .build_for_loopback()
    .ok()
}

fn request<'a>(
    client: &'a AsyncBasicClient,
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
fn asynchronous_basic_client_sends_exact_authorization_and_target() {
    run_async_test(async {
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
        let mut body = [0_u8; 8];
        let mut headers = [0_u8; 512];
        let mut response = ResponseBuffer::new(&mut body, 8, &mut headers);
        let result = AsyncAuthenticatedTransport::send_authenticated(
            &client,
            request(&client, target),
            response.writer(),
        )
        .await;
        assert_eq!(result, Ok(()));

        let recorded = server.request.recv_timeout(Duration::from_secs(2));
        assert!(recorded.is_ok());
        if let Ok(recorded) = recorded {
            let wire = String::from_utf8_lossy(&recorded.bytes).to_ascii_lowercase();
            assert!(wire.starts_with("get /v1/server/321 http/1.1\r\n"));
            assert!(wire.contains("authorization: basic qwxhzgrpbjpvcgvuihnlc2ftzq==\r\n"));
        }
    });
}

#[test]
fn asynchronous_basic_builder_rejects_a_different_credential_endpoint() {
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
        AsyncBasicClientBuilder::new(configured, credential, user_agent, timeouts)
            .build_for_loopback(),
        Err(BuildError::CredentialEndpointMismatch)
    ));
}
