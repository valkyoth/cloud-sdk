//! Transactional byte, chunk, observation, and backpressure accounting.

use super::policy::partial_state;
use super::{StreamCompletion, StreamFraming, StreamKind, StreamPolicy};

/// Public nonsensitive streaming counters.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StreamProgress {
    bytes: u64,
    chunks: u32,
    observations: u32,
    consecutive_zero_progress: u16,
    pending_bytes: usize,
}

impl StreamProgress {
    /// Returns actual bytes accepted by the sink.
    #[must_use]
    pub const fn bytes(self) -> u64 {
        self.bytes
    }

    /// Returns source chunks admitted, including empty chunks.
    #[must_use]
    pub const fn chunks(self) -> u32 {
        self.chunks
    }

    /// Returns all source, sink, and waiting observations.
    #[must_use]
    pub const fn observations(self) -> u32 {
        self.observations
    }

    /// Returns the current zero-progress streak.
    #[must_use]
    pub const fn consecutive_zero_progress(self) -> u16 {
        self.consecutive_zero_progress
    }

    /// Returns bytes that must be accepted before another chunk may begin.
    #[must_use]
    pub const fn pending_bytes(self) -> usize {
        self.pending_bytes
    }
}

/// Final lifecycle state recorded even when an asynchronous attempt is dropped.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StreamState {
    /// No attempt has borrowed the outcome slot yet.
    NotStarted,
    /// An attempt currently owns the slot.
    Active,
    /// Length and all hard limits were satisfied.
    Complete,
    /// The caller or executor cancelled before completion.
    Cancelled,
    /// A source, sink, or policy failure terminated the attempt.
    Failed,
}

/// Visibility and cleanup requirement after incomplete transfer.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StreamPartialState {
    /// No sink write was attempted.
    Clean,
    /// A transactional write was attempted and must be rolled back.
    RollbackRequired,
    /// A direct sink write may already have produced an external effect.
    Dirty,
}

/// Attempt outcome stored in caller-owned state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StreamOutcome {
    state: StreamState,
    progress: StreamProgress,
    partial: StreamPartialState,
}

impl StreamOutcome {
    /// Creates an untouched outcome slot.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: StreamState::NotStarted,
            progress: StreamProgress {
                bytes: 0,
                chunks: 0,
                observations: 0,
                consecutive_zero_progress: 0,
                pending_bytes: 0,
            },
            partial: StreamPartialState::Clean,
        }
    }

    /// Returns the final or current lifecycle state.
    #[must_use]
    pub const fn state(self) -> StreamState {
        self.state
    }

    /// Returns counters captured when the outcome was last updated.
    #[must_use]
    pub const fn progress(self) -> StreamProgress {
        self.progress
    }

    /// Returns incomplete-state visibility and cleanup requirements.
    #[must_use]
    pub const fn partial_state(self) -> StreamPartialState {
        self.partial
    }
}

impl Default for StreamOutcome {
    fn default() -> Self {
        Self::new()
    }
}

/// Stream accounting or lifecycle failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamProgressError {
    /// The attempt already completed or failed.
    AttemptClosed,
    /// A new source chunk was offered before the prior chunk was consumed.
    BackpressurePending,
    /// A sink advance was reported without pending bytes.
    NoPendingChunk,
    /// Sink progress was reported without a preflight sink observation.
    NoSinkObservation,
    /// A second sink observation was started before classifying the first.
    SinkObservationPending,
    /// A source result was reported without a preflight observation.
    NoSourceObservation,
    /// A second source observation was started before classifying the first.
    SourceObservationPending,
    /// A sink claimed to accept more than the pending chunk remainder.
    InvalidSinkProgress,
    /// One chunk exceeds the per-operation chunk-size limit.
    ChunkTooLarge,
    /// The stream exceeded its chunk budget.
    ChunkLimitExceeded,
    /// The stream exceeded its observation budget.
    ObservationLimitExceeded,
    /// Actual or offered bytes exceed the operation limit.
    ByteLimitExceeded,
    /// Offered bytes exceed the declared wire length.
    DeclaredLengthExceeded,
    /// End-of-stream actual bytes differ from the declared length.
    DeclaredLengthMismatch,
    /// Caller-cancelled event streams cannot end as finite streams.
    UnexpectedEventEnd,
    /// Consecutive observations made no progress beyond policy tolerance.
    ZeroProgressLimitExceeded,
    /// Counter arithmetic or platform length conversion overflowed.
    ArithmeticOverflow,
}

