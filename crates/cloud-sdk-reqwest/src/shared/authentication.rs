use cloud_sdk::authentication::{AuthenticationScopePolicy, ScopeRequirement};
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme};
use std::net::IpAddr;

use super::BearerCredentialScope;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuthenticationValidationError {
    InsecureEndpoint,
    EndpointMismatch,
    IncompletePolicy,
    ScopeRejected,
}

pub(crate) fn validate_bearer_authentication<'a>(
    endpoint: EndpointIdentity<'a>,
    scope: &'a BearerCredentialScope,
    policy: AuthenticationScopePolicy<'a>,
    allow_insecure_loopback: bool,
) -> Result<(), AuthenticationValidationError> {
    let secure_destination = endpoint.scheme() == EndpointScheme::Https;
    let admitted_test_loopback = cfg!(test)
        && allow_insecure_loopback
        && endpoint.scheme() == EndpointScheme::Http
        && is_numeric_loopback(endpoint.host());
    if !secure_destination && !admitted_test_loopback {
        return Err(AuthenticationValidationError::InsecureEndpoint);
    }
    if scope
        .endpoint_identity()
        .map_err(|_| AuthenticationValidationError::ScopeRejected)?
        != endpoint
    {
        return Err(AuthenticationValidationError::EndpointMismatch);
    }
    if !matches!(
        policy.provider_requirement(),
        ScopeRequirement::Required(expected) if expected == scope.provider()
    ) || !matches!(
        policy.service_requirement(),
        ScopeRequirement::Required(expected) if expected == scope.service()
    ) {
        return Err(AuthenticationValidationError::IncompletePolicy);
    }
    match policy.endpoint_requirement() {
        ScopeRequirement::Required(expected) if expected == endpoint => {}
        ScopeRequirement::Required(_) => {
            return Err(AuthenticationValidationError::EndpointMismatch);
        }
        ScopeRequirement::Optional(_) | ScopeRequirement::Forbidden => {
            return Err(AuthenticationValidationError::IncompletePolicy);
        }
    }
    let credential_scope = scope
        .borrowed()
        .map_err(|_| AuthenticationValidationError::ScopeRejected)?;
    if admitted_test_loopback {
        return Ok(());
    }
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

    #[test]
    fn exact_https_endpoint_provider_and_service_scope_is_admitted() {
        let configured = endpoint("https://api.example.test/v1");
        let credential_scope =
            BearerCredentialScope::new(provider(), service(), configured.clone());
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
        let correct_scope = BearerCredentialScope::new(provider(), service(), configured.clone());
        assert_eq!(
            validate_bearer_authentication(identity, &correct_scope, policy(other_identity), false,),
            Err(AuthenticationValidationError::EndpointMismatch)
        );

        let wrong_scope = BearerCredentialScope::new(provider(), service(), other);
        assert_eq!(
            validate_bearer_authentication(identity, &wrong_scope, policy(identity), false,),
            Err(AuthenticationValidationError::EndpointMismatch)
        );
    }

    #[test]
    fn optional_or_forbidden_base_identity_rules_cannot_downgrade_binding() {
        let configured = endpoint("https://api.example.test/v1");
        let identity = configured.identity().unwrap_or_else(|_| unreachable!());
        let credential = BearerCredentialScope::new(provider(), service(), configured.clone());
        let policies = [
            AuthenticationScopePolicy::new(
                ScopeRequirement::Optional(provider()),
                ScopeRequirement::Required(service()),
                ScopeRequirement::Required(identity),
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
            ),
            AuthenticationScopePolicy::new(
                ScopeRequirement::Required(provider()),
                ScopeRequirement::Forbidden,
                ScopeRequirement::Required(identity),
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
            ),
            AuthenticationScopePolicy::new(
                ScopeRequirement::Required(provider()),
                ScopeRequirement::Required(service()),
                ScopeRequirement::Optional(identity),
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
            ),
        ];
        for downgraded in policies {
            assert_eq!(
                validate_bearer_authentication(identity, &credential, downgraded, false),
                Err(AuthenticationValidationError::IncompletePolicy)
            );
        }
    }

    #[test]
    fn production_validation_rejects_plain_http_with_complete_scope() {
        let configured = HttpsEndpoint::local_http("http://127.0.0.1:3000/v1")
            .unwrap_or_else(|_| unreachable!());
        let identity = configured.identity().unwrap_or_else(|_| unreachable!());
        let credential = BearerCredentialScope::new(provider(), service(), configured.clone());
        assert_eq!(
            validate_bearer_authentication(identity, &credential, policy(identity), false,),
            Err(AuthenticationValidationError::InsecureEndpoint)
        );
    }

    #[test]
    fn test_exception_admits_only_numeric_http_loopback_destinations() {
        for value in ["http://127.0.0.1:3000/v1", "http://[::1]:3000/v1"] {
            let configured = HttpsEndpoint::local_http(value).unwrap_or_else(|_| unreachable!());
            let identity = configured.identity().unwrap_or_else(|_| unreachable!());
            let credential = BearerCredentialScope::new(provider(), service(), configured.clone());
            assert_eq!(
                validate_bearer_authentication(identity, &credential, policy(identity), true,),
                Ok(())
            );
        }

        let configured = HttpsEndpoint::local_http("http://127.0.0.1:3000/v1")
            .unwrap_or_else(|_| unreachable!());
        let credential = BearerCredentialScope::new(provider(), service(), configured);
        for host in ["192.0.2.1", "localhost", "api.example.test"] {
            let identity = cloud_sdk::transport::EndpointIdentity::new(
                cloud_sdk::transport::EndpointScheme::Http,
                host,
                3000,
                "/v1",
            )
            .unwrap_or_else(|_| unreachable!());
            assert_eq!(
                validate_bearer_authentication(identity, &credential, policy(identity), true,),
                Err(AuthenticationValidationError::InsecureEndpoint)
            );
        }
    }
}
