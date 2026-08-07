//! Source-bound OVHcloud task and generic event-model conformance.

use cloud_sdk::async_resource::{
    AsyncEvent, AsyncEventParts, AsyncPollDisposition, AsyncProgressStep, AsyncResourceId,
    AsyncResourceLink, AsyncResourceStatus, AsyncResourceText, AsyncResourceTimestamp, AsyncTask,
    AsyncTaskError, AsyncTaskParts,
};

const REVIEWED_TASKS: &str = include_str!("fixtures/ovhcloud-task-contracts.tsv");

fn status(value: &str) -> Result<AsyncResourceStatus, ()> {
    match value {
        "DONE" => Ok(AsyncResourceStatus::Succeeded),
        "ERROR" => Ok(AsyncResourceStatus::Failed),
        "PENDING" => Ok(AsyncResourceStatus::Pending),
        "RUNNING" => Ok(AsyncResourceStatus::Running),
        "SCHEDULED" => Ok(AsyncResourceStatus::Scheduled),
        "WAITING_USER_INPUT" => Ok(AsyncResourceStatus::WaitingForInput),
        _ => Err(()),
    }
}

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

#[test]
fn exact_source_statuses_map_to_the_neutral_lifecycle() {
    assert!(REVIEWED_TASKS.contains("/notification/contactMean/{contactMeanId}/task"));
    assert!(REVIEWED_TASKS.contains("common.Task[]\tcommon.Task"));
    assert!(REVIEWED_TASKS.ends_with("\t/event\tfixture-only\n"));
    assert_eq!(status("DONE"), Ok(AsyncResourceStatus::Succeeded));
    assert_eq!(status("ERROR"), Ok(AsyncResourceStatus::Failed));
    assert_eq!(status("PENDING"), Ok(AsyncResourceStatus::Pending));
    assert_eq!(status("RUNNING"), Ok(AsyncResourceStatus::Running));
    assert_eq!(status("SCHEDULED"), Ok(AsyncResourceStatus::Scheduled));
    assert_eq!(
        status("WAITING_USER_INPUT"),
        Ok(AsyncResourceStatus::WaitingForInput)
    );
    assert_eq!(status("UNKNOWN_PROVIDER_STATE"), Err(()));
}

#[test]
fn complete_source_task_shape_is_bounded_redacted_and_pollable() {
    let progress = [AsyncProgressStep::new(
        text("sentinel-progress-name"),
        AsyncResourceStatus::Succeeded,
    )];
    let errors = [AsyncTaskError::new(text("sentinel-provider-error"))];
    let task = AsyncTask::new(AsyncTaskParts {
        id: id("018f4a55-8970-7db9-b6b5-4afed75820af"),
        kind: text("sentinel-task-type"),
        status: status("ERROR").unwrap_or_else(|_| unreachable!()),
        link: Some(link("/v2/notification/contactMean/sentinel-resource")),
        message: Some(text("sentinel-task-message")),
        created_at: timestamp("2026-08-07T08:00:00Z"),
        updated_at: timestamp("2026-08-07T08:00:02Z"),
        started_at: Some(timestamp("2026-08-07T08:00:01Z")),
        finished_at: Some(timestamp("2026-08-07T08:00:02Z")),
        progress: &progress,
        errors: &errors,
    })
    .unwrap_or_else(|_| unreachable!());
    assert_eq!(
        task.poll_disposition(),
        AsyncPollDisposition::Update(cloud_sdk::action_polling::ActionUpdate::Failed(
            errors.as_slice()
        ))
    );
    assert_eq!(task.created_at(), timestamp("2026-08-07T08:00:00Z"));
    assert_eq!(task.updated_at(), timestamp("2026-08-07T08:00:02Z"));
    assert_eq!(task.started_at(), Some(timestamp("2026-08-07T08:00:01Z")));
    assert_eq!(task.finished_at(), Some(timestamp("2026-08-07T08:00:02Z")));
    let debug = format!("{task:?}");
    for secret in [
        "sentinel-progress-name",
        "sentinel-provider-error",
        "sentinel-task-type",
        "sentinel-resource",
        "sentinel-task-message",
    ] {
        assert!(!debug.contains(secret));
    }
}

#[test]
fn omitted_optional_task_fields_and_nullable_errors_remain_representable() {
    let task = AsyncTask::new(AsyncTaskParts {
        id: id("018f4a55-8970-7db9-b6b5-4afed75820af"),
        kind: text("source-example"),
        status: AsyncResourceStatus::WaitingForInput,
        link: None,
        message: None,
        created_at: timestamp("2026-08-07T08:00:00Z"),
        updated_at: timestamp("2026-08-07T08:00:01Z"),
        started_at: None,
        finished_at: None,
        progress: &[],
        errors: &[],
    })
    .unwrap_or_else(|_| unreachable!());
    assert_eq!(task.link(), None);
    assert_eq!(task.message(), None);
    assert!(task.errors().is_empty());
    assert_eq!(
        task.poll_disposition(),
        AsyncPollDisposition::WaitingForInput
    );
}

#[test]
fn successful_task_with_provider_errors_fails_closed() {
    let errors = [AsyncTaskError::new(text("contradictory-provider-error"))];
    let result = AsyncTask::new(AsyncTaskParts {
        id: id("018f4a55-8970-7db9-b6b5-4afed75820af"),
        kind: text("source-example"),
        status: AsyncResourceStatus::Succeeded,
        link: None,
        message: None,
        created_at: timestamp("2026-08-07T08:00:00Z"),
        updated_at: timestamp("2026-08-07T08:00:01Z"),
        started_at: None,
        finished_at: Some(timestamp("2026-08-07T08:00:01Z")),
        progress: &[],
        errors: &errors,
    });
    assert!(matches!(
        result,
        Err(cloud_sdk::async_resource::AsyncResourceValidationError::StatusErrorMismatch)
    ));
}

#[test]
fn generic_event_model_does_not_create_an_endpoint_claim() {
    let event = AsyncEvent::new(AsyncEventParts {
        id: id("event-fixture"),
        kind: text("fixture.event"),
        observed_at: timestamp("2026-08-07T08:00:00Z"),
        link: Some(link("/fixture/not-an-endpoint")),
        message: Some(text("sentinel-event-message")),
    });
    let debug = format!("{event:?}");
    assert_eq!(debug, "AsyncEvent([redacted])");
    assert_eq!(event.observed_at(), timestamp("2026-08-07T08:00:00Z"));
    assert_eq!(event.link(), Some(link("/fixture/not-an-endpoint")));
    assert_eq!(event.message(), Some(text("sentinel-event-message")));
}
