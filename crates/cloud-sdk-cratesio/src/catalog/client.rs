use super::{CatalogError, CatalogExecutionError, CatalogRequest, CatalogResponse};
use crate::{discovery::DiscoveryClient, wire::IdentifyingUserAgent};
use cloud_sdk::transport::{BoundTransport, BoundUserAgent};

/// Official-origin catalog client using the same admission and cleanup runner
/// as taxonomy discovery. No retries, redirects, sleeps or bulk enumeration.
pub struct CatalogClient<'a, T: ?Sized>(pub(super) DiscoveryClient<'a, T>);
impl<T: ?Sized> core::fmt::Debug for CatalogClient<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("CatalogClient([redacted])")
    }
}
impl<'a, T: BoundTransport + BoundUserAgent + ?Sized> CatalogClient<'a, T> {
    /// Fixed production API with an identifying user-agent and response cap.
    pub fn production(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, CatalogError> {
        DiscoveryClient::production(executor, identity, maximum).map(Self)
    }
    /// Fixed staging API; never reuses production credentials implicitly.
    pub fn staging(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, CatalogError> {
        DiscoveryClient::staging(executor, identity, maximum).map(Self)
    }
}
#[cfg(feature = "blocking")]
impl<T: BoundTransport + BoundUserAgent + cloud_sdk::transport::BlockingRawHttpExecutor + ?Sized>
    CatalogClient<'_, T>
{
    /// One anonymous checked request. Following-filter requests fail before I/O.
    pub fn execute(
        &self,
        request: CatalogRequest<'_>,
        storage: &mut [u8],
        headers: &mut [u8],
    ) -> Result<CatalogResponse, CatalogExecutionError<T::Error>> {
        self.0.execute_get(request, storage, headers)
    }
}
#[cfg(feature = "async")]
impl<T: BoundTransport + BoundUserAgent + cloud_sdk::transport::LocalAsyncRawHttpExecutor + ?Sized>
    CatalogClient<'_, T>
{
    /// One anonymous local async request; unpolled/cancelled futures clear storage.
    pub fn execute_local<'s, 'r: 's, 'b: 's>(
        &'s self,
        request: CatalogRequest<'r>,
        storage: &'b mut [u8],
        headers: &'b mut [u8],
    ) -> impl core::future::Future<Output = Result<CatalogResponse, CatalogExecutionError<T::Error>>> + 's
    {
        self.0.execute_get_local(request, storage, headers)
    }
}
#[cfg(feature = "async")]
impl<
    T: BoundTransport + BoundUserAgent + cloud_sdk::transport::AsyncRawHttpExecutor + Sync + ?Sized,
> CatalogClient<'_, T>
{
    /// One anonymous Send request with identical wire, model and rate policies.
    pub fn execute_async<'s, 'r: 's, 'b: 's>(
        &'s self,
        request: CatalogRequest<'r>,
        storage: &'b mut [u8],
        headers: &'b mut [u8],
    ) -> impl core::future::Future<Output = Result<CatalogResponse, CatalogExecutionError<T::Error>>>
    + Send
    + 's {
        self.0.execute_get_async(request, storage, headers)
    }
}

#[cfg(all(test, feature = "blocking", feature = "async"))]
mod tests;
#[cfg(feature = "blocking")]
mod token;
