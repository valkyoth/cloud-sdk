use core::fmt;

use super::PaginationError;

/// Provider-neutral page strategy driven without transport or allocation.
pub trait PageStrategy {
    /// Exact request-position token for the next page.
    type Request: Copy;
    /// Decoded response observation, optionally borrowing provider state.
    type Observation<'observation>;
    /// Validated accepted page boundary.
    type Boundary;

    /// Returns the next request token or [`PaginationError::Complete`].
    fn next_request(&self) -> Result<Self::Request, PaginationError>;

    /// Transactionally validates and accepts one response.
    fn observe<'observation>(
        &mut self,
        observation: Self::Observation<'observation>,
    ) -> Result<Self::Boundary, PaginationError>;
}

/// Caller decision independent from provider pagination state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PagerControl {
    /// Continue traversal within the strategy's hard budgets.
    Continue,
    /// Cancel before another response is accepted.
    Cancel,
}

/// Next caller action from the pure pager driver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PagerStep<R> {
    /// Send one request for this validated position.
    Request(R),
    /// The strategy has no continuation.
    Complete,
    /// Caller cancellation made the driver terminal.
    Cancelled,
}

/// Pager sequencing or strategy failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PagerDriverError {
    /// A request is already awaiting one response.
    ResponsePending,
    /// A response was supplied without one admitted request.
    UnexpectedObservation,
    /// The driver already completed or was cancelled.
    Terminal,
    /// The underlying transactional strategy rejected the response.
    Strategy(PaginationError),
}

impl fmt::Display for PagerDriverError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ResponsePending => "a pagination response is still pending",
            Self::UnexpectedObservation => "pagination observation has no admitted request",
            Self::Terminal => "pagination driver already reached a terminal state",
            Self::Strategy(_) => "pagination strategy rejected the response",
        })
    }
}

impl core::error::Error for PagerDriverError {}

/// Single-owner request/response sequencer for one pagination strategy.
///
/// ```compile_fail
/// use cloud_sdk::pagination::{NumberedPagination, PagerDriver};
/// fn duplicate(driver: PagerDriver<NumberedPagination>) {
///     let _copy = driver.clone();
/// }
/// ```
pub struct PagerDriver<S> {
    strategy: S,
    response_pending: bool,
    terminal: bool,
}

impl<S> PagerDriver<S>
where
    S: PageStrategy,
{
    /// Wraps one fresh strategy without executing a request.
    #[must_use]
    pub const fn new(strategy: S) -> Self {
        Self {
            strategy,
            response_pending: false,
            terminal: false,
        }
    }

    /// Returns read-only access to strategy progress and limits.
    #[must_use]
    pub const fn strategy(&self) -> &S {
        &self.strategy
    }

    /// Reports whether no response can be accepted.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        self.terminal
    }

    /// Admits one next request, completes, or cancels.
    pub fn next_request(
        &mut self,
        control: PagerControl,
    ) -> Result<PagerStep<S::Request>, PagerDriverError> {
        if self.terminal {
            return Err(PagerDriverError::Terminal);
        }
        if control == PagerControl::Cancel {
            self.terminal = true;
            self.response_pending = false;
            return Ok(PagerStep::Cancelled);
        }
        if self.response_pending {
            return Err(PagerDriverError::ResponsePending);
        }
        match self.strategy.next_request() {
            Ok(request) => {
                self.response_pending = true;
                Ok(PagerStep::Request(request))
            }
            Err(PaginationError::Complete) => {
                self.terminal = true;
                Ok(PagerStep::Complete)
            }
            Err(error) => Err(PagerDriverError::Strategy(error)),
        }
    }

    /// Accepts exactly one decoded response for the admitted request.
    pub fn observe<'observation>(
        &mut self,
        observation: S::Observation<'observation>,
    ) -> Result<S::Boundary, PagerDriverError> {
        if self.terminal {
            return Err(PagerDriverError::Terminal);
        }
        if !self.response_pending {
            return Err(PagerDriverError::UnexpectedObservation);
        }
        let boundary = self
            .strategy
            .observe(observation)
            .map_err(PagerDriverError::Strategy)?;
        self.response_pending = false;
        Ok(boundary)
    }
}

impl<S> fmt::Debug for PagerDriver<S>
where
    S: PageStrategy + fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PagerDriver")
            .field("strategy", &self.strategy)
            .field("response_pending", &self.response_pending)
            .field("terminal", &self.terminal)
            .finish()
    }
}
