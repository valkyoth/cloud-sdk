//! Successful streaming completion metadata.

use super::{StreamProgress, StreamSinkMode};

/// Successful accounting completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StreamCompletion {
    pub(super) progress: StreamProgress,
    pub(super) sink_mode: StreamSinkMode,
}

impl StreamCompletion {
    /// Returns final actual counters.
    #[must_use]
    pub const fn progress(self) -> StreamProgress {
        self.progress
    }

    /// Reports whether the sink must commit its hidden transactional state.
    #[must_use]
    pub const fn requires_sink_commit(self) -> bool {
        matches!(self.sink_mode, StreamSinkMode::Transactional)
    }
}
