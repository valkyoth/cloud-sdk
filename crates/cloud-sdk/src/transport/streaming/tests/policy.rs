use super::super::{
    MAX_CONSECUTIVE_ZERO_PROGRESS, MAX_STREAM_BYTES, MAX_STREAM_CHUNK_BYTES, MAX_STREAM_CHUNKS,
    MAX_STREAM_OBSERVATIONS, StreamFraming, StreamKind, StreamLimits, StreamLimitsError,
    StreamPolicy, StreamPolicyError, StreamSinkMode,
};

#[test]
fn exact_global_limits_are_admitted_and_excess_is_rejected() {
    let exact = StreamLimits::new(
        MAX_STREAM_BYTES,
        MAX_STREAM_CHUNK_BYTES,
        MAX_STREAM_CHUNKS,
        MAX_STREAM_OBSERVATIONS,
        MAX_CONSECUTIVE_ZERO_PROGRESS,
    );
    assert!(exact.is_ok());
    assert_eq!(
        StreamLimits::new(MAX_STREAM_BYTES.saturating_add(1), 1, 1, 1, 0,),
        Err(StreamLimitsError::ByteLimitTooLarge)
    );
    assert_eq!(
        StreamLimits::new(1, MAX_STREAM_CHUNK_BYTES.saturating_add(1), 1, 1, 0,),
        Err(StreamLimitsError::ChunkBytesTooLarge)
    );
    assert_eq!(
        StreamLimits::new(1, 1, MAX_STREAM_CHUNKS.saturating_add(1), 1, 0),
        Err(StreamLimitsError::ChunkLimitTooLarge)
    );
    assert_eq!(
        StreamLimits::new(1, 1, 1, MAX_STREAM_OBSERVATIONS.saturating_add(1), 0,),
        Err(StreamLimitsError::ObservationLimitTooLarge)
    );
}

#[test]
fn zero_and_incoherent_limits_are_rejected_independently() {
    assert_eq!(
        StreamLimits::new(0, 1, 1, 1, 0),
        Err(StreamLimitsError::ByteLimitZero)
    );
    assert_eq!(
        StreamLimits::new(1, 0, 1, 1, 0),
        Err(StreamLimitsError::ChunkBytesZero)
    );
    assert_eq!(
        StreamLimits::new(1, 1, 0, 1, 0),
        Err(StreamLimitsError::ChunkLimitZero)
    );
    assert_eq!(
        StreamLimits::new(1, 1, 2, 1, 0),
        Err(StreamLimitsError::ObservationLimitTooSmall)
    );
    assert_eq!(
        StreamLimits::new(1, 1, 1, 1, 2),
        Err(StreamLimitsError::ZeroProgressLimitTooLarge)
    );
}

#[test]
fn policy_makes_length_and_event_framing_explicit() {
    let Ok(limits) = StreamLimits::new(8, 8, 2, 4, 1) else {
        return;
    };
    assert_eq!(
        StreamPolicy::new(
            StreamKind::FiniteUpload,
            StreamFraming::Declared(9),
            StreamSinkMode::Direct,
            limits,
        ),
        Err(StreamPolicyError::DeclaredLengthTooLarge)
    );
    assert_eq!(
        StreamPolicy::new(
            StreamKind::CallerCancelledEvent,
            StreamFraming::Declared(8),
            StreamSinkMode::Direct,
            limits,
        ),
        Err(StreamPolicyError::EventRequiresExecutorFraming)
    );
    let event = StreamPolicy::new(
        StreamKind::CallerCancelledEvent,
        StreamFraming::ExecutorOwned,
        StreamSinkMode::Transactional,
        limits,
    );
    assert!(event.is_ok());
    if let Ok(event) = event {
        assert_eq!(event.kind(), StreamKind::CallerCancelledEvent);
        assert_eq!(event.framing(), StreamFraming::ExecutorOwned);
        assert_eq!(event.sink_mode(), StreamSinkMode::Transactional);
        assert_eq!(event.limits(), limits);
    }
}
