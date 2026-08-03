use super::{
    AbortGuard, AsyncStreamSink, AsyncStreamSource, LocalAsyncStreamSink, LocalAsyncStreamSource,
    ScratchGuard, StreamExecutionError, StreamRead,
};
use crate::transport::{
    StreamAttempt, StreamCompletion, StreamOutcome, StreamPartialState, StreamPolicy,
    StreamProgressError,
};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

const MAX_CALLBACKS_BEFORE_YIELD: u16 = 64;

/// Drives one local asynchronous transfer without owning an executor.
pub async fn drive_local_stream<S, D>(
    policy: StreamPolicy,
    source: &mut S,
    sink: &mut D,
    scratch: &mut [u8],
    outcome: &mut StreamOutcome,
) -> Result<StreamCompletion, StreamExecutionError<S::Error, D::Error>>
where
    S: LocalAsyncStreamSource,
    D: LocalAsyncStreamSink,
{
    *outcome = StreamOutcome::new();
    if scratch.is_empty() {
        return Err(StreamExecutionError::EmptyScratch);
    }
    let mut scratch = ScratchGuard::new(scratch);
    let mut guard = AbortGuard::new(sink, abort_local::<D>);
    let mut attempt = StreamAttempt::new(policy, outcome);
    let mut cooperation = CooperativeBudget::new();
    let completion = loop {
        attempt
            .begin_source_observation()
            .map_err(StreamExecutionError::Progress)?;
        let read_limit = core::cmp::min(scratch.bytes().len(), policy.limits().max_chunk_bytes());
        let Some(output) = scratch.bytes().get_mut(..read_limit) else {
            attempt.mark_failed();
            return Err(StreamExecutionError::EmptyScratch);
        };
        let read = match source.read_chunk_local(output).await {
            Ok(read) => read,
            Err(error) => {
                attempt.mark_failed();
                return Err(StreamExecutionError::Source(error));
            }
        };
        match read {
            StreamRead::End => {
                let completion = attempt.finish().map_err(StreamExecutionError::Progress)?;
                cooperation.after_callback().await;
                break completion;
            }
            StreamRead::Wait => {
                attempt
                    .observe_wait()
                    .map_err(StreamExecutionError::Progress)?;
                cooperation.after_callback().await;
            }
            StreamRead::Chunk(len) => {
                validate_source_length(len, output.len(), &mut attempt)?;
                attempt
                    .begin_chunk(len)
                    .map_err(StreamExecutionError::Progress)?;
                cooperation.after_callback().await;
                let mut offset = 0_usize;
                while offset < len {
                    let Some(input) = output.get(offset..len) else {
                        attempt.mark_failed();
                        return Err(arithmetic_error());
                    };
                    attempt
                        .begin_sink_observation()
                        .map_err(StreamExecutionError::Progress)?;
                    guard.record_write_attempt(policy);
                    let accepted = match guard.sink().write_chunk_local(input).await {
                        Ok(accepted) => accepted,
                        Err(error) => {
                            attempt.mark_failed();
                            return Err(StreamExecutionError::Sink(error));
                        }
                    };
                    advance::<S::Error, D::Error>(&mut attempt, &mut offset, accepted)?;
                    cooperation.after_callback().await;
                }
            }
        }
    };
    if let Err(error) = guard.sink().commit_local().await {
        attempt.mark_failed();
        return Err(StreamExecutionError::Sink(error));
    }
    attempt
        .commit_sink()
        .map_err(StreamExecutionError::Progress)?;
    guard.disarm();
    Ok(completion)
}

