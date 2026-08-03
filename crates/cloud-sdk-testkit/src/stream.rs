//! Deterministic bounded streaming sources and sinks.

use cloud_sdk::buffer::sanitize_bytes;
use cloud_sdk::transport::{
    AsyncStreamSink, AsyncStreamSource, BlockingStreamSink, BlockingStreamSource,
    StreamPartialState, StreamRead, StreamReplayability,
};

/// Maximum chunks in one borrowed stream fixture.
pub const MAX_STREAM_FIXTURE_CHUNKS: usize = 1_024;

/// Invalid fixture or deterministic fixture I/O failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamFixtureError {
    /// The fixture contains too many chunks.
    TooManyChunks,
    /// One source chunk exceeds supplied scratch storage.
    SourceScratchTooSmall,
    /// Sink storage cannot accept the next deterministic write.
    SinkStorageTooSmall,
    /// The configured maximum sink write is zero.
    ZeroWriteLimit,
}

impl_static_error!(StreamFixtureError,
    Self::TooManyChunks => "stream fixture contains too many chunks",
    Self::SourceScratchTooSmall => "stream fixture scratch storage is too small",
    Self::SinkStorageTooSmall => "stream fixture sink storage is too small",
    Self::ZeroWriteLimit => "stream fixture sink write limit is zero",
);

/// Borrowed ordered chunks for one deterministic finite source.
pub struct StreamFixtureSource<'fixture> {
    chunks: &'fixture [&'fixture [u8]],
    index: usize,
    observations: usize,
    replayability: StreamReplayability<'fixture>,
}

impl<'fixture> StreamFixtureSource<'fixture> {
    /// Creates one bounded source. Empty borrowed slices represent explicit
    /// empty chunks rather than end-of-stream.
    pub const fn new(chunks: &'fixture [&'fixture [u8]]) -> Result<Self, StreamFixtureError> {
        if chunks.len() > MAX_STREAM_FIXTURE_CHUNKS {
            return Err(StreamFixtureError::TooManyChunks);
        }
        Ok(Self {
            chunks,
            index: 0,
            observations: 0,
            replayability: StreamReplayability::NotReplayable,
        })
    }

    /// Creates a bounded source with an explicit replay capability.
    pub const fn with_replayability(
        chunks: &'fixture [&'fixture [u8]],
        replayability: StreamReplayability<'fixture>,
    ) -> Result<Self, StreamFixtureError> {
        if chunks.len() > MAX_STREAM_FIXTURE_CHUNKS {
            return Err(StreamFixtureError::TooManyChunks);
        }
        Ok(Self {
            chunks,
            index: 0,
            observations: 0,
            replayability,
        })
    }

    /// Returns source observations including the final end marker.
    #[must_use]
    pub const fn observations(&self) -> usize {
        self.observations
    }

    fn read(&mut self, output: &mut [u8]) -> Result<StreamRead, StreamFixtureError> {
        self.observations = self.observations.saturating_add(1);
        let Some(chunk) = self.chunks.get(self.index) else {
            return Ok(StreamRead::End);
        };
        let target = output
            .get_mut(..chunk.len())
            .ok_or(StreamFixtureError::SourceScratchTooSmall)?;
        target.copy_from_slice(chunk);
        self.index = self.index.saturating_add(1);
        Ok(StreamRead::Chunk(chunk.len()))
    }
}

impl BlockingStreamSource for StreamFixtureSource<'_> {
    type Error = StreamFixtureError;

    fn replayability(&self) -> StreamReplayability<'_> {
        self.replayability
    }

    fn read_chunk(&mut self, output: &mut [u8]) -> Result<StreamRead, Self::Error> {
        self.read(output)
    }
}

impl AsyncStreamSource for StreamFixtureSource<'_> {
    type Error = StreamFixtureError;

    fn replayability(&self) -> StreamReplayability<'_> {
        self.replayability
    }

    async fn read_chunk<'operation>(
        &'operation mut self,
        output: &'operation mut [u8],
    ) -> Result<StreamRead, Self::Error> {
        self.read(output)
    }
}

/// Caller-buffered deterministic sink with configurable short writes.
pub struct StreamFixtureSink<'storage> {
    output: &'storage mut [u8],
    initialized_len: usize,
    max_write_bytes: usize,
    writes: usize,
    committed: bool,
    aborted: Option<StreamPartialState>,
}

impl<'storage> StreamFixtureSink<'storage> {
    /// Creates a sink and clears all caller storage before first use.
    pub fn new(
        output: &'storage mut [u8],
        max_write_bytes: usize,
    ) -> Result<Self, StreamFixtureError> {
        sanitize_bytes(output);
        if max_write_bytes == 0 {
            return Err(StreamFixtureError::ZeroWriteLimit);
        }
        Ok(Self {
            output,
            initialized_len: 0,
            max_write_bytes,
            writes: 0,
            committed: false,
            aborted: None,
        })
    }

    /// Returns initialized sink bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        self.output.get(..self.initialized_len).unwrap_or_default()
    }

    /// Returns deterministic sink write count.
    #[must_use]
    pub const fn writes(&self) -> usize {
        self.writes
    }

    /// Reports whether the successful stream was committed.
    #[must_use]
    pub const fn is_committed(&self) -> bool {
        self.committed
    }

    /// Returns the last incomplete-state abort classification.
    #[must_use]
    pub const fn aborted_with(&self) -> Option<StreamPartialState> {
        self.aborted
    }

    fn write(&mut self, input: &[u8]) -> Result<usize, StreamFixtureError> {
        let accepted = core::cmp::min(input.len(), self.max_write_bytes);
        let end = self
            .initialized_len
            .checked_add(accepted)
            .ok_or(StreamFixtureError::SinkStorageTooSmall)?;
        let target = self
            .output
            .get_mut(self.initialized_len..end)
            .ok_or(StreamFixtureError::SinkStorageTooSmall)?;
        let source = input
            .get(..accepted)
            .ok_or(StreamFixtureError::SinkStorageTooSmall)?;
        target.copy_from_slice(source);
        self.initialized_len = end;
        self.writes = self.writes.saturating_add(1);
        Ok(accepted)
    }

    fn commit_inner(&mut self) {
        self.committed = true;
    }

    fn abort_inner(&mut self, partial: StreamPartialState) {
        self.aborted = Some(partial);
        if matches!(partial, StreamPartialState::RollbackRequired) {
            sanitize_bytes(self.output);
            self.initialized_len = 0;
        }
    }
}

impl BlockingStreamSink for StreamFixtureSink<'_> {
    type Error = StreamFixtureError;

    fn write_chunk(&mut self, input: &[u8]) -> Result<usize, Self::Error> {
        self.write(input)
    }

    fn commit(&mut self) -> Result<(), Self::Error> {
        self.commit_inner();
        Ok(())
    }

    fn abort(&mut self, partial: StreamPartialState) {
        self.abort_inner(partial);
    }
}

impl AsyncStreamSink for StreamFixtureSink<'_> {
    type Error = StreamFixtureError;

    async fn write_chunk<'operation>(
        &'operation mut self,
        input: &'operation [u8],
    ) -> Result<usize, Self::Error> {
        self.write(input)
    }

    async fn commit(&mut self) -> Result<(), Self::Error> {
        self.commit_inner();
        Ok(())
    }

    fn abort(&mut self, partial: StreamPartialState) {
        self.abort_inner(partial);
    }
}
