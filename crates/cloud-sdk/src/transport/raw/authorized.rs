//! Explicit authorization handoff for provider-owned credential policies.
//!
//! Unlike anonymous raw execution, these contracts accept a complete, already
//! encoded Authorization value. The provider must validate credential kind,
//! scope and operation authority before calling them. Implementations must
//! compare the expected destination before copying credentials or doing I/O,
//! send exactly once, disable redirects/retries/cookies, and clear their owned
//! credential copies. They must not infer a Bearer or Basic prefix.

use super::{AsyncRawHttpExecutor, BlockingRawHttpExecutor, RawResponsePolicy};
use crate::transport::{
    AsyncExecutionError, AsyncResponseStaging, BoundTransport, EndpointIdentity, HeaderValue,
    ResponseCompletion, ResponseWriter, TransportRequest,
};
use core::future::Future;

/// Blocking raw execution with an explicit provider-validated credential.
pub trait BlockingAuthorizedRawHttpExecutor: BlockingRawHttpExecutor + BoundTransport {
    /// Sends one request only when the actual destination equals `expected`.
    /// The caller retains ownership and cleanup responsibility for `authorization`.
    fn execute_authorized(
        &self,
        expected: EndpointIdentity<'_>,
        authorization: HeaderValue<'_>,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>;
}

/// Send-async raw execution with explicit provider-validated authorization.
pub trait AsyncAuthorizedRawHttpExecutor: AsyncRawHttpExecutor + BoundTransport {
    /// Stages one response without commit access. Destination validation must
    /// precede dispatch and credential copying; cancellation must drop owned copies.
    fn execute_authorized<'executor, 'request, 'policy, 'writer, 'buffer>(
        &'executor self,
        expected: EndpointIdentity<'request>,
        authorization: HeaderValue<'request>,
        request: TransportRequest<'request>,
        policy: RawResponsePolicy<'policy>,
        response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> impl Future<Output = Result<ResponseCompletion, Self::Error>> + Send + 'writer
    where
        'executor: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer;
}

/// Local-async authorized execution, including adapters with `!Send` futures.
pub trait LocalAuthorizedRawHttpExecutor:
    super::LocalAsyncRawHttpExecutor + BoundTransport
{
    /// Stages one explicitly bound authorized exchange, without commit access.
    fn execute_authorized_local<'executor, 'request, 'policy, 'writer, 'buffer>(
        &'executor self,
        expected: EndpointIdentity<'request>,
        authorization: HeaderValue<'request>,
        request: TransportRequest<'request>,
        policy: RawResponsePolicy<'policy>,
        response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> impl Future<Output = Result<ResponseCompletion, Self::Error>> + 'writer
    where
        'executor: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer;
}

impl<T: AsyncAuthorizedRawHttpExecutor + ?Sized> LocalAuthorizedRawHttpExecutor for T {
    async fn execute_authorized_local<'executor, 'request, 'policy, 'writer, 'buffer>(
        &'executor self,
        expected: EndpointIdentity<'request>,
        authorization: HeaderValue<'request>,
        request: TransportRequest<'request>,
        policy: RawResponsePolicy<'policy>,
        response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'executor: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
    {
        self.execute_authorized(expected, authorization, request, policy, response)
            .await
    }
}

/// Commits a Send-async authorized response only after successful completion.
pub async fn drive_async_authorized_raw<T: AsyncAuthorizedRawHttpExecutor + ?Sized>(
    executor: &T,
    expected: EndpointIdentity<'_>,
    authorization: HeaderValue<'_>,
    request: TransportRequest<'_>,
    policy: RawResponsePolicy<'_>,
    response: &mut ResponseWriter<'_>,
) -> Result<(), AsyncExecutionError<T::Error>> {
    let mut attempt = response
        .begin_attempt()
        .map_err(AsyncExecutionError::Response)?;
    let completion = executor
        .execute_authorized(expected, authorization, request, policy, attempt.staging())
        .await
        .map_err(AsyncExecutionError::Transport)?;
    attempt
        .commit_completion(completion)
        .map_err(AsyncExecutionError::Response)
}

/// Commits a local-async authorized response only after successful completion.
pub async fn drive_local_authorized_raw<T: LocalAuthorizedRawHttpExecutor + ?Sized>(
    executor: &T,
    expected: EndpointIdentity<'_>,
    authorization: HeaderValue<'_>,
    request: TransportRequest<'_>,
    policy: RawResponsePolicy<'_>,
    response: &mut ResponseWriter<'_>,
) -> Result<(), AsyncExecutionError<T::Error>> {
    let mut attempt = response
        .begin_attempt()
        .map_err(AsyncExecutionError::Response)?;
    let completion = executor
        .execute_authorized_local(expected, authorization, request, policy, attempt.staging())
        .await
        .map_err(AsyncExecutionError::Transport)?;
    attempt
        .commit_completion(completion)
        .map_err(AsyncExecutionError::Response)
}
