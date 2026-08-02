//! One-use retry execution permits.

use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;

use super::policy::observe_monotonic;
use super::{MonotonicDuration, MonotonicInstant};
use crate::authentication::{AsyncAuthenticatedTransport, BlockingAuthenticatedTransport};
use crate::operation::{CheckedResponseGuard, PreparedExecutionError, PreparedRequest};
use crate::transport::BoundTransport;

/// Why a one-use retry permit did not authorize execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryPermitError {
    /// The caller observed a time before the authorized delay completed.
    TooEarly,
    /// The caller observation moved backward relative to controller state.
    MonotonicRollback,
    /// The hard elapsed budget expired before execution.
    ElapsedBudgetExhausted,
}

impl_static_error!(RetryPermitError,
    Self::TooEarly => "retry permit used before its authorized delay",
    Self::MonotonicRollback => "retry permit monotonic observation moved backward",
    Self::ElapsedBudgetExhausted => "retry permit elapsed budget is exhausted",
);

/// Retry authorization or prepared transport execution failure.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum RetryExecutionError<E> {
    /// The permit rejected its final monotonic observation.
    Permit(RetryPermitError),
    /// The exact prepared request failed during its single execution.
    Execution(PreparedExecutionError<E>),
}

impl<E> fmt::Debug for RetryExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Permit(error) => formatter.debug_tuple("Permit").field(error).finish(),
            Self::Execution(error) => formatter.debug_tuple("Execution").field(error).finish(),
        }
    }
}

impl<E> fmt::Display for RetryExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Permit(_) => "retry permit rejected execution",
            Self::Execution(_) => "retry prepared execution failed",
        })
    }
}

impl<E: fmt::Debug> core::error::Error for RetryExecutionError<E> {}

/// Non-cloneable authorization for one exact prepared replay.
///
/// The permit exclusively borrows controller monotonic state until it is
/// consumed. Safe code therefore cannot retain two permits from one owner.
/// Execution is performed by the permit and never returns a reusable request.
///
/// ```compile_fail
/// use cloud_sdk::retry::RetryPermit;
///
/// fn duplicate(permit: RetryPermit<'_, '_, '_>) {
///     let _second = permit.clone();
/// }
/// ```
///
/// ```compile_fail
/// use cloud_sdk::retry::{
///     MonotonicDuration, MonotonicInstant, RetryController, RetryEvent,
///     RetrySubject,
/// };
/// use cloud_sdk::transport::DeliveryPhase;
///
/// fn fan_out<'initial, 'binding, 'replay, 'subject>(
///     controller: &mut RetryController<'initial, 'binding>,
///     replay: RetrySubject<'replay, 'subject>,
/// ) {
///     let first = controller.decide_retry(
///         RetryEvent::Transport(DeliveryPhase::NotSent),
///         replay,
///         MonotonicDuration::new(0),
///         MonotonicInstant::new(1),
///     );
///     let _second = controller.decide_retry(
///         RetryEvent::Transport(DeliveryPhase::NotSent),
///         replay,
///         MonotonicDuration::new(0),
///         MonotonicInstant::new(1),
///     );
///     drop(first);
/// }
/// ```
#[must_use]
pub struct RetryPermit<'controller, 'request, 'subject> {
    last_observed: &'controller mut MonotonicInstant,
    prepared: &'subject PreparedRequest<'request>,
    attempt: u16,
    delay: MonotonicDuration,
    not_before: MonotonicInstant,
    started: MonotonicInstant,
    max_elapsed: MonotonicDuration,
}

impl<'controller, 'request, 'subject> RetryPermit<'controller, 'request, 'subject> {
    pub(crate) fn new(
        last_observed: &'controller mut MonotonicInstant,
        prepared: &'subject PreparedRequest<'request>,
        attempt: u16,
        delay: MonotonicDuration,
        not_before: MonotonicInstant,
        started: MonotonicInstant,
        max_elapsed: MonotonicDuration,
    ) -> Self {
        Self {
            last_observed,
            prepared,
            attempt,
            delay,
            not_before,
            started,
            max_elapsed,
        }
    }

    /// Returns the authorized attempt number, including the initial attempt.
    #[must_use]
    pub const fn attempt(&self) -> u16 {
        self.attempt
    }

    /// Returns the exact caller-selected delay charged to retry budgets.
    #[must_use]
    pub const fn delay(&self) -> MonotonicDuration {
        self.delay
    }

    /// Authorizes and executes the exact replay once on a blocking transport.
    pub fn execute_blocking<'buffer, T>(
        mut self,
        now: MonotonicInstant,
        transport: &T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponseGuard<'buffer>, RetryExecutionError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundTransport,
    {
        sanitize_bytes(response_storage);
        sanitize_bytes(response_header_storage);
        self.authorize(now).map_err(RetryExecutionError::Permit)?;
        self.prepared
            .execute_blocking(transport, response_storage, response_header_storage)
            .map_err(RetryExecutionError::Execution)
    }

    /// Authorizes and executes the exact replay once without owning an executor.
    pub async fn execute_async<'transport, 'buffer, T>(
        mut self,
        now: MonotonicInstant,
        transport: &'transport T,
        response_storage: &'buffer mut [u8],
        response_header_storage: &'buffer mut [u8],
    ) -> Result<CheckedResponseGuard<'buffer>, RetryExecutionError<T::Error>>
    where
        T: AsyncAuthenticatedTransport + BoundTransport,
        'request: 'transport,
        'subject: 'transport,
    {
        sanitize_bytes(response_storage);
        sanitize_bytes(response_header_storage);
        self.authorize(now).map_err(RetryExecutionError::Permit)?;
        self.prepared
            .execute_async(transport, response_storage, response_header_storage)
            .await
            .map_err(RetryExecutionError::Execution)
    }

    fn authorize(&mut self, now: MonotonicInstant) -> Result<(), RetryPermitError> {
        let elapsed = observe_monotonic(self.last_observed, self.started, now)
            .map_err(|_| RetryPermitError::MonotonicRollback)?;
        if now < self.not_before {
            return Err(RetryPermitError::TooEarly);
        }
        if elapsed > self.max_elapsed {
            return Err(RetryPermitError::ElapsedBudgetExhausted);
        }
        Ok(())
    }
}

impl fmt::Debug for RetryPermit<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RetryPermit")
            .field("prepared", &"[bound request]")
            .field("attempt", &self.attempt)
            .field("delay", &self.delay)
            .field("not_before", &self.not_before)
            .field("started", &self.started)
            .field("max_elapsed", &self.max_elapsed)
            .finish()
    }
}
