use core::fmt;
use core::future::Future;

use crate::transport::{
    AsyncExecutionError, AsyncResponseStaging, RawResponsePolicy, ResponseCompletion,
    ResponseWriter, TransportRequest,
};

use super::AuthenticationScopePolicy;

/// Request plus mandatory provider or operation-owned authentication policy.
///
/// Construction is internal so application code cannot bypass prepared or
/// permit-authorized execution through the public transport traits.
///
/// ```compile_fail
/// use cloud_sdk::authentication::{AuthenticatedRequest, AuthenticationScopePolicy};
/// use cloud_sdk::transport::{RawResponsePolicy, TransportRequest};
///
/// fn forge(
///     request: TransportRequest<'_>,
///     authentication: AuthenticationScopePolicy<'_>,
///     response: RawResponsePolicy<'_>,
/// ) {
///     let _ = AuthenticatedRequest::new(request, authentication, &response);
/// }
/// ```
#[derive(Clone, Copy)]
pub struct AuthenticatedRequest<'request, 'policy> {
    request: TransportRequest<'request>,
    policy: AuthenticationScopePolicy<'policy>,
    response_policy: RawResponsePolicy<'policy>,
}

impl<'request, 'policy> AuthenticatedRequest<'request, 'policy> {
    /// Binds a transport request to its complete authentication policy.
    #[must_use]
    pub(crate) const fn new(
        request: TransportRequest<'request>,
        policy: AuthenticationScopePolicy<'policy>,
        response_policy: &RawResponsePolicy<'policy>,
    ) -> Self {
        Self {
            request,
            policy,
            response_policy: *response_policy,
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

/// Executor-neutral Send async transport requiring authentication policy.
///
/// Implementations stage responses without commit access. Callers use
/// [`drive_async_authenticated`].
pub trait AsyncAuthenticatedTransport {
    /// Transport-specific failure.
    type Error;

    /// Validates scope and stages one authenticated response.
    fn send_authenticated<'transport, 'request, 'policy, 'writer, 'buffer>(
        &'transport self,
        request: AuthenticatedRequest<'request, 'policy>,
        response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> impl Future<Output = Result<ResponseCompletion, Self::Error>> + Send + 'writer
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer;
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
    fn send_authenticated_local<'transport, 'request, 'policy, 'writer, 'buffer>(
        &'transport self,
        request: AuthenticatedRequest<'request, 'policy>,
        response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> impl Future<Output = Result<ResponseCompletion, Self::Error>> + 'writer
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer;
}

/// Drives one authenticated local attempt and commits after `Ready(Ok)`.
pub async fn drive_local_authenticated<'transport, 'request, 'policy, 'writer, 'buffer, T>(
    transport: &'transport T,
    request: AuthenticatedRequest<'request, 'policy>,
    response: &'writer mut ResponseWriter<'buffer>,
) -> Result<(), AsyncExecutionError<T::Error>>
where
    T: LocalAsyncAuthenticatedTransport + ?Sized,
    'transport: 'writer,
    'request: 'writer,
    'policy: 'writer,
    'buffer: 'writer,
{
    let mut attempt = response
        .begin_attempt()
        .map_err(AsyncExecutionError::Response)?;
    let completion = transport
        .send_authenticated_local(request, attempt.staging())
        .await
        .map_err(AsyncExecutionError::Transport)?;
    attempt
        .commit_completion(completion)
        .map_err(AsyncExecutionError::Response)
}

/// Drives one authenticated cross-thread async attempt and commits after `Ready(Ok)`.
pub async fn drive_async_authenticated<'transport, 'request, 'policy, 'writer, 'buffer, T>(
    transport: &'transport T,
    request: AuthenticatedRequest<'request, 'policy>,
    response: &'writer mut ResponseWriter<'buffer>,
) -> Result<(), AsyncExecutionError<T::Error>>
where
    T: AsyncAuthenticatedTransport + ?Sized,
    'transport: 'writer,
    'request: 'writer,
    'policy: 'writer,
    'buffer: 'writer,
{
    let mut attempt = response
        .begin_attempt()
        .map_err(AsyncExecutionError::Response)?;
    let completion = transport
        .send_authenticated(request, attempt.staging())
        .await
        .map_err(AsyncExecutionError::Transport)?;
    attempt
        .commit_completion(completion)
        .map_err(AsyncExecutionError::Response)
}

impl<T> LocalAsyncAuthenticatedTransport for T
where
    T: AsyncAuthenticatedTransport + ?Sized,
{
    type Error = T::Error;

    async fn send_authenticated_local<'transport, 'request, 'policy, 'writer, 'buffer>(
        &'transport self,
        request: AuthenticatedRequest<'request, 'policy>,
        response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
    {
        AsyncAuthenticatedTransport::send_authenticated(self, request, response).await
    }
}
