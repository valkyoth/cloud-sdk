use super::super::super::{
    BlockingStreamSink, BlockingStreamSource, StreamExecutionError, StreamFraming, StreamOutcome,
    StreamPartialState, StreamProgressError, StreamRead, StreamReplayability, StreamSinkMode,
    StreamState, drive_blocking_stream,
};
use super::super::{limits, policy};
use cloud_sdk_sanitization::sanitize_bytes;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SourceError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SinkError;

struct SliceSource<'a> {
    chunks: &'a [&'a [u8]],
    index: usize,
    reads: usize,
}

impl BlockingStreamSource for SliceSource<'_> {
    type Error = SourceError;

    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }

    fn read_chunk(&mut self, output: &mut [u8]) -> Result<StreamRead, Self::Error> {
        self.reads = self.reads.saturating_add(1);
        let Some(chunk) = self.chunks.get(self.index) else {
            return Ok(StreamRead::End);
        };
        self.index = self.index.saturating_add(1);
        let Some(target) = output.get_mut(..chunk.len()) else {
            return Ok(StreamRead::Chunk(output.len().saturating_add(1)));
        };
        target.copy_from_slice(chunk);
        Ok(StreamRead::Chunk(chunk.len()))
    }
}

struct FixtureSink {
    output: [u8; 32],
    len: usize,
    max_write: usize,
    writes: usize,
    fail_after: Option<usize>,
    overreport: bool,
    committed: bool,
    aborted: Option<StreamPartialState>,
}

impl FixtureSink {
    const fn new(max_write: usize) -> Self {
        Self {
            output: [0; 32],
            len: 0,
            max_write,
            writes: 0,
            fail_after: None,
            overreport: false,
            committed: false,
            aborted: None,
        }
    }

    fn bytes(&self) -> &[u8] {
        self.output.get(..self.len).unwrap_or_default()
    }
}

impl BlockingStreamSink for FixtureSink {
    type Error = SinkError;

    fn write_chunk(&mut self, input: &[u8]) -> Result<usize, Self::Error> {
        if self.fail_after.is_some_and(|limit| self.len >= limit) {
            return Err(SinkError);
        }
        self.writes = self.writes.saturating_add(1);
        if self.overreport {
            return Ok(input.len().saturating_add(1));
        }
        let accepted = core::cmp::min(input.len(), self.max_write);
        let end = self.len.checked_add(accepted).ok_or(SinkError)?;
        let target = self.output.get_mut(self.len..end).ok_or(SinkError)?;
        let source = input.get(..accepted).ok_or(SinkError)?;
        target.copy_from_slice(source);
        self.len = end;
        Ok(accepted)
    }

    fn commit(&mut self) -> Result<(), Self::Error> {
        self.committed = true;
        Ok(())
    }

    fn abort(&mut self, partial: StreamPartialState) {
        self.aborted = Some(partial);
        if matches!(partial, StreamPartialState::RollbackRequired) {
            sanitize_bytes(&mut self.output);
            self.len = 0;
        }
    }
}

#[test]
fn blocking_driver_handles_chunk_boundaries_empty_chunks_and_short_writes() {
    let chunks: &[&[u8]] = &[b"ab", b"", b"cde"];
    let mut source = SliceSource {
        chunks,
        index: 0,
        reads: 0,
    };
    let mut sink = FixtureSink::new(2);
    let mut scratch = [0_u8; 4];
    let mut outcome = StreamOutcome::new();
    let policy = policy(
        StreamFraming::Declared(5),
        StreamSinkMode::Transactional,
        limits(5, 4, 4, 8, 1),
    );
    let result = drive_blocking_stream(policy, &mut source, &mut sink, &mut scratch, &mut outcome);
    assert!(result.is_ok());
    assert_eq!(sink.bytes(), b"abcde");
    assert_eq!(sink.writes, 3);
    assert_eq!(source.reads, 4);
    assert!(sink.committed);
    assert_eq!(sink.aborted, None);
    assert_eq!(outcome.state(), StreamState::Complete);
    assert!(scratch.iter().all(|byte| *byte == 0));
}

#[test]
fn declared_under_and_over_length_abort_with_precise_partial_state() {
    let chunks: &[&[u8]] = &[b"ab"];
    let mut source = SliceSource {
        chunks,
        index: 0,
        reads: 0,
    };
    let mut sink = FixtureSink::new(8);
    let mut scratch = [0_u8; 8];
    let mut outcome = StreamOutcome::new();
    let under = policy(
        StreamFraming::Declared(3),
        StreamSinkMode::Direct,
        limits(4, 4, 2, 4, 1),
    );
    assert_eq!(
        drive_blocking_stream(under, &mut source, &mut sink, &mut scratch, &mut outcome,),
        Err(StreamExecutionError::Progress(
            StreamProgressError::DeclaredLengthMismatch
        ))
    );
    assert_eq!(sink.aborted, Some(StreamPartialState::Dirty));
    assert_eq!(sink.bytes(), b"ab");

    let chunks: &[&[u8]] = &[b"abcd"];
    let mut source = SliceSource {
        chunks,
        index: 0,
        reads: 0,
    };
    let mut sink = FixtureSink::new(8);
    let mut outcome = StreamOutcome::new();
    let over = policy(
        StreamFraming::Declared(3),
        StreamSinkMode::Direct,
        limits(4, 4, 2, 4, 1),
    );
    assert_eq!(
        drive_blocking_stream(over, &mut source, &mut sink, &mut scratch, &mut outcome,),
        Err(StreamExecutionError::Progress(
            StreamProgressError::DeclaredLengthExceeded
        ))
    );
    assert_eq!(sink.aborted, Some(StreamPartialState::Clean));
}

struct EmptySource;

