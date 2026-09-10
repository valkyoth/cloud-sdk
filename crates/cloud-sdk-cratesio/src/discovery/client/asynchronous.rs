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
        let mut response = ResponseBuffer::new(storage, self.maximum, header_storage);
        async move {
            self.verify().map_err(DiscoveryExecutionError::Model)?;
            let mut target = [0; MAX_TARGET_BYTES];
            let target = request
                .write_target(&mut target)
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
        let mut response = ResponseBuffer::new(storage, self.maximum, header_storage);
        async move {
            self.verify().map_err(DiscoveryExecutionError::Model)?;
            let mut target = [0; MAX_TARGET_BYTES];
            let target = request
                .write_target(&mut target)
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
