use crate::shared::{RawHttpError, RawTransportFailure, StreamingResponse};
use cloud_sdk::transport::{
    AsyncStreamSource, BlockingStreamSource, StreamRead, StreamReplayability,
};

/// Live unpooled response with its private blocking executor. Dropping the
/// source closes unfinished I/O. Never use it from an active Tokio runtime.
pub struct BlockingStreamingResponse {
    pub(super) response: StreamingResponse,
    pub(super) runtime: Option<tokio::runtime::Runtime>,
}
impl BlockingStreamingResponse {
    /// Optional validated Content-Length; EOF also checks this declaration.
    #[must_use]
    pub const fn content_length(&self) -> Option<u64> {
        self.response.content_length()
    }
}
impl core::fmt::Debug for BlockingStreamingResponse {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("BlockingStreamingResponse([redacted])")
    }
}
impl BlockingStreamSource for BlockingStreamingResponse {
    type Error = RawTransportFailure;
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    fn read_chunk(&mut self, output: &mut [u8]) -> Result<StreamRead, Self::Error> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(cloud_sdk::transport::TransportFailure::response_started(
                RawHttpError::BlockingRuntimeContext,
            ));
        }
        let runtime = self.runtime.as_ref().ok_or_else(|| {
            cloud_sdk::transport::TransportFailure::response_started(RawHttpError::RequestFailed)
        })?;
        runtime.block_on(self.response.read_chunk(output))
    }
}
impl Drop for BlockingStreamingResponse {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}
