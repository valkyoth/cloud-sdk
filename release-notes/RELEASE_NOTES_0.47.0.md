# cloud-sdk 0.47.0 Release Notes

Status: implementation stop reached; pentest required.

Release date: pending

## Overview

v0.47 adds a complete local asynchronous execution family for `!Send`
browser-WASM, embedded, and single-threaded futures without changing the
default no_std dependency boundary or existing cross-thread APIs.

## Local Async

- Added `LocalAsyncTransport`, `LocalAsyncAuthenticatedTransport`, and
  `LocalAsyncRawHttpExecutor` without allocator or executor ownership.
- Added automatic local compatibility for every existing `Send` async
  transport and raw executor.
- Added local execution for checked prepared requests, operation-bound
  provider links, and one-use retry permits.
- Added explicit `PossiblySent` cancellation classification through
  `ASYNC_CANCELLATION_DELIVERY_PHASE`.
- Required uncommitted response cleanup across local future cancellation.
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
| `cloud-sdk-reqwest` | `0.31.2` | dependency-only patch |
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
