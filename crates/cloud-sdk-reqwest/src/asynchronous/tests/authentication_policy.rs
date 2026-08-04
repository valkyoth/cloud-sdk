use cloud_sdk::Method;
use cloud_sdk::ProviderId;
use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::operation::{PreparedExecutionError, ProviderService};
use cloud_sdk::transport::{BoundTransport, EndpointPolicy, RequestTarget, TransportRequest};

use super::{
    AsyncClientBuilder, BearerToken, HttpsEndpoint, TransportError, UserAgent, build_loopback,
    run_async_test, send_test, test_credential, test_timeouts,
};
use crate::shared::CustomEndpointAcknowledgement;

#[test]
fn missing_content_type_fails_before_async_network_access() {
    run_async_test(async {
        let Some(client) = build_loopback("http://127.0.0.1:9/v1") else {
            unreachable!("security fixture construction failed");
        };
        let Ok(target) = RequestTarget::new("/servers") else {
            unreachable!("security fixture construction failed");
        };
        let mut output = [0xa5_u8; 8];
        let result = send_test(
            &client,
            TransportRequest::new(Method::Post, target).with_body(b"{}"),
            &mut output,
        )
        .await;
        assert_eq!(
            result.map(|_| ()),
            Err(TransportError::RawHttp(
                super::super::RawHttpError::MissingContentType
            ))
        );
        assert_eq!(output, [0_u8; 8]);
    });
}

#[test]
fn scope_rejection_happens_before_async_network_or_header_work() {
    run_async_test(async {
        let Some(client) = build_loopback("http://127.0.0.1:9/v1") else {
            unreachable!("security fixture construction failed");
        };
        let Ok(provider) = ProviderId::new("example") else {
            unreachable!("security fixture construction failed");
        };
        let policy = AuthenticationScopePolicy::new(
            ScopeRequirement::Required(provider),
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
        );
        let Ok(target) = RequestTarget::new("/must-not-send") else {
            unreachable!("security fixture construction failed");
        };
        let Ok(endpoint) = client.endpoint_identity() else {
            unreachable!("security fixture construction failed");
        };
        let request = super::support::prepared_with_policy(
            TransportRequest::new(Method::Get, target),
            ProviderService::new(
                provider,
                cloud_sdk::service_id!("compute"),
                EndpointPolicy::fixed(endpoint),
            ),
            policy,
        );
        let mut body = [0xa5_u8; 8];
        let mut headers = [0xa5_u8; 8192];
        {
            assert!(matches!(
                request.execute_async(&client, &mut body, &mut headers).await,
                Err(PreparedExecutionError::Transport(failure))
                    if failure == cloud_sdk::transport::TransportFailure::not_sent(
                        TransportError::AuthenticationScopeRejected
                    )
            ));
        }
        assert_eq!(body, [0_u8; 8]);
        assert_eq!(headers, [0_u8; 8192]);
    });
}

#[test]
fn async_builder_debug_redacts_endpoint_scope_and_token() {
    let endpoint = HttpsEndpoint::new_custom(
        "https://api.example.test/v1",
        CustomEndpointAcknowledgement::trusted_operator_configuration(),
    );
    let token = BearerToken::new("secret-token");
    let user_agent = UserAgent::new("cloud-sdk-test/0.41");
    let timeouts = test_timeouts();
    let (Ok(endpoint), Ok(token), Ok(user_agent), Some(timeouts)) =
        (endpoint, token, user_agent, timeouts)
    else {
        unreachable!("security fixture construction failed");
    };
    let credential = test_credential(token, &endpoint);
    let builder = AsyncClientBuilder::new(endpoint, credential, user_agent, timeouts);
    let debug = std::format!("{builder:?}");
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("secret-token"));
    assert!(!debug.contains("api.example.test"));
}
