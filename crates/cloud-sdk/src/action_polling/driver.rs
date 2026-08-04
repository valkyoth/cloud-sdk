use core::fmt;

use crate::rate_limit::RateLimit;
use crate::retry::{MonotonicDuration, MonotonicInstant};

use super::progress::{ProgressError, ProgressTracker};
use super::{
    PollBackoff, PollContext, PollControl, PollRequestStep, ProgressObservation, ProgressPolicy,
    ProviderTimeObservation,
};

/// One provider action observation.
#[derive(Eq, PartialEq)]
pub enum ActionUpdate<E> {
    /// The action remains in progress.
    Running,
    /// The action completed successfully.
    Success,
    /// The action completed with a provider failure.
    Failed(E),
}

impl<E> fmt::Debug for ActionUpdate<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Running => formatter.write_str("Running"),
            Self::Success => formatter.write_str("Success"),
            Self::Failed(_) => formatter.write_str("Failed([redacted])"),
        }
    }
}

/// Invalid hard action-poll limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionPollLimitsError {
    /// Every hard limit must be nonzero.
    Zero,
    /// One delay may not exceed the cumulative delay budget.
    DelayExceedsCumulative,
    /// One delay may not meet or exceed the elapsed budget.
    DelayExceedsElapsed,
}

impl_static_error!(ActionPollLimitsError,
    Self::Zero => "action polling limits must be nonzero",
    Self::DelayExceedsCumulative => "maximum poll delay exceeds the cumulative budget",
    Self::DelayExceedsElapsed => "maximum poll delay reaches the elapsed budget",
);

/// Unconditional request, delay, and elapsed limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionPollLimits {
    max_observations: u32,
    max_delay: MonotonicDuration,
    max_cumulative_delay: MonotonicDuration,
    max_elapsed: MonotonicDuration,
}

impl ActionPollLimits {
    /// Creates complete hard limits selected before the first request.
    pub const fn new(
        max_observations: u32,
        max_delay: MonotonicDuration,
        max_cumulative_delay: MonotonicDuration,
        max_elapsed: MonotonicDuration,
    ) -> Result<Self, ActionPollLimitsError> {
        if max_observations == 0
            || max_delay.get() == 0
            || max_cumulative_delay.get() == 0
            || max_elapsed.get() == 0
        {
            return Err(ActionPollLimitsError::Zero);
        }
        if max_delay.get() > max_cumulative_delay.get() {
            return Err(ActionPollLimitsError::DelayExceedsCumulative);
        }
        if max_delay.get() >= max_elapsed.get() {
            return Err(ActionPollLimitsError::DelayExceedsElapsed);
        }
        Ok(Self {
            max_observations,
            max_delay,
            max_cumulative_delay,
            max_elapsed,
        })
    }

    /// Returns the maximum accepted provider observations.
    #[must_use]
    pub const fn max_observations(self) -> u32 {
        self.max_observations
    }

    /// Returns the maximum delay admitted from any backoff policy.
    #[must_use]
    pub const fn max_delay(self) -> MonotonicDuration {
        self.max_delay
    }

    /// Returns the cumulative requested-delay budget.
    #[must_use]
    pub const fn max_cumulative_delay(self) -> MonotonicDuration {
        self.max_cumulative_delay
    }

    /// Returns the monotonic elapsed-time budget.
    #[must_use]
    pub const fn max_elapsed(self) -> MonotonicDuration {
        self.max_elapsed
    }
}

