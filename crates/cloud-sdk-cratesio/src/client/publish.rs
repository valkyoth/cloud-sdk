use super::{RegistryBuffers, RegistryClient, buffers::Guard};
use crate::{
    discovery::DiscoveryExecutionError,
    publishing::{PublishBuffers, PublishClient, PublishPermit, PublishResponse},
};
use cloud_sdk::transport::{BlockingRawUploadExecutor, BlockingStreamSource, BoundUserAgent};

impl<T: BlockingRawUploadExecutor + BoundUserAgent + ?Sized> RegistryClient<'_, T> {
    /// Consumes one publish permit and streams an already packaged archive.
    /// `buffers.body` is bounded upload scratch, not whole-archive storage.
    /// The caller must not replay an ambiguous publication or block indefinitely
    /// inside its source. No caller HTTP assembly or trusted callback is required.
    pub fn publish<S: BlockingStreamSource>(
        &self,
        permit: PublishPermit<'_>,
        source: &mut S,
        buffers: RegistryBuffers<'_>,
    ) -> Result<PublishResponse, DiscoveryExecutionError<T::Error>> {
        let mut guard = Guard::new(buffers);
        let parts = guard.parts();
        let client = if self.staging {
            PublishClient::staging(self.executor, self.identity, self.maximum)
        } else {
            PublishClient::production(self.executor, self.identity, self.maximum)
        }
        .map_err(DiscoveryExecutionError::Model)?;
        client.execute_bundled(
            permit,
            source,
            PublishBuffers {
                credential: parts.credential,
                response: parts.response,
                headers: parts.headers,
            },
            parts.body,
        )
    }
}
