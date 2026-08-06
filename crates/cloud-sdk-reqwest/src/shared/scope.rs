use core::fmt;
use std::string::String;

use cloud_sdk::authentication::{
    AuthenticationScope, CredentialLifetime, ScopeValue, ScopeValueError,
};
use cloud_sdk::transport::EndpointIdentity;
use cloud_sdk::{ProviderId, ServiceId};

use super::{BearerToken, HttpsEndpoint};

/// Owned authentication-scope construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialScopeError {
    /// A provider-owned scope value failed core validation.
    ValueRejected(ScopeValueError),
    /// Adapter-owned scope storage could not be allocated.
    AllocationFailed,
    /// A previously admitted endpoint identity could not be recovered.
    EndpointIdentityRejected,
}

impl fmt::Display for CredentialScopeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ValueRejected(_) => "credential scope value was rejected",
            Self::AllocationFailed => "credential scope allocation failed",
            Self::EndpointIdentityRejected => "credential endpoint identity was rejected",
        })
    }
}

impl core::error::Error for CredentialScopeError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::ValueRejected(error) => Some(error),
            Self::AllocationFailed | Self::EndpointIdentityRejected => None,
        }
    }
}

/// Compatibility name for bearer-scope construction failures.
pub type BearerCredentialScopeError = CredentialScopeError;
/// Basic-scope construction failure.
pub type BasicCredentialScopeError = CredentialScopeError;

struct OwnedCredentialScope {
    provider: ProviderId,
    service: ServiceId,
    endpoint: HttpsEndpoint,
    audience: Option<String>,
    account: Option<String>,
    tenant: Option<String>,
}

impl OwnedCredentialScope {
    const fn new(provider: ProviderId, service: ServiceId, endpoint: HttpsEndpoint) -> Self {
        Self {
            provider,
            service,
            endpoint,
            audience: None,
            account: None,
            tenant: None,
        }
    }

    fn try_with_audience(mut self, value: &str) -> Result<Self, CredentialScopeError> {
        self.audience = Some(copy_scope_value(value)?);
        Ok(self)
    }

    fn try_with_account(mut self, value: &str) -> Result<Self, CredentialScopeError> {
        self.account = Some(copy_scope_value(value)?);
        Ok(self)
    }

    fn try_with_tenant(mut self, value: &str) -> Result<Self, CredentialScopeError> {
        self.tenant = Some(copy_scope_value(value)?);
        Ok(self)
    }

    fn borrowed(&self) -> Result<AuthenticationScope<'_>, CredentialScopeError> {
        self.borrowed_with_endpoint(self.endpoint_identity()?)
    }

    fn borrowed_with_endpoint<'a>(
        &'a self,
        endpoint: EndpointIdentity<'a>,
    ) -> Result<AuthenticationScope<'a>, CredentialScopeError> {
        let mut scope = AuthenticationScope::unscoped()
            .with_provider(self.provider)
            .with_service(self.service)
            .with_endpoint(endpoint);
        if let Some(value) = self.audience.as_deref() {
            scope = scope.with_audience(
                ScopeValue::new(value).map_err(CredentialScopeError::ValueRejected)?,
            );
        }
        if let Some(value) = self.account.as_deref() {
            scope = scope
                .with_account(ScopeValue::new(value).map_err(CredentialScopeError::ValueRejected)?);
        }
        if let Some(value) = self.tenant.as_deref() {
            scope = scope
                .with_tenant(ScopeValue::new(value).map_err(CredentialScopeError::ValueRejected)?);
        }
        Ok(scope)
    }

    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, CredentialScopeError> {
        self.endpoint
            .identity()
            .map_err(|_| CredentialScopeError::EndpointIdentityRejected)
    }

    fn matches_endpoint(&self, endpoint: &HttpsEndpoint) -> bool {
        self.endpoint_identity()
            .ok()
            .zip(endpoint.identity().ok())
            .is_some_and(|(credential, configured)| credential == configured)
    }
}

pub(crate) trait CredentialScopeView {
    fn provider(&self) -> ProviderId;
    fn service(&self) -> ServiceId;
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, CredentialScopeError>;
    fn borrowed(&self) -> Result<AuthenticationScope<'_>, CredentialScopeError>;
    fn borrowed_with_endpoint<'a>(
        &'a self,
        endpoint: EndpointIdentity<'a>,
    ) -> Result<AuthenticationScope<'a>, CredentialScopeError>;
}

