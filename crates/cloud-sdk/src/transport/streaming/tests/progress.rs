use super::super::{
    StreamAttempt, StreamCompletion, StreamFraming, StreamOutcome, StreamPartialState,
    StreamProgressError, StreamSinkMode, StreamState,
};
use super::{limits, policy};

fn chunk(attempt: &mut StreamAttempt<'_>, len: usize) -> Result<(), StreamProgressError> {
    attempt.begin_source_observation()?;
    attempt.begin_chunk(len)
}

fn accept(attempt: &mut StreamAttempt<'_>, len: usize) -> Result<(), StreamProgressError> {
    attempt.begin_sink_observation()?;
    attempt.advance(len)
}

fn wait(attempt: &mut StreamAttempt<'_>) -> Result<(), StreamProgressError> {
    attempt.begin_source_observation()?;
    attempt.observe_wait()
}

fn end(attempt: &mut StreamAttempt<'_>) -> Result<StreamCompletion, StreamProgressError> {
    attempt.begin_source_observation()?;
    attempt.finish()
}

#[test]
fn exact_length_counts_actual_partial_sink_progress() {
    let policy = policy(
        StreamFraming::Declared(4),
        StreamSinkMode::Transactional,
        limits(4, 4, 2, 5, 1),
    );
    let mut outcome = StreamOutcome::new();
    let mut attempt = StreamAttempt::new(policy, &mut outcome);
    assert_eq!(chunk(&mut attempt, 4), Ok(()));
    assert_eq!(attempt.progress().bytes(), 0);
    assert_eq!(attempt.progress().pending_bytes(), 4);
    assert_eq!(accept(&mut attempt, 1), Ok(()));
    assert_eq!(accept(&mut attempt, 3), Ok(()));
    assert_eq!(chunk(&mut attempt, 0), Ok(()));
    let completion = end(&mut attempt);
    assert!(completion.is_ok());
    if let Ok(completion) = completion {
        assert_eq!(completion.progress().bytes(), 4);
        assert_eq!(completion.progress().chunks(), 2);
        assert_eq!(completion.progress().observations(), 5);
        assert!(completion.requires_sink_commit());
    } else {
        unreachable!("stream completion fixture construction failed");
    }
    assert_eq!(attempt.commit_sink(), Ok(()));
    assert_eq!(
        attempt.commit_sink(),
        Err(StreamProgressError::AttemptClosed)
    );
    attempt.mark_failed();
    drop(attempt);
    assert_eq!(outcome.state(), StreamState::Complete);
    assert_eq!(outcome.partial_state(), StreamPartialState::Clean);
}

#[test]
fn pending_chunk_blocks_next_source_read_and_overreported_progress() {
    let policy = policy(
        StreamFraming::ExecutorOwned,
        StreamSinkMode::Direct,
        limits(8, 8, 2, 4, 1),
    );
    let mut outcome = StreamOutcome::new();
    let mut attempt = StreamAttempt::new(policy, &mut outcome);
    assert_eq!(chunk(&mut attempt, 2), Ok(()));
    assert_eq!(
        attempt.begin_source_observation(),
        Err(StreamProgressError::BackpressurePending)
    );
    assert_eq!(
        accept(&mut attempt, 1),
        Err(StreamProgressError::AttemptClosed)
    );
    drop(attempt);
    assert_eq!(outcome.state(), StreamState::Failed);

    let mut outcome = StreamOutcome::new();
    let mut attempt = StreamAttempt::new(policy, &mut outcome);
    assert_eq!(chunk(&mut attempt, 2), Ok(()));
    assert_eq!(
        accept(&mut attempt, 3),
        Err(StreamProgressError::InvalidSinkProgress)
    );
}

#[test]
fn declared_under_over_and_operation_byte_limits_fail_closed() {
    let exact = policy(
        StreamFraming::Declared(3),
        StreamSinkMode::Direct,
        limits(4, 4, 2, 4, 1),
    );
    let mut outcome = StreamOutcome::new();
    let mut under = StreamAttempt::new(exact, &mut outcome);
    assert_eq!(chunk(&mut under, 2), Ok(()));
    assert_eq!(accept(&mut under, 2), Ok(()));
    assert_eq!(
        end(&mut under),
        Err(StreamProgressError::DeclaredLengthMismatch)
    );

    let mut outcome = StreamOutcome::new();
    let mut over = StreamAttempt::new(exact, &mut outcome);
    assert_eq!(
        chunk(&mut over, 4),
        Err(StreamProgressError::DeclaredLengthExceeded)
    );

    let unknown = policy(
        StreamFraming::ExecutorOwned,
        StreamSinkMode::Direct,
        limits(3, 4, 2, 4, 1),
    );
    let mut outcome = StreamOutcome::new();
    let mut too_large = StreamAttempt::new(unknown, &mut outcome);
    assert_eq!(
        chunk(&mut too_large, 4),
        Err(StreamProgressError::ByteLimitExceeded)
    );
}

