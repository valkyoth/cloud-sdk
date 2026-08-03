use core::future::{Future, pending};
use core::marker::PhantomData;
use core::task::{Context, Poll, Waker};

use super::super::super::{
    AsyncStreamSink, AsyncStreamSource, LocalAsyncStreamSink, LocalAsyncStreamSource,
    StreamFraming, StreamOutcome, StreamPartialState, StreamRead, StreamReplayability,
    StreamSinkMode, StreamState, drive_async_stream, drive_local_stream,
};
use super::super::{limits, policy};
use cloud_sdk_sanitization::sanitize_bytes;

struct SendSource {
    done: bool,
}

impl AsyncStreamSource for SendSource {
    type Error = ();

    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }

    async fn read_chunk<'operation>(
        &'operation mut self,
        output: &'operation mut [u8],
    ) -> Result<StreamRead, Self::Error> {
        if self.done {
            return Ok(StreamRead::End);
        }
        self.done = true;
        let target = output.get_mut(..2).ok_or(())?;
        target.copy_from_slice(b"ok");
        Ok(StreamRead::Chunk(2))
    }
}

struct SendSink {
    output: [u8; 2],
    len: usize,
    committed: bool,
    aborted: Option<StreamPartialState>,
}

impl AsyncStreamSink for SendSink {
    type Error = ();

    async fn write_chunk<'operation>(
        &'operation mut self,
        input: &'operation [u8],
    ) -> Result<usize, Self::Error> {
        let end = self.len.checked_add(input.len()).ok_or(())?;
        self.output
            .get_mut(self.len..end)
            .ok_or(())?
            .copy_from_slice(input);
        self.len = end;
        Ok(input.len())
    }

    async fn commit(&mut self) -> Result<(), Self::Error> {
        self.committed = true;
        Ok(())
    }

    fn abort(&mut self, partial: StreamPartialState) {
        self.aborted = Some(partial);
    }
}

#[test]
fn send_async_driver_is_executor_neutral_and_send_sources_are_local_compatible() {
    let policy = policy(
        StreamFraming::Declared(2),
        StreamSinkMode::Direct,
        limits(2, 2, 1, 3, 0),
    );
    let mut source = SendSource { done: false };
    let mut sink = SendSink {
        output: [0; 2],
        len: 0,
        committed: false,
        aborted: None,
    };
    let mut scratch = [0_u8; 2];
    let mut outcome = StreamOutcome::new();
    {
        let future = require_send(drive_async_stream(
            policy,
            &mut source,
            &mut sink,
            &mut scratch,
            &mut outcome,
        ));
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready(Ok(_))
        ));
    }
    assert_eq!(sink.output, *b"ok");
    assert!(sink.committed);
    assert_eq!(outcome.state(), StreamState::Complete);

    let mut source = SendSource { done: false };
    let mut sink = SendSink {
        output: [0; 2],
        len: 0,
        committed: false,
        aborted: None,
    };
    let mut outcome = StreamOutcome::new();
    {
        let future = drive_local_stream(policy, &mut source, &mut sink, &mut scratch, &mut outcome);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready(Ok(_))
        ));
    }
}

fn require_send<T: Send>(value: T) -> T {
    value
}

struct PendingLocalSource {
    emitted: bool,
    _not_send: PhantomData<*const ()>,
}

impl LocalAsyncStreamSource for PendingLocalSource {
    type Error = ();

    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }

    async fn read_chunk_local<'operation>(
        &'operation mut self,
        output: &'operation mut [u8],
    ) -> Result<StreamRead, Self::Error> {
        if !self.emitted {
            self.emitted = true;
            output.get_mut(..2).ok_or(())?.copy_from_slice(b"ok");
            return Ok(StreamRead::Chunk(2));
        }
        pending::<()>().await;
        Ok(StreamRead::End)
    }
}

struct LocalTransactionalSink {
    output: [u8; 2],
    len: usize,
    aborted: Option<StreamPartialState>,
    pending_commit: bool,
}

impl LocalAsyncStreamSink for LocalTransactionalSink {
    type Error = ();

    async fn write_chunk_local<'operation>(
        &'operation mut self,
        input: &'operation [u8],
    ) -> Result<usize, Self::Error> {
        let end = self.len.checked_add(input.len()).ok_or(())?;
        self.output
            .get_mut(self.len..end)
            .ok_or(())?
            .copy_from_slice(input);
        self.len = end;
        Ok(input.len())
    }

    async fn commit_local(&mut self) -> Result<(), Self::Error> {
        if self.pending_commit {
            pending::<()>().await;
        }
        Ok(())
    }

    fn abort_local(&mut self, partial: StreamPartialState) {
        self.aborted = Some(partial);
        if matches!(partial, StreamPartialState::RollbackRequired) {
            sanitize_bytes(&mut self.output);
            self.len = 0;
        }
    }
}

#[test]
fn local_async_cancellation_aborts_transactional_sink_and_records_progress() {
    let policy = policy(
        StreamFraming::ExecutorOwned,
        StreamSinkMode::Transactional,
        limits(8, 2, 4, 8, 1),
    );
    let mut source = PendingLocalSource {
        emitted: false,
        _not_send: PhantomData,
    };
    let mut sink = LocalTransactionalSink {
        output: [0; 2],
        len: 0,
        aborted: None,
        pending_commit: false,
    };
    let mut scratch = [0_u8; 2];
    let mut outcome = StreamOutcome::new();
    {
        let future = drive_local_stream(policy, &mut source, &mut sink, &mut scratch, &mut outcome);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Pending
        ));
    }
    assert_eq!(sink.aborted, Some(StreamPartialState::RollbackRequired));
    assert_eq!(sink.output, [0; 2]);
    assert_eq!(outcome.state(), StreamState::Cancelled);
    assert_eq!(outcome.progress().bytes(), 2);
    assert_eq!(scratch, [0; 2]);
}

