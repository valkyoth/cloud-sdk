use super::*;
use crate::action_polling::ActionUpdate;
use core::fmt::{self, Write};

fn id(value: &str) -> AsyncResourceId<'_> {
    AsyncResourceId::new(value).unwrap_or_else(|_| unreachable!())
}

fn text(value: &str) -> AsyncResourceText<'_> {
    AsyncResourceText::new(value).unwrap_or_else(|_| unreachable!())
}

fn link(value: &str) -> AsyncResourceLink<'_> {
    AsyncResourceLink::new(value).unwrap_or_else(|_| unreachable!())
}

fn timestamp(value: &str) -> AsyncResourceTimestamp<'_> {
    AsyncResourceTimestamp::parse(value).unwrap_or_else(|_| unreachable!())
}

fn parts<'a>(
    status: AsyncResourceStatus,
    started_at: Option<AsyncResourceTimestamp<'a>>,
    finished_at: Option<AsyncResourceTimestamp<'a>>,
    progress: &'a [AsyncProgressStep<'a>],
    errors: &'a [AsyncTaskError<'a>],
) -> AsyncTaskParts<'a> {
    AsyncTaskParts {
        id: id("018f4a55-8970-7db9-b6b5-4afed75820af"),
        kind: text("contact-mean-validation"),
        status,
        link: Some(link("/v2/notification/contactMean/example")),
        message: Some(text("provider-controlled task message")),
        created_at: timestamp("2026-08-07T08:00:00Z"),
        updated_at: timestamp("2026-08-07T08:00:02.500000000Z"),
        started_at,
        finished_at,
        progress,
        errors,
    }
}

#[test]
fn sensitive_values_enforce_exact_bounds_and_redact_debug() {
    let max_id = "x".repeat(MAX_ASYNC_ID_BYTES);
    assert!(AsyncResourceId::new(&max_id).is_ok());
    assert_eq!(
        AsyncResourceId::new(&(max_id + "x")),
        Err(AsyncResourceValidationError::IdTooLong)
    );
    assert_eq!(
        AsyncResourceId::new("id with space"),
        Err(AsyncResourceValidationError::InvalidId)
    );
    let max_text = "x".repeat(MAX_ASYNC_TEXT_BYTES);
    assert!(AsyncResourceText::new(&max_text).is_ok());
    assert_eq!(
        AsyncResourceText::new("message\nsecret"),
        Err(AsyncResourceValidationError::TextControl)
    );
    assert_eq!(
        AsyncResourceLink::new("/resource token"),
        Err(AsyncResourceValidationError::InvalidLink)
    );
    let mut output = DebugBuffer::new();
    write!(output, "{:?}", text("sentinel-secret")).unwrap_or_else(|_| unreachable!());
    assert_eq!(output.as_str(), "AsyncResourceText([redacted])");
}

#[test]
fn utc_timestamps_validate_calendar_fraction_and_order() {
    for value in [
        "2024-02-29T23:59:59Z",
        "2026-08-07T08:00:00.1Z",
        "2026-08-07T08:00:00.000000001Z",
    ] {
        assert!(AsyncResourceTimestamp::parse(value).is_ok(), "{value}");
    }
    for value in [
        "2023-02-29T00:00:00Z",
        "2026-13-01T00:00:00Z",
        "2026-01-01t00:00:00Z",
        "2026-01-01T24:00:00Z",
        "2026-01-01T00:00:60Z",
        "2026-01-01T00:00:00.Z",
        "2026-01-01T00:00:00+00:00",
    ] {
        assert_eq!(
            AsyncResourceTimestamp::parse(value),
            Err(AsyncResourceValidationError::InvalidTimestamp),
            "{value}"
        );
    }
    assert_eq!(
        timestamp("2026-01-01T00:00:00.9Z").compare(timestamp("2026-01-01T00:00:01Z")),
        core::cmp::Ordering::Less
    );
    assert_eq!(
        timestamp("2026-01-01T00:00:00.1Z"),
        timestamp("2026-01-01T00:00:00.100Z")
    );
    assert_eq!(
        timestamp("2026-01-01T00:00:00Z"),
        timestamp("2026-01-01T00:00:00.0Z")
    );
}

