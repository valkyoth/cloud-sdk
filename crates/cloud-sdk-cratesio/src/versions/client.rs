use super::{VersionError, VersionExecutionError, VersionRequest, VersionResponse};
use crate::{discovery::DiscoveryClient, wire::IdentifyingUserAgent};
use cloud_sdk::transport::{BoundTransport, BoundUserAgent};

/// Official-origin version client using the same admission and cleanup runner
/// as taxonomy discovery. No retries, redirects, sleeps or bulk enumeration.
pub struct VersionClient<'a, T: ?Sized>(pub(super) DiscoveryClient<'a, T>);
impl<T: ?Sized> core::fmt::Debug for VersionClient<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("VersionClient([redacted])")
    }
}
impl<'a, T: BoundTransport + BoundUserAgent + ?Sized> VersionClient<'a, T> {
    /// Fixed production API with an identifying user-agent and response cap.
    pub fn production(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, VersionError> {
        DiscoveryClient::production(executor, identity, maximum).map(Self)
    }
    /// Fixed staging API; never reuses production credentials implicitly.
    pub fn staging(
        executor: &'a T,
        identity: IdentifyingUserAgent<'a>,
        maximum: usize,
    ) -> Result<Self, VersionError> {
        DiscoveryClient::staging(executor, identity, maximum).map(Self)
    }
}
#[cfg(feature = "blocking")]
impl<T: BoundTransport + BoundUserAgent + cloud_sdk::transport::BlockingRawHttpExecutor + ?Sized>
    VersionClient<'_, T>
{
    /// One anonymous checked request. No credentials or automatic redirects.
    pub fn execute(
        &self,
        request: VersionRequest<'_>,
        storage: &mut [u8],
        headers: &mut [u8],
    ) -> Result<VersionResponse, VersionExecutionError<T::Error>> {
        self.0.execute_get(request, storage, headers)
    }
}
#[cfg(feature = "async")]
impl<T: BoundTransport + BoundUserAgent + cloud_sdk::transport::LocalAsyncRawHttpExecutor + ?Sized>
    VersionClient<'_, T>
{
    /// One anonymous local async request; unpolled/cancelled futures clear storage.
    pub fn execute_local<'s, 'r: 's, 'b: 's>(
        &'s self,
        request: VersionRequest<'r>,
        storage: &'b mut [u8],
        headers: &'b mut [u8],
    ) -> impl core::future::Future<Output = Result<VersionResponse, VersionExecutionError<T::Error>>> + 's
    {
        self.0.execute_get_local(request, storage, headers)
    }
}
#[cfg(feature = "async")]
impl<
    T: BoundTransport + BoundUserAgent + cloud_sdk::transport::AsyncRawHttpExecutor + Sync + ?Sized,
> VersionClient<'_, T>
{
    /// One anonymous Send request with identical wire, model and rate policies.
    pub fn execute_async<'s, 'r: 's, 'b: 's>(
        &'s self,
        request: VersionRequest<'r>,
        storage: &'b mut [u8],
        headers: &'b mut [u8],
    ) -> impl core::future::Future<Output = Result<VersionResponse, VersionExecutionError<T::Error>>>
    + Send
    + 's {
        self.0.execute_get_async(request, storage, headers)
    }
}

#[cfg(all(test, feature = "blocking", feature = "async"))]
mod tests;