impl_static_error!(StreamProgressError,
    Self::AttemptClosed => "stream attempt is closed",
    Self::BackpressurePending => "stream chunk remains pending",
    Self::NoPendingChunk => "stream has no pending chunk",
    Self::NoSinkObservation => "stream sink observation was not preflighted",
    Self::SinkObservationPending => "stream sink observation remains unclassified",
    Self::NoSourceObservation => "stream source observation was not preflighted",
    Self::SourceObservationPending => "stream source observation remains unclassified",
    Self::InvalidSinkProgress => "stream sink reported invalid progress",
    Self::ChunkTooLarge => "stream chunk exceeds the operation limit",
    Self::ChunkLimitExceeded => "stream chunk budget is exhausted",
    Self::ObservationLimitExceeded => "stream observation budget is exhausted",
    Self::ByteLimitExceeded => "stream byte budget is exhausted",
    Self::DeclaredLengthExceeded => "stream exceeds its declared length",
    Self::DeclaredLengthMismatch => "stream length differs from its declaration",
    Self::UnexpectedEventEnd => "caller-cancelled event stream ended unexpectedly",
    Self::ZeroProgressLimitExceeded => "stream made no progress beyond its tolerance",
    Self::ArithmeticOverflow => "stream accounting overflowed",
);

/// SDK-owned accounting attempt over caller-owned outcome state.
///
/// Only one source chunk may be outstanding. Dropping an active attempt marks
/// it cancelled and records whether a sink write was never attempted, needs
/// transactional rollback, or may already be externally visible.
pub struct StreamAttempt<'outcome> {
    policy: StreamPolicy,
    progress: StreamProgress,
    outcome: &'outcome mut StreamOutcome,
    closed: bool,
    failed: bool,
    end_validated: bool,
    source_observation_pending: bool,
    sink_observation_pending: bool,
    sink_write_attempted: bool,
}

impl<'outcome> StreamAttempt<'outcome> {
    /// Starts one attempt and marks the supplied outcome slot active.
    #[must_use]
    pub fn new(policy: StreamPolicy, outcome: &'outcome mut StreamOutcome) -> Self {
        *outcome = StreamOutcome {
            state: StreamState::Active,
            progress: StreamProgress::default(),
            partial: StreamPartialState::Clean,
        };
        Self {
            policy,
            progress: StreamProgress::default(),
            outcome,
            closed: false,
            failed: false,
            end_validated: false,
            source_observation_pending: false,
            sink_observation_pending: false,
            sink_write_attempted: false,
        }
    }

    /// Returns current counters.
    #[must_use]
    pub const fn progress(&self) -> StreamProgress {
        self.progress
    }

    /// Reserves one source observation before external source code is called.
    ///
    /// The returned source result must be classified with [`Self::begin_chunk`],
    /// [`Self::observe_wait`], or [`Self::finish`].
    pub fn begin_source_observation(&mut self) -> Result<(), StreamProgressError> {
        self.ensure_open()?;
        if self.progress.pending_bytes != 0 {
            return self.reject(StreamProgressError::BackpressurePending);
        }
        if self.source_observation_pending {
            return self.reject(StreamProgressError::SourceObservationPending);
        }
        if self.sink_observation_pending {
            return self.reject(StreamProgressError::SinkObservationPending);
        }
        let observations = self.next_observation()?;
        self.progress.observations = observations;
        self.source_observation_pending = true;
        self.sync_active();
        Ok(())
    }