#[test]
fn task_lifecycle_is_bounded_coherent_and_pollable() {
    let progress = [AsyncProgressStep::new(
        text("validation"),
        AsyncResourceStatus::Running,
    )];
    let running = AsyncTask::new(parts(
        AsyncResourceStatus::Running,
        Some(timestamp("2026-08-07T08:00:01Z")),
        None,
        &progress,
        &[],
    ))
    .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        running.poll_disposition(),
        AsyncPollDisposition::Update(ActionUpdate::Running)
    );
    assert_eq!(running.progress(), &progress);

    let errors = [AsyncTaskError::new(text("sentinel-provider-error"))];
    let failed = AsyncTask::new(parts(
        AsyncResourceStatus::Failed,
        Some(timestamp("2026-08-07T08:00:01Z")),
        Some(timestamp("2026-08-07T08:00:02Z")),
        &progress,
        &errors,
    ))
    .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        failed.poll_disposition(),
        AsyncPollDisposition::Update(ActionUpdate::Failed(errors.as_slice()))
    );

    let mut waiting_parts = parts(
        AsyncResourceStatus::WaitingForInput,
        Some(timestamp("2026-08-07T08:00:01Z")),
        None,
        &progress,
        &[],
    );
    waiting_parts.link = None;
    waiting_parts.message = None;
    let waiting = AsyncTask::new(waiting_parts).unwrap_or_else(|_| unreachable!());
    assert_eq!(
        waiting.poll_disposition(),
        AsyncPollDisposition::WaitingForInput
    );
    assert_eq!(waiting.link(), None);
    assert_eq!(waiting.message(), None);

    assert!(matches!(
        AsyncTask::new(parts(AsyncResourceStatus::Succeeded, None, None, &[], &[],)),
        Err(AsyncResourceValidationError::TerminalTimeMismatch)
    ));
    assert!(matches!(
        AsyncTask::new(parts(
            AsyncResourceStatus::Running,
            Some(timestamp("2026-08-07T08:00:02.600Z")),
            None,
            &[],
            &[],
        )),
        Err(AsyncResourceValidationError::TimestampOrder)
    ));
}

#[test]
fn generic_events_are_bounded_without_claiming_an_endpoint() {
    let event = AsyncEvent::new(AsyncEventParts {
        id: id("event-1"),
        kind: text("resource.changed"),
        observed_at: timestamp("2026-08-07T08:00:00Z"),
        link: Some(link("/fixture/resource")),
        message: Some(text("sentinel-event-message")),
    });
    let events = [event];
    let batch = AsyncEventBatch::new(&events).unwrap_or_else(|_| unreachable!());
    assert_eq!(batch.events().len(), 1);
    let mut output = DebugBuffer::new();
    write!(output, "{batch:?}").unwrap_or_else(|_| unreachable!());
    assert_eq!(output.as_str(), "AsyncEventBatch { events: 1 }");
}

#[test]
fn collection_bounds_accept_exact_limits_and_reject_plus_one() {
    let progress = AsyncProgressStep::new(text("phase"), AsyncResourceStatus::Running);
    let progress_limit = [progress; MAX_ASYNC_PROGRESS_STEPS];
    assert!(
        AsyncTask::new(parts(
            AsyncResourceStatus::Running,
            None,
            None,
            &progress_limit,
            &[],
        ))
        .is_ok()
    );
    let progress_over = [progress; MAX_ASYNC_PROGRESS_STEPS + 1];
    assert!(matches!(
        AsyncTask::new(parts(
            AsyncResourceStatus::Running,
            None,
            None,
            &progress_over,
            &[],
        )),
        Err(AsyncResourceValidationError::TooManyProgressSteps)
    ));

    let task_error = AsyncTaskError::new(text("provider error"));
    let errors_over = [task_error; MAX_ASYNC_ERRORS + 1];
    assert!(matches!(
        AsyncTask::new(parts(
            AsyncResourceStatus::Running,
            None,
            None,
            &[],
            &errors_over,
        )),
        Err(AsyncResourceValidationError::TooManyErrors)
    ));

    let event = AsyncEvent::new(AsyncEventParts {
        id: id("event-1"),
        kind: text("fixture"),
        observed_at: timestamp("2026-08-07T08:00:00Z"),
        link: None,
        message: None,
    });
    let event_limit = [event; MAX_ASYNC_EVENTS];
    assert!(AsyncEventBatch::new(&event_limit).is_ok());
    let event_over = [event; MAX_ASYNC_EVENTS + 1];
    assert!(matches!(
        AsyncEventBatch::new(&event_over),
        Err(AsyncResourceValidationError::TooManyEvents)
    ));
}

struct DebugBuffer {
    bytes: [u8; 128],
    len: usize,
}

impl DebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 128],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
    }
}

impl Write for DebugBuffer {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(fmt::Error)?;
        let target = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        target.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}
