use core::{cmp::Ordering, fmt};

use crate::action_polling::ActionUpdate;

use super::{
    AsyncResourceId, AsyncResourceLink, AsyncResourceText, AsyncResourceTimestamp,
    AsyncResourceValidationError, MAX_ASYNC_ERRORS, MAX_ASYNC_EVENTS, MAX_ASYNC_PROGRESS_STEPS,
};

/// Provider-neutral lifecycle classification for asynchronous resources.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AsyncResourceStatus {
    /// Accepted but not started.
    Pending,
    /// Scheduled for later execution.
    Scheduled,
    /// Currently executing.
    Running,
    /// Waiting for explicit caller input.
    WaitingForInput,
    /// Completed successfully.
    Succeeded,
    /// Completed with provider errors.
    Failed,
}

/// Exhaustive polling disposition for one asynchronous task snapshot.
#[derive(Debug, Eq, PartialEq)]
pub enum AsyncPollDisposition<'a> {
    /// The task can be consumed by the ordinary bounded action-polling driver.
    Update(ActionUpdate<&'a [AsyncTaskError<'a>]>),
    /// The provider requires explicit caller intervention before polling resumes.
    WaitingForInput,
}

impl AsyncResourceStatus {
    /// Reports whether the provider lifecycle is complete.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed)
    }
}

/// One bounded progress step.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct AsyncProgressStep<'a> {
    name: AsyncResourceText<'a>,
    status: AsyncResourceStatus,
}

impl<'a> AsyncProgressStep<'a> {
    /// Creates one validated progress step.
    #[must_use]
    pub const fn new(name: AsyncResourceText<'a>, status: AsyncResourceStatus) -> Self {
        Self { name, status }
    }

    /// Returns the sensitive progress-step name.
    #[must_use]
    pub const fn name(self) -> AsyncResourceText<'a> {
        self.name
    }

    /// Returns the normalized progress status.
    #[must_use]
    pub const fn status(self) -> AsyncResourceStatus {
        self.status
    }
}

impl fmt::Debug for AsyncProgressStep<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AsyncProgressStep")
            .field("name", &"[redacted]")
            .field("status", &self.status)
            .finish()
    }
}

/// One bounded provider task error with redacted diagnostics.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct AsyncTaskError<'a> {
    message: AsyncResourceText<'a>,
}

impl<'a> AsyncTaskError<'a> {
    /// Creates one task error.
    #[must_use]
    pub const fn new(message: AsyncResourceText<'a>) -> Self {
        Self { message }
    }

    /// Returns the sensitive provider error message.
    #[must_use]
    pub const fn message(self) -> AsyncResourceText<'a> {
        self.message
    }
}

impl fmt::Debug for AsyncTaskError<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AsyncTaskError([redacted])")
    }
}

/// Complete borrowed fields used to validate one task snapshot.
#[derive(Clone, Copy)]
pub struct AsyncTaskParts<'a> {
    /// Opaque task identifier.
    pub id: AsyncResourceId<'a>,
    /// Provider task type.
    pub kind: AsyncResourceText<'a>,
    /// Current normalized status.
    pub status: AsyncResourceStatus,
    /// Optional related-resource link. This is metadata, not an executable target.
    pub link: Option<AsyncResourceLink<'a>>,
    /// Optional provider task description.
    pub message: Option<AsyncResourceText<'a>>,
    /// Creation timestamp.
    pub created_at: AsyncResourceTimestamp<'a>,
    /// Last-update timestamp.
    pub updated_at: AsyncResourceTimestamp<'a>,
    /// Optional start timestamp.
    pub started_at: Option<AsyncResourceTimestamp<'a>>,
    /// Optional terminal timestamp.
    pub finished_at: Option<AsyncResourceTimestamp<'a>>,
    /// Borrowed bounded progress collection.
    pub progress: &'a [AsyncProgressStep<'a>],
    /// Borrowed bounded provider errors.
    pub errors: &'a [AsyncTaskError<'a>],
}

/// Validated borrowed asynchronous task snapshot.
pub struct AsyncTask<'a> {
    parts: AsyncTaskParts<'a>,
}

impl<'a> AsyncTask<'a> {
    /// Validates collection limits, timestamp ordering, and terminal coherence.
    pub fn new(parts: AsyncTaskParts<'a>) -> Result<Self, AsyncResourceValidationError> {
        if parts.progress.len() > MAX_ASYNC_PROGRESS_STEPS {
            return Err(AsyncResourceValidationError::TooManyProgressSteps);
        }
        if parts.errors.len() > MAX_ASYNC_ERRORS {
            return Err(AsyncResourceValidationError::TooManyErrors);
        }
        if parts.updated_at.compare(parts.created_at) == Ordering::Less
            || parts
                .started_at
                .is_some_and(|value| value.compare(parts.created_at) == Ordering::Less)
            || parts
                .started_at
                .is_some_and(|value| value.compare(parts.updated_at) == Ordering::Greater)
            || parts
                .finished_at
                .is_some_and(|value| value.compare(parts.created_at) == Ordering::Less)
            || parts
                .finished_at
                .is_some_and(|value| value.compare(parts.updated_at) == Ordering::Greater)
            || matches!((parts.started_at, parts.finished_at), (Some(started), Some(finished))
                if finished.compare(started) == Ordering::Less)
        {
            return Err(AsyncResourceValidationError::TimestampOrder);
        }
        if parts.status.is_terminal() != parts.finished_at.is_some() {
            return Err(AsyncResourceValidationError::TerminalTimeMismatch);
        }
        Ok(Self { parts })
    }

