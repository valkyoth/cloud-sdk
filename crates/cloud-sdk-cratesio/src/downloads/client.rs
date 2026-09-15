use super::{DownloadError, DownloadExecutionError, DownloadRequest, DownloadResponse};
use crate::{discovery::DiscoveryClient, wire::IdentifyingUserAgent};
use cloud_sdk::transport::{BoundTransport, BoundUserAgent};

#[cfg(all(test, feature = "blocking", feature = "async"))]
mod tests;

/// Official-origin download client using the same admission and cleanup runner
/// as taxonomy discovery. No retries, redirects, sleeps or bulk enumeration.
pub struct DownloadClient<'a, T: ?Sized>(pub(super) DiscoveryClient<'a, T>);
impl<T: ?Sized> core::fmt::Debug for DownloadClient<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("DownloadClient([redacted])")
    }
}
impl<'a, T: BoundTransport + BoundUserAgent + ?Sized> DownloadClient<'a, T> {
    /// Fixed production API with an identifying user-agent and response cap.
    pub fn production(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, DownloadError> {
        DiscoveryClient::production(executor, identity, maximum).map(Self)
    }
    /// Fixed staging API; never reuses production credentials implicitly.
    pub fn staging(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, DownloadError> {
        DiscoveryClient::staging(executor, identity, maximum).map(Self)
    }
}
#[cfg(feature = "blocking")]
impl<T: BoundTransport + BoundUserAgent + cloud_sdk::transport::BlockingRawHttpExecutor + ?Sized>
    DownloadClient<'_, T>
{
    /// One anonymous checked request. No credentials or automatic redirects.
    pub fn execute(
        &self,
        request: DownloadRequest<'_>,
        storage: &mut [u8],
        headers: &mut [u8],
    ) -> Result<DownloadResponse, DownloadExecutionError<T::Error>> {
        self.0.execute_get(request, storage, headers)
    }
}
#[cfg(feature = "async")]
impl<T: BoundTransport + BoundUserAgent + cloud_sdk::transport::LocalAsyncRawHttpExecutor + ?Sized>
    DownloadClient<'_, T>
{
    /// One anonymous local async request; unpolled/cancelled futures clear storage.
    pub fn execute_local<'s, 'r: 's, 'b: 's>(
        &'s self,
        request: DownloadRequest<'r>,
        storage: &'b mut [u8],
        headers: &'b mut [u8],
    ) -> impl core::future::Future<
        Output = Result<DownloadResponse, DownloadExecutionError<T::Error>>,
    > + 's {
        self.0.execute_get_local(request, storage, headers)
    }
}
#[cfg(feature = "async")]
impl<
    T: BoundTransport + BoundUserAgent + cloud_sdk::transport::AsyncRawHttpExecutor + Sync + ?Sized,
> DownloadClient<'_, T>
{
    /// One anonymous Send request with identical wire, model and rate policies.
    pub fn execute_async<'s, 'r: 's, 'b: 's>(
        &'s self,
        request: DownloadRequest<'r>,
        storage: &'b mut [u8],
        headers: &'b mut [u8],
    ) -> impl core::future::Future<
        Output = Result<DownloadResponse, DownloadExecutionError<T::Error>>,
    > + Send
    + 's {
        self.0.execute_get_async(request, storage, headers)
    }
}
