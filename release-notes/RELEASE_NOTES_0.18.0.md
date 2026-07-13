# cloud-sdk 0.18.0 Release Notes

Status: implementation stop reached; pentest and retest required before tagging.

## Overview

`0.18.0` adds explicit provider-neutral pagination and action polling state
machines, validated rate-limit transport metadata, strict Hetzner pagination
response parsing, and deterministic fixture support. The helpers remain
no_std, allocation-free, runtime-neutral, clock-free, and transport-neutral.

## Pagination

- `PaginationCursor` requires a caller-selected starting page and hard page
  limit.
- Callers fetch and decode each page, then submit validated metadata and the
  exact decoded entry count.
- Repeated or unexpected pages, contradictory navigation, empty non-terminal
  pages, zero page values, and page-limit exhaustion fail without advancing.
- Advertised previous and next pages must be exactly adjacent to the current
  page; checked arithmetic rejects gaps and page-number overflow.
- Known last pages must agree with terminal state. Decoded entries cannot
  exceed `per_page`, and supplied totals must match the exact page entry count
  and continuation state before the cursor advances.
- Every accepted `PageBoundary` exposes page metadata, decoded entry count,
  terminal state, and optional response rate-limit metadata.
- Hetzner's optional Serde boundary extracts strict `meta.pagination` fields
  from any list response while accepting additive provider fields.
- Hetzner page defaults now match the official source: default 25 and maximum
  50 unless an endpoint explicitly documents another limit.

## Action Polling

- `ActionPoller` accepts decoded running, successful, or failed updates.
- Terminal provider failure payloads are returned unchanged to the caller.
- Terminal success and failure take precedence over progress telemetry, so a
  lower or malformed final progress value cannot suppress the provider result.
- Running observations invoke caller-owned `PollPolicy`, which explicitly
  chooses a nonzero delay, cancellation, or timeout.
- Progress regression, progress above 100, zero-delay busy loops, observation
  overflow, and polling after a terminal step fail closed.
- The SDK performs no request, sleep, retry, timeout measurement, or runtime
  selection on the caller's behalf.

## Transport Metadata

- `TransportResponse` carries optional validated provider-neutral rate-limit
  metadata while preserving its initialized caller-buffer body contract.
- Blocking and async reqwest adapters parse `RateLimit-Limit`,
  `RateLimit-Remaining`, and `RateLimit-Reset` as one complete decimal set with
  each header occurring exactly once.
- Partial, empty, signed, non-decimal, overflowing, zero-limit, and incoherent
  or duplicate header sets fail before a response body is exposed as
  successful.
- Testkit fixtures can attach coherent rate-limit metadata to paginated,
  action, success, error, or `429` responses.

## Release Source Integrity

- Live Hetzner and IANA drift checks use explicit default validating TLS
  contexts and require exact pinned HTTPS URLs without redirects.
- Bounded response bytes must match pinned SHA-256 digests before OpenAPI JSON
  or registry CSV parsing begins.

## Version Plan

- `cloud-sdk` publishes its code release as `0.18.0`.
- `cloud-sdk-hetzner` publishes its code release as `0.16.0`.
- `cloud-sdk-reqwest` publishes its code release as `0.15.0`.
- `cloud-sdk-testkit` publishes its code release as `0.15.0`.
- `cloud-sdk-sanitization` publishes dependency-only patch `0.13.4`.
- Retired and provider-scoped helper packages remain rejected and unpublished.

## Verification

- `scripts/checks.sh`
- `scripts/check_reqwest_boundary.sh`
- `scripts/check_rust_version_matrix.sh`
- `scripts/release_0_18_gate.sh`
- `cargo deny check`
- `cargo audit`
- `git diff --check`

## Pentest

Pentest and retest are required before tagging. The final report must be the
only change in the direct child of the reviewed implementation commit.
