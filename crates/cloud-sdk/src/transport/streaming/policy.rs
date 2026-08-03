//! Complete per-operation streaming policy.

/// Global ceiling for one stream's actual transferred bytes.
pub const MAX_STREAM_BYTES: u64 = 1_125_899_906_842_624;
/// Global ceiling for one source chunk.
pub const MAX_STREAM_CHUNK_BYTES: usize = 16_777_216;
/// Global ceiling for chunks observed in one stream attempt.
pub const MAX_STREAM_CHUNKS: u32 = 16_777_216;
/// Global ceiling for source, sink, and waiting observations in one attempt.
pub const MAX_STREAM_OBSERVATIONS: u32 = 67_108_864;
/// Global ceiling for consecutive observations that transfer no bytes.
pub const MAX_CONSECUTIVE_ZERO_PROGRESS: u16 = 4_096;

/// Streaming operation shape.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StreamKind {
    /// One finite request-body upload.
    FiniteUpload,
    /// One finite response-body download.
    FiniteDownload,
    /// An event download whose lifetime is bounded by caller cancellation and
    /// the observation policy.
    CallerCancelledEvent,
}

/// Wire framing responsibility.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StreamFraming {
    /// The exact stream length is known before execution.
    Declared(u64),
    /// The executor owns framing for an explicitly unknown-length stream.
    ExecutorOwned,
}

/// Visibility of accepted bytes before successful completion.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StreamSinkMode {
    /// Accepted bytes remain hidden and the sink can roll them back on abort.
    Transactional,
    /// Accepted bytes may be externally visible and cannot be rolled back.
    Direct,
}

/// Invalid hard streaming limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamLimitsError {
    /// The byte limit is zero.
    ByteLimitZero,
    /// The byte limit exceeds the global ceiling.
    ByteLimitTooLarge,
    /// The chunk-size limit is zero.
    ChunkBytesZero,
    /// The chunk-size limit exceeds the global ceiling.
    ChunkBytesTooLarge,
    /// The chunk-count limit is zero.
    ChunkLimitZero,
    /// The chunk-count limit exceeds the global ceiling.
    ChunkLimitTooLarge,
    /// The observation limit is zero or cannot admit every allowed chunk.
    ObservationLimitTooSmall,
    /// The observation limit exceeds the global ceiling.
    ObservationLimitTooLarge,
    /// The zero-progress tolerance exceeds its global or observation bound.
    ZeroProgressLimitTooLarge,
}

impl_static_error!(StreamLimitsError,
    Self::ByteLimitZero => "stream byte limit is zero",
    Self::ByteLimitTooLarge => "stream byte limit exceeds the global ceiling",
    Self::ChunkBytesZero => "stream chunk-size limit is zero",
    Self::ChunkBytesTooLarge => "stream chunk-size limit exceeds the global ceiling",
    Self::ChunkLimitZero => "stream chunk limit is zero",
    Self::ChunkLimitTooLarge => "stream chunk limit exceeds the global ceiling",
    Self::ObservationLimitTooSmall => "stream observation limit cannot admit every chunk",
    Self::ObservationLimitTooLarge => "stream observation limit exceeds the global ceiling",
    Self::ZeroProgressLimitTooLarge => "stream zero-progress limit is too large",
);

/// Hard byte, chunk, observation, and zero-progress limits.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StreamLimits {
    max_bytes: u64,
    max_chunk_bytes: usize,
    max_chunks: u32,
    max_observations: u32,
    max_consecutive_zero_progress: u16,
}