/// Structural action workflow failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionPollError {
    /// Another request cannot start before its response is observed.
    ResponsePending,
    /// A response was supplied without one admitted request.
    UnexpectedObservation,
    /// The workflow already reached a terminal state.
    Terminal,
    /// Caller monotonic time moved backwards.
    MonotonicRollback,
    /// Monotonic timestamp arithmetic overflowed.
    TimeOverflow,
    /// The unconditional observation limit was reached while still running.
    ObservationLimitExceeded,
    /// Provider progress exceeded 100.
    InvalidProgress,
    /// Provider progress regressed without an explicit reset.
    ProgressRegressed,
    /// Provider progress reset was not admitted.
    ProgressResetForbidden,
    /// Provider progress exhausted its reset budget.
    ProgressResetLimitExceeded,
    /// Backoff requested a zero delay.
    ZeroDelay,
    /// Backoff requested more than the per-delay bound.
    DelayLimitExceeded,
    /// Backoff exhausted the cumulative delay budget.
    CumulativeDelayExceeded,
    /// Backoff would reach or exceed the elapsed budget.
    ElapsedBudgetExceeded,
}

impl_static_error!(ActionPollError,
    Self::ResponsePending => "an action response is still pending",
    Self::UnexpectedObservation => "action observation has no admitted request",
    Self::Terminal => "action polling already reached a terminal state",
    Self::MonotonicRollback => "action polling monotonic time moved backwards",
    Self::TimeOverflow => "action polling monotonic time overflowed",
    Self::ObservationLimitExceeded => "action polling observation limit was exceeded",
    Self::InvalidProgress => "action progress exceeds 100",
    Self::ProgressRegressed => "action progress moved backwards",
    Self::ProgressResetForbidden => "action progress reset is forbidden",
    Self::ProgressResetLimitExceeded => "action progress reset limit was exceeded",
    Self::ZeroDelay => "action poll backoff requested a zero delay",
    Self::DelayLimitExceeded => "action poll backoff exceeded the delay limit",
    Self::CumulativeDelayExceeded => "action polling cumulative delay was exceeded",
    Self::ElapsedBudgetExceeded => "action polling elapsed budget would be exceeded",
);

/// Observation failure with payload-redacted backoff diagnostics.
#[derive(Eq, PartialEq)]
pub enum ActionObserveError<E> {
    /// Driver validation failed.
    Driver(ActionPollError),
    /// Caller-owned backoff policy failed.
    Backoff(E),
}

impl<E> fmt::Debug for ActionObserveError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Driver(error) => formatter.debug_tuple("Driver").field(error).finish(),
            Self::Backoff(_) => formatter.write_str("Backoff([redacted])"),
        }
    }
}

impl<E> fmt::Display for ActionObserveError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Driver(error) => fmt::Display::fmt(error, formatter),
            Self::Backoff(_) => formatter.write_str("action poll backoff policy failed"),
        }
    }
}

impl<E> core::error::Error for ActionObserveError<E> {}

/// Step returned after one accepted provider observation.
#[derive(Eq, PartialEq)]
pub enum ActionPollStep<E> {
    /// Wait before asking the driver to admit another request.
    Delay(MonotonicDuration),
    /// The provider action completed successfully.
    Complete,
    /// The provider action completed with an error.
    Failed(E),
}