#[test]
fn every_chunk_observation_and_zero_progress_boundary_is_hard() {
    let chunk_limited = policy(
        StreamFraming::ExecutorOwned,
        StreamSinkMode::Direct,
        limits(8, 4, 1, 3, 1),
    );
    let mut outcome = StreamOutcome::new();
    let mut attempt = StreamAttempt::new(chunk_limited, &mut outcome);
    assert_eq!(chunk(&mut attempt, 1), Ok(()));
    assert_eq!(accept(&mut attempt, 1), Ok(()));
    assert_eq!(
        chunk(&mut attempt, 1),
        Err(StreamProgressError::ChunkLimitExceeded)
    );

    let observation_limited = policy(
        StreamFraming::ExecutorOwned,
        StreamSinkMode::Direct,
        limits(8, 4, 1, 2, 2),
    );
    let mut outcome = StreamOutcome::new();
    let mut attempt = StreamAttempt::new(observation_limited, &mut outcome);
    assert_eq!(chunk(&mut attempt, 2), Ok(()));
    assert_eq!(accept(&mut attempt, 1), Ok(()));
    assert_eq!(
        accept(&mut attempt, 1),
        Err(StreamProgressError::ObservationLimitExceeded)
    );

    let zero_limited = policy(
        StreamFraming::ExecutorOwned,
        StreamSinkMode::Direct,
        limits(8, 4, 3, 6, 1),
    );
    let mut outcome = StreamOutcome::new();
    let mut attempt = StreamAttempt::new(zero_limited, &mut outcome);
    assert_eq!(chunk(&mut attempt, 0), Ok(()));
    assert_eq!(
        chunk(&mut attempt, 0),
        Err(StreamProgressError::ZeroProgressLimitExceeded)
    );
}

#[test]
fn caller_cancelled_event_waits_are_observation_and_progress_bounded() {
    use super::super::{StreamKind, StreamLimits, StreamPolicy};

    let Ok(limits) = StreamLimits::new(8, 1, 1, 2, 1) else {
        unreachable!("stream-progress limits fixture construction failed");
    };
    let Ok(policy) = StreamPolicy::new(
        StreamKind::CallerCancelledEvent,
        StreamFraming::ExecutorOwned,
        StreamSinkMode::Direct,
        limits,
    ) else {
        unreachable!("stream-progress policy fixture construction failed");
    };
    let mut outcome = StreamOutcome::new();
    let mut attempt = StreamAttempt::new(policy, &mut outcome);
    assert_eq!(wait(&mut attempt), Ok(()));
    assert_eq!(
        wait(&mut attempt),
        Err(StreamProgressError::ZeroProgressLimitExceeded)
    );

    let mut outcome = StreamOutcome::new();
    let mut attempt = StreamAttempt::new(policy, &mut outcome);
    assert_eq!(
        end(&mut attempt),
        Err(StreamProgressError::UnexpectedEventEnd)
    );
}

#[test]
fn alternating_empty_and_data_resets_only_after_actual_acceptance() {
    let policy = policy(
        StreamFraming::Declared(2),
        StreamSinkMode::Direct,
        limits(2, 1, 4, 8, 1),
    );
    let mut outcome = StreamOutcome::new();
    let mut attempt = StreamAttempt::new(policy, &mut outcome);
    assert_eq!(chunk(&mut attempt, 0), Ok(()));
    assert_eq!(chunk(&mut attempt, 1), Ok(()));
    assert_eq!(accept(&mut attempt, 1), Ok(()));
    assert_eq!(chunk(&mut attempt, 0), Ok(()));
    assert_eq!(chunk(&mut attempt, 1), Ok(()));
    assert_eq!(accept(&mut attempt, 1), Ok(()));
    assert!(end(&mut attempt).is_ok());
    assert_eq!(attempt.commit_sink(), Ok(()));
}

#[test]
fn drop_records_cancellation_and_transactional_or_dirty_partial_state() {
    for (mode, expected) in [
        (
            StreamSinkMode::Transactional,
            StreamPartialState::RollbackRequired,
        ),
        (StreamSinkMode::Direct, StreamPartialState::Dirty),
    ] {
        let policy = policy(StreamFraming::ExecutorOwned, mode, limits(8, 4, 2, 4, 1));
        let mut outcome = StreamOutcome::new();
        {
            let mut attempt = StreamAttempt::new(policy, &mut outcome);
            assert_eq!(chunk(&mut attempt, 2), Ok(()));
            assert_eq!(accept(&mut attempt, 1), Ok(()));
        }
        assert_eq!(outcome.state(), StreamState::Cancelled);
        assert_eq!(outcome.partial_state(), expected);
        assert_eq!(outcome.progress().bytes(), 1);
    }

    let policy = policy(
        StreamFraming::ExecutorOwned,
        StreamSinkMode::Direct,
        limits(8, 4, 2, 4, 1),
    );
    let mut outcome = StreamOutcome::new();
    drop(StreamAttempt::new(policy, &mut outcome));
    assert_eq!(outcome.partial_state(), StreamPartialState::Clean);
}

#[test]
fn sink_observation_makes_partial_state_sticky_before_progress() {
    for (mode, expected) in [
        (
            StreamSinkMode::Transactional,
            StreamPartialState::RollbackRequired,
        ),
        (StreamSinkMode::Direct, StreamPartialState::Dirty),
    ] {
        let policy = policy(StreamFraming::ExecutorOwned, mode, limits(8, 4, 1, 3, 1));
        let mut outcome = StreamOutcome::new();
        {
            let mut attempt = StreamAttempt::new(policy, &mut outcome);
            assert_eq!(chunk(&mut attempt, 1), Ok(()));
            assert_eq!(attempt.begin_sink_observation(), Ok(()));
        }
        assert_eq!(outcome.state(), StreamState::Cancelled);
        assert_eq!(outcome.partial_state(), expected);
        assert_eq!(outcome.progress().bytes(), 0);
    }
}