impl StreamLimits {
    /// Creates complete nonzero limits beneath global ceilings.
    pub const fn new(
        max_bytes: u64,
        max_chunk_bytes: usize,
        max_chunks: u32,
        max_observations: u32,
        max_consecutive_zero_progress: u16,
    ) -> Result<Self, StreamLimitsError> {
        if max_bytes == 0 {
            return Err(StreamLimitsError::ByteLimitZero);
        }
        if max_bytes > MAX_STREAM_BYTES {
            return Err(StreamLimitsError::ByteLimitTooLarge);
        }
        if max_chunk_bytes == 0 {
            return Err(StreamLimitsError::ChunkBytesZero);
        }
        if max_chunk_bytes > MAX_STREAM_CHUNK_BYTES {
            return Err(StreamLimitsError::ChunkBytesTooLarge);
        }
        if max_chunks == 0 {
            return Err(StreamLimitsError::ChunkLimitZero);
        }
        if max_chunks > MAX_STREAM_CHUNKS {
            return Err(StreamLimitsError::ChunkLimitTooLarge);
        }
        if max_observations < max_chunks {
            return Err(StreamLimitsError::ObservationLimitTooSmall);
        }
        if max_observations > MAX_STREAM_OBSERVATIONS {
            return Err(StreamLimitsError::ObservationLimitTooLarge);
        }
        if max_consecutive_zero_progress > MAX_CONSECUTIVE_ZERO_PROGRESS
            || max_consecutive_zero_progress as u32 > max_observations
        {
            return Err(StreamLimitsError::ZeroProgressLimitTooLarge);
        }
        Ok(Self {
            max_bytes,
            max_chunk_bytes,
            max_chunks,
            max_observations,
            max_consecutive_zero_progress,
        })
    }

    /// Returns the actual-byte ceiling.
    #[must_use]
    pub const fn max_bytes(self) -> u64 {
        self.max_bytes
    }

    /// Returns the per-chunk byte ceiling.
    #[must_use]
    pub const fn max_chunk_bytes(self) -> usize {
        self.max_chunk_bytes
    }

    /// Returns the source-chunk ceiling.
    #[must_use]
    pub const fn max_chunks(self) -> u32 {
        self.max_chunks
    }

    /// Returns the source, sink, and waiting observation ceiling.
    #[must_use]
    pub const fn max_observations(self) -> u32 {
        self.max_observations
    }

    /// Returns the admitted consecutive zero-progress observations.
    #[must_use]
    pub const fn max_consecutive_zero_progress(self) -> u16 {
        self.max_consecutive_zero_progress
    }
}

/// Incoherent stream policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamPolicyError {
    /// The declared length exceeds the operation byte limit.
    DeclaredLengthTooLarge,
    /// Event streams require executor-owned unknown-length framing.
    EventRequiresExecutorFraming,
}

impl_static_error!(StreamPolicyError,
    Self::DeclaredLengthTooLarge => "declared stream length exceeds the operation limit",
    Self::EventRequiresExecutorFraming => "event stream requires executor-owned framing",
);

/// Complete stream behavior fixed before the first observation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StreamPolicy {
    kind: StreamKind,
    framing: StreamFraming,
    sink_mode: StreamSinkMode,
    limits: StreamLimits,
}

impl StreamPolicy {
    /// Creates a coherent stream policy without permissive defaults.
    pub const fn new(
        kind: StreamKind,
        framing: StreamFraming,
        sink_mode: StreamSinkMode,
        limits: StreamLimits,
    ) -> Result<Self, StreamPolicyError> {
        if let StreamFraming::Declared(length) = framing
            && length > limits.max_bytes
        {
            return Err(StreamPolicyError::DeclaredLengthTooLarge);
        }
        if matches!(kind, StreamKind::CallerCancelledEvent)
            && !matches!(framing, StreamFraming::ExecutorOwned)
        {
            return Err(StreamPolicyError::EventRequiresExecutorFraming);
        }
        Ok(Self {
            kind,
            framing,
            sink_mode,
            limits,
        })
    }

    /// Returns the operation shape.
    #[must_use]
    pub const fn kind(self) -> StreamKind {
        self.kind
    }

    /// Returns the explicit wire framing policy.
    #[must_use]
    pub const fn framing(self) -> StreamFraming {
        self.framing
    }

    /// Returns partial-byte visibility policy.
    #[must_use]
    pub const fn sink_mode(self) -> StreamSinkMode {
        self.sink_mode
    }

    /// Returns all hard limits.
    #[must_use]
    pub const fn limits(self) -> StreamLimits {
        self.limits
    }
}

pub(super) const fn partial_state(mode: StreamSinkMode, bytes: u64) -> super::StreamPartialState {
    if bytes == 0 {
        super::StreamPartialState::Clean
    } else {
        match mode {
            StreamSinkMode::Transactional => super::StreamPartialState::RollbackRequired,
            StreamSinkMode::Direct => super::StreamPartialState::Dirty,
        }
    }
}
