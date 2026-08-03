use cloud_sdk::transport::{
    StreamFraming, StreamKind, StreamLimits, StreamOutcome, StreamPolicy, StreamSinkMode,
    drive_blocking_stream,
};

use crate::{StreamFixtureSink, StreamFixtureSource};

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
