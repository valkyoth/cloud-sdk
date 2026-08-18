# cloud-sdk 0.89.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-14

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.90.0

## Overview

v0.89 implements all eight active Hetzner Robot firewall and firewall-template
operations and continues the v0.86-v0.90 cumulative train. This internal
milestone will be tagged only after green CI/CodeQL on the unchanged release
evidence commit; it publishes no crate.

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
- Aligned port/protocol validation and all eight 500-per-hour quotas with the
  official source examples; digest-bound examples now execute through the Rust
  decoder.
- Added complete protected rule/template accessors and fixed-work comparison.
  Detailed responses may omit the documented template name, so mutation
  decoding reports non-erasable pending state instead of unconfirmed success.
  Confirmation consumes that state with a same-ID, name-bearing list summary
  and checks the protected name, policy flags, and detailed ordered rules
  against the exact mutation configuration retained by the pending type.
  Callers cannot substitute replacement intent during reconciliation.
- Documented Robot's non-atomic list/detail observation boundary: callers must
  exclude concurrent template mutation or repeat reconciliation after a
  possible race because Robot supplies no revision binding the two reads.
- Rejected additional bidirectional and invisible Unicode formatting and made
  all form-builder allocations explicitly fallible.
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

- [`docs/PUBLIC_API_REVIEW.md#v0890`](../docs/PUBLIC_API_REVIEW.md#v0890)
- [`docs/DEPENDENCY_REVIEW.md#v0890`](../docs/DEPENDENCY_REVIEW.md#v0890)
- [`docs/THREAT_MODEL_DELTA.md#v0890`](../docs/THREAT_MODEL_DELTA.md#v0890)
- [`docs/REJECTED_ABSTRACTIONS.md#v0890`](../docs/REJECTED_ABSTRACTIONS.md#v0890)
- [`docs/MIGRATION.md#v0890`](../docs/MIGRATION.md#v0890)
- [`security/pentest/v0.89.0.md`](../security/pentest/v0.89.0.md)

## Stop Gate

The incremental pentest and final remediation retest passed. Run
`scripts/release_0_89_gate.sh` on the committed release evidence and require
green GitHub CI/CodeQL on that unchanged commit before tagging. Do not publish
crates for v0.89.