impl<E> fmt::Debug for ActionPollStep<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Delay(delay) => formatter.debug_tuple("Delay").field(delay).finish(),
            Self::Complete => formatter.write_str("Complete"),
            Self::Failed(_) => formatter.write_str("Failed([redacted])"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PollPhase {
    Ready,
    AwaitingResponse,
    Delayed(MonotonicInstant),
    Terminal,
}

/// Single-owner bounded action workflow driver.
///
/// ```compile_fail
/// use cloud_sdk::action_polling::ActionPoller;
/// fn duplicate(driver: ActionPoller) {
///     let _copy = driver.clone();
/// }
/// ```
pub struct ActionPoller {
    limits: ActionPollLimits,
    progress: ProgressTracker,
    started: MonotonicInstant,
    last_observed: MonotonicInstant,
    observations: u32,
    cumulative_delay: u64,
    phase: PollPhase,
}

impl ActionPoller {
    /// Creates a workflow without reading a clock or issuing a request.
    #[must_use]
    pub const fn new(
        limits: ActionPollLimits,
        progress_policy: ProgressPolicy,
        started: MonotonicInstant,
    ) -> Self {
        Self {
            limits,
            progress: ProgressTracker::new(progress_policy),
            started,
            last_observed: started,
            observations: 0,
            cumulative_delay: 0,
            phase: PollPhase::Ready,
        }
    }

    /// Returns accepted provider observations.
    #[must_use]
    pub const fn observations(&self) -> u32 {
        self.observations
    }

    /// Returns cumulative delay selected by backoff.
    #[must_use]
    pub const fn cumulative_delay(&self) -> MonotonicDuration {
        MonotonicDuration::new(self.cumulative_delay)
    }

    /// Reports whether no further request can be admitted.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self.phase, PollPhase::Terminal)
    }

    /// Admits, delays, cancels, or times out the next request.
    pub fn next_request(
        &mut self,
        control: PollControl,
        now: MonotonicInstant,
    ) -> Result<PollRequestStep, ActionPollError> {
        if self.is_terminal() {
            return Err(ActionPollError::Terminal);
        }
        if now < self.last_observed {
            self.phase = PollPhase::Terminal;
            return Err(ActionPollError::MonotonicRollback);
        }
        self.last_observed = now;
        if self.elapsed_exhausted(now)? {
            self.phase = PollPhase::Terminal;
            return Ok(PollRequestStep::TimedOut);
        }
        if control == PollControl::Cancel {
            self.phase = PollPhase::Terminal;
            return Ok(PollRequestStep::Cancelled);
        }
        match self.phase {
            PollPhase::AwaitingResponse => Err(ActionPollError::ResponsePending),
            PollPhase::Delayed(not_before) if now < not_before => {
                let remaining = not_before
                    .checked_duration_since(now)
                    .ok_or(ActionPollError::MonotonicRollback)?;
                Ok(PollRequestStep::Delay(remaining))
            }
            PollPhase::Ready | PollPhase::Delayed(_) => {
                self.phase = PollPhase::AwaitingResponse;
                Ok(PollRequestStep::Request)
            }
            PollPhase::Terminal => Err(ActionPollError::Terminal),
        }
    }

    /// Accepts exactly one response for the last admitted request.
    pub fn observe<E, B>(
        &mut self,
        update: ActionUpdate<E>,
        progress: ProgressObservation,
        rate_limit: Option<RateLimit>,
        provider_time: ProviderTimeObservation,
        now: MonotonicInstant,
        backoff: &mut B,
    ) -> Result<ActionPollStep<E>, ActionObserveError<B::Error>>
    where
        B: PollBackoff,
    {
        if self.phase != PollPhase::AwaitingResponse {
            return Err(ActionObserveError::Driver(if self.is_terminal() {
                ActionPollError::Terminal
            } else {
                ActionPollError::UnexpectedObservation
            }));
        }
        self.validate_observation_time(now)
            .map_err(ActionObserveError::Driver)?;
        let observations = self.observations.checked_add(1).ok_or_else(|| {
            self.phase = PollPhase::Terminal;
            ActionObserveError::Driver(ActionPollError::ObservationLimitExceeded)
        })?;
        self.observations = observations;
        match update {
            ActionUpdate::Success => {
                self.phase = PollPhase::Terminal;
                Ok(ActionPollStep::Complete)
            }
            ActionUpdate::Failed(error) => {
                self.phase = PollPhase::Terminal;
                Ok(ActionPollStep::Failed(error))
            }
            ActionUpdate::Running => self.observe_running(
                observations,
                progress,
                rate_limit,
                provider_time,
                now,
                backoff,
            ),
        }
    }

    fn observe_running<E, B>(
        &mut self,
        observations: u32,
        progress: ProgressObservation,
        rate_limit: Option<RateLimit>,
        provider_time: ProviderTimeObservation,
        now: MonotonicInstant,
        backoff: &mut B,
    ) -> Result<ActionPollStep<E>, ActionObserveError<B::Error>>
    where
        B: PollBackoff,
    {
        if observations >= self.limits.max_observations {
            self.phase = PollPhase::Terminal;
            return Err(ActionObserveError::Driver(
                ActionPollError::ObservationLimitExceeded,
            ));
        }
        let progress_change = self.progress.observe(progress).map_err(|error| {
            self.phase = PollPhase::Terminal;
            ActionObserveError::Driver(map_progress_error(error))
        })?;
        let context = PollContext {
            observation: observations,
            progress,
            progress_change,
            rate_limit,
            provider_time,
        };
        let delay = backoff.delay(context).map_err(|error| {
            self.phase = PollPhase::Terminal;
            ActionObserveError::Backoff(error)
        })?;
        self.schedule(delay, now)
            .map_err(ActionObserveError::Driver)?;
        Ok(ActionPollStep::Delay(delay))
    }

    fn schedule(
        &mut self,
        delay: MonotonicDuration,
        now: MonotonicInstant,
    ) -> Result<(), ActionPollError> {
        if delay.get() == 0 {
            self.phase = PollPhase::Terminal;
            return Err(ActionPollError::ZeroDelay);
        }
        if delay > self.limits.max_delay {
            self.phase = PollPhase::Terminal;
            return Err(ActionPollError::DelayLimitExceeded);
        }
        let Some(cumulative) = self.cumulative_delay.checked_add(delay.get()) else {
            self.phase = PollPhase::Terminal;
            return Err(ActionPollError::CumulativeDelayExceeded);
        };
        if cumulative > self.limits.max_cumulative_delay.get() {
            self.phase = PollPhase::Terminal;
            return Err(ActionPollError::CumulativeDelayExceeded);
        }
        let Some(not_before) = now.checked_add(delay) else {
            self.phase = PollPhase::Terminal;
            return Err(ActionPollError::TimeOverflow);
        };
        let Some(elapsed) = not_before.checked_duration_since(self.started) else {
            self.phase = PollPhase::Terminal;
            return Err(ActionPollError::MonotonicRollback);
        };
        if elapsed >= self.limits.max_elapsed {
            self.phase = PollPhase::Terminal;
            return Err(ActionPollError::ElapsedBudgetExceeded);
        }
        self.cumulative_delay = cumulative;
        self.phase = PollPhase::Delayed(not_before);
        Ok(())
    }

    fn validate_observation_time(&mut self, now: MonotonicInstant) -> Result<(), ActionPollError> {
        if now < self.last_observed {
            self.phase = PollPhase::Terminal;
            return Err(ActionPollError::MonotonicRollback);
        }
        self.last_observed = now;
        if self.elapsed_exhausted(now)? {
            self.phase = PollPhase::Terminal;
            return Err(ActionPollError::ElapsedBudgetExceeded);
        }
        Ok(())
    }

    fn elapsed_exhausted(&self, now: MonotonicInstant) -> Result<bool, ActionPollError> {
        let elapsed = now
            .checked_duration_since(self.started)
            .ok_or(ActionPollError::MonotonicRollback)?;
        Ok(elapsed >= self.limits.max_elapsed)
    }
}

impl fmt::Debug for ActionPoller {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ActionPoller")
            .field("limits", &self.limits)
            .field("observations", &self.observations)
            .field("cumulative_delay", &self.cumulative_delay())
            .field("phase", &self.phase)
            .finish_non_exhaustive()
    }
}

fn map_progress_error(error: ProgressError) -> ActionPollError {
    match error {
        ProgressError::Invalid => ActionPollError::InvalidProgress,
        ProgressError::Regressed => ActionPollError::ProgressRegressed,
        ProgressError::ResetForbidden => ActionPollError::ProgressResetForbidden,
        ProgressError::ResetLimit => ActionPollError::ProgressResetLimitExceeded,
    }
}
