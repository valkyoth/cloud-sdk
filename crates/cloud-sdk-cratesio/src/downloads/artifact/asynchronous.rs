use super::*;
use cloud_sdk::transport::{
    AsyncStreamSink, AsyncStreamSource, LocalAsyncStreamSink, LocalAsyncStreamSource,
    drive_async_stream, drive_local_stream,
};
use core::future::Future;

/// Trusted local anonymous streaming adapter; follows the same
/// origin, header, TLS, deadline and no-credential contract as BlockingArtifactTransport.
pub trait LocalArtifactTransport: BoundTransport + BoundUserAgent {
    /// Live body source tied to the transport.
    type Source<'a>: LocalAsyncStreamSource
    where
        Self: 'a;
    /// Opens one SDK-owned GET without retaining request storage.
    fn open_local<'a, 'r>(
        &'a self,
        request: TransportRequest<'r>,
    ) -> impl Future<Output = Result<OpenedArtifact<Self::Source<'a>>, ArtifactError>> + 'r
    where
        'a: 'r;
}
impl<'a> ArtifactDownload<'a> {
    /// One local finite transfer; even unpolled cancellation aborts the
    /// sink and clears scratch. The caller supplies a transactional sink.
    pub fn execute_local<'s, T, D, C>(
        self,
        transport: &'s T,
        identity: IdentifyingUserAgent<'s>,
        sink: &'s mut D,
        checksum: C,
        scratch: &'s mut [u8],
    ) -> impl Future<Output = Result<StreamCompletion, ArtifactError>> + 's
    where
        'a: 's,
        T: LocalArtifactTransport,
        D: LocalAsyncStreamSink,
        C: ArtifactChecksum + 's,
    {
        let mut sink = LocalSink {
            sink,
            checksum,
            expected: self.sha256,
            armed: true,
        };
        cloud_sdk::buffer::sanitize_bytes(scratch);
        let scratch = Scratch(scratch);
        async move {
            if scratch.0.is_empty() {
                return Err(ArtifactError::Policy);
            }
            OfficialCratesIoEndpoint::static_downloads()
                .verify_transport(transport)
                .map_err(|_| ArtifactError::Binding)?;
            if transport.configured_user_agent() != identity.as_str().as_bytes() {
                return Err(ArtifactError::Binding);
            }
            let mut target = [0; crate::query::MAX_TARGET_BYTES];
            let target = self.write_target(&mut target)?;
            let mut opened = transport
                .open_local(TransportRequest::new(
                    Method::Get,
                    target.as_request_target(),
                ))
                .await?;
            if opened.length.is_some_and(|len| len != self.expected_bytes) {
                return Err(ArtifactError::Headers);
            }
            drive_local_stream(
                self.policy,
                &mut opened.source,
                &mut sink,
                scratch.0,
                &mut StreamOutcome::new(),
            )
            .await
            .map_err(stream_error)
        }
    }
}
struct LocalSink<'a, D: LocalAsyncStreamSink, C> {
    sink: &'a mut D,
    checksum: C,
    expected: [u8; 32],
    armed: bool,
}
impl<D: LocalAsyncStreamSink, C: ArtifactChecksum> LocalAsyncStreamSink for LocalSink<'_, D, C> {
    type Error = ArtifactError;
    async fn write_chunk_local<'b>(&'b mut self, input: &'b [u8]) -> Result<usize, Self::Error> {
        let len = self
            .sink
            .write_chunk_local(input)
            .await
            .map_err(|_| ArtifactError::Stream)?;
        self.checksum
            .update(input.get(..len).ok_or(ArtifactError::Stream)?)?;
        Ok(len)
    }
    async fn commit_local(&mut self) -> Result<(), Self::Error> {
        if self.checksum.finish()? != self.expected {
            return Err(ArtifactError::Checksum);
        }
        self.sink
            .commit_local()
            .await
            .map_err(|_| ArtifactError::Stream)?;
        self.armed = false;
        Ok(())
    }
    fn abort_local(&mut self, partial: StreamPartialState) {
        if self.armed {
            self.sink.abort_local(partial);
            self.armed = false;
        }
    }
}
impl<D: LocalAsyncStreamSink, C> Drop for LocalSink<'_, D, C> {
    fn drop(&mut self) {
        if self.armed {
            self.sink.abort_local(StreamPartialState::RollbackRequired);
        }
    }
}
/// Trusted Send anonymous streaming adapter; follows the same
/// origin, header, TLS, deadline and no-credential contract as BlockingArtifactTransport.
pub trait AsyncArtifactTransport: BoundTransport + BoundUserAgent + Sync {
    /// Live body source tied to the transport.
    type Source<'a>: AsyncStreamSource + Send
    where
        Self: 'a;
    /// Opens one SDK-owned GET without retaining request storage.
    fn open_async<'a, 'r>(
        &'a self,
        request: TransportRequest<'r>,
    ) -> impl Future<Output = Result<OpenedArtifact<Self::Source<'a>>, ArtifactError>> + Send + 'r
    where
        'a: 'r;
}
impl<'a> ArtifactDownload<'a> {
    /// One Send finite transfer; even unpolled cancellation aborts the
    /// sink and clears scratch. The caller supplies a transactional sink.
    pub fn execute_async<'s, T, D, C>(
        self,
        transport: &'s T,
        identity: IdentifyingUserAgent<'s>,
        sink: &'s mut D,
        checksum: C,
        scratch: &'s mut [u8],
    ) -> impl Future<Output = Result<StreamCompletion, ArtifactError>> + Send + 's
    where
        'a: 's,
        T: AsyncArtifactTransport,
        D: AsyncStreamSink + Send,
        C: ArtifactChecksum + Send + 's,
    {
        let mut sink = SendSink {
            sink,
            checksum,
            expected: self.sha256,
            armed: true,
        };
        cloud_sdk::buffer::sanitize_bytes(scratch);
        let scratch = Scratch(scratch);
        async move {
            if scratch.0.is_empty() {
                return Err(ArtifactError::Policy);
            }
            OfficialCratesIoEndpoint::static_downloads()
                .verify_transport(transport)
                .map_err(|_| ArtifactError::Binding)?;
            if transport.configured_user_agent() != identity.as_str().as_bytes() {
                return Err(ArtifactError::Binding);
            }
            let mut target = [0; crate::query::MAX_TARGET_BYTES];
            let target = self.write_target(&mut target)?;
            let mut opened = transport
                .open_async(TransportRequest::new(
                    Method::Get,
                    target.as_request_target(),
                ))
                .await?;
            if opened.length.is_some_and(|len| len != self.expected_bytes) {
                return Err(ArtifactError::Headers);
            }
            drive_async_stream(
                self.policy,
                &mut opened.source,
                &mut sink,
                scratch.0,
                &mut StreamOutcome::new(),
            )
            .await
            .map_err(stream_error)
        }
    }
}
struct SendSink<'a, D: AsyncStreamSink, C> {
    sink: &'a mut D,
    checksum: C,
    expected: [u8; 32],
    armed: bool,
}
impl<D: AsyncStreamSink + Send, C: ArtifactChecksum + Send> AsyncStreamSink for SendSink<'_, D, C> {
    type Error = ArtifactError;
    async fn write_chunk<'b>(&'b mut self, input: &'b [u8]) -> Result<usize, Self::Error> {
        let len = self
            .sink
            .write_chunk(input)
            .await
            .map_err(|_| ArtifactError::Stream)?;
        self.checksum
            .update(input.get(..len).ok_or(ArtifactError::Stream)?)?;
        Ok(len)
    }
    async fn commit(&mut self) -> Result<(), Self::Error> {
        if self.checksum.finish()? != self.expected {
            return Err(ArtifactError::Checksum);
        }
        self.sink
            .commit()
            .await
            .map_err(|_| ArtifactError::Stream)?;
        self.armed = false;
        Ok(())
    }
    fn abort(&mut self, partial: StreamPartialState) {
        if self.armed {
            self.sink.abort(partial);
            self.armed = false;
        }
    }
}
impl<D: AsyncStreamSink, C> Drop for SendSink<'_, D, C> {
    fn drop(&mut self) {
        if self.armed {
            self.sink.abort(StreamPartialState::RollbackRequired);
        }
    }
}
