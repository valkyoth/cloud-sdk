use core::fmt;
use core::future::Future;

use crate::transport::{ResponseWriter, TransportRequest};

use super::AuthenticationScopePolicy;

/// Request plus mandatory provider or operation-owned authentication policy.
#[derive(Clone, Copy)]
pub struct AuthenticatedRequest<'request, 'policy> {
    request: TransportRequest<'request>,
    policy: AuthenticationScopePolicy<'policy>,
}

impl<'request, 'policy> AuthenticatedRequest<'request, 'policy> {
    /// Binds a transport request to its complete authentication policy.
    #[must_use]
    pub const fn new(
        request: TransportRequest<'request>,
        policy: AuthenticationScopePolicy<'policy>,
    ) -> Self {
        Self { request, policy }
    }

    /// Returns the credential-free transport request.
    #[must_use]
    pub const fn transport_request(self) -> TransportRequest<'request> {
        self.request
    }

    /// Returns the complete authentication policy.
    #[must_use]
    pub const fn policy(self) -> AuthenticationScopePolicy<'policy> {
        self.policy
    }
}

impl fmt::Debug for AuthenticatedRequest<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthenticatedRequest")
            .field("request", &self.request)
            .field("policy", &self.policy)
            .finish()
    }
}

/// Blocking transport that cannot execute without an authentication policy.
pub trait BlockingAuthenticatedTransport {
    /// Transport-specific failure.
    type Error;

    /// Validates scope and sends one authenticated request.
    fn send_authenticated(
        &self,
        request: AuthenticatedRequest<'_, '_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>;
}

/// Executor-neutral async transport requiring an authentication policy.
pub trait AsyncAuthenticatedTransport {
    /// Transport-specific failure.
    type Error;

    /// Validates scope and sends one authenticated request.
    fn send_authenticated<'transport, 'request, 'policy, 'writer>(
        &'transport self,
        request: AuthenticatedRequest<'request, 'policy>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'writer
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer;
}
