use super::{
    AuthenticationScope, AuthenticationScopePolicy, CredentialGeneration,
    CredentialGenerationError, ScopeField, ScopeRequirement, ScopeValue, ScopeValueError,
    ScopeViolation,
};
use crate::transport::{EndpointIdentity, EndpointScheme};
use crate::{ProviderId, ServiceId};

struct FormatBuffer {
    bytes: [u8; 128],
    len: usize,
}

impl FormatBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 128],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        let Some(bytes) = self.bytes.get(..self.len) else {
            return "";
        };
        core::str::from_utf8(bytes).unwrap_or("")
    }
}

impl core::fmt::Write for FormatBuffer {
    fn write_str(&mut self, value: &str) -> core::fmt::Result {
        let Some(end) = self.len.checked_add(value.len()) else {
            return Err(core::fmt::Error);
        };
        let Some(output) = self.bytes.get_mut(self.len..end) else {
            return Err(core::fmt::Error);
        };
        output.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}

fn provider() -> ProviderId {
    ProviderId::new("example").unwrap_or_else(|_| unreachable!())
}

fn other_provider() -> ProviderId {
    ProviderId::new("other").unwrap_or_else(|_| unreachable!())
}

fn service() -> ServiceId {
    ServiceId::new("compute").unwrap_or_else(|_| unreachable!())
}

fn endpoint() -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, "api.example.test", 443, "/v1")
        .unwrap_or_else(|_| unreachable!())
}

fn other_endpoint() -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, "other.example.test", 443, "/v1")
        .unwrap_or_else(|_| unreachable!())
}

fn value(text: &'static str) -> ScopeValue<'static> {
    ScopeValue::new(text).unwrap_or_else(|_| unreachable!())
}

fn all_forbidden() -> AuthenticationScopePolicy<'static> {
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
fn scope_values_are_bounded_validated_and_redacted() {
    assert_eq!(ScopeValue::new(""), Err(ScopeValueError::Empty));
    assert_eq!(
        ScopeValue::new("contains space"),
        Err(ScopeValueError::InvalidByte)
    );
    assert_eq!(
        ScopeValue::new("contains\\backslash"),
        Err(ScopeValueError::InvalidByte)
    );
    let too_long = "a".repeat(super::MAX_SCOPE_VALUE_BYTES.saturating_add(1));
    assert_eq!(ScopeValue::new(&too_long), Err(ScopeValueError::TooLong));
    let accepted = ScopeValue::new("urn:example:tenant/a?b=c");
    assert!(accepted.is_ok());
    if let Ok(accepted) = accepted {
        let mut debug = FormatBuffer::new();
        assert!(core::fmt::write(&mut debug, format_args!("{accepted:?}")).is_ok());
        let debug = debug.as_str();
        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains("tenant"));
    }
}

#[test]
fn unscoped_credentials_cannot_bypass_required_bindings() {
    let policy = AuthenticationScopePolicy::new(
        ScopeRequirement::Required(provider()),
        ScopeRequirement::Required(service()),
        ScopeRequirement::Required(endpoint()),
        ScopeRequirement::Required(value("audience")),
        ScopeRequirement::Required(value("account")),
        ScopeRequirement::Required(value("tenant")),
    );
    for field in [
        ScopeField::Provider,
        ScopeField::Service,
        ScopeField::Endpoint,
        ScopeField::Audience,
        ScopeField::Account,
        ScopeField::Tenant,
    ] {
        let result = match field {
            ScopeField::Provider => policy.validate(AuthenticationScope::unscoped()),
            ScopeField::Service => {
                policy.validate(AuthenticationScope::unscoped().with_provider(provider()))
            }
            ScopeField::Endpoint => policy.validate(
                AuthenticationScope::unscoped()
                    .with_provider(provider())
                    .with_service(service()),
            ),
            ScopeField::Audience => policy.validate(
                AuthenticationScope::unscoped()
                    .with_provider(provider())
                    .with_service(service())
                    .with_endpoint(endpoint()),
            ),
            ScopeField::Account => policy.validate(
                AuthenticationScope::unscoped()
                    .with_provider(provider())
                    .with_service(service())
                    .with_endpoint(endpoint())
                    .with_audience(value("audience")),
            ),
            ScopeField::Tenant => policy.validate(
                AuthenticationScope::unscoped()
                    .with_provider(provider())
                    .with_service(service())
                    .with_endpoint(endpoint())
                    .with_audience(value("audience"))
                    .with_account(value("account")),
            ),
        };
        assert!(result.is_err());
        let Err(error) = result else {
            unreachable!("security fixture unexpectedly succeeded");
        };
        assert_eq!(error.field(), field);
        assert_eq!(error.violation(), ScopeViolation::MissingRequired);
    }
}

