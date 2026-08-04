use super::super::{
    MAX_STREAM_SOURCE_ID_BYTES, StreamReplayError, StreamReplayability, StreamSourceId,
    StreamSourceIdError, validate_stream_replay,
};

#[test]
fn source_id_is_bounded_exact_and_redacted() {
    assert_eq!(StreamSourceId::new(b""), Err(StreamSourceIdError::Empty));
    let oversized = [0_u8; MAX_STREAM_SOURCE_ID_BYTES + 1];
    assert_eq!(
        StreamSourceId::new(&oversized),
        Err(StreamSourceIdError::TooLong)
    );
    let Ok(identity) = StreamSourceId::new(b"version-1") else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(identity.as_bytes(), b"version-1");
    assert_eq!(identity, identity);
}

#[test]
fn replay_requires_both_replayability_and_exact_source_version() {
    let (Ok(first), Ok(same), Ok(changed)) = (
        StreamSourceId::new(b"source-v1"),
        StreamSourceId::new(b"source-v1"),
        StreamSourceId::new(b"source-v2"),
    ) else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(
        validate_stream_replay(
            StreamReplayability::Replayable(first),
            StreamReplayability::Replayable(same),
        ),
        Ok(())
    );
    assert_eq!(
        validate_stream_replay(
            StreamReplayability::Replayable(first),
            StreamReplayability::Replayable(changed),
        ),
        Err(StreamReplayError::SourceChanged)
    );
    assert_eq!(
        validate_stream_replay(
            StreamReplayability::Replayable(first),
            StreamReplayability::NotReplayable,
        ),
        Err(StreamReplayError::NonReplayable)
    );
}
