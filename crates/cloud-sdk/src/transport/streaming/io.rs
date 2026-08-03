//! Caller-buffered blocking, Send-async, and local-async stream drivers.

mod asynchronous;
mod blocking;

pub use asynchronous::{drive_async_stream, drive_local_stream};
pub use blocking::drive_blocking_stream;

use cloud_sdk_sanitization::sanitize_bytes;
use core::{fmt, future::Future};

use super::policy::partial_state;
use super::{StreamPartialState, StreamPolicy, StreamProgressError, StreamReplayability};

/// One source observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamRead {
    /// One chunk was initialized in the supplied scratch prefix.
    Chunk(usize),
    /// The source was observed but produced no chunk in this turn.
    Wait,
    /// The finite source reached end-of-stream.
    End,
}

/// Blocking source that initializes caller-owned scratch storage.
pub trait BlockingStreamSource {
    /// Source-specific payload-free failure.
    type Error;

    /// Returns explicit body replayability and exact source version.
    fn replayability(&self) -> StreamReplayability<'_>;

    /// Produces at most one chunk and never retains `output`.
    fn read_chunk(&mut self, output: &mut [u8]) -> Result<StreamRead, Self::Error>;
}

/// Blocking sink with explicit commit and infallible abort cleanup.
pub trait BlockingStreamSink {
    /// Sink-specific payload-free failure.
    type Error;

    /// Accepts some or all bytes. A zero result is a bounded no-progress event.
    fn write_chunk(&mut self, input: &[u8]) -> Result<usize, Self::Error>;

    /// Commits a successfully validated stream.
    fn commit(&mut self) -> Result<(), Self::Error>;

    /// Aborts an incomplete stream according to its recorded partial state.
    fn abort(&mut self, partial: StreamPartialState);
}

/// Local asynchronous source whose future may be `!Send`.
pub trait LocalAsyncStreamSource {
    /// Source-specific payload-free failure.
    type Error;

    /// Returns explicit body replayability and exact source version.
    fn replayability(&self) -> StreamReplayability<'_>;

    /// Produces at most one chunk without retaining `output` after completion.
    fn read_chunk_local<'operation>(
        &'operation mut self,
        output: &'operation mut [u8],
    ) -> impl Future<Output = Result<StreamRead, Self::Error>> + 'operation;
}

/// Cross-thread asynchronous source.
pub trait AsyncStreamSource {
    /// Source-specific payload-free failure.
    type Error;

    /// Returns explicit body replayability and exact source version.
    fn replayability(&self) -> StreamReplayability<'_>;

    /// Produces at most one chunk in a `Send` future.
    fn read_chunk<'operation>(
        &'operation mut self,
        output: &'operation mut [u8],
    ) -> impl Future<Output = Result<StreamRead, Self::Error>> + Send + 'operation;
}

impl<T: AsyncStreamSource> LocalAsyncStreamSource for T {
    type Error = T::Error;

    fn replayability(&self) -> StreamReplayability<'_> {
        AsyncStreamSource::replayability(self)
    }

    async fn read_chunk_local<'operation>(
        &'operation mut self,
        output: &'operation mut [u8],
    ) -> Result<StreamRead, Self::Error> {
        AsyncStreamSource::read_chunk(self, output).await
    }
}

/// Local asynchronous sink whose futures may be `!Send`.
pub trait LocalAsyncStreamSink {
    /// Sink-specific payload-free failure.
    type Error;

    /// Accepts some or all bytes without retaining `input` after completion.
    fn write_chunk_local<'operation>(
        &'operation mut self,
        input: &'operation [u8],
    ) -> impl Future<Output = Result<usize, Self::Error>> + 'operation;

    /// Commits a successfully validated stream.
    fn commit_local(&mut self) -> impl Future<Output = Result<(), Self::Error>> + '_;

    /// Synchronously aborts on error or future cancellation.
    fn abort_local(&mut self, partial: StreamPartialState);
}

