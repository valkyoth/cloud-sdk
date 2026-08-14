# cloud-sdk 0.89.0 Release Notes

Status: implementation stop; pentest required.

Release date: pending

Security-Review: PENDING
Pentest: PENDING
Publication: DEFERRED TO v0.90.0

## Overview

v0.89 implements all eight active Hetzner Robot firewall and firewall-template
operations and continues the v0.86-v0.90 cumulative train. This internal
milestone will be tagged only after its incremental pentest and green
CI/CodeQL; it publishes no crate.

## Robot Firewalls

- Added server firewall get, complete replacement, and clear plus template
  list, create, get, complete replacement, and delete.
- Added bounded ordered rules, canonical IPv4 selectors and ports, exact
  protocols/actions/statuses, bounded TCP flags, protected names, and non-zero
  template identities.
- Made inline and template replacement mutually exclusive and rejected every
  source-locked IP/protocol/port/flag conflict.
- Added strict protected firewall, template, and inventory models with exact
  request identity and mutation-outcome association.
- Added request-bound mutation/destructive permits, immutable source evidence,
  mutation-resistant checks, hostile tests, and direct response fuzzing across
  the complete 2 MiB template-list range.

## Versions

| Crate | Published | v0.89 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.89.0` | deferred |
| `cloud-sdk-hetzner` | `0.44.0` | `0.44.0` | deferred |
| `cloud-sdk-reqwest` | `0.35.2` | `0.35.2` | unchanged |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.4` | `0.30.4` | unchanged |

## Evidence

- [`docs/PUBLIC_API_REVIEW_0.89.0.md`](../docs/PUBLIC_API_REVIEW_0.89.0.md)
- [`docs/DEPENDENCY_REVIEW_0.89.0.md`](../docs/DEPENDENCY_REVIEW_0.89.0.md)
- [`docs/THREAT_MODEL_DELTA_0.89.0.md`](../docs/THREAT_MODEL_DELTA_0.89.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.89.0.md`](../docs/REJECTED_ABSTRACTIONS_0.89.0.md)
- [`docs/MIGRATION_0.89.0.md`](../docs/MIGRATION_0.89.0.md)

## Stop Gate

Run the incremental pentest for the exact implementation commit. After every
finding is remediated and retested, run `scripts/release_0_89_gate.sh` and
require green GitHub CI/CodeQL on the unchanged evidence commit before tagging.
Do not publish crates for v0.89.
