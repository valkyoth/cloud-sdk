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
    /// Fault observation indices are one-based.
    ZeroFaultIndex,
    /// The source reached its configured injected failure.
    InjectedSourceFault,
    /// The sink reached its configured injected failure.
    InjectedSinkFault,
    /// Alternating stream patterns require a nonempty data chunk.
    EmptyPatternData,
}

impl_static_error!(StreamFixtureError,
    Self::TooManyChunks => "stream fixture contains too many chunks",
    Self::SourceScratchTooSmall => "stream fixture scratch storage is too small",
    Self::SinkStorageTooSmall => "stream fixture sink storage is too small",
    Self::ZeroWriteLimit => "stream fixture sink write limit is zero",
    Self::ZeroFaultIndex => "stream fixture fault index must be nonzero",
    Self::InjectedSourceFault => "stream fixture injected a source failure",
    Self::InjectedSinkFault => "stream fixture injected a sink failure",
    Self::EmptyPatternData => "alternating stream pattern data is empty",
);

/// Borrowed ordered chunks for one deterministic finite source.
pub struct StreamFixtureSource<'fixture> {
    chunks: &'fixture [&'fixture [u8]],
    index: usize,
    observations: usize,
    replayability: StreamReplayability<'fixture>,
    fault_at_observation: Option<usize>,
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
            fault_at_observation: None,
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
            fault_at_observation: None,
        })
    }

    /// Injects a source failure at one one-based read observation.
    pub const fn with_fault_at_observation(
        mut self,
        observation: usize,
    ) -> Result<Self, StreamFixtureError> {
        if observation == 0 {
            return Err(StreamFixtureError::ZeroFaultIndex);
        }
        self.fault_at_observation = Some(observation);
        Ok(self)
    }

    /// Returns source observations including the final end marker.
    #[must_use]
    pub const fn observations(&self) -> usize {
        self.observations
    }

    fn read(&mut self, output: &mut [u8]) -> Result<StreamRead, StreamFixtureError> {
        self.observations = self.observations.saturating_add(1);
        if self.fault_at_observation == Some(self.observations) {
            return Err(StreamFixtureError::InjectedSourceFault);
        }
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

/// Non-terminating deterministic source pattern for hard-limit tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamPattern<'fixture> {
    /// Every observation is an explicit empty chunk.
    EndlessEmpty,
    /// Observations alternate between an empty chunk and borrowed data.
    AlternatingEmptyData(&'fixture [u8]),
}

/// Non-terminating stream source used to verify cancellation and hard bounds.
pub struct StreamPatternSource<'fixture> {
    pattern: StreamPattern<'fixture>,
    observations: usize,
}

impl<'fixture> StreamPatternSource<'fixture> {
    /// Creates a deterministic non-terminating source pattern.
    pub const fn new(pattern: StreamPattern<'fixture>) -> Result<Self, StreamFixtureError> {
        if matches!(pattern, StreamPattern::AlternatingEmptyData(data) if data.is_empty()) {
            return Err(StreamFixtureError::EmptyPatternData);
        }
        Ok(Self {
            pattern,
            observations: 0,
        })
    }

    /// Returns the number of source observations.
    #[must_use]
    pub const fn observations(&self) -> usize {
        self.observations
    }

    fn read(&mut self, output: &mut [u8]) -> Result<StreamRead, StreamFixtureError> {
        self.observations = self.observations.saturating_add(1);
        match self.pattern {
            StreamPattern::EndlessEmpty => Ok(StreamRead::Chunk(0)),
            StreamPattern::AlternatingEmptyData(_) if self.observations % 2 == 1 => {
                Ok(StreamRead::Chunk(0))
            }
            StreamPattern::AlternatingEmptyData(data) => {
                let target = output
                    .get_mut(..data.len())
                    .ok_or(StreamFixtureError::SourceScratchTooSmall)?;
                target.copy_from_slice(data);
                Ok(StreamRead::Chunk(data.len()))
            }
        }
    }
}

impl BlockingStreamSource for StreamPatternSource<'_> {
    type Error = StreamFixtureError;

    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }

    fn read_chunk(&mut self, output: &mut [u8]) -> Result<StreamRead, Self::Error> {
        self.read(output)
    }
}

impl AsyncStreamSource for StreamPatternSource<'_> {
    type Error = StreamFixtureError;

    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }

    async fn read_chunk<'operation>(
        &'operation mut self,
        output: &'operation mut [u8],
    ) -> Result<StreamRead, Self::Error> {
        self.read(output)
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
    fault_at_write: Option<usize>,
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
            fault_at_write: None,
        })
    }

    /// Injects a sink failure at one one-based write attempt.
    pub const fn with_fault_at_write(mut self, write: usize) -> Result<Self, StreamFixtureError> {
        if write == 0 {
            return Err(StreamFixtureError::ZeroFaultIndex);
        }
        self.fault_at_write = Some(write);
        Ok(self)
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
        let next_write = self.writes.saturating_add(1);
        if self.fault_at_write == Some(next_write) {
            return Err(StreamFixtureError::InjectedSinkFault);
        }
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
