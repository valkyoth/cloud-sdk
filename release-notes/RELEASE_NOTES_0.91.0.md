# cloud-sdk 0.91.0 Release Notes

Status: implementation stop; pentest required.

Release date: pending

Security-Review: PENDING
Pentest: PENDING
Publication: DEFERRED TO v0.95.0

## Overview

v0.91 implements all six active Hetzner Robot ordering-catalog operations and
starts the v0.91-v0.95 cumulative train. This internal milestone will be tagged
only after its incremental pentest and green CI/CodeQL; it publishes no crate.

## Robot Ordering Catalogs

- Added standard-server list/detail, Server Auction list/detail, per-server
  addon list, and account-currency requests with exact official paths and
  500-request/hour quotas.
- Added bounded protected identifiers, locations, distribution/language
  choices, response text, and exact non-floating-point decimal prices.
- Added strict standard, market, addon, currency, nested price, and nested
  orderable-addon response models with duplicate and cross-field checks.
- Bound detail decoding to the exact requested product identity and required
  structured prices to reference advertised locations.
- Added non-executable standard, auction, and addon plan inputs carrying a
  mandatory revalidation warning for every observed price.
- Added immutable official-source evidence, response examples, mutation-
  resistant contract checks, hostile tests, and a dedicated 4 MiB response
  fuzz target.

## Versions

| Crate | Published | v0.91 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.91.0` | deferred |
| `cloud-sdk-hetzner` | `0.45.0` | `0.45.0` | deferred |
| `cloud-sdk-reqwest` | `0.35.3` | `0.35.3` | unchanged |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.5` | `0.30.5` | unchanged |

## Evidence

- [`docs/PUBLIC_API_REVIEW_0.91.0.md`](../docs/PUBLIC_API_REVIEW_0.91.0.md)
- [`docs/DEPENDENCY_REVIEW_0.91.0.md`](../docs/DEPENDENCY_REVIEW_0.91.0.md)
- [`docs/THREAT_MODEL_DELTA_0.91.0.md`](../docs/THREAT_MODEL_DELTA_0.91.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.91.0.md`](../docs/REJECTED_ABSTRACTIONS_0.91.0.md)
- [`docs/MIGRATION_0.91.0.md`](../docs/MIGRATION_0.91.0.md)

## Stop Gate

Run the incremental pentest for the exact implementation commit. After every
finding is remediated and retested, run `scripts/release_0_91_gate.sh` and
require green GitHub CI/CodeQL on the unchanged evidence commit before tagging.
Do not publish crates for v0.91.
