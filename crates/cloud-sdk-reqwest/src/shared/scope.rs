use core::fmt;
use std::string::String;

use cloud_sdk::authentication::{AuthenticationScope, ScopeValue, ScopeValueError};
use cloud_sdk::{ProviderId, ServiceId};

use super::{BearerToken, HttpsEndpoint};

/// Owned authentication-scope construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BearerCredentialScopeError {
    /// A provider-owned scope value failed core validation.
    ValueRejected(ScopeValueError),
    /// Adapter-owned scope storage could not be allocated.
    AllocationFailed,
    /// A previously admitted endpoint identity could not be recovered.
    EndpointIdentityRejected,
}

impl fmt::Display for BearerCredentialScopeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ValueRejected(_) => "bearer credential scope value was rejected",
            Self::AllocationFailed => "bearer credential scope allocation failed",
            Self::EndpointIdentityRejected => "bearer credential endpoint identity was rejected",
        })
    }
}

impl core::error::Error for BearerCredentialScopeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::ValueRejected(error) => Some(error),
            Self::AllocationFailed => None,
            Self::EndpointIdentityRejected => None,
        }
    }
}

/// Immutable owned scope attached to one bearer-credential lifecycle.
///
/// Rotation and refresh replace only the token. They cannot change this scope.
pub struct BearerCredentialScope {
    provider: Option<ProviderId>,
    service: Option<ServiceId>,
    endpoint: Option<HttpsEndpoint>,
    audience: Option<String>,
    account: Option<String>,
    tenant: Option<String>,
}

impl BearerCredentialScope {
    /// Creates an unscoped credential.
    ///
    /// Provider policies with required fields reject this scope before an
    /// authorization header is constructed.
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

    /// Binds a provider namespace.
    #[must_use]
    pub const fn with_provider(mut self, value: ProviderId) -> Self {
        self.provider = Some(value);
        self
    }

    /// Binds a provider-owned service namespace.
    #[must_use]
    pub const fn with_service(mut self, value: ServiceId) -> Self {
        self.service = Some(value);
        self
    }

    /// Binds a normalized credential endpoint.
    #[must_use]
    pub fn with_endpoint(mut self, value: HttpsEndpoint) -> Self {
        self.endpoint = Some(value);
        self
    }

    /// Binds a provider-owned audience.
    pub fn try_with_audience(mut self, value: &str) -> Result<Self, BearerCredentialScopeError> {
        self.audience = Some(copy_scope_value(value)?);
        Ok(self)
    }

    /// Binds a provider-owned account.
    pub fn try_with_account(mut self, value: &str) -> Result<Self, BearerCredentialScopeError> {
        self.account = Some(copy_scope_value(value)?);
        Ok(self)
    }

    /// Binds a provider-owned tenant.
    pub fn try_with_tenant(mut self, value: &str) -> Result<Self, BearerCredentialScopeError> {
        self.tenant = Some(copy_scope_value(value)?);
        Ok(self)
    }

    pub(crate) fn borrowed(&self) -> Result<AuthenticationScope<'_>, BearerCredentialScopeError> {
        let mut scope = AuthenticationScope::unscoped();
        if let Some(value) = self.provider {
            scope = scope.with_provider(value);
        }
        if let Some(value) = self.service {
            scope = scope.with_service(value);
        }
        if let Some(value) = &self.endpoint {
            scope = scope.with_endpoint(
                value
                    .identity()
                    .map_err(|_| BearerCredentialScopeError::EndpointIdentityRejected)?,
            );
        }
        if let Some(value) = self.audience.as_deref() {
            scope = scope.with_audience(
                ScopeValue::new(value).map_err(BearerCredentialScopeError::ValueRejected)?,
            );
        }
        if let Some(value) = self.account.as_deref() {
            scope = scope.with_account(
                ScopeValue::new(value).map_err(BearerCredentialScopeError::ValueRejected)?,
            );
        }
        if let Some(value) = self.tenant.as_deref() {
            scope = scope.with_tenant(
                ScopeValue::new(value).map_err(BearerCredentialScopeError::ValueRejected)?,
            );
        }
        Ok(scope)
    }
}

impl fmt::Debug for BearerCredentialScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BearerCredentialScope")
            .field("provider", &self.provider)
            .field("service", &self.service)
            .field("endpoint", &self.endpoint.as_ref().map(|_| "[redacted]"))
            .field("audience", &self.audience.as_ref().map(|_| "[redacted]"))
            .field("account", &self.account.as_ref().map(|_| "[redacted]"))
            .field("tenant", &self.tenant.as_ref().map(|_| "[redacted]"))
            .finish()
    }
}

/// Initial bearer token and its immutable authentication scope.
pub struct BearerCredential {
    pub(crate) token: BearerToken,
    pub(crate) scope: BearerCredentialScope,
}

impl BearerCredential {
    /// Binds a validated token to an immutable scope.
    #[must_use]
    pub const fn new(token: BearerToken, scope: BearerCredentialScope) -> Self {
        Self { token, scope }
    }
}

impl fmt::Debug for BearerCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BearerCredential([redacted])")
    }
}

fn copy_scope_value(value: &str) -> Result<String, BearerCredentialScopeError> {
    ScopeValue::new(value).map_err(BearerCredentialScopeError::ValueRejected)?;
    let mut owned = String::new();
    owned
        .try_reserve_exact(value.len())
        .map_err(|_| BearerCredentialScopeError::AllocationFailed)?;
    owned.push_str(value);
    Ok(owned)
}

#[cfg(test)]
mod tests {
    use cloud_sdk::transport::CustomEndpointAcknowledgement;
    use cloud_sdk::{ProviderId, ServiceId};

    use super::{BearerCredentialScope, BearerCredentialScopeError, ScopeValueError};
    use crate::shared::HttpsEndpoint;

    #[test]
    fn owned_scope_validates_values_and_redacts_all_provider_fields() {
        let provider = ProviderId::new("example").unwrap_or_else(|_| unreachable!());
        let service = ServiceId::new("compute").unwrap_or_else(|_| unreachable!());
        let endpoint = HttpsEndpoint::new_custom(
            "https://api.example.test/v1",
            CustomEndpointAcknowledgement::trusted_operator_configuration(),
        )
        .unwrap_or_else(|_| unreachable!());
        let scope = BearerCredentialScope::unscoped()
            .with_provider(provider)
            .with_service(service)
            .with_endpoint(endpoint)
            .try_with_audience("secret-audience")
            .and_then(|scope| scope.try_with_account("secret-account"))
            .and_then(|scope| scope.try_with_tenant("secret-tenant"));
        assert!(scope.is_ok());
        if let Ok(scope) = scope {
            let debug = std::format!("{scope:?}");
            assert!(debug.contains("[redacted]"));
            assert!(!debug.contains("secret-audience"));
            assert!(!debug.contains("secret-account"));
            assert!(!debug.contains("secret-tenant"));
            assert!(scope.borrowed().is_ok());
        }
    }

    #[test]
    fn invalid_scope_value_does_not_create_partial_owned_scope() {
        let result = BearerCredentialScope::unscoped().try_with_audience("contains space");
        assert!(matches!(
            result,
            Err(BearerCredentialScopeError::ValueRejected(
                ScopeValueError::InvalidByte
            ))
        ));
    }
}
