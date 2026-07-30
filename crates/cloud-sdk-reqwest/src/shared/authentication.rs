use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme};
use std::net::IpAddr;

use super::BearerCredentialScope;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuthenticationValidationError {
    InsecureEndpoint,
    EndpointMismatch,
    ScopeRejected,
}

pub(crate) fn validate_bearer_authentication<'a>(
    endpoint: EndpointIdentity<'a>,
    scope: &'a BearerCredentialScope,
    policy: AuthenticationScopePolicy<'a>,
    allow_insecure_loopback: bool,
) -> Result<(), AuthenticationValidationError> {
    let secure_destination = endpoint.scheme() == EndpointScheme::Https;
    let admitted_test_loopback = allow_insecure_loopback
        && endpoint.scheme() == EndpointScheme::Http
        && is_numeric_loopback(endpoint.host());
    if !secure_destination && !admitted_test_loopback {
        return Err(AuthenticationValidationError::InsecureEndpoint);
    }
    match policy.endpoint_requirement() {
        ScopeRequirement::Required(expected) | ScopeRequirement::Optional(expected)
            if expected != endpoint =>
        {
            return Err(AuthenticationValidationError::EndpointMismatch);
        }
        ScopeRequirement::Required(_)
        | ScopeRequirement::Optional(_)
        | ScopeRequirement::Forbidden => {}
    }
    let credential_scope = scope
        .borrowed()
        .map_err(|_| AuthenticationValidationError::ScopeRejected)?;
    policy
        .validate(credential_scope)
        .map_err(|_| AuthenticationValidationError::ScopeRejected)
}

fn is_numeric_loopback(host: &str) -> bool {
    let address = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host);
    address.parse::<IpAddr>().is_ok_and(|ip| ip.is_loopback())
}

#[cfg(test)]
mod tests {
    use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
    use cloud_sdk::transport::CustomEndpointAcknowledgement;
    use cloud_sdk::{ProviderId, ServiceId};

    use super::{AuthenticationValidationError, validate_bearer_authentication};
    use crate::shared::{BearerCredentialScope, HttpsEndpoint};

    fn endpoint(value: &str) -> HttpsEndpoint {
        HttpsEndpoint::new_custom(
            value,
            CustomEndpointAcknowledgement::trusted_operator_configuration(),
        )
        .unwrap_or_else(|_| unreachable!())
    }

    fn provider() -> ProviderId {
        ProviderId::new("example").unwrap_or_else(|_| unreachable!())
    }

    fn service() -> ServiceId {
        ServiceId::new("compute").unwrap_or_else(|_| unreachable!())
    }

    fn policy<'a>(
        endpoint: cloud_sdk::transport::EndpointIdentity<'a>,
    ) -> AuthenticationScopePolicy<'a> {
        AuthenticationScopePolicy::new(
            ScopeRequirement::Required(provider()),
            ScopeRequirement::Required(service()),
            ScopeRequirement::Required(endpoint),
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
        )
    }

    fn unscoped_policy() -> AuthenticationScopePolicy<'static> {
        AuthenticationScopePolicy::new(
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
        )
    }

    #[test]
    fn exact_https_endpoint_provider_and_service_scope_is_admitted() {
        let configured = endpoint("https://api.example.test/v1");
        let credential_scope = BearerCredentialScope::unscoped()
            .with_provider(provider())
            .with_service(service())
            .with_endpoint(configured.clone());
        let identity = configured.identity().unwrap_or_else(|_| unreachable!());
        assert_eq!(
            validate_bearer_authentication(identity, &credential_scope, policy(identity), false,),
            Ok(())
        );
    }

    #[test]
    fn configured_and_credential_endpoint_mismatches_fail_closed() {
        let configured = endpoint("https://api.example.test/v1");
        let other = endpoint("https://other.example.test/v1");
        let identity = configured.identity().unwrap_or_else(|_| unreachable!());
        let other_identity = other.identity().unwrap_or_else(|_| unreachable!());
        let correct_scope = BearerCredentialScope::unscoped()
            .with_provider(provider())
            .with_service(service())
            .with_endpoint(configured.clone());
        assert_eq!(
            validate_bearer_authentication(identity, &correct_scope, policy(other_identity), false,),
            Err(AuthenticationValidationError::EndpointMismatch)
        );

        let wrong_scope = BearerCredentialScope::unscoped()
            .with_provider(provider())
            .with_service(service())
            .with_endpoint(other);
        assert_eq!(
            validate_bearer_authentication(identity, &wrong_scope, policy(identity), false,),
            Err(AuthenticationValidationError::ScopeRejected)
        );
    }

    #[test]
    fn unscoped_credential_cannot_bypass_required_adapter_policy() {
        let configured = endpoint("https://api.example.test/v1");
        let identity = configured.identity().unwrap_or_else(|_| unreachable!());
        assert_eq!(
            validate_bearer_authentication(
                identity,
                &BearerCredentialScope::unscoped(),
                policy(identity),
                false,
            ),
            Err(AuthenticationValidationError::ScopeRejected)
        );
    }

    #[test]
    fn production_validation_rejects_plain_http_even_with_forbidden_endpoint_scope() {
        let configured = HttpsEndpoint::local_http("http://127.0.0.1:3000/v1")
            .unwrap_or_else(|_| unreachable!());
        let identity = configured.identity().unwrap_or_else(|_| unreachable!());
        let policy = AuthenticationScopePolicy::new(
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
            ScopeRequirement::Forbidden,
        );
        assert_eq!(
            validate_bearer_authentication(
                identity,
                &BearerCredentialScope::unscoped(),
                policy,
                false,
            ),
            Err(AuthenticationValidationError::InsecureEndpoint)
        );
    }

    #[test]
    fn test_exception_admits_only_numeric_http_loopback_destinations() {
        let credential = BearerCredentialScope::unscoped();
        for host in ["127.0.0.1", "[::1]"] {
            let identity = cloud_sdk::transport::EndpointIdentity::new(
                cloud_sdk::transport::EndpointScheme::Http,
                host,
                3000,
                "/v1",
            )
            .unwrap_or_else(|_| unreachable!());
            assert_eq!(
                validate_bearer_authentication(identity, &credential, unscoped_policy(), true,),
                Ok(())
            );
        }

        for host in ["192.0.2.1", "localhost", "api.example.test"] {
            let identity = cloud_sdk::transport::EndpointIdentity::new(
                cloud_sdk::transport::EndpointScheme::Http,
                host,
                3000,
                "/v1",
            )
            .unwrap_or_else(|_| unreachable!());
            assert_eq!(
                validate_bearer_authentication(identity, &credential, unscoped_policy(), true,),
                Err(AuthenticationValidationError::InsecureEndpoint)
            );
        }
    }
}
