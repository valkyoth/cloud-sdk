# v0.53.0 Public API Review

Date: 2026-08-04

Scope: provider-neutral pager sequencing, bounded action polling, progress,
backoff, cancellation, and time-domain separation.

## Added API

Pagination exports `PageStrategy`, `PagerDriver`, `PagerControl`, `PagerStep`,
`PagerDriverError`, and grouped numbered/offset observations. The generic
strategy uses a GAT so decoded snapshot state may remain borrowed.

Action polling exports hard limits, request and observation steps, separate
control and backoff contracts, deterministic bounded exponential backoff,
progress observations/policies/transitions, and typed provider-time telemetry.

## Changed API

The old `PollPolicy` and `PollDecision` API is removed. `ActionPoller::new`
requires complete limits, progress policy, and a monotonic start. Every response
must follow `next_request`; `observe` accepts typed progress, provider time,
monotonic time, and a separate backoff owner.

This pre-1.0 break closes unbounded observations and policy-controlled timeout.
The migration is documented in `MIGRATION_0.53.0.md`.

## Security And Semver Review

Drivers are non-cloneable, allocation-free, and transport-free. Pagination
strategy failures remain transactional. Action structural errors are
payload-free; generic backoff and provider failure values have manually
redacted Debug paths and do not implement `Copy` or `Clone`. Wall-clock
provider values cannot affect local budgets.

The API remains `no_std`, uses no unsafe code, and supports Rust 1.92.0 through
the pinned stable toolchain.
