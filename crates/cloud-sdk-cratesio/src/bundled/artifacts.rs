use super::{BundledBuildError, RequestTimeouts};
use crate::{downloads::*, endpoint::OfficialCratesIoEndpoint, wire::IdentifyingUserAgent};
use cloud_sdk::transport::{
    BoundTransport, BoundUserAgent, EndpointIdentity, EndpointIdentityError, MAX_STREAM_BYTES,
    StatusCode, TransportRequest,
};

/// Separate, anonymous static-CDN transport. No API credential can be installed.
/// Reads retain only bounded HTTP frames, never the complete archive.
pub struct ArtifactTransport<T> {
    inner: T,
    maximum: u64,
}
impl<T> core::fmt::Debug for ArtifactTransport<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("ArtifactTransport([redacted])")
    }
}
impl<T: BoundTransport> BoundTransport for ArtifactTransport<T> {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.inner.endpoint_identity()
    }
}
impl<T: BoundUserAgent> BoundUserAgent for ArtifactTransport<T> {
    fn configured_user_agent(&self) -> &[u8] {
        self.inner.configured_user_agent()
    }
}
fn bound(maximum: u64) -> Result<(), BundledBuildError> {
    if maximum == 0 || maximum > MAX_STREAM_BYTES {
        Err(BundledBuildError)
    } else {
        Ok(())
    }
}

#[cfg(feature = "blocking-rustls")]
impl ArtifactTransport<cloud_sdk_reqwest::blocking::RawBlockingClient> {
    /// Constructs a credential-free static.crates.io blocking transport with an
    /// explicit finite body bound and a deadline covering head plus body reads.
    pub fn blocking(
        identity: IdentifyingUserAgent<'_>,
        timeouts: RequestTimeouts,
        maximum: u64,
    ) -> Result<Self, BundledBuildError> {
        bound(maximum)?;
        Ok(Self {
            inner: super::build_blocking(
                OfficialCratesIoEndpoint::static_downloads(),
                identity,
                timeouts,
            )?,
            maximum,
        })
    }
}
#[cfg(feature = "blocking-rustls")]
impl BlockingArtifactTransport
    for ArtifactTransport<cloud_sdk_reqwest::blocking::RawBlockingClient>
{
    type Source<'a> = cloud_sdk_reqwest::blocking::BlockingStreamingResponse;
    fn open(
        &self,
        request: TransportRequest<'_>,
    ) -> Result<OpenedArtifact<Self::Source<'_>>, ArtifactError> {
        let source = self
            .inner
            .open_stream(request, self.maximum)
            .map_err(|_| ArtifactError::Transport)?;
        let length = source.content_length();
        OpenedArtifact::new(source, StatusCode::OK, length, None)
    }
}

#[cfg(feature = "async-rustls")]
impl ArtifactTransport<cloud_sdk_reqwest::asynchronous::RawAsyncClient> {
    /// Constructs a credential-free static.crates.io async transport. Poll only
    /// from Tokio; cancellation closes the unpooled exchange.
    pub fn asynchronous(
        identity: IdentifyingUserAgent<'_>,
        timeouts: RequestTimeouts,
        maximum: u64,
    ) -> Result<Self, BundledBuildError> {
        bound(maximum)?;
        Ok(Self {
            inner: super::build_async(
                OfficialCratesIoEndpoint::static_downloads(),
                identity,
                timeouts,
            )?,
            maximum,
        })
    }
}
#[cfg(feature = "async-rustls")]
impl AsyncArtifactTransport for ArtifactTransport<cloud_sdk_reqwest::asynchronous::RawAsyncClient> {
    type Source<'a> = cloud_sdk_reqwest::asynchronous::StreamingResponse;
    async fn open_async<'a, 'r>(
        &'a self,
        request: TransportRequest<'r>,
    ) -> Result<OpenedArtifact<Self::Source<'a>>, ArtifactError>
    where
        'a: 'r,
    {
        let source = self
            .inner
            .open_stream(request, self.maximum)
            .await
            .map_err(|_| ArtifactError::Transport)?;
        let length = source.content_length();
        OpenedArtifact::new(source, StatusCode::OK, length, None)
    }
}
#[cfg(feature = "async-rustls")]
impl LocalArtifactTransport for ArtifactTransport<cloud_sdk_reqwest::asynchronous::RawAsyncClient> {
    type Source<'a> = cloud_sdk_reqwest::asynchronous::StreamingResponse;
    async fn open_local<'a, 'r>(
        &'a self,
        request: TransportRequest<'r>,
    ) -> Result<OpenedArtifact<Self::Source<'a>>, ArtifactError>
    where
        'a: 'r,
    {
        self.open_async(request).await
    }
}
