mod io;
mod policy;
mod progress;
mod replay;

use super::{StreamFraming, StreamKind, StreamLimits, StreamPolicy, StreamSinkMode};

fn limits(
    bytes: u64,
    chunk_bytes: usize,
    chunks: u32,
    observations: u32,
    zero_progress: u16,
) -> StreamLimits {
    let Ok(limits) = StreamLimits::new(bytes, chunk_bytes, chunks, observations, zero_progress)
    else {
        unreachable!();
    };
    limits
}

fn policy(framing: StreamFraming, sink_mode: StreamSinkMode, limits: StreamLimits) -> StreamPolicy {
    let Ok(policy) = StreamPolicy::new(StreamKind::FiniteDownload, framing, sink_mode, limits)
    else {
        unreachable!();
    };
    policy
}
