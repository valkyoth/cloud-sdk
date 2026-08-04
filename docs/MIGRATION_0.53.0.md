# Migrating Source Users To v0.53.0

v0.53.0 is an internal source milestone and is not published separately to
crates.io. It intentionally tightens the pre-1.0 action polling API.

## Action Polling

Replace `ActionPoller::new()` with a constructor that receives
`ActionPollLimits`, `ProgressPolicy`, and the initial `MonotonicInstant`.

Replace the combined `PollPolicy`/`PollDecision` contract with:

- `PollControl` for continue or cancel decisions;
- `PollBackoff` for delay selection;
- `ProgressObservation` and `ProgressPolicy` for provider progress semantics;
- `ProviderTimeObservation` for provider wall-clock telemetry;
- `MonotonicDuration` and `MonotonicInstant` for all local budgets.

Call `next_request` before each transport request. Pass exactly one decoded
response to `observe`. When `observe` returns `ActionPollStep::Delay`, ask
`next_request` again after caller-owned sleep; it returns the remaining delay
until the monotonic boundary is reached.

The old API let policy choose timeout and had no structural observation limit.
The new driver enforces timeout, request sequencing, observations, delay, and
elapsed budgets even for custom backoff implementations.

## Pagination

Existing direct `NumberedPagination` and `OffsetPagination` use remains valid.
For sequenced workflows, wrap either in `PagerDriver` and group decoded values
in `NumberedPageObservation` or `OffsetPageObservation`. `PagerControl::Cancel`
is separate from provider strategy and is terminal.

Custom providers can implement `PageStrategy` with a borrowed observation GAT.
The strategy must retain its own unconditional request/item/state budgets.

## Rust Version

The workspace MSRV remains Rust 1.92.0. v0.53 adds no dependency or feature.
