# cloud-sdk 0.17.0 Release Notes

Status: implementation stop reached; pentest and retest required before tagging.

## Overview

`0.17.0` adds a runtime-neutral async transport contract to the no_std core,
an async deterministic testkit implementation, and an optional hardened async
reqwest/rustls adapter. No transport, allocator, TLS stack, or async runtime
enters any default provider or facade graph.

## Async Contract

- `cloud-sdk::transport::AsyncTransport` returns a `Send` future without
  selecting an executor, allocator, TLS stack, or retry policy.
- `cloud-sdk-testkit::MockTransport` implements blocking and async contracts
  from one validation/write path and needs no runtime.
- Dropping an unpolled mock future does not consume an exchange or modify the
  caller buffer.

## Async Reqwest Adapter

- `cloud-sdk-reqwest/async-rustls` uses the same validated endpoint, bearer
  token, timeout, user-agent, error, and hardened client policies as the
  blocking adapter.
- Callers provide an active Tokio executor; the adapter does not own or install
  a runtime.
- Request and temporary response allocations are sanitized through
  `cloud-sdk-sanitization`.
- Responses accumulate only up to caller capacity and copy into the cleared
  caller buffer after complete success. Cancellation, timeout, read failure,
  and overflow cannot expose a partial successful response.
- Redirects and automatic retries remain disabled. A `429` or other status is
  returned to provider/caller policy without hidden replay.

## Tests And Gates

- Added separate async-only, blocking-only, and combined feature checks.
- Added deterministic Tokio loopback tests for exact headers/targets/bodies,
  redirect refusal, timeout, cancellation, body bounds, and redaction.
- Extended the locked HTTP/2/Hickory feature-unification fixture to both
  adapters.
- Enforced that reqwest, bytes, Tokio, hyper, and rustls remain absent from the
  default facade, provider, and testkit graphs.
- Bound the complete release gate to a clean unchanged commit at entry and
  exit. Shared readiness rejects modified tracked files and all untracked files.

## Version Plan

- `cloud-sdk` publishes its code release as `0.17.0`.
- `cloud-sdk-reqwest` publishes its code release as `0.14.0`.
- `cloud-sdk-testkit` publishes its code release as `0.14.0`.
- `cloud-sdk-hetzner` publishes dependency-only patch `0.15.2`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.3`.
- Retired and provider-scoped helper packages remain rejected and unpublished.

## Verification

- `scripts/checks.sh`
- `scripts/check_reqwest_boundary.sh`
- `scripts/check_rust_version_matrix.sh`
- `scripts/release_0_17_gate.sh`
- `cargo deny check`
- `cargo audit`
- `git diff --check`

## Pentest

Pentest and retest are required before tagging. The final report must be the
only change in the direct child of the reviewed implementation commit.