impl BlockingStreamSource for EmptySource {
    type Error = SourceError;

    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }

    fn read_chunk(&mut self, _output: &mut [u8]) -> Result<StreamRead, Self::Error> {
        Ok(StreamRead::Chunk(0))
    }
}

#[test]
fn endless_empty_source_exhausts_zero_progress_before_general_budgets() {
    let mut source = EmptySource;
    let mut sink = FixtureSink::new(1);
    let mut scratch = [0_u8; 1];
    let mut outcome = StreamOutcome::new();
    let policy = policy(
        StreamFraming::ExecutorOwned,
        StreamSinkMode::Transactional,
        limits(8, 1, 8, 8, 2),
    );
    assert_eq!(
        drive_blocking_stream(policy, &mut source, &mut sink, &mut scratch, &mut outcome,),
        Err(StreamExecutionError::Progress(
            StreamProgressError::ZeroProgressLimitExceeded
        ))
    );
    assert_eq!(outcome.progress().chunks(), 2);
    assert_eq!(sink.aborted, Some(StreamPartialState::Clean));
}

#[test]
fn transactional_sink_failure_rolls_back_and_overreport_is_rejected() {
    let chunks: &[&[u8]] = &[b"abcd"];
    let mut source = SliceSource {
        chunks,
        index: 0,
        reads: 0,
    };
    let mut sink = FixtureSink::new(2);
    sink.fail_after = Some(2);
    let mut scratch = [0_u8; 4];
    let mut outcome = StreamOutcome::new();
    let transactional = policy(
        StreamFraming::Declared(4),
        StreamSinkMode::Transactional,
        limits(4, 4, 1, 4, 1),
    );
    assert_eq!(
        drive_blocking_stream(
            transactional,
            &mut source,
            &mut sink,
            &mut scratch,
            &mut outcome,
        ),
        Err(StreamExecutionError::Sink(SinkError))
    );
    assert_eq!(sink.aborted, Some(StreamPartialState::RollbackRequired));
    assert!(sink.bytes().is_empty());
    assert!(scratch.iter().all(|byte| *byte == 0));

    let mut source = SliceSource {
        chunks,
        index: 0,
        reads: 0,
    };
    let mut sink = FixtureSink::new(4);
    sink.overreport = true;
    let mut outcome = StreamOutcome::new();
    assert_eq!(
        drive_blocking_stream(
            transactional,
            &mut source,
            &mut sink,
            &mut scratch,
            &mut outcome,
        ),
        Err(StreamExecutionError::Progress(
            StreamProgressError::InvalidSinkProgress
        ))
    );
    assert_eq!(sink.aborted, Some(StreamPartialState::RollbackRequired));

    let mut source = SliceSource {
        chunks,
        index: 0,
        reads: 0,
    };
    let mut sink = FixtureSink::new(4);
    sink.fail_after = Some(0);
    let mut outcome = StreamOutcome::new();
    let direct = policy(
        StreamFraming::Declared(4),
        StreamSinkMode::Direct,
        limits(4, 4, 1, 4, 1),
    );
    assert_eq!(
        drive_blocking_stream(direct, &mut source, &mut sink, &mut scratch, &mut outcome,),
        Err(StreamExecutionError::Sink(SinkError))
    );
    assert_eq!(sink.aborted, Some(StreamPartialState::Dirty));
}

#[test]
fn empty_scratch_fails_before_source_or_sink_access() {
    let chunks: &[&[u8]] = &[b"a"];
    let mut source = SliceSource {
        chunks,
        index: 0,
        reads: 0,
    };
    let mut sink = FixtureSink::new(1);
    let mut outcome = StreamOutcome::new();
    let policy = policy(
        StreamFraming::Declared(1),
        StreamSinkMode::Direct,
        limits(1, 1, 1, 2, 0),
    );
    assert_eq!(
        drive_blocking_stream(policy, &mut source, &mut sink, &mut [], &mut outcome,),
        Err(StreamExecutionError::EmptyScratch)
    );
    assert_eq!(source.reads, 0);
    assert_eq!(sink.writes, 0);
    assert_eq!(outcome.state(), StreamState::NotStarted);
}

#[test]
fn observation_limits_fail_before_external_source_or_sink_calls() {
    let chunks: &[&[u8]] = &[b"a", b"b"];
    let mut source = SliceSource {
        chunks,
        index: 0,
        reads: 0,
    };
    let mut sink = FixtureSink::new(1);
    let mut scratch = [0_u8; 1];
    let mut outcome = StreamOutcome::new();
    let source_limited = policy(
        StreamFraming::ExecutorOwned,
        StreamSinkMode::Direct,
        limits(2, 1, 2, 2, 0),
    );
    assert_eq!(
        drive_blocking_stream(
            source_limited,
            &mut source,
            &mut sink,
            &mut scratch,
            &mut outcome,
        ),
        Err(StreamExecutionError::Progress(
            StreamProgressError::ObservationLimitExceeded
        ))
    );
    assert_eq!(source.reads, 1);
    assert_eq!(sink.writes, 1);

    let chunks: &[&[u8]] = &[b"a"];
    let mut source = SliceSource {
        chunks,
        index: 0,
        reads: 0,
    };
    let mut sink = FixtureSink::new(1);
    let mut outcome = StreamOutcome::new();
    let sink_limited = policy(
        StreamFraming::ExecutorOwned,
        StreamSinkMode::Direct,
        limits(1, 1, 1, 1, 0),
    );
    assert_eq!(
        drive_blocking_stream(
            sink_limited,
            &mut source,
            &mut sink,
            &mut scratch,
            &mut outcome,
        ),
        Err(StreamExecutionError::Progress(
            StreamProgressError::ObservationLimitExceeded
        ))
    );
    assert_eq!(source.reads, 1);
    assert_eq!(sink.writes, 0);
}
