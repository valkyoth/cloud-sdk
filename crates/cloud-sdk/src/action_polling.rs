//! Bounded caller-driven action polling without clocks, sleep, or executors.

mod backoff;
mod context;
mod driver;
mod progress;

pub use backoff::{
    ExponentialBackoff, ExponentialBackoffError, MAX_BACKOFF_MULTIPLIER, PollBackoff,
};
pub use context::{
    PollContext, PollControl, PollRequestStep, ProviderTimeError, ProviderTimeObservation,
};
pub use driver::{
    ActionObserveError, ActionPollError, ActionPollLimits, ActionPollLimitsError, ActionPollStep,
    ActionPoller, ActionUpdate,
};
pub use progress::{ProgressChange, ProgressObservation, ProgressPolicy};

#[cfg(test)]
mod tests;