    /// Classifies a preflight source observation as one complete chunk.
    ///
    /// The next chunk is rejected until [`Self::advance`] accepts every pending
    /// byte, which makes backpressure deterministic.
    pub fn begin_chunk(&mut self, len: usize) -> Result<(), StreamProgressError> {
        self.ensure_open()?;
        if !self.source_observation_pending {
            return self.reject(StreamProgressError::NoSourceObservation);
        }
        if self.progress.pending_bytes != 0 {
            return self.reject(StreamProgressError::BackpressurePending);
        }
        let limits = self.policy.limits();
        if len > limits.max_chunk_bytes() {
            return self.reject(StreamProgressError::ChunkTooLarge);
        }
        let Some(chunks) = self.progress.chunks.checked_add(1) else {
            return self.reject(StreamProgressError::ArithmeticOverflow);
        };
        if chunks > limits.max_chunks() {
            return self.reject(StreamProgressError::ChunkLimitExceeded);
        }
        let Ok(offered) = u64::try_from(len) else {
            return self.reject(StreamProgressError::ArithmeticOverflow);
        };
        let Some(projected) = self.progress.bytes.checked_add(offered) else {
            return self.reject(StreamProgressError::ArithmeticOverflow);
        };
        if projected > limits.max_bytes() {
            return self.reject(StreamProgressError::ByteLimitExceeded);
        }
        if let StreamFraming::Declared(declared) = self.policy.framing()
            && projected > declared
        {
            return self.reject(StreamProgressError::DeclaredLengthExceeded);
        }
        let zero = if len == 0 {
            self.next_zero_progress()?
        } else {
            self.progress.consecutive_zero_progress
        };
        self.progress.chunks = chunks;
        self.progress.consecutive_zero_progress = zero;
        self.progress.pending_bytes = len;
        self.source_observation_pending = false;
        self.sync_active();
        Ok(())
    }

    /// Reserves one sink observation and conservatively records a write attempt
    /// before external sink code is called.
    pub fn begin_sink_observation(&mut self) -> Result<(), StreamProgressError> {
        self.ensure_open()?;
        if self.source_observation_pending {
            return self.reject(StreamProgressError::SourceObservationPending);
        }
        if self.sink_observation_pending {
            return self.reject(StreamProgressError::SinkObservationPending);
        }
        if self.progress.pending_bytes == 0 {
            return self.reject(StreamProgressError::NoPendingChunk);
        }
        let observations = self.next_observation()?;
        self.progress.observations = observations;
        self.sink_write_attempted = true;
        self.sink_observation_pending = true;
        self.sync_active();
        Ok(())
    }

    /// Classifies a preflight sink observation with actual accepted bytes.
    pub fn advance(&mut self, accepted: usize) -> Result<(), StreamProgressError> {
        self.ensure_open()?;
        if !self.sink_observation_pending {
            return self.reject(StreamProgressError::NoSinkObservation);
        }
        if self.progress.pending_bytes == 0 {
            return self.reject(StreamProgressError::NoPendingChunk);
        }
        if accepted > self.progress.pending_bytes {
            return self.reject(StreamProgressError::InvalidSinkProgress);
        }
        let Ok(accepted_u64) = u64::try_from(accepted) else {
            return self.reject(StreamProgressError::ArithmeticOverflow);
        };
        let Some(bytes) = self.progress.bytes.checked_add(accepted_u64) else {
            return self.reject(StreamProgressError::ArithmeticOverflow);
        };
        let Some(pending) = self.progress.pending_bytes.checked_sub(accepted) else {
            return self.reject(StreamProgressError::ArithmeticOverflow);
        };
        let zero = if accepted == 0 {
            self.next_zero_progress()?
        } else {
            0
        };
        self.progress.bytes = bytes;
        self.progress.pending_bytes = pending;
        self.progress.consecutive_zero_progress = zero;
        self.sink_observation_pending = false;
        self.sync_active();
        Ok(())
    }

