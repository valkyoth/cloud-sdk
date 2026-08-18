# cloud-sdk 0.76.0 Milestone Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-11

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.80.0

## Overview

v0.76 adds protected Hetzner Robot credentials and a provider-neutral
lockout-aware attempt lifecycle. This is an internal milestone; no crate is
selected for crates.io publication.

## Credential Attempt Lifecycle

- Added allocation-free caller-owned atomic state with nonzero monotonic
  generations and open or rejected status.
- Authentication rejection closes one exact generation globally and
  idempotently; stale attempts cannot close replacement credentials.
- Allocation-free attempts borrow their issuing state; validation and
  rejection return `ForeignState` before inspecting equal generation numbers
  from another owner.
- Added an `alloc`-gated owned lineage variant for Robot and other task-owned
  integrations. Attempt creation only clones the existing lineage, does not
  borrow credentials, and permits rotation while older requests are in flight.
- Removed `Hash` from owner-bound attempts so caller-supplied hashers cannot
  observe state addresses.
- Newly supplied credentials may advance from open or rejected state.
- Unchanged credentials require an explicit consumed reconfirmation token and
  can be reconfirmed only after rejection.
- Added concurrent same-generation, stale transition, rejection, replacement,
  reconfirmation, exhaustion, cross-owner confusion, in-flight rotation, and
  stale-response tests.

## Residual Operational Boundary

The alloc-backed owned attempt lineage uses infallible `Arc::new` state
construction. Allocator exhaustion may therefore abort the process instead of
returning an SDK error. This is within the documented process-abort threat
model boundary. High-availability and regulated deployments must enforce
external memory limits and process supervision.

## Robot Credentials

- Added `RobotService`, `ROBOT_SERVICE_ID`, and the exact official Robot HTTPS
  endpoint policy.
- Added `alloc`-gated non-`Clone` protected username/password ownership with
  mutable and guarded ingestion and rotation.
- Made the provider `alloc` feature explicitly activate the first-party
  sanitization allocation support and added standalone production checks for
  the no-default, `alloc`, and `std` feature modes.
- Fixed every credential to Hetzner, Robot, and
  `https://robot-ws.your-server.de/`; Cloud and altered endpoint scope fail.
- Revalidate attempt state before closure-scoped secret access and prevent
  borrowed secrets from escaping through a compile-fail contract.
- Clear caller sources on every return and retain payload-free diagnostics.
- Keep Basic authorization encoding, typed 401 decoding, retries, clients, and
  live execution outside this milestone.

## Versions

| Crate | Published | v0.76 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.75.0` | `0.76.0` | deferred to v0.80.0 |
| `cloud-sdk-hetzner` | `0.42.0` | `0.42.0` | code accumulated, no publication |
| `cloud-sdk-reqwest` | `0.35.0` | `0.35.0` | unchanged |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.2` | `0.30.2` | unchanged |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW.md#v0760`](../docs/PUBLIC_API_REVIEW.md#v0760)
- [`docs/DEPENDENCY_REVIEW.md#v0760`](../docs/DEPENDENCY_REVIEW.md#v0760)
- [`docs/THREAT_MODEL_DELTA.md#v0760`](../docs/THREAT_MODEL_DELTA.md#v0760)
- [`docs/REJECTED_ABSTRACTIONS.md#v0760`](../docs/REJECTED_ABSTRACTIONS.md#v0760)
- [`docs/MIGRATION.md#v0760`](../docs/MIGRATION.md#v0760)
- [`security/pentest/v0.76.0.md`](../security/pentest/v0.76.0.md)

## Release Gate

Run `scripts/release_0_76_gate.sh` on the clean final evidence commit. GitHub
CI and CodeQL must be green on that unchanged commit before the signed
internal tag. Do not publish crates.
