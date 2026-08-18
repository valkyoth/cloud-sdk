# cloud-sdk 0.79.0 Milestone Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-11

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.80.0

## Overview

v0.79 implements all nine active Hetzner Robot cancellation operations for
servers, individual IPs, and subnets. This is an internal milestone; no crate
is selected for crates.io publication.

## Robot Cancellations

- Added named GET, POST, and DELETE request types for server, IP, and subnet
  cancellation routes.
- Added canonical protected IP/subnet identities and calendar-valid protected
  dates without ordinary owned secret copies or payload-bearing diagnostics.
- Added explicit immediate/date scheduling, bounded redacted server reasons,
  and explicit location-reservation intent.
- Classified create and revoke operations as destructive with automatic retry
  forbidden. Create is non-idempotent; revoke is idempotent but still requires
  explicit reconciliation after uncertain delivery.
- Added exact `200` JSON policies for reads, creates, and IP/subnet revokes;
  only server revoke uses the documented empty-body, no-content-type policy.
- Added strict bounded models and decoders with request/response identity
  binding, date/state consistency, earliest-date enforcement, reservation
  conflict checks, source-specific reason shapes, and canonical subnet checks.
- Source-locked both documented IP/subnet cancellation date spellings and
  requires exactly one, reflecting the official table/example inconsistency.
- Added a dedicated source fixture/checker, regression suite, and direct
  checked-response fuzz target with deterministic server/IP/subnet seeds.
- Pentest remediation binds checked responses to the exact cancellation
  request type and instance across direct validation and blocking, Send-async,
  or local-async destructive permit execution. The permit path returns the
  bound checked response directly and exposes no caller-rebinding operation.
- Pentest remediation also validates complete POST intent, including distinct
  unavailable omission, available reservation, and explicit non-reservation
  acknowledgements; validates inactive revocation outcomes; shares Unicode
  display-safety policy; and preserves protected-date allocation errors.
- Added cancellation-specific direct/shared and
  blocking/Send-async/local-async permit tests, POST digest and DELETE exact
  fingerprint coverage, post-execution mismatch rejection, and unpolled
  cleanup/reconciliation evidence. Documentation now states that sensitive
  POST forms require the digest builder while bodyless DELETE permits either
  exact or digest fingerprints.

## Versions

| Crate | Published | v0.79 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.75.0` | `0.79.0` | deferred to v0.80.0 |
| `cloud-sdk-hetzner` | `0.42.0` | `0.42.0` | code accumulated, no publication |
| `cloud-sdk-reqwest` | `0.35.0` | `0.35.0` | unchanged |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.2` | `0.30.2` | unchanged |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW.md#v0790`](../docs/PUBLIC_API_REVIEW.md#v0790)
- [`docs/DEPENDENCY_REVIEW.md#v0790`](../docs/DEPENDENCY_REVIEW.md#v0790)
- [`docs/THREAT_MODEL_DELTA.md#v0790`](../docs/THREAT_MODEL_DELTA.md#v0790)
- [`docs/REJECTED_ABSTRACTIONS.md#v0790`](../docs/REJECTED_ABSTRACTIONS.md#v0790)
- [`docs/MIGRATION.md#v0790`](../docs/MIGRATION.md#v0790)
- [`security/pentest/v0.79.0.md`](../security/pentest/v0.79.0.md)

## Release Gate

Run `scripts/release_0_79_gate.sh` only after the pentest report is committed.
GitHub CI and CodeQL must be green on the unchanged final evidence commit
before the signed internal tag. Do not publish crates for this milestone.
