//! Bounded provider-neutral asynchronous task and event response models.

mod model;
mod value;

pub use model::{
    AsyncEvent, AsyncEventBatch, AsyncEventParts, AsyncProgressStep, AsyncResourceStatus,
    AsyncTask, AsyncTaskError, AsyncTaskParts,
};
pub use value::{
    AsyncResourceId, AsyncResourceLink, AsyncResourceText, AsyncResourceTimestamp,
    AsyncResourceValidationError, MAX_ASYNC_ERRORS, MAX_ASYNC_EVENTS, MAX_ASYNC_ID_BYTES,
    MAX_ASYNC_LINK_BYTES, MAX_ASYNC_PROGRESS_STEPS, MAX_ASYNC_TEXT_BYTES,
};

#[cfg(test)]
mod tests;
