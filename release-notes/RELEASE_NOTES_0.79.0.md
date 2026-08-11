# cloud-sdk 0.79.0 Milestone Notes

Status: implementation stop reached; pentest required.

Release date: pending

Security-Review: PASS
Pentest: REQUIRED
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
- Added exact `200` JSON policies for reads/creates and exact empty-body,
  no-content-type policies for revoke.
- Added strict bounded models and decoders with request/response identity
  binding, date/state consistency, earliest-date enforcement, reservation
  conflict checks, source-specific reason shapes, and canonical subnet checks.
- Source-locked both documented IP/subnet cancellation date spellings and
  requires exactly one, reflecting the official table/example inconsistency.
- Added a dedicated source fixture/checker, regression suite, and direct
  checked-response fuzz target with deterministic server/IP/subnet seeds.

## Versions

| Crate | Published | v0.79 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.75.0` | `0.79.0` | deferred to v0.80.0 |
| `cloud-sdk-hetzner` | `0.42.0` | `0.42.0` | code accumulated, no publication |
| `cloud-sdk-reqwest` | `0.35.0` | `0.35.0` | unchanged |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.2` | `0.30.2` | unchanged |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.79.0.md`](../docs/PUBLIC_API_REVIEW_0.79.0.md)
- [`docs/DEPENDENCY_REVIEW_0.79.0.md`](../docs/DEPENDENCY_REVIEW_0.79.0.md)
- [`docs/THREAT_MODEL_DELTA_0.79.0.md`](../docs/THREAT_MODEL_DELTA_0.79.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.79.0.md`](../docs/REJECTED_ABSTRACTIONS_0.79.0.md)
- [`docs/MIGRATION_0.79.0.md`](../docs/MIGRATION_0.79.0.md)

## Release Gate

Do not run `scripts/release_0_79_gate.sh` until the pentest report is committed.
After the final evidence commit, GitHub CI and CodeQL must be green before the
signed internal tag. Do not publish crates for this milestone.
