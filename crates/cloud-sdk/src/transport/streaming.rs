//! Allocation-free, runtime-neutral streaming transport contracts.

mod io;
mod policy;
mod progress;
mod replay;

pub use io::{
    AsyncStreamSink, AsyncStreamSource, BlockingStreamSink, BlockingStreamSource,
    LocalAsyncStreamSink, LocalAsyncStreamSource, StreamExecutionError, StreamRead,
    drive_async_stream, drive_blocking_stream, drive_local_stream,
};
pub use policy::{
    MAX_CONSECUTIVE_ZERO_PROGRESS, MAX_STREAM_BYTES, MAX_STREAM_CHUNK_BYTES, MAX_STREAM_CHUNKS,
    MAX_STREAM_OBSERVATIONS, StreamFraming, StreamKind, StreamLimits, StreamLimitsError,
    StreamPolicy, StreamPolicyError, StreamSinkMode,
};
pub use progress::{
    StreamAttempt, StreamCompletion, StreamOutcome, StreamPartialState, StreamProgress,
    StreamProgressError, StreamState,
};
pub use replay::{
    MAX_STREAM_SOURCE_ID_BYTES, StreamReplayError, StreamReplayability, StreamSourceId,
    StreamSourceIdError, validate_stream_replay,
};

#[cfg(test)]
mod tests;
