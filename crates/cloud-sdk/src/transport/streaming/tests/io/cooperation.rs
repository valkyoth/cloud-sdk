use core::future::Future;
use core::sync::atomic::{AtomicU32, Ordering};
use core::task::{Context, Poll, Waker};

use super::super::super::{
    AsyncStreamSink, AsyncStreamSource, StreamFraming, StreamOutcome, StreamPartialState,
    StreamRead, StreamReplayability, StreamSinkMode, StreamState, drive_async_stream,
    drive_local_stream,
};
use super::super::{limits, policy};

const CHUNKS_AFTER_FORCED_YIELD: u32 = 32;
const TOTAL_CHUNKS: u32 = 40;

struct ReadySource<'counter> {
    remaining: u32,
    reads: &'counter AtomicU32,
}

impl AsyncStreamSource for ReadySource<'_> {
    type Error = ();

    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }

    async fn read_chunk<'operation>(
        &'operation mut self,
        output: &'operation mut [u8],
    ) -> Result<StreamRead, Self::Error> {
        self.reads.fetch_add(1, Ordering::Relaxed);
        if self.remaining == 0 {
            return Ok(StreamRead::End);
        }
        self.remaining = self.remaining.checked_sub(1).ok_or(())?;
        *output.first_mut().ok_or(())? = b'x';
        Ok(StreamRead::Chunk(1))
    }
}

struct ReadySink<'counter> {
    writes: &'counter AtomicU32,
    committed: bool,
}

impl AsyncStreamSink for ReadySink<'_> {
    type Error = ();

    async fn write_chunk<'operation>(
        &'operation mut self,
        input: &'operation [u8],
    ) -> Result<usize, Self::Error> {
        self.writes.fetch_add(1, Ordering::Relaxed);
        Ok(input.len())
    }

    async fn commit(&mut self) -> Result<(), Self::Error> {
        self.committed = true;
        Ok(())
    }

    fn abort(&mut self, _partial: StreamPartialState) {}
}

fn ready_policy() -> super::super::super::StreamPolicy {
    policy(
        StreamFraming::Declared(u64::from(TOTAL_CHUNKS)),
        StreamSinkMode::Direct,
        limits(
            u64::from(TOTAL_CHUNKS),
            1,
            TOTAL_CHUNKS,
            TOTAL_CHUNKS.saturating_mul(2).saturating_add(1),
            0,
        ),
    )
}

#[test]
fn send_async_driver_forces_a_yield_after_bounded_ready_callbacks() {
    let reads = AtomicU32::new(0);
    let writes = AtomicU32::new(0);
    let mut source = ReadySource {
        remaining: TOTAL_CHUNKS,
        reads: &reads,
    };
    let mut sink = ReadySink {
        writes: &writes,
        committed: false,
    };
    let mut scratch = [0_u8; 1];
    let mut outcome = StreamOutcome::new();
    {
        let future = drive_async_stream(
            ready_policy(),
            &mut source,
            &mut sink,
            &mut scratch,
            &mut outcome,
        );
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Pending
        ));
        assert_eq!(reads.load(Ordering::Relaxed), CHUNKS_AFTER_FORCED_YIELD);
        assert_eq!(writes.load(Ordering::Relaxed), CHUNKS_AFTER_FORCED_YIELD);
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready(Ok(_))
        ));
    }
    assert_eq!(
        reads.load(Ordering::Relaxed),
        TOTAL_CHUNKS.saturating_add(1)
    );
    assert_eq!(writes.load(Ordering::Relaxed), TOTAL_CHUNKS);
    assert!(sink.committed);
    assert_eq!(outcome.state(), StreamState::Complete);
}

#[test]
fn local_async_driver_forces_a_yield_after_bounded_ready_callbacks() {
    let reads = AtomicU32::new(0);
    let writes = AtomicU32::new(0);
    let mut source = ReadySource {
        remaining: TOTAL_CHUNKS,
        reads: &reads,
    };
    let mut sink = ReadySink {
        writes: &writes,
        committed: false,
    };
    let mut scratch = [0_u8; 1];
    let mut outcome = StreamOutcome::new();
    {
        let future = drive_local_stream(
            ready_policy(),
            &mut source,
            &mut sink,
            &mut scratch,
            &mut outcome,
        );
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Pending
        ));
        assert_eq!(reads.load(Ordering::Relaxed), CHUNKS_AFTER_FORCED_YIELD);
        assert_eq!(writes.load(Ordering::Relaxed), CHUNKS_AFTER_FORCED_YIELD);
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready(Ok(_))
        ));
    }
    assert_eq!(
        reads.load(Ordering::Relaxed),
        TOTAL_CHUNKS.saturating_add(1)
    );
    assert_eq!(writes.load(Ordering::Relaxed), TOTAL_CHUNKS);
    assert!(sink.committed);
    assert_eq!(outcome.state(), StreamState::Complete);
}