#[test]
fn every_field_enforces_forbidden_and_mismatch_rules() {
    let supplied = [
        AuthenticationScope::unscoped().with_provider(provider()),
        AuthenticationScope::unscoped().with_service(service()),
        AuthenticationScope::unscoped().with_endpoint(endpoint()),
        AuthenticationScope::unscoped().with_audience(value("audience")),
        AuthenticationScope::unscoped().with_account(value("account")),
        AuthenticationScope::unscoped().with_tenant(value("tenant")),
    ];
    for (field, scope) in [
        ScopeField::Provider,
        ScopeField::Service,
        ScopeField::Endpoint,
        ScopeField::Audience,
        ScopeField::Account,
        ScopeField::Tenant,
    ]
    .into_iter()
    .zip(supplied)
    {
        let result = all_forbidden().validate(scope);
        assert!(result.is_err());
        let Err(error) = result else {
            unreachable!("security fixture unexpectedly succeeded");
        };
        assert_eq!(error.field(), field);
        assert_eq!(error.violation(), ScopeViolation::SuppliedForbidden);
    }

    let mismatch_cases = [
        (
            AuthenticationScopePolicy::new(
                ScopeRequirement::Required(other_provider()),
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
            ),
            AuthenticationScope::unscoped().with_provider(provider()),
            ScopeField::Provider,
        ),
        (
            AuthenticationScopePolicy::new(
                ScopeRequirement::Forbidden,
                ScopeRequirement::Required(ServiceId::new("storage").unwrap_or(service())),
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
            ),
            AuthenticationScope::unscoped().with_service(service()),
            ScopeField::Service,
        ),
        (
            AuthenticationScopePolicy::new(
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Required(other_endpoint()),
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
            ),
            AuthenticationScope::unscoped().with_endpoint(endpoint()),
            ScopeField::Endpoint,
        ),
    ];
    for (policy, scope, field) in mismatch_cases {
        let result = policy.validate(scope);
        assert!(result.is_err());
        let Err(error) = result else {
            unreachable!("security fixture unexpectedly succeeded");
        };
        assert_eq!(error.field(), field);
        assert_eq!(error.violation(), ScopeViolation::Mismatch);
    }

    for (field, policy, scope) in [
        (
            ScopeField::Audience,
            AuthenticationScopePolicy::new(
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Required(value("expected")),
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
            ),
            AuthenticationScope::unscoped().with_audience(value("actual")),
        ),
        (
            ScopeField::Account,
            AuthenticationScopePolicy::new(
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Required(value("expected")),
                ScopeRequirement::Forbidden,
            ),
            AuthenticationScope::unscoped().with_account(value("actual")),
        ),
        (
            ScopeField::Tenant,
            AuthenticationScopePolicy::new(
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Forbidden,
                ScopeRequirement::Required(value("expected")),
            ),
            AuthenticationScope::unscoped().with_tenant(value("actual")),
        ),
    ] {
        let result = policy.validate(scope);
        assert!(result.is_err());
        let Err(error) = result else {
            unreachable!("security fixture unexpectedly succeeded");
        };
        assert_eq!(error.field(), field);
        assert_eq!(error.violation(), ScopeViolation::Mismatch);
    }
}

#[test]
fn optional_fields_accept_omission_and_exact_presence() {
    let policy = AuthenticationScopePolicy::new(
        ScopeRequirement::Optional(provider()),
        ScopeRequirement::Optional(service()),
        ScopeRequirement::Optional(endpoint()),
        ScopeRequirement::Optional(value("audience")),
        ScopeRequirement::Optional(value("account")),
        ScopeRequirement::Optional(value("tenant")),
    );
    assert!(policy.validate(AuthenticationScope::unscoped()).is_ok());
    let complete = AuthenticationScope::unscoped()
        .with_provider(provider())
        .with_service(service())
        .with_endpoint(endpoint())
        .with_audience(value("audience"))
        .with_account(value("account"))
        .with_tenant(value("tenant"));
    assert!(policy.validate(complete).is_ok());
}

#[test]
fn endpoint_policy_rejects_plain_http_before_scope_comparison() {
    let http = EndpointIdentity::new(EndpointScheme::Http, "127.0.0.1", 80, "/")
        .unwrap_or_else(|_| unreachable!());
    let policy = AuthenticationScopePolicy::new(
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Required(http),
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
        ScopeRequirement::Forbidden,
    );
    let result = policy.validate(AuthenticationScope::unscoped().with_endpoint(http));
    assert!(result.is_err());
    let Err(error) = result else {
        unreachable!("security fixture unexpectedly succeeded");
    };
    assert_eq!(error.field(), ScopeField::Endpoint);
    assert_eq!(error.violation(), ScopeViolation::InsecureEndpoint);
}

#[test]
fn credential_generations_never_wrap_and_create_stable_handoffs() {
    let initial = CredentialGeneration::INITIAL;
    assert_eq!(initial.get(), 1);
    let handoff = initial.refresh_handoff();
    assert_eq!(handoff.expected_generation(), initial);
    assert_eq!(
        CredentialGeneration::from_raw_for_test(u64::MAX).checked_next(),
        Err(CredentialGenerationError::Exhausted)
    );
}
