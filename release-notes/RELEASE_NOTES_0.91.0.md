# cloud-sdk 0.91.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-14

Security-Review: PASS
Pentest: PASS
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
- Retained the exact per-server request in decoded addon catalogs so safe Rust
  cannot relabel one server's catalog or plan with another server identity.
- Completely redacted addon-selection diagnostics, volatile-cleared decimal
  scalar mirrors on drop, and made the ordering example own successful-path
  request-target cleanup.
- Stored validated plan selections as direct references, removing production
  impossible-state panic paths from ordering plan accessors.
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
- [`security/pentest/v0.91.0.md`](../security/pentest/v0.91.0.md)

## Stop Gate

The incremental pentest and final remediation retest are green. Run
`scripts/release_0_91_gate.sh`, then require green GitHub CI and CodeQL on the
unchanged evidence commit before tagging. Do not publish crates for v0.91.
