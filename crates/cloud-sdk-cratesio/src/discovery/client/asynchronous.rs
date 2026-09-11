use super::*;
use cloud_sdk::transport::{
    AsyncExecutionError, AsyncRawHttpExecutor, LocalAsyncRawHttpExecutor, drive_async_raw,
    drive_local_raw,
};
use core::future::Future;

impl<T: BoundTransport + BoundUserAgent + LocalAsyncRawHttpExecutor + ?Sized>
    DiscoveryClient<'_, T>
{
    /// Executes one anonymous GET with a possibly non-Send executor.
    /// Admission starts on first poll; dropping even an unpolled future clears
    /// response storage. Cancellation imposes a fresh quiet interval.
    pub fn execute_local<'s, 'r: 's, 'b: 's>(
        &'s self,
        request: DiscoveryRequest<'r>,
        storage: &'b mut [u8],
        header_storage: &'b mut [u8],
    ) -> impl Future<Output = Result<DiscoveryResponse, DiscoveryExecutionError<T::Error>>> + 's
    {
        self.execute_get_local(request, storage, header_storage)
    }

    pub(crate) fn execute_get_local<'s, 'r: 's, 'b: 's, R: CheckedGet + 'r>(
        &'s self,
        request: R,
        storage: &'b mut [u8],
        header_storage: &'b mut [u8],
    ) -> impl Future<Output = Result<R::Response, DiscoveryExecutionError<T::Error>>> + 's {
        let mut response = ResponseBuffer::new(storage, self.maximum, header_storage);
        async move {
            request
                .anonymous()
                .map_err(DiscoveryExecutionError::Model)?;
            self.verify().map_err(DiscoveryExecutionError::Model)?;
            let mut target = [0; MAX_TARGET_BYTES];
            let target = request
                .target(&mut target)
                .map_err(|_| DiscoveryExecutionError::Model(DiscoveryError::Binding))?;
            let headers = self.headers().map_err(DiscoveryExecutionError::Model)?;
            let wire = TransportRequest::new(Method::Get, target.as_request_target()).with_headers(
                RequestHeaders::new(&headers)
                    .map_err(|_| DiscoveryExecutionError::Model(DiscoveryError::Value))?,
            );
            let policy = self.policy().map_err(DiscoveryExecutionError::Model)?;
            let mut attempt = self
                .gate
                .begin()
                .map_err(DiscoveryExecutionError::Schedule)?;
            drive_local_raw(self.executor, wire, policy, response.writer())
                .await
                .map_err(|e| match e {
                    AsyncExecutionError::Transport(e) => DiscoveryExecutionError::Transport(e),
                    AsyncExecutionError::Response(_) => DiscoveryExecutionError::Staging,
                })?;
            self.decode(request, response, &mut attempt)
        }
    }
}

impl<T: BoundTransport + BoundUserAgent + AsyncRawHttpExecutor + Sync + ?Sized>
    DiscoveryClient<'_, T>
{
    /// Cross-thread execution with the same admission, response and cleanup path.
    pub fn execute_async<'s, 'r: 's, 'b: 's>(
        &'s self,
        request: DiscoveryRequest<'r>,
        storage: &'b mut [u8],
        header_storage: &'b mut [u8],
    ) -> impl Future<Output = Result<DiscoveryResponse, DiscoveryExecutionError<T::Error>>> + Send + 's
    {
        self.execute_get_async(request, storage, header_storage)
    }

    pub(crate) fn execute_get_async<'s, 'r: 's, 'b: 's, R: CheckedGet + Send + 'r>(
        &'s self,
        request: R,
        storage: &'b mut [u8],
        header_storage: &'b mut [u8],
    ) -> impl Future<Output = Result<R::Response, DiscoveryExecutionError<T::Error>>> + Send + 's
    {
        let mut response = ResponseBuffer::new(storage, self.maximum, header_storage);
        async move {
            request
                .anonymous()
                .map_err(DiscoveryExecutionError::Model)?;
            self.verify().map_err(DiscoveryExecutionError::Model)?;
            let mut target = [0; MAX_TARGET_BYTES];
            let target = request
                .target(&mut target)
                .map_err(|_| DiscoveryExecutionError::Model(DiscoveryError::Binding))?;
            let headers = self.headers().map_err(DiscoveryExecutionError::Model)?;
            let wire = TransportRequest::new(Method::Get, target.as_request_target()).with_headers(
                RequestHeaders::new(&headers)
                    .map_err(|_| DiscoveryExecutionError::Model(DiscoveryError::Value))?,
            );
            let policy = self.policy().map_err(DiscoveryExecutionError::Model)?;
            let mut attempt = self
                .gate
                .begin()
                .map_err(DiscoveryExecutionError::Schedule)?;
            drive_async_raw(self.executor, wire, policy, response.writer())
                .await
                .map_err(|e| match e {
                    AsyncExecutionError::Transport(e) => DiscoveryExecutionError::Transport(e),
                    AsyncExecutionError::Response(_) => DiscoveryExecutionError::Staging,
                })?;
            self.decode(request, response, &mut attempt)
        }
    }
}
