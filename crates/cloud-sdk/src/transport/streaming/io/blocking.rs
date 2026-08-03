use super::{
    AbortGuard, BlockingStreamSink, BlockingStreamSource, ScratchGuard, StreamExecutionError,
    StreamRead,
};
use crate::transport::{
    StreamAttempt, StreamCompletion, StreamOutcome, StreamPartialState, StreamPolicy,
    StreamProgressError,
};

/// Drives one finite blocking transfer with caller-owned scratch storage.
pub fn drive_blocking_stream<S, D>(
    policy: StreamPolicy,
    source: &mut S,
    sink: &mut D,
    scratch: &mut [u8],
    outcome: &mut StreamOutcome,
) -> Result<StreamCompletion, StreamExecutionError<S::Error, D::Error>>
where
    S: BlockingStreamSource,
    D: BlockingStreamSink,
{
    *outcome = StreamOutcome::new();
    if scratch.is_empty() {
        return Err(StreamExecutionError::EmptyScratch);
    }
    let mut scratch = ScratchGuard::new(scratch);
    let mut guard = AbortGuard::new(sink, abort_blocking::<D>);
    let mut attempt = StreamAttempt::new(policy, outcome);
    let completion = pump(policy, source, &mut guard, &mut scratch, &mut attempt)?;
    if let Err(error) = guard.sink().commit() {
        attempt.mark_failed();
        return Err(StreamExecutionError::Sink(error));
    }
    attempt
        .commit_sink()
        .map_err(StreamExecutionError::Progress)?;
    guard.disarm();
    Ok(completion)
}

fn pump<S, D>(
    policy: StreamPolicy,
    source: &mut S,
    sink: &mut AbortGuard<'_, D>,
    scratch: &mut ScratchGuard<'_>,
    attempt: &mut StreamAttempt<'_>,
) -> Result<StreamCompletion, StreamExecutionError<S::Error, D::Error>>
where
    S: BlockingStreamSource,
    D: BlockingStreamSink,
{
    loop {
        attempt
            .begin_source_observation()
            .map_err(StreamExecutionError::Progress)?;
        let read_limit = core::cmp::min(scratch.bytes().len(), policy.limits().max_chunk_bytes());
        let Some(output) = scratch.bytes().get_mut(..read_limit) else {
            attempt.mark_failed();
            return Err(StreamExecutionError::EmptyScratch);
        };
        let read = source.read_chunk(output).map_err(|error| {
            attempt.mark_failed();
            StreamExecutionError::Source(error)
        })?;
        match read {
            StreamRead::End => return attempt.finish().map_err(StreamExecutionError::Progress),
            StreamRead::Wait => attempt
                .observe_wait()
                .map_err(StreamExecutionError::Progress)?,
            StreamRead::Chunk(len) => {
                if len > output.len() {
                    attempt.mark_failed();
                    return Err(StreamExecutionError::InvalidSourceLength);
                }
                transfer(policy, sink, output, len, attempt)?;
            }
        }
    }
}

fn transfer<S, D>(
    policy: StreamPolicy,
    sink: &mut AbortGuard<'_, D>,
    chunk: &[u8],
    len: usize,
    attempt: &mut StreamAttempt<'_>,
) -> Result<(), StreamExecutionError<S, D::Error>>
where
    D: BlockingStreamSink,
{
    attempt
        .begin_chunk(len)
        .map_err(StreamExecutionError::Progress)?;
    let mut offset = 0_usize;
    while offset < len {
        let Some(input) = chunk.get(offset..len) else {
            attempt.mark_failed();
            return Err(StreamExecutionError::Progress(
                StreamProgressError::ArithmeticOverflow,
            ));
        };
        attempt
            .begin_sink_observation()
            .map_err(StreamExecutionError::Progress)?;
        sink.record_write_attempt(policy);
        let accepted = sink.sink().write_chunk(input).map_err(|error| {
            attempt.mark_failed();
            StreamExecutionError::Sink(error)
        })?;
        attempt
            .advance(accepted)
            .map_err(StreamExecutionError::Progress)?;
        offset = offset.checked_add(accepted).ok_or_else(|| {
            attempt.mark_failed();
            StreamExecutionError::Progress(StreamProgressError::ArithmeticOverflow)
        })?;
    }
    Ok(())
}

fn abort_blocking<S: BlockingStreamSink>(sink: &mut S, state: StreamPartialState) {
    sink.abort(state);
}