/// Drives one cross-thread asynchronous transfer without owning an executor.
pub async fn drive_async_stream<S, D>(
    policy: StreamPolicy,
    source: &mut S,
    sink: &mut D,
    scratch: &mut [u8],
    outcome: &mut StreamOutcome,
) -> Result<StreamCompletion, StreamExecutionError<S::Error, D::Error>>
where
    S: AsyncStreamSource + Send,
    D: AsyncStreamSink + Send,
{
    *outcome = StreamOutcome::new();
    if scratch.is_empty() {
        return Err(StreamExecutionError::EmptyScratch);
    }
    let mut scratch = ScratchGuard::new(scratch);
    let mut guard = AbortGuard::new(sink, abort_async::<D>);
    let mut attempt = StreamAttempt::new(policy, outcome);
    let mut cooperation = CooperativeBudget::new();
    let completion = loop {
        attempt
            .begin_source_observation()
            .map_err(StreamExecutionError::Progress)?;
        let read_limit = core::cmp::min(scratch.bytes().len(), policy.limits().max_chunk_bytes());
        let Some(output) = scratch.bytes().get_mut(..read_limit) else {
            attempt.mark_failed();
            return Err(StreamExecutionError::EmptyScratch);
        };
        let read = match source.read_chunk(output).await {
            Ok(read) => read,
            Err(error) => {
                attempt.mark_failed();
                return Err(StreamExecutionError::Source(error));
            }
        };
        match read {
            StreamRead::End => {
                let completion = attempt.finish().map_err(StreamExecutionError::Progress)?;
                cooperation.after_callback().await;
                break completion;
            }
            StreamRead::Wait => {
                attempt
                    .observe_wait()
                    .map_err(StreamExecutionError::Progress)?;
                cooperation.after_callback().await;
            }
            StreamRead::Chunk(len) => {
                validate_source_length(len, output.len(), &mut attempt)?;
                attempt
                    .begin_chunk(len)
                    .map_err(StreamExecutionError::Progress)?;
                cooperation.after_callback().await;
                let mut offset = 0_usize;
                while offset < len {
                    let Some(input) = output.get(offset..len) else {
                        attempt.mark_failed();
                        return Err(arithmetic_error());
                    };
                    attempt
                        .begin_sink_observation()
                        .map_err(StreamExecutionError::Progress)?;
                    guard.record_write_attempt(policy);
                    let accepted = match guard.sink().write_chunk(input).await {
                        Ok(accepted) => accepted,
                        Err(error) => {
                            attempt.mark_failed();
                            return Err(StreamExecutionError::Sink(error));
                        }
                    };
                    advance::<S::Error, D::Error>(&mut attempt, &mut offset, accepted)?;
                    cooperation.after_callback().await;
                }
            }
        }
    };
    if let Err(error) = guard.sink().commit().await {
        attempt.mark_failed();
        return Err(StreamExecutionError::Sink(error));
    }
    attempt
        .commit_sink()
        .map_err(StreamExecutionError::Progress)?;
    guard.disarm();
    Ok(completion)
}

fn validate_source_length<S, D>(
    len: usize,
    capacity: usize,
    attempt: &mut StreamAttempt<'_>,
) -> Result<(), StreamExecutionError<S, D>> {
    if len > capacity {
        attempt.mark_failed();
        return Err(StreamExecutionError::InvalidSourceLength);
    }
    Ok(())
}

fn advance<S, E>(
    attempt: &mut StreamAttempt<'_>,
    offset: &mut usize,
    accepted: usize,
) -> Result<(), StreamExecutionError<S, E>> {
    attempt
        .advance(accepted)
        .map_err(StreamExecutionError::Progress)?;
    *offset = offset.checked_add(accepted).ok_or_else(|| {
        attempt.mark_failed();
        arithmetic_error()
    })?;
    Ok(())
}

const fn arithmetic_error<S, D>() -> StreamExecutionError<S, D> {
    StreamExecutionError::Progress(StreamProgressError::ArithmeticOverflow)
}

fn abort_local<S: LocalAsyncStreamSink>(sink: &mut S, state: StreamPartialState) {
    sink.abort_local(state);
}

fn abort_async<S: AsyncStreamSink>(sink: &mut S, state: StreamPartialState) {
    sink.abort(state);
}

struct CooperativeBudget {
    completed_callbacks: u16,
}

impl CooperativeBudget {
    const fn new() -> Self {
        Self {
            completed_callbacks: 0,
        }
    }

    async fn after_callback(&mut self) {
        if self.completed_callbacks == MAX_CALLBACKS_BEFORE_YIELD - 1 {
            self.completed_callbacks = 0;
            YieldOnce { yielded: false }.await;
        } else if let Some(next) = self.completed_callbacks.checked_add(1) {
            self.completed_callbacks = next;
        } else {
            self.completed_callbacks = 0;
            YieldOnce { yielded: false }.await;
        }
    }
}

struct YieldOnce {
    yielded: bool,
}

impl Future for YieldOnce {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            context.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
