//! Bounded incremental JSON decoding for large provider responses.
//!
//! This API validates one JSON document across arbitrary input chunks and
//! emits borrowed events without constructing a complete JSON tree. Callers
//! must observe [`IncrementalJsonProgress::Complete`] before treating the
//! document as fully validated. A visitor-requested stop is deliberately a
//! separate terminal outcome.

mod decoder;
mod event;
mod lexical;
mod limits;
mod state;

pub use decoder::IncrementalJsonDecoder;
pub use event::{
    IncrementalJsonError, IncrementalJsonEvent, IncrementalJsonProgress, IncrementalJsonVisitor,
    VisitControl,
};
pub use limits::{IncrementalJsonLimits, IncrementalJsonLimitsError};

#[cfg(test)]
mod tests;