    /// Returns the opaque task identifier.
    #[must_use]
    pub const fn id(&self) -> AsyncResourceId<'a> {
        self.parts.id
    }

    /// Returns the provider task type.
    #[must_use]
    pub const fn kind(&self) -> AsyncResourceText<'a> {
        self.parts.kind
    }

    /// Returns the normalized lifecycle status.
    #[must_use]
    pub const fn status(&self) -> AsyncResourceStatus {
        self.parts.status
    }

    /// Returns the non-executable related-resource link.
    #[must_use]
    pub const fn link(&self) -> Option<AsyncResourceLink<'a>> {
        self.parts.link
    }

    /// Returns the optional sensitive provider task description.
    #[must_use]
    pub const fn message(&self) -> Option<AsyncResourceText<'a>> {
        self.parts.message
    }

    /// Returns the creation timestamp.
    #[must_use]
    pub const fn created_at(&self) -> AsyncResourceTimestamp<'a> {
        self.parts.created_at
    }

    /// Returns the last-update timestamp.
    #[must_use]
    pub const fn updated_at(&self) -> AsyncResourceTimestamp<'a> {
        self.parts.updated_at
    }

    /// Returns the optional start timestamp.
    #[must_use]
    pub const fn started_at(&self) -> Option<AsyncResourceTimestamp<'a>> {
        self.parts.started_at
    }

    /// Returns the optional completion timestamp.
    #[must_use]
    pub const fn finished_at(&self) -> Option<AsyncResourceTimestamp<'a>> {
        self.parts.finished_at
    }

    /// Returns the provider progress steps.
    #[must_use]
    pub const fn progress(&self) -> &'a [AsyncProgressStep<'a>] {
        self.parts.progress
    }

    /// Returns the provider task errors.
    #[must_use]
    pub const fn errors(&self) -> &'a [AsyncTaskError<'a>] {
        self.parts.errors
    }

    /// Classifies the snapshot without collapsing caller-intervention states.
    #[must_use]
    pub fn poll_disposition(&self) -> AsyncPollDisposition<'a> {
        match self.parts.status {
            AsyncResourceStatus::Succeeded => AsyncPollDisposition::Update(ActionUpdate::Success),
            AsyncResourceStatus::Failed => {
                AsyncPollDisposition::Update(ActionUpdate::Failed(self.parts.errors))
            }
            AsyncResourceStatus::WaitingForInput => AsyncPollDisposition::WaitingForInput,
            _ => AsyncPollDisposition::Update(ActionUpdate::Running),
        }
    }
}

impl fmt::Debug for AsyncTask<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AsyncTask")
            .field("id", &"[redacted]")
            .field("kind", &"[redacted]")
            .field("status", &self.parts.status)
            .field("link", &"[redacted]")
            .field("message", &"[redacted]")
            .field("timestamps", &"[redacted]")
            .field("progress_steps", &self.parts.progress.len())
            .field("errors", &self.parts.errors.len())
            .finish()
    }
}

/// Complete borrowed fields for one generic asynchronous event fixture.
#[derive(Clone, Copy)]
pub struct AsyncEventParts<'a> {
    /// Opaque event identifier.
    pub id: AsyncResourceId<'a>,
    /// Provider event type.
    pub kind: AsyncResourceText<'a>,
    /// Event observation timestamp.
    pub observed_at: AsyncResourceTimestamp<'a>,
    /// Optional non-executable related-resource link.
    pub link: Option<AsyncResourceLink<'a>>,
    /// Optional sensitive event message.
    pub message: Option<AsyncResourceText<'a>>,
}

/// Bounded generic event model that makes no provider endpoint claim.
#[derive(Clone, Copy)]
pub struct AsyncEvent<'a> {
    parts: AsyncEventParts<'a>,
}

impl<'a> AsyncEvent<'a> {
    /// Creates an event from individually validated bounded fields.
    #[must_use]
    pub const fn new(parts: AsyncEventParts<'a>) -> Self {
        Self { parts }
    }

    /// Returns the opaque event identifier.
    #[must_use]
    pub const fn id(&self) -> AsyncResourceId<'a> {
        self.parts.id
    }

    /// Returns the sensitive provider event type.
    #[must_use]
    pub const fn kind(&self) -> AsyncResourceText<'a> {
        self.parts.kind
    }

    /// Returns the event observation timestamp.
    #[must_use]
    pub const fn observed_at(&self) -> AsyncResourceTimestamp<'a> {
        self.parts.observed_at
    }

    /// Returns the optional non-executable related-resource link.
    #[must_use]
    pub const fn link(&self) -> Option<AsyncResourceLink<'a>> {
        self.parts.link
    }

    /// Returns the optional sensitive event message.
    #[must_use]
    pub const fn message(&self) -> Option<AsyncResourceText<'a>> {
        self.parts.message
    }
}

impl fmt::Debug for AsyncEvent<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AsyncEvent([redacted])")
    }
}

/// Borrowed event batch with an unconditional event-count bound.
pub struct AsyncEventBatch<'a> {
    events: &'a [AsyncEvent<'a>],
}

impl<'a> AsyncEventBatch<'a> {
    /// Validates the complete event-count bound before exposing the batch.
    pub fn new(events: &'a [AsyncEvent<'a>]) -> Result<Self, AsyncResourceValidationError> {
        if events.len() > MAX_ASYNC_EVENTS {
            return Err(AsyncResourceValidationError::TooManyEvents);
        }
        Ok(Self { events })
    }

    /// Returns the complete bounded event slice.
    #[must_use]
    pub const fn events(&self) -> &'a [AsyncEvent<'a>] {
        self.events
    }
}

impl fmt::Debug for AsyncEventBatch<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AsyncEventBatch")
            .field("events", &self.events.len())
            .finish()
    }
}