macro_rules! define_scope {
    ($name:ident, $label:literal) => {
        /// Immutable owned scope attached to one credential lifecycle.
        pub struct $name {
            inner: OwnedCredentialScope,
        }

        impl $name {
            /// Binds a credential to one provider, service, and transport endpoint.
            #[must_use]
            pub const fn new(
                provider: ProviderId,
                service: ServiceId,
                endpoint: HttpsEndpoint,
            ) -> Self {
                Self {
                    inner: OwnedCredentialScope::new(provider, service, endpoint),
                }
            }

            /// Binds a provider-owned audience.
            pub fn try_with_audience(mut self, value: &str) -> Result<Self, CredentialScopeError> {
                self.inner = self.inner.try_with_audience(value)?;
                Ok(self)
            }

            /// Binds a provider-owned account.
            pub fn try_with_account(mut self, value: &str) -> Result<Self, CredentialScopeError> {
                self.inner = self.inner.try_with_account(value)?;
                Ok(self)
            }

            /// Binds a provider-owned tenant.
            pub fn try_with_tenant(mut self, value: &str) -> Result<Self, CredentialScopeError> {
                self.inner = self.inner.try_with_tenant(value)?;
                Ok(self)
            }

            pub(crate) const fn provider(&self) -> ProviderId {
                self.inner.provider
            }

            pub(crate) const fn service(&self) -> ServiceId {
                self.inner.service
            }

            pub(crate) fn endpoint_identity(
                &self,
            ) -> Result<EndpointIdentity<'_>, CredentialScopeError> {
                self.inner.endpoint_identity()
            }

            pub(crate) fn borrowed(&self) -> Result<AuthenticationScope<'_>, CredentialScopeError> {
                self.inner.borrowed()
            }

            pub(crate) fn borrowed_with_endpoint<'a>(
                &'a self,
                endpoint: EndpointIdentity<'a>,
            ) -> Result<AuthenticationScope<'a>, CredentialScopeError> {
                self.inner.borrowed_with_endpoint(endpoint)
            }

            pub(crate) fn matches_endpoint(&self, endpoint: &HttpsEndpoint) -> bool {
                self.inner.matches_endpoint(endpoint)
            }
        }

        impl CredentialScopeView for $name {
            fn provider(&self) -> ProviderId {
                self.provider()
            }

            fn service(&self) -> ServiceId {
                self.service()
            }

            fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, CredentialScopeError> {
                self.endpoint_identity()
            }

            fn borrowed(&self) -> Result<AuthenticationScope<'_>, CredentialScopeError> {
                self.borrowed()
            }

            fn borrowed_with_endpoint<'a>(
                &'a self,
                endpoint: EndpointIdentity<'a>,
            ) -> Result<AuthenticationScope<'a>, CredentialScopeError> {
                self.borrowed_with_endpoint(endpoint)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter
                    .debug_struct($label)
                    .field("provider", &self.inner.provider)
                    .field("service", &self.inner.service)
                    .field("endpoint", &"[redacted]")
                    .field(
                        "audience",
                        &self.inner.audience.as_ref().map(|_| "[redacted]"),
                    )
                    .field(
                        "account",
                        &self.inner.account.as_ref().map(|_| "[redacted]"),
                    )
                    .field("tenant", &self.inner.tenant.as_ref().map(|_| "[redacted]"))
                    .finish()
            }
        }
    };
}

define_scope!(BearerCredentialScope, "BearerCredentialScope");
define_scope!(BasicCredentialScope, "BasicCredentialScope");

/// Initial bearer token and its immutable authentication scope.
pub struct BearerCredential {
    pub(crate) token: BearerToken,
    pub(crate) scope: BearerCredentialScope,
    pub(crate) lifetime: Option<CredentialLifetime>,
}

impl BearerCredential {
    /// Binds a validated token to an immutable scope.
    #[must_use]
    pub const fn new(token: BearerToken, scope: BearerCredentialScope) -> Self {
        Self {
            token,
            scope,
            lifetime: None,
        }
    }

    /// Binds an expiring token and its caller-clock lifetime to one scope.
    #[must_use]
    pub const fn new_expiring(
        token: BearerToken,
        scope: BearerCredentialScope,
        lifetime: CredentialLifetime,
    ) -> Self {
        Self {
            token,
            scope,
            lifetime: Some(lifetime),
        }
    }
}

impl fmt::Debug for BearerCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BearerCredential([redacted])")
    }
}

fn copy_scope_value(value: &str) -> Result<String, CredentialScopeError> {
    ScopeValue::new(value).map_err(CredentialScopeError::ValueRejected)?;
    let mut owned = String::new();
    owned
        .try_reserve_exact(value.len())
        .map_err(|_| CredentialScopeError::AllocationFailed)?;
    owned.push_str(value);
    Ok(owned)
}

#[cfg(test)]
mod tests {
    use cloud_sdk::transport::CustomEndpointAcknowledgement;
    use cloud_sdk::{ProviderId, ServiceId};

    use super::{BasicCredentialScope, CredentialScopeError, ScopeValueError};
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
        let scope = BasicCredentialScope::new(provider, service, endpoint)
            .try_with_audience("secret-audience")
            .and_then(|scope| scope.try_with_account("secret-account"))
            .and_then(|scope| scope.try_with_tenant("secret-tenant"));
        assert!(scope.is_ok());
        let Ok(scope) = scope else {
            unreachable!("security fixture construction failed");
        };
        let debug = std::format!("{scope:?}");
        assert!(debug.contains("[redacted]"));
        assert!(!debug.contains("secret-audience"));
        assert!(!debug.contains("secret-account"));
        assert!(!debug.contains("secret-tenant"));
        assert!(scope.borrowed().is_ok());
    }

    #[test]
    fn invalid_scope_value_does_not_create_partial_owned_scope() {
        let provider = ProviderId::new("example").unwrap_or_else(|_| unreachable!());
        let service = ServiceId::new("compute").unwrap_or_else(|_| unreachable!());
        let endpoint = HttpsEndpoint::new_custom(
            "https://api.example.test/v1",
            CustomEndpointAcknowledgement::trusted_operator_configuration(),
        )
        .unwrap_or_else(|_| unreachable!());
        let result =
            BasicCredentialScope::new(provider, service, endpoint).try_with_audience("has space");
        assert!(matches!(
            result,
            Err(CredentialScopeError::ValueRejected(
                ScopeValueError::InvalidByte
            ))
        ));
    }
}
