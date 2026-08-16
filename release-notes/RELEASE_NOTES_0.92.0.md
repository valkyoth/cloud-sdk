# cloud-sdk 0.92.0 Release Notes

Status: implementation stop; incremental pentest required.

Release date: TBD

Security-Review: PENDING
Pentest: PENDING
Publication: DEFERRED TO v0.95.0

## Overview

v0.92 implements all six active read-only Hetzner Robot transaction operations
and continues the v0.91-v0.95 cumulative train. This internal milestone will be
tagged only after its incremental pentest and green CI/CodeQL; it publishes no
crate.

## Robot Transactions

- Added standard-server, Server Auction, and per-server addon transaction
  list/detail requests with exact source-locked paths and one typed shared
  500-request-per-hour account quota.
- Modeled Robot's fixed 30-day list window without inventing pagination.
- Added protected transaction identifiers, calendar-valid timestamps, finite
  status, server results, SSH metadata, product snapshots, exact prices, and
  addon resources.
- Added strict bounded response decoding with duplicate, unknown-field,
  nullability, timestamp, decimal, key-shape, and identity-substitution checks.
- Bound every detail response to its exact request and admitted only the
  source-locked `404 NOT_FOUND` provider failure.
- Added official-source fixtures, mutation-resistant contract checks,
  adversarial tests, and a dedicated 4 MiB response fuzz target that invokes
  every decoder from valid deep list/detail corpus entries.
- Replaced panic-only production preparation invariants with payload-free typed
  target and policy errors while preserving complete caller-buffer cleanup.
- Added no purchase request, cost permit, automatic retry, client, runtime,
  transport, dependency, feature, or unsafe code.

## Versions

| Crate | Published | v0.92 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.92.0` | deferred |
| `cloud-sdk-hetzner` | `0.45.0` | `0.45.0` | deferred |
| `cloud-sdk-reqwest` | `0.35.3` | `0.35.3` | unchanged |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.5` | `0.30.5` | unchanged |

## Evidence

- [`docs/PUBLIC_API_REVIEW_0.92.0.md`](../docs/PUBLIC_API_REVIEW_0.92.0.md)
- [`docs/DEPENDENCY_REVIEW_0.92.0.md`](../docs/DEPENDENCY_REVIEW_0.92.0.md)
- [`docs/THREAT_MODEL_DELTA_0.92.0.md`](../docs/THREAT_MODEL_DELTA_0.92.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.92.0.md`](../docs/REJECTED_ABSTRACTIONS_0.92.0.md)
- [`docs/MIGRATION_0.92.0.md`](../docs/MIGRATION_0.92.0.md)

## Stop Gate

Run the incremental pentest on the exact implementation commit. After a green
retest, add the permanent report, run `scripts/release_0_92_gate.sh`, and
require green GitHub CI and CodeQL on the unchanged evidence commit before
tagging. Do not publish crates for v0.92.
