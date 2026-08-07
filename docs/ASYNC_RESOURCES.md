# Bounded Asynchronous Resources

`cloud_sdk::async_resource` models provider task snapshots and generic event
fixtures without allocation, transport, parsing, clocks, or provider endpoint
assumptions. It is available in the default `no_std` graph.

## Hard Bounds

| Data | Maximum |
| --- | ---: |
| identifier | 256 bytes |
| text or non-executable link | 4,096 bytes |
| progress steps per task | 1,024 |
| errors per task | 1,024 |
| events per batch | 1,024 |

Identifiers must be visible ASCII. Text rejects controls. Links reject
whitespace and controls and are metadata only: they are never executable
request targets. All value, task, progress, error, event, and timestamp Debug
implementations redact sensitive content.

## Task Validation

`AsyncTask::new` accepts borrowed `AsyncTaskParts` after each scalar has been
validated. It rejects collection overflow, timestamps outside canonical UTC
`YYYY-MM-DDTHH:MM:SS[.fraction]Z`, invalid calendar values, contradictory
creation/start/update/finish ordering, and terminal states without a finish
time. Nonterminal states reject a finish time.

```rust
use cloud_sdk::async_resource::{
    AsyncResourceId, AsyncResourceLink, AsyncResourceStatus, AsyncResourceText,
    AsyncResourceTimestamp, AsyncTask, AsyncTaskParts,
};

let task = AsyncTask::new(AsyncTaskParts {
    id: AsyncResourceId::new("task-42")?,
    kind: AsyncResourceText::new("resource.update")?,
    status: AsyncResourceStatus::Running,
    link: AsyncResourceLink::new("/resources/42")?,
    message: AsyncResourceText::new("update running")?,
    created_at: AsyncResourceTimestamp::parse("2026-08-07T08:00:00Z")?,
    updated_at: AsyncResourceTimestamp::parse("2026-08-07T08:00:01Z")?,
    started_at: Some(AsyncResourceTimestamp::parse("2026-08-07T08:00:01Z")?),
    finished_at: None,
    progress: &[],
    errors: &[],
})?;

assert!(!task.status().is_terminal());
# Ok::<(), cloud_sdk::async_resource::AsyncResourceValidationError>(())
```

`AsyncTask::action_update` maps the snapshot into the existing bounded action
polling contract. Provider adapters remain responsible for exact source-status
mapping and bounded response decoding.

## Source-Locked Evidence

v0.60 binds this model to the official OVHcloud API v2 production reads
`GET /notification/contactMean/{contactMeanId}/task` and
`GET /notification/contactMean/{contactMeanId}/task/{taskId}`. The lock covers
the complete `common.Task`, `common.TaskProgress`, `common.TaskError`, and
`common.TaskStatusEnum` schemas.

`AsyncEvent` and `AsyncEventBatch` are generic fixture models only. v0.60 does
not source-lock an OVHcloud event route, publish an OVHcloud provider, or make
task links executable. See
[`provider-probes/ovhcloud-v2/README.md`](../provider-probes/ovhcloud-v2/README.md).
