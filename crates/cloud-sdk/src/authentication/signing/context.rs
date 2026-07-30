use core::fmt;

use crate::transport::EndpointIdentity;
use crate::{ProviderId, ServiceId};

use super::super::ScopeValue;

/// Maximum bytes in a signing key identifier.
pub const MAX_SIGNING_KEY_ID_BYTES: usize = 256;
/// Maximum bytes in a signing algorithm identifier.
pub const MAX_SIGNING_ALGORITHM_BYTES: usize = 128;

/// Signing-context text validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SigningContextValueError {
    /// The value must not be empty.
    Empty,
    /// The value exceeds its type-specific byte limit.
    TooLong,
    /// The value must be visible ASCII without backslashes.
    InvalidByte,
}

impl_static_error!(SigningContextValueError,
    Self::Empty => "signing context value is empty",
    Self::TooLong => "signing context value exceeds the length limit",
    Self::InvalidByte => "signing context value contains an invalid byte",
);

macro_rules! define_context_value {
    ($name:ident, $maximum:ident, $label:literal) => {
        #[doc = $label]
        #[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name<'a>(&'a str);

        impl<'a> $name<'a> {
            /// Validates a provider-selected canonical identifier.
            pub fn new(value: &'a str) -> Result<Self, SigningContextValueError> {
                validate_context_value(value, $maximum)?;
                Ok(Self(value))
            }

            /// Returns the exact canonical provider-selected identifier.
            #[must_use]
            pub const fn as_str(self) -> &'a str {
                self.0
            }
        }

        impl fmt::Debug for $name<'_> {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(concat!(stringify!($name), "([redacted])"))
            }
        }
    };
}

define_context_value!(
    SigningKeyId,
    MAX_SIGNING_KEY_ID_BYTES,
    "Borrowed provider-owned signing key identifier."
);
define_context_value!(
    SigningAlgorithm,
    MAX_SIGNING_ALGORITHM_BYTES,
    "Borrowed provider-owned signature algorithm identifier."
);

/// Complete security domain bound into one canonical signature.
#[derive(Clone, Copy)]
pub struct SigningContext<'a> {
    provider: ProviderId,
    service: ServiceId,
    endpoint: EndpointIdentity<'a>,
    audience: Option<ScopeValue<'a>>,
    account: Option<ScopeValue<'a>>,
    tenant: Option<ScopeValue<'a>>,
    key_id: SigningKeyId<'a>,
    algorithm: SigningAlgorithm<'a>,
}

impl<'a> SigningContext<'a> {
    /// Binds required provider, service, endpoint, key, and algorithm identity.
    #[must_use]
    pub const fn new(
        provider: ProviderId,
        service: ServiceId,
        endpoint: EndpointIdentity<'a>,
        key_id: SigningKeyId<'a>,
        algorithm: SigningAlgorithm<'a>,
    ) -> Self {
        Self {
            provider,
            service,
            endpoint,
            audience: None,
            account: None,
            tenant: None,
            key_id,
            algorithm,
        }
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

    /// Returns the bound provider.
    #[must_use]
    pub const fn provider(self) -> ProviderId {
        self.provider
    }

    /// Returns the bound service.
    #[must_use]
    pub const fn service(self) -> ServiceId {
        self.service
    }

    /// Returns the normalized bound endpoint.
    #[must_use]
    pub const fn endpoint(self) -> EndpointIdentity<'a> {
        self.endpoint
    }

    /// Returns the optional bound audience.
    #[must_use]
    pub const fn audience(self) -> Option<ScopeValue<'a>> {
        self.audience
    }

    /// Returns the optional bound account.
    #[must_use]
    pub const fn account(self) -> Option<ScopeValue<'a>> {
        self.account
    }

    /// Returns the optional bound tenant.
    #[must_use]
    pub const fn tenant(self) -> Option<ScopeValue<'a>> {
        self.tenant
    }

    /// Returns the bound key identifier.
    #[must_use]
    pub const fn key_id(self) -> SigningKeyId<'a> {
        self.key_id
    }

    /// Returns the bound signature algorithm identifier.
    #[must_use]
    pub const fn algorithm(self) -> SigningAlgorithm<'a> {
        self.algorithm
    }
}

impl fmt::Debug for SigningContext<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SigningContext")
            .field("provider", &self.provider)
            .field("service", &self.service)
            .field("endpoint", &"[redacted]")
            .field("audience", &self.audience.map(|_| "[redacted]"))
            .field("account", &self.account.map(|_| "[redacted]"))
            .field("tenant", &self.tenant.map(|_| "[redacted]"))
            .field("key_id", &"[redacted]")
            .field("algorithm", &"[redacted]")
            .finish()
    }
}

fn validate_context_value(value: &str, maximum: usize) -> Result<(), SigningContextValueError> {
    if value.is_empty() {
        return Err(SigningContextValueError::Empty);
    }
    if value.len() > maximum {
        return Err(SigningContextValueError::TooLong);
    }
    if !value
        .bytes()
        .all(|byte| byte.is_ascii_graphic() && byte != b'\\')
    {
        return Err(SigningContextValueError::InvalidByte);
    }
    Ok(())
}
