use cloud_sdk::Method;
use cloud_sdk::ProviderId;
use cloud_sdk::authentication::{
    AuthenticatedRequest, AuthenticationScopePolicy, BlockingAuthenticatedTransport,
    ScopeRequirement,
};
use cloud_sdk::transport::{RequestTarget, ResponseBuffer, TransportRequest};

use super::super::BearerTokenError;
use super::{BearerToken, TransportError, build_loopback};

#[test]
fn bearer_tokens_are_bounded_validated_redacted_and_sensitive() {
    assert!(matches!(BearerToken::new(""), Err(BearerTokenError::Empty)));
    assert!(matches!(
        BearerToken::new("token with space"),
        Err(BearerTokenError::InvalidByte)
    ));
    assert!(matches!(
        BearerToken::new("token=bad"),
        Err(BearerTokenError::InvalidByte)
    ));
    for invalid in ["=", "====", "=token"] {
        assert!(matches!(
            BearerToken::new(invalid),
            Err(BearerTokenError::InvalidByte)
        ));
    }
    let token = BearerToken::new("token-value==");
    assert!(token.is_ok());
    if let Ok(token) = token {
        assert_eq!(token.owned_bytes(), b"Bearer token-value==");
        let debug = std::format!("{token:?}");
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
fn client_builder_rejects_a_credential_bound_to_another_endpoint() {
    use std::time::Duration;

    use cloud_sdk::ServiceId;

    use super::super::{
        BearerCredential, BearerCredentialScope, BlockingClientBuilder, BuildError,
        CustomEndpointAcknowledgement, HttpsEndpoint, RequestTimeouts, UserAgent,
    };

    let acknowledgement = CustomEndpointAcknowledgement::trusted_operator_configuration();
    let configured = HttpsEndpoint::new_custom("https://api.example.test/v1", acknowledgement);
    let credential_endpoint =
        HttpsEndpoint::new_custom("https://other.example.test/v1", acknowledgement);
    let token = BearerToken::new("token");
    let provider = ProviderId::new("example");
    let service = ServiceId::new("compute");
    let user_agent = UserAgent::new("cloud-sdk-test/0.41");
    let timeouts = RequestTimeouts::new(Duration::from_secs(2), Duration::from_secs(1));
    let (
        Ok(configured),
        Ok(credential_endpoint),
        Ok(token),
        Ok(provider),
        Ok(service),
        Ok(user_agent),
        Ok(timeouts),
    ) = (
        configured,
        credential_endpoint,
        token,
        provider,
        service,
        user_agent,
        timeouts,
    )
    else {
        return;
    };
    let credential = BearerCredential::new(
        token,
        BearerCredentialScope::new(provider, service, credential_endpoint),
    );
    assert_eq!(
        BlockingClientBuilder::new(configured, credential, user_agent, timeouts)
            .build()
            .map(|_| ()),
        Err(BuildError::CredentialEndpointMismatch)
    );
}

#[test]
fn scope_rejection_happens_before_blocking_network_or_header_work() {
    let Some(client) = build_loopback("http://127.0.0.1:9/v1") else {
        return;
    };
    let Ok(provider) = ProviderId::new("example") else {
        return;
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
        return;
    };
    let request = AuthenticatedRequest::new(TransportRequest::new(Method::Get, target), policy);
    let mut body = [0xa5_u8; 8];
    let mut headers = [0xa5_u8; 8192];
    let mut response = ResponseBuffer::new(&mut body, 8, &mut headers);
    assert_eq!(
        BlockingAuthenticatedTransport::send_authenticated(&client, request, response.writer(),),
        Err(TransportError::AuthenticationScopeRejected)
    );
    drop(response);
    assert_eq!(body, [0_u8; 8]);
    assert_eq!(headers, [0_u8; 8192]);
}