struct FiniteLocalSource {
    done: bool,
}

impl LocalAsyncStreamSource for FiniteLocalSource {
    type Error = ();

    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }

    async fn read_chunk_local<'operation>(
        &'operation mut self,
        output: &'operation mut [u8],
    ) -> Result<StreamRead, Self::Error> {
        if self.done {
            return Ok(StreamRead::End);
        }
        self.done = true;
        output.get_mut(..2).ok_or(())?.copy_from_slice(b"ok");
        Ok(StreamRead::Chunk(2))
    }
}

#[test]
fn cancellation_while_transactional_commit_is_pending_is_not_complete() {
    let policy = policy(
        StreamFraming::Declared(2),
        StreamSinkMode::Transactional,
        limits(2, 2, 1, 3, 0),
    );
    let mut source = FiniteLocalSource { done: false };
    let mut sink = LocalTransactionalSink {
        output: [0; 2],
        len: 0,
        aborted: None,
        pending_commit: true,
    };
    let mut scratch = [0_u8; 2];
    let mut outcome = StreamOutcome::new();
    {
        let future = drive_local_stream(policy, &mut source, &mut sink, &mut scratch, &mut outcome);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Pending
        ));
    }
    assert_eq!(sink.aborted, Some(StreamPartialState::RollbackRequired));
    assert_eq!(sink.output, [0; 2]);
    assert_eq!(outcome.state(), StreamState::Cancelled);
    assert_eq!(scratch, [0; 2]);
}

struct PendingFirstWriteSink {
    effect_observed: bool,
    aborted: Option<StreamPartialState>,
}

impl LocalAsyncStreamSink for PendingFirstWriteSink {
    type Error = ();

    async fn write_chunk_local<'operation>(
        &'operation mut self,
        _input: &'operation [u8],
    ) -> Result<usize, Self::Error> {
        self.effect_observed = true;
        pending::<()>().await;
        Ok(0)
    }

    async fn commit_local(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn abort_local(&mut self, partial: StreamPartialState) {
        self.aborted = Some(partial);
    }
}

#[test]
fn first_sink_future_cancellation_is_never_reported_clean() {
    for (mode, expected) in [
        (
            StreamSinkMode::Transactional,
            StreamPartialState::RollbackRequired,
        ),
        (StreamSinkMode::Direct, StreamPartialState::Dirty),
    ] {
        let policy = policy(StreamFraming::Declared(2), mode, limits(2, 2, 1, 3, 0));
        let mut source = FiniteLocalSource { done: false };
        let mut sink = PendingFirstWriteSink {
            effect_observed: false,
            aborted: None,
        };
        let mut scratch = [0_u8; 2];
        let mut outcome = StreamOutcome::new();
        {
            let future =
                drive_local_stream(policy, &mut source, &mut sink, &mut scratch, &mut outcome);
            let mut future = core::pin::pin!(future);
            let mut context = Context::from_waker(Waker::noop());
            assert!(matches!(
                Future::poll(future.as_mut(), &mut context),
                Poll::Pending
            ));
        }
        assert!(sink.effect_observed);
        assert_eq!(sink.aborted, Some(expected));
        assert_eq!(outcome.state(), StreamState::Cancelled);
        assert_eq!(outcome.partial_state(), expected);
        assert_eq!(outcome.progress().bytes(), 0);
        assert_eq!(scratch, [0; 2]);
    }
}

#[test]
fn async_empty_scratch_resets_reused_complete_outcomes() {
    let policy = policy(
        StreamFraming::Declared(2),
        StreamSinkMode::Direct,
        limits(2, 2, 1, 3, 0),
    );
    let mut scratch = [0_u8; 2];
    let mut outcome = StreamOutcome::new();

    let mut source = SendSource { done: false };
    let mut sink = SendSink {
        output: [0; 2],
        len: 0,
        committed: false,
        aborted: None,
    };
    {
        let future = drive_async_stream(policy, &mut source, &mut sink, &mut scratch, &mut outcome);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready(Ok(_))
        ));
    }
    assert_eq!(outcome.state(), StreamState::Complete);
    {
        let future = drive_async_stream(policy, &mut source, &mut sink, &mut [], &mut outcome);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready(Err(super::super::super::StreamExecutionError::EmptyScratch))
        ));
    }
    assert_eq!(outcome.state(), StreamState::NotStarted);

    let mut source = SendSource { done: false };
    let mut sink = SendSink {
        output: [0; 2],
        len: 0,
        committed: false,
        aborted: None,
    };
    {
        let future = drive_local_stream(policy, &mut source, &mut sink, &mut scratch, &mut outcome);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready(Ok(_))
        ));
    }
    assert_eq!(outcome.state(), StreamState::Complete);
    {
        let future = drive_local_stream(policy, &mut source, &mut sink, &mut [], &mut outcome);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready(Err(super::super::super::StreamExecutionError::EmptyScratch))
        ));
    }
    assert_eq!(outcome.state(), StreamState::NotStarted);
}
