use core::fmt;
use core::future::Future;

use crate::transport::{RawResponsePolicy, ResponseWriter, TransportRequest};

use super::AuthenticationScopePolicy;

/// Request plus mandatory provider or operation-owned authentication policy.
#[derive(Clone, Copy)]
pub struct AuthenticatedRequest<'request, 'policy> {
    request: TransportRequest<'request>,
    policy: AuthenticationScopePolicy<'policy>,
    response_policy: RawResponsePolicy<'policy>,
}

impl<'request, 'policy> AuthenticatedRequest<'request, 'policy> {
    /// Binds a transport request to its complete authentication policy.
    #[must_use]
    pub const fn new(
        request: TransportRequest<'request>,
        policy: AuthenticationScopePolicy<'policy>,
        response_policy: RawResponsePolicy<'policy>,
    ) -> Self {
        Self {
            request,
            policy,
            response_policy,
        }
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

    /// Returns the complete status-class raw response policy.
    #[must_use]
    pub const fn response_policy(self) -> RawResponsePolicy<'policy> {
        self.response_policy
    }
}

impl fmt::Debug for AuthenticatedRequest<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthenticatedRequest")
            .field("request", &self.request)
            .field("policy", &self.policy)
            .field("response_policy", &self.response_policy)
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

/// Executor-neutral authenticated transport for `!Send` local futures.
///
/// Dropping the returned future leaves the response uncommitted and clears
/// partial response state, but does not prove that the authenticated request
/// was not delivered. Cancellation is conservatively
/// [`DeliveryPhase::PossiblySent`](crate::transport::DeliveryPhase::PossiblySent).
pub trait LocalAsyncAuthenticatedTransport {
    /// Transport-specific failure.
    type Error;

    /// Validates scope and sends one authenticated request locally.
    fn send_authenticated_local<'transport, 'request, 'policy, 'writer>(
        &'transport self,
        request: AuthenticatedRequest<'request, 'policy>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> impl Future<Output = Result<(), Self::Error>> + 'writer
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer;
}

impl<T> LocalAsyncAuthenticatedTransport for T
where
    T: AsyncAuthenticatedTransport + ?Sized,
{
    type Error = T::Error;

    fn send_authenticated_local<'transport, 'request, 'policy, 'writer>(
        &'transport self,
        request: AuthenticatedRequest<'request, 'policy>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> impl Future<Output = Result<(), Self::Error>> + 'writer
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
    {
        AsyncAuthenticatedTransport::send_authenticated(self, request, response)
    }
}
