//! Exact bounded source identity for explicit streaming replay.

use core::fmt;

/// Maximum exact source-version identity length.
pub const MAX_STREAM_SOURCE_ID_BYTES: usize = 256;

/// Invalid source identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamSourceIdError {
    /// Source identities cannot be empty.
    Empty,
    /// The identity exceeds [`MAX_STREAM_SOURCE_ID_BYTES`].
    TooLong,
}

impl_static_error!(StreamSourceIdError,
    Self::Empty => "stream source identity is empty",
    Self::TooLong => "stream source identity is too long",
);

/// Exact source version supplied by the source owner.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StreamSourceId<'a>(&'a [u8]);

impl<'a> StreamSourceId<'a> {
    /// Admits one nonempty bounded exact identity.
    pub const fn new(value: &'a [u8]) -> Result<Self, StreamSourceIdError> {
        if value.is_empty() {
            return Err(StreamSourceIdError::Empty);
        }
        if value.len() > MAX_STREAM_SOURCE_ID_BYTES {
            return Err(StreamSourceIdError::TooLong);
        }
        Ok(Self(value))
    }

    /// Returns exact identity bytes for caller-controlled comparison or hashing.
    #[must_use]
    pub const fn as_bytes(self) -> &'a [u8] {
        self.0
    }
}

impl fmt::Debug for StreamSourceId<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("StreamSourceId([redacted])")
    }
}

/// Whether a streaming body can be reproduced for another attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamReplayability<'a> {
    /// The source cannot guarantee byte-for-byte reproduction.
    NotReplayable,
    /// The source promises stable bytes while this exact version remains current.
    Replayable(StreamSourceId<'a>),
}

/// Streaming replay rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamReplayError {
    /// Either attempt uses a non-replayable source.
    NonReplayable,
    /// The source version changed between attempts.
    SourceChanged,
}

impl_static_error!(StreamReplayError,
    Self::NonReplayable => "stream source is not replayable",
    Self::SourceChanged => "stream source changed between attempts",
);

/// Validates that a later attempt uses the same explicit replayable source.
pub fn validate_stream_replay(
    initial: StreamReplayability<'_>,
    replay: StreamReplayability<'_>,
) -> Result<(), StreamReplayError> {
    let (StreamReplayability::Replayable(initial), StreamReplayability::Replayable(replay)) =
        (initial, replay)
    else {
        return Err(StreamReplayError::NonReplayable);
    };
    if initial.as_bytes() != replay.as_bytes() {
        return Err(StreamReplayError::SourceChanged);
    }
    Ok(())
}
