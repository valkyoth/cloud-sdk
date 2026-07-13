//! Caller-driven action polling state machine.

use core::time::Duration;

use crate::rate_limit::RateLimit;

/// One provider action observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionUpdate<E> {
    /// The action remains in progress.
    Running,
    /// The action completed successfully.
    Success,
    /// The action completed with a provider failure.
    Failed(E),
}

/// Context passed to caller-owned delay and stop policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PollContext {
    observation: u32,
    progress: u8,
    rate_limit: Option<RateLimit>,
}

impl PollContext {
    /// Returns the one-based running observation count.
    #[must_use]
    pub const fn observation(self) -> u32 {
        self.observation
    }

    /// Returns provider-reported progress in `0..=100`.
    #[must_use]
    pub const fn progress(self) -> u8 {
        self.progress
    }

    /// Returns rate-limit metadata associated with the observation.
    #[must_use]
    pub const fn rate_limit(self) -> Option<RateLimit> {
        self.rate_limit
    }
}

/// Caller-owned decision after a running action observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PollDecision {
    /// Wait for this nonzero duration before the caller sends another request.
    Delay(Duration),
    /// Stop because caller cancellation was requested.
    Cancel,
    /// Stop because the caller's deadline or attempt budget expired.
    Timeout,
}

/// Caller-supplied delay, backoff, timeout, and cancellation policy.
pub trait PollPolicy {
    /// Policy-specific error.
    type Error;

    /// Chooses the next explicit step after a running observation.
    fn decide(&mut self, context: PollContext) -> Result<PollDecision, Self::Error>;
}

/// Step returned after one accepted observation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionPollStep<E> {
    /// The caller must wait before requesting the action again.
    Delay(Duration),
    /// The action completed successfully.
    Complete,
    /// The provider reported a terminal action failure.
    Failed(E),
    /// Caller policy cancelled polling.
    Cancelled,
    /// Caller policy stopped polling at its deadline or attempt budget.
    TimedOut,
}

/// Action observation or policy failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionPollError<E> {
    /// Provider-reported progress exceeded 100.
    InvalidProgress,
    /// Progress moved backwards across observations.
    ProgressRegressed,
    /// A caller policy requested a zero-duration busy loop.
    ZeroDelay,
    /// The observation counter overflowed.
    ObservationOverflow,
    /// Polling was attempted after a terminal step.
    Terminal,
    /// Caller policy failed.
    Policy(E),
}

/// Explicit state for one caller-driven action polling operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionPoller {
    observations: u32,
    last_progress: Option<u8>,
    terminal: bool,
}

impl ActionPoller {
    /// Creates an action poller without selecting time or retry policy.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            observations: 0,
            last_progress: None,
            terminal: false,
        }
    }

    /// Returns the number of accepted observations.
    #[must_use]
    pub const fn observations(self) -> u32 {
        self.observations
    }

    /// Reports whether polling reached a terminal step.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        self.terminal
    }

    /// Records one decoded action response and asks caller policy only when it
    /// remains running.
    pub fn observe<E, P>(
        &mut self,
        update: ActionUpdate<E>,
        progress: u8,
        rate_limit: Option<RateLimit>,
        policy: &mut P,
    ) -> Result<ActionPollStep<E>, ActionPollError<P::Error>>
    where
        P: PollPolicy,
    {
        if self.terminal {
            return Err(ActionPollError::Terminal);
        }
        if progress > 100 {
            return Err(ActionPollError::InvalidProgress);
        }
        if self.last_progress.is_some_and(|last| progress < last) {
            return Err(ActionPollError::ProgressRegressed);
        }
        let observations = self
            .observations
            .checked_add(1)
            .ok_or(ActionPollError::ObservationOverflow)?;

        let step = match update {
            ActionUpdate::Success => ActionPollStep::Complete,
            ActionUpdate::Failed(error) => ActionPollStep::Failed(error),
            ActionUpdate::Running => {
                let context = PollContext {
                    observation: observations,
                    progress,
                    rate_limit,
                };
                match policy.decide(context).map_err(ActionPollError::Policy)? {
                    PollDecision::Delay(delay) if delay.is_zero() => {
                        return Err(ActionPollError::ZeroDelay);
                    }
                    PollDecision::Delay(delay) => ActionPollStep::Delay(delay),
                    PollDecision::Cancel => ActionPollStep::Cancelled,
                    PollDecision::Timeout => ActionPollStep::TimedOut,
                }
            }
        };

        self.observations = observations;
        self.last_progress = Some(progress);
        self.terminal = !matches!(step, ActionPollStep::Delay(_));
        Ok(step)
    }
}

impl Default for ActionPoller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
