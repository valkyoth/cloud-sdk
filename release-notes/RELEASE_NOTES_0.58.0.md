# cloud-sdk 0.58.0 Milestone Notes

Status: implementation stop reached; incremental pentest required.

Release date: pending

Security-Review: PASS
Pentest: REQUIRED
Publication: DEFERRED TO v0.60.0

## Overview

v0.58 challenges neutral endpoint and authentication contracts against the
source-locked OVHcloud API v2 probe. It adds exact geographic API/token pairing
and an explicit-time expiring bearer lifecycle without publishing an OVHcloud
provider or adding a default dependency.

This milestone receives an incremental pentest from signed v0.57.0, the full
local and GitHub gates, and a normal signed tag. No crate is published until
the v0.60.0 checkpoint.

## Authority Policy

- Added bounded allocation-free regional API/token pair contracts.
- Bound EU and CA probe identities to their exact reviewed token authorities.
- Rejected aliases, cross-region combinations, unknown regions, duplicate
  identities, HTTP, and credentialed redirects.
- Kept the OVHcloud probe excluded from packages and support claims.

## Expiring Credentials

- Added explicit caller-clock lifetime construction from `expires_in`.
- Added exclusive expiry, rollback detection, and nonempty bounded refresh
  windows; zero refresh margins fail closed.
- Required time-qualified refresh handoffs for expiring credentials.
- Rotated token and replacement lifetime atomically under lineage and
  generation compare-and-swap.
- Preserved in-flight snapshots and complete mutable-source cleanup.
- Kept static and expiring lifecycle modes type-visible and non-convertible by
  ordinary rotation.

## Versions

| Crate | Source version | Cumulative change | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.58.0` | code | deferred to v0.60.0 |
| `cloud-sdk-hetzner` | `0.39.0` | dependency | deferred |
| `cloud-sdk-reqwest` | `0.32.4` | code | deferred |
| `cloud-sdk-sanitization` | `0.18.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.29.0` | dependency | deferred |

## Documentation

- [`provider-probes/ovhcloud-v2/README.md`](../provider-probes/ovhcloud-v2/README.md)
- [`docs/AUTHENTICATION_POLICY.md`](../docs/AUTHENTICATION_POLICY.md)
- [`docs/SECURITY_RECIPES.md`](../docs/SECURITY_RECIPES.md)
- [`docs/MIGRATION_0.58.0.md`](../docs/MIGRATION_0.58.0.md)
- [`docs/PUBLIC_API_REVIEW_0.58.0.md`](../docs/PUBLIC_API_REVIEW_0.58.0.md)
- [`docs/DEPENDENCY_REVIEW_0.58.0.md`](../docs/DEPENDENCY_REVIEW_0.58.0.md)

## Release Gate

Run `scripts/release_0_58_gate.sh` only after the incremental pentest and final
retest pass and permanent evidence is committed at
`security/pentest/v0.58.0.md`. Tag only after that clean gate plus GitHub CI and
CodeQL are green. Do not publish crates for this internal milestone.
