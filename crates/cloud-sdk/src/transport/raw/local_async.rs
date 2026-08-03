use core::future::Future;

use super::{AsyncRawHttpExecutor, RawResponsePolicy};
use crate::transport::{
    AsyncExecutionError, AsyncResponseStaging, ResponseCompletion, ResponseWriter, TransportRequest,
};

/// Runtime-neutral raw HTTP execution for `!Send` local futures.
pub trait LocalAsyncRawHttpExecutor {
    /// Executor-specific phased failure.
    type Error;

    /// Stages exactly one response without requiring a `Send` future.
    fn execute_local<'executor, 'request, 'policy, 'writer, 'buffer>(
        &'executor self,
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

/// Drives one raw local attempt and commits only after `Ready(Ok)`.
pub async fn drive_local_raw<'executor, 'request, 'policy, 'writer, 'buffer, T>(
    executor: &'executor T,
    request: TransportRequest<'request>,
    policy: RawResponsePolicy<'policy>,
    response: &'writer mut ResponseWriter<'buffer>,
) -> Result<(), AsyncExecutionError<T::Error>>
where
    T: LocalAsyncRawHttpExecutor + ?Sized,
    'executor: 'writer,
    'request: 'writer,
    'policy: 'writer,
    'buffer: 'writer,
{
    let mut attempt = response
        .begin_attempt()
        .map_err(AsyncExecutionError::Response)?;
    let completion = executor
        .execute_local(request, policy, attempt.staging())
        .await
        .map_err(AsyncExecutionError::Transport)?;
    attempt
        .commit_completion(completion)
        .map_err(AsyncExecutionError::Response)
}

impl<T> LocalAsyncRawHttpExecutor for T
where
    T: AsyncRawHttpExecutor + ?Sized,
{
    type Error = T::Error;

    async fn execute_local<'executor, 'request, 'policy, 'writer, 'buffer>(
        &'executor self,
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
        AsyncRawHttpExecutor::execute(self, request, policy, response).await
    }
}
