use core::fmt;

use crate::transport::{EndpointIdentity, EndpointScheme};
use crate::{ProviderId, ServiceId};

use super::ScopeValue;

/// Provider or operation requirement for one authentication-scope field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeRequirement<T> {
    /// The credential must contain the exact expected value.
    Required(T),
    /// The credential may omit the field; a supplied value must match.
    Optional(T),
    /// The credential must omit the field.
    Forbidden,
}

/// Immutable borrowed scope bound to one admitted credential.
#[derive(Clone, Copy)]
pub struct AuthenticationScope<'a> {
    provider: Option<ProviderId>,
    service: Option<ServiceId>,
    endpoint: Option<EndpointIdentity<'a>>,
    audience: Option<ScopeValue<'a>>,
    account: Option<ScopeValue<'a>>,
    tenant: Option<ScopeValue<'a>>,
}

impl<'a> AuthenticationScope<'a> {
    /// Creates a completely unscoped credential identity.
    ///
    /// A policy with any required field rejects this value.
    #[must_use]
    pub const fn unscoped() -> Self {
        Self {
            provider: None,
            service: None,
            endpoint: None,
            audience: None,
            account: None,
            tenant: None,
        }
    }

    /// Binds a provider.
    #[must_use]
    pub const fn with_provider(mut self, value: ProviderId) -> Self {
        self.provider = Some(value);
        self
    }

    /// Binds a provider-owned service.
    #[must_use]
    pub const fn with_service(mut self, value: ServiceId) -> Self {
        self.service = Some(value);
        self
    }

    /// Binds a normalized endpoint identity.
    #[must_use]
    pub const fn with_endpoint(mut self, value: EndpointIdentity<'a>) -> Self {
        self.endpoint = Some(value);
        self
    }

    /// Binds a provider-owned audience.
    #[must_use]
    pub const fn with_audience(mut self, value: ScopeValue<'a>) -> Self {
        self.audience = Some(value);
        self
    }

    /// Binds a provider-owned account.
    #[must_use]
    pub const fn with_account(mut self, value: ScopeValue<'a>) -> Self {
        self.account = Some(value);
        self
    }

    /// Binds a provider-owned tenant.
    #[must_use]
    pub const fn with_tenant(mut self, value: ScopeValue<'a>) -> Self {
        self.tenant = Some(value);
        self
    }
}

impl fmt::Debug for AuthenticationScope<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthenticationScope")
            .field("provider", &self.provider)
            .field("service", &self.service)
            .field("endpoint", &self.endpoint)
            .field("audience", &self.audience.map(|_| "[redacted]"))
            .field("account", &self.account.map(|_| "[redacted]"))
            .field("tenant", &self.tenant.map(|_| "[redacted]"))
            .finish()
    }
}

/// Complete provider or operation-owned authentication-scope policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthenticationScopePolicy<'a> {
    provider: ScopeRequirement<ProviderId>,
    service: ScopeRequirement<ServiceId>,
    endpoint: ScopeRequirement<EndpointIdentity<'a>>,
    audience: ScopeRequirement<ScopeValue<'a>>,
    account: ScopeRequirement<ScopeValue<'a>>,
    tenant: ScopeRequirement<ScopeValue<'a>>,
}

impl<'a> AuthenticationScopePolicy<'a> {
    /// Creates a policy with an explicit rule for every scope field.
    #[must_use]
    pub const fn new(
        provider: ScopeRequirement<ProviderId>,
        service: ScopeRequirement<ServiceId>,
        endpoint: ScopeRequirement<EndpointIdentity<'a>>,
        audience: ScopeRequirement<ScopeValue<'a>>,
        account: ScopeRequirement<ScopeValue<'a>>,
        tenant: ScopeRequirement<ScopeValue<'a>>,
    ) -> Self {
        Self {
            provider,
            service,
            endpoint,
            audience,
            account,
            tenant,
        }
    }

    /// Returns the provider rule for adapter identity verification.
    #[must_use]
    pub const fn provider_requirement(self) -> ScopeRequirement<ProviderId> {
        self.provider
    }

