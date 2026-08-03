# cloud-sdk 0.47.0 Release Notes

Status: implementation stop reached; pentest required.

Release date: pending

## Overview

v0.47 adds a complete local asynchronous execution family for `!Send`
browser-WASM, embedded, and single-threaded futures. All async transport APIs
now use one non-committing response contract without changing the default
no_std dependency boundary.

## Local Async

- Added `LocalAsyncTransport`, `LocalAsyncAuthenticatedTransport`, and
  `LocalAsyncRawHttpExecutor` without allocator or executor ownership.
- Added automatic local compatibility for every existing `Send` async
  transport and raw executor.
- Added local execution for checked prepared requests, operation-bound
  provider links, and one-use retry permits.
- Added explicit `PossiblySent` cancellation classification through
  `ASYNC_CANCELLATION_DELIVERY_PHASE`.
- Added non-committing `AsyncResponseStaging`, `ResponseCompletion`, and
  SDK-owned Send/local drivers that commit only after `Ready(Ok)`.
- Required rollback of partial response state across every async future
  cancellation; implementations have no commit capability.
- Added `LocalMockTransport`, a no-allocation deliberately `!Sync` testkit
  fixture for local basic and authenticated workflows.
- Added partial-secret cancellation, same-thread cooperative concurrency,
  prepared request, pagination, raw fault, retry permit, blanket adaptation,
  compile-fail, portable-target, and doctest coverage.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.47.0` | local async contract code |
| `cloud-sdk-hetzner` | `0.36.1` | dependency-only patch |
| `cloud-sdk-reqwest` | `0.32.0` | Send async staging migration |
| `cloud-sdk-sanitization` | `0.16.0` | unchanged; not published |
| `cloud-sdk-testkit` | `0.27.0` | local mock and conformance code |

## Documentation

- [`docs/LOCAL_ASYNC.md`](../docs/LOCAL_ASYNC.md)
- [`docs/MIGRATION_0.47.0.md`](../docs/MIGRATION_0.47.0.md)
- [`docs/PUBLIC_API_REVIEW_0.47.0.md`](../docs/PUBLIC_API_REVIEW_0.47.0.md)
- [`docs/DEPENDENCY_REVIEW_0.47.0.md`](../docs/DEPENDENCY_REVIEW_0.47.0.md)

## Pentest

Pentest is required for the exact implementation-stop commit. Temporary
findings belong in root `PENTEST.md` and must be removed after remediation.

## Release Gate

```text
v0.47.0 implementation stop reached. Run pentest for this exact commit.
```