    /// Classifies a preflight source observation that produced no chunk.
    pub fn observe_wait(&mut self) -> Result<(), StreamProgressError> {
        self.ensure_open()?;
        if !self.source_observation_pending {
            return self.reject(StreamProgressError::NoSourceObservation);
        }
        let zero = self.next_zero_progress()?;
        self.progress.consecutive_zero_progress = zero;
        self.source_observation_pending = false;
        self.sync_active();
        Ok(())
    }

    /// Marks an external source or sink error so drop records failure, not cancellation.
    pub fn mark_failed(&mut self) {
        if self.closed {
            return;
        }
        self.failed = true;
        self.sync(StreamState::Failed);
    }

    /// Classifies a preflight source observation as end and validates length.
    pub fn finish(&mut self) -> Result<StreamCompletion, StreamProgressError> {
        self.ensure_open()?;
        if !self.source_observation_pending {
            return self.reject(StreamProgressError::NoSourceObservation);
        }
        if matches!(self.policy.kind(), StreamKind::CallerCancelledEvent) {
            return self.reject(StreamProgressError::UnexpectedEventEnd);
        }
        if self.progress.pending_bytes != 0 {
            return self.reject(StreamProgressError::BackpressurePending);
        }
        if let StreamFraming::Declared(declared) = self.policy.framing()
            && self.progress.bytes != declared
        {
            return self.reject(StreamProgressError::DeclaredLengthMismatch);
        }
        self.source_observation_pending = false;
        self.end_validated = true;
        self.sync_active();
        Ok(StreamCompletion {
            progress: self.progress,
            sink_mode: self.policy.sink_mode(),
        })
    }

    /// Records successful sink commitment after [`Self::finish`] validated end.
    pub fn commit_sink(&mut self) -> Result<(), StreamProgressError> {
        if self.closed || self.failed {
            return Err(StreamProgressError::AttemptClosed);
        }
        if !self.end_validated {
            return self.reject(StreamProgressError::AttemptClosed);
        }
        self.closed = true;
        self.sync(StreamState::Complete);
        Ok(())
    }

    fn ensure_open(&mut self) -> Result<(), StreamProgressError> {
        if self.closed || self.failed || self.end_validated {
            return Err(StreamProgressError::AttemptClosed);
        }
        Ok(())
    }

    fn next_observation(&mut self) -> Result<u32, StreamProgressError> {
        let Some(next) = self.progress.observations.checked_add(1) else {
            return self.reject(StreamProgressError::ArithmeticOverflow);
        };
        if next > self.policy.limits().max_observations() {
            return self.reject(StreamProgressError::ObservationLimitExceeded);
        }
        Ok(next)
    }

    fn next_zero_progress(&mut self) -> Result<u16, StreamProgressError> {
        let Some(next) = self.progress.consecutive_zero_progress.checked_add(1) else {
            return self.reject(StreamProgressError::ArithmeticOverflow);
        };
        if next > self.policy.limits().max_consecutive_zero_progress() {
            return self.reject(StreamProgressError::ZeroProgressLimitExceeded);
        }
        Ok(next)
    }

    fn reject<T>(&mut self, error: StreamProgressError) -> Result<T, StreamProgressError> {
        self.failed = true;
        self.sync(StreamState::Failed);
        Err(error)
    }

    fn sync_active(&mut self) {
        self.sync(StreamState::Active);
    }

    fn sync(&mut self, state: StreamState) {
        let partial = if matches!(state, StreamState::Complete) || !self.sink_write_attempted {
            StreamPartialState::Clean
        } else {
            partial_state(self.policy.sink_mode(), true)
        };
        *self.outcome = StreamOutcome {
            state,
            progress: self.progress,
            partial,
        };
    }
}

impl Drop for StreamAttempt<'_> {
    fn drop(&mut self) {
        if !self.closed {
            let state = if self.failed {
                StreamState::Failed
            } else {
                StreamState::Cancelled
            };
            self.sync(state);
        }
    }
}