/// Cross-thread asynchronous sink.
pub trait AsyncStreamSink {
    /// Sink-specific payload-free failure.
    type Error;

    /// Accepts some or all bytes in a `Send` future.
    fn write_chunk<'operation>(
        &'operation mut self,
        input: &'operation [u8],
    ) -> impl Future<Output = Result<usize, Self::Error>> + Send + 'operation;

    /// Commits a successfully validated stream in a `Send` future.
    fn commit(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send + '_;

    /// Synchronously aborts on error or future cancellation.
    fn abort(&mut self, partial: StreamPartialState);
}

impl<T: AsyncStreamSink> LocalAsyncStreamSink for T {
    type Error = T::Error;

    async fn write_chunk_local<'operation>(
        &'operation mut self,
        input: &'operation [u8],
    ) -> Result<usize, Self::Error> {
        AsyncStreamSink::write_chunk(self, input).await
    }

    async fn commit_local(&mut self) -> Result<(), Self::Error> {
        AsyncStreamSink::commit(self).await
    }

    fn abort_local(&mut self, partial: StreamPartialState) {
        AsyncStreamSink::abort(self, partial);
    }
}

/// One bounded stream execution failure.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum StreamExecutionError<S, D> {
    /// Caller scratch storage is empty.
    EmptyScratch,
    /// The source claimed more initialized bytes than the supplied prefix.
    InvalidSourceLength,
    /// SDK-owned accounting or lifecycle validation failed.
    Progress(StreamProgressError),
    /// The source failed.
    Source(S),
    /// The sink failed while writing or committing.
    Sink(D),
}

impl<S, D> fmt::Debug for StreamExecutionError<S, D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyScratch => formatter.write_str("EmptyScratch"),
            Self::InvalidSourceLength => formatter.write_str("InvalidSourceLength"),
            Self::Progress(error) => formatter.debug_tuple("Progress").field(error).finish(),
            Self::Source(_) => formatter.write_str("Source([redacted])"),
            Self::Sink(_) => formatter.write_str("Sink([redacted])"),
        }
    }
}

impl<S, D> fmt::Display for StreamExecutionError<S, D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::EmptyScratch => "stream scratch storage is empty",
            Self::InvalidSourceLength => "stream source reported an invalid length",
            Self::Progress(_) => "stream progress policy rejected execution",
            Self::Source(_) => "stream source failed",
            Self::Sink(_) => "stream sink failed",
        })
    }
}

impl<S, D> core::error::Error for StreamExecutionError<S, D> {}

struct AbortGuard<'sink, S> {
    sink: &'sink mut S,
    partial: StreamPartialState,
    abort: fn(&mut S, StreamPartialState),
    armed: bool,
}

impl<'sink, S> AbortGuard<'sink, S> {
    fn new(sink: &'sink mut S, abort: fn(&mut S, StreamPartialState)) -> Self {
        Self {
            sink,
            partial: StreamPartialState::Clean,
            abort,
            armed: true,
        }
    }

    fn sink(&mut self) -> &mut S {
        self.sink
    }

    fn record_write_attempt(&mut self, policy: StreamPolicy) {
        self.partial = partial_state(policy.sink_mode(), 1);
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl<S> Drop for AbortGuard<'_, S> {
    fn drop(&mut self) {
        if self.armed {
            (self.abort)(self.sink, self.partial);
        }
    }
}

struct ScratchGuard<'scratch> {
    bytes: &'scratch mut [u8],
}

impl<'scratch> ScratchGuard<'scratch> {
    fn new(bytes: &'scratch mut [u8]) -> Self {
        sanitize_bytes(bytes);
        Self { bytes }
    }

    fn bytes(&mut self) -> &mut [u8] {
        self.bytes
    }
}

impl Drop for ScratchGuard<'_> {
    fn drop(&mut self) {
        sanitize_bytes(self.bytes);
    }
}
