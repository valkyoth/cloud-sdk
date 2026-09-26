use super::{RegistryBuffers, RegistryClient, buffers::Guard};
use crate::{
    discovery::DiscoveryExecutionError as Failure,
    publishing::{PublishBuffers, PublishClient, PublishPermit, PublishResponse},
};
use cloud_sdk::transport::{AsyncStreamSource, LocalAsyncStreamSource};
use cloud_sdk_reqwest::asynchronous::{RawAsyncClient, RawTransportFailure};
use core::future::Future;

macro_rules! publish {
    ($name:ident, $execute:ident, $source:path $(, $send:ident)?) => {
        /// Streams one explicitly permitted publication without HTTP assembly.
        /// `buffers.body` is upload scratch; all regions clear even if the future
        /// is never polled. No retry or implicit archive packaging occurs.
        pub fn $name<'s, S: $source $(+ $send)? + 's>(&'s self, permit: PublishPermit<'s>, source: &'s mut S, buffers: RegistryBuffers<'s>) -> impl Future<Output = Result<PublishResponse, Failure<RawTransportFailure>>> $(+ $send)? + 's {
            let mut guard = Guard::new(buffers);
            async move {
                let parts = guard.parts();
                let client = if self.staging {
                    PublishClient::staging(self.executor, self.identity, self.maximum)
                } else {
                    PublishClient::production(self.executor, self.identity, self.maximum)
                }.map_err(Failure::Model)?;
                client.$execute(permit, source, PublishBuffers { credential: parts.credential, response: parts.response, headers: parts.headers }, parts.body).await
            }
        }
    }
}
impl RegistryClient<'_, RawAsyncClient> {
    publish!(
        publish_async,
        execute_bundled_async,
        AsyncStreamSource,
        Send
    );
    publish!(publish_local, execute_bundled_local, LocalAsyncStreamSource);
}
