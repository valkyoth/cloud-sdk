use cloud_sdk::transport::{
    BlockingStreamSource, StreamExecutionError, StreamFraming, StreamKind, StreamLimits,
    StreamOutcome, StreamPolicy, StreamProgressError, StreamRead, StreamSinkMode,
    drive_blocking_stream,
};

use crate::{
    StreamFixtureError, StreamFixtureSink, StreamFixtureSource, StreamPattern, StreamPatternSource,
};

#[test]
fn fixtures_support_empty_chunks_short_writes_and_commit() {
    let chunks: &[&[u8]] = &[b"ab", b"", b"cde"];
    let source = StreamFixtureSource::new(chunks);
    let mut output = [0xa5_u8; 5];
    let sink = StreamFixtureSink::new(&mut output, 2);
    let limits = StreamLimits::new(5, 3, 3, 7, 1);
    let (Ok(mut source), Ok(mut sink), Ok(limits)) = (source, sink, limits) else {
        return;
    };
    let policy = StreamPolicy::new(
        StreamKind::FiniteDownload,
        StreamFraming::Declared(5),
        StreamSinkMode::Transactional,
        limits,
    );
    let Ok(policy) = policy else {
        return;
    };
    let mut scratch = [0_u8; 3];
    let mut outcome = StreamOutcome::new();
    let result = drive_blocking_stream(policy, &mut source, &mut sink, &mut scratch, &mut outcome);
    assert!(result.is_ok());
    assert_eq!(source.observations(), 4);
    assert_eq!(sink.bytes(), b"abcde");
    assert_eq!(sink.writes(), 3);
    assert!(sink.is_committed());
    assert_eq!(sink.aborted_with(), None);
}

#[test]
fn endless_empty_source_stops_at_the_zero_progress_bound() {
    let Ok(mut source) = StreamPatternSource::new(StreamPattern::EndlessEmpty) else {
        return;
    };
    let mut output = [0xa5_u8; 1];
    let Ok(mut sink) = StreamFixtureSink::new(&mut output, 1) else {
        return;
    };
    let Ok(limits) = StreamLimits::new(1, 1, 8, 8, 2) else {
        return;
    };
    let Ok(policy) = StreamPolicy::new(
        StreamKind::FiniteDownload,
        StreamFraming::Declared(1),
        StreamSinkMode::Transactional,
        limits,
    ) else {
        return;
    };
    let mut scratch = [0_u8; 1];
    let mut outcome = StreamOutcome::new();
    let result = drive_blocking_stream(policy, &mut source, &mut sink, &mut scratch, &mut outcome);
    assert!(matches!(
        result,
        Err(StreamExecutionError::Progress(
            StreamProgressError::ZeroProgressLimitExceeded
        ))
    ));
    assert_eq!(source.observations(), 3);
    assert_eq!(sink.bytes(), b"");
}

#[test]
fn alternating_pattern_is_exact_and_checks_scratch_size() {
    let Ok(mut source) = StreamPatternSource::new(StreamPattern::AlternatingEmptyData(b"xy"))
    else {
        return;
    };
    let mut scratch = [0_u8; 2];
    assert_eq!(source.read_chunk(&mut scratch), Ok(StreamRead::Chunk(0)));
    assert_eq!(source.read_chunk(&mut scratch), Ok(StreamRead::Chunk(2)));
    assert_eq!(scratch, *b"xy");
    assert_eq!(source.read_chunk(&mut scratch), Ok(StreamRead::Chunk(0)));

    let Ok(mut source) = StreamPatternSource::new(StreamPattern::AlternatingEmptyData(b"xy"))
    else {
        return;
    };
    let mut short = [0_u8; 1];
    assert_eq!(source.read_chunk(&mut short), Ok(StreamRead::Chunk(0)));
    assert_eq!(
        source.read_chunk(&mut short),
        Err(StreamFixtureError::SourceScratchTooSmall)
    );
    assert!(matches!(
        StreamPatternSource::new(StreamPattern::AlternatingEmptyData(b"")),
        Err(StreamFixtureError::EmptyPatternData)
    ));
}

#[test]
fn source_and_sink_faults_are_injected_at_exact_one_based_attempts() {
    let chunks: &[&[u8]] = &[b"a", b"b"];
    let Ok(source) = StreamFixtureSource::new(chunks) else {
        return;
    };
    assert!(matches!(
        source.with_fault_at_observation(0),
        Err(StreamFixtureError::ZeroFaultIndex)
    ));
    let Ok(mut source) =
        StreamFixtureSource::new(chunks).and_then(|source| source.with_fault_at_observation(2))
    else {
        return;
    };
    let mut scratch = [0_u8; 1];
    assert_eq!(source.read_chunk(&mut scratch), Ok(StreamRead::Chunk(1)));
    assert_eq!(
        source.read_chunk(&mut scratch),
        Err(StreamFixtureError::InjectedSourceFault)
    );

    let mut output = [0xa5_u8; 2];
    let Ok(sink) = StreamFixtureSink::new(&mut output, 1) else {
        return;
    };
    assert!(matches!(
        sink.with_fault_at_write(0),
        Err(StreamFixtureError::ZeroFaultIndex)
    ));
    let Ok(mut sink) =
        StreamFixtureSink::new(&mut output, 1).and_then(|sink| sink.with_fault_at_write(2))
    else {
        return;
    };
    assert_eq!(
        cloud_sdk::transport::BlockingStreamSink::write_chunk(&mut sink, b"a"),
        Ok(1)
    );
    assert_eq!(
        cloud_sdk::transport::BlockingStreamSink::write_chunk(&mut sink, b"b"),
        Err(StreamFixtureError::InjectedSinkFault)
    );
    assert_eq!(sink.bytes(), b"a");
}
