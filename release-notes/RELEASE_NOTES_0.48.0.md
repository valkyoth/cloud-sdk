# cloud-sdk 0.48.0 Release Notes

Status: implementation stop reached; pentest required.

Release date: pending

## Overview

v0.48 adds allocation-free, runtime-neutral streaming contracts for bounded
uploads, downloads, and caller-cancelled event streams without changing
buffered request behavior or the default no_std dependency graph.

## Streaming

- Added complete per-operation byte, chunk-size, chunk-count, observation, and
  consecutive zero-progress limits under global ceilings.
- Added exact declared-length and explicit executor-owned unknown framing.
- Added actual-byte accounting, one-pending-chunk backpressure, short-write
  handling, and pre-accept overflow rejection.
- Added source and sink observation preflight so hard caps reject before
  external I/O, including terminal, error, and invalid-progress observations.
- Added sticky sink-attempt state so first-write failure, invalid or zero
  progress, and cancellation cannot falsely report a clean outcome.
- Added explicit transactional rollback and dirty direct-sink outcomes.
- Added blocking, Send-async, and local-async source/sink traits and drivers
  over caller-owned scratch storage.
- Added a forced cooperative yield after every 64 completed async callbacks.
- Added complete scratch clearing before use and on success, failure, or
  cancellation.
- Reset reused outcomes before empty-scratch rejection to prevent stale
  completion state.
- Re-exported the audited `sanitize_bytes` boundary through
  `cloud_sdk::buffer` for provider-neutral companion crates.
- Added separate end validation and sink commitment so cancellation during
  commit cannot be reported as completion.
- Added bounded `Wait` observations and caller-cancelled event policy.
- Added exact redacted source identities and replay invalidation when a source
  version changes; drivers remain single-attempt and never retry.
- Added deterministic testkit sources and short-write sinks.

## Versions

| Crate | Version | Change |
| --- | --- | --- |
| `cloud-sdk` | `0.48.0` | streaming contract code |
| `cloud-sdk-hetzner` | `0.36.2` | dependency-only patch |
| `cloud-sdk-reqwest` | `0.32.1` | dependency-only patch |
| `cloud-sdk-sanitization` | `0.16.0` | unchanged; not published |
| `cloud-sdk-testkit` | `0.28.0` | streaming fixture code |

## Documentation

- [`docs/STREAMING.md`](../docs/STREAMING.md)
- [`docs/MIGRATION_0.48.0.md`](../docs/MIGRATION_0.48.0.md)
- [`docs/PUBLIC_API_REVIEW_0.48.0.md`](../docs/PUBLIC_API_REVIEW_0.48.0.md)
- [`docs/DEPENDENCY_REVIEW_0.48.0.md`](../docs/DEPENDENCY_REVIEW_0.48.0.md)

## Pentest

Pentest is required for the exact implementation-stop commit. Temporary
findings belong in root `PENTEST.md` and must be removed after remediation.

## Release Gate

```text
v0.48.0 implementation stop reached. Run pentest for this exact commit.
```