    /// Returns the service rule for adapter identity verification.
    #[must_use]
    pub const fn service_requirement(self) -> ScopeRequirement<ServiceId> {
        self.service
    }

    /// Returns the endpoint rule for adapter destination verification.
    #[must_use]
    pub const fn endpoint_requirement(self) -> ScopeRequirement<EndpointIdentity<'a>> {
        self.endpoint
    }

    /// Validates one credential scope before authorization-header construction.
    pub fn validate(self, scope: AuthenticationScope<'a>) -> Result<(), AuthenticationScopeError> {
        validate_endpoint_security(self.endpoint)?;
        validate_field(scope.provider, self.provider, ScopeField::Provider)?;
        validate_field(scope.service, self.service, ScopeField::Service)?;
        validate_field(scope.endpoint, self.endpoint, ScopeField::Endpoint)?;
        validate_field(scope.audience, self.audience, ScopeField::Audience)?;
        validate_field(scope.account, self.account, ScopeField::Account)?;
        validate_field(scope.tenant, self.tenant, ScopeField::Tenant)
    }
}

/// Authentication-scope field that failed policy validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeField {
    /// Provider namespace.
    Provider,
    /// Provider-owned service namespace.
    Service,
    /// Normalized endpoint identity.
    Endpoint,
    /// Provider-owned audience.
    Audience,
    /// Provider-owned account.
    Account,
    /// Provider-owned tenant.
    Tenant,
}

/// Payload-free reason a scope field failed validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScopeViolation {
    /// A required field was omitted.
    MissingRequired,
    /// A forbidden field was supplied.
    SuppliedForbidden,
    /// A supplied field did not equal the expected value.
    Mismatch,
    /// An endpoint policy expected a non-HTTPS identity.
    InsecureEndpoint,
}

/// Payload-free authentication-scope policy failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthenticationScopeError {
    field: ScopeField,
    violation: ScopeViolation,
}

impl AuthenticationScopeError {
    /// Returns the rejected field without exposing its value.
    #[must_use]
    pub const fn field(self) -> ScopeField {
        self.field
    }

    /// Returns the payload-free rejection category.
    #[must_use]
    pub const fn violation(self) -> ScopeViolation {
        self.violation
    }
}

impl fmt::Display for AuthenticationScopeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.violation {
            ScopeViolation::MissingRequired => "required authentication scope field is missing",
            ScopeViolation::SuppliedForbidden => {
                "forbidden authentication scope field was supplied"
            }
            ScopeViolation::Mismatch => "authentication scope field does not match policy",
            ScopeViolation::InsecureEndpoint => "authentication endpoint policy is not HTTPS",
        })
    }
}

impl core::error::Error for AuthenticationScopeError {}

fn validate_field<T: Copy + Eq>(
    actual: Option<T>,
    requirement: ScopeRequirement<T>,
    field: ScopeField,
) -> Result<(), AuthenticationScopeError> {
    let violation = match (actual, requirement) {
        (None, ScopeRequirement::Required(_)) => Some(ScopeViolation::MissingRequired),
        (Some(_), ScopeRequirement::Forbidden) => Some(ScopeViolation::SuppliedForbidden),
        (
            Some(actual),
            ScopeRequirement::Required(expected) | ScopeRequirement::Optional(expected),
        ) if actual != expected => Some(ScopeViolation::Mismatch),
        _ => None,
    };
    violation.map_or(Ok(()), |violation| {
        Err(AuthenticationScopeError { field, violation })
    })
}

fn validate_endpoint_security(
    requirement: ScopeRequirement<EndpointIdentity<'_>>,
) -> Result<(), AuthenticationScopeError> {
    let expected = match requirement {
        ScopeRequirement::Required(value) | ScopeRequirement::Optional(value) => value,
        ScopeRequirement::Forbidden => return Ok(()),
    };
    if expected.scheme() == EndpointScheme::Https {
        Ok(())
    } else {
        Err(AuthenticationScopeError {
            field: ScopeField::Endpoint,
            violation: ScopeViolation::InsecureEndpoint,
        })
    }
}
