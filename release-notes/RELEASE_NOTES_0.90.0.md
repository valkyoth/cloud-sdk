# cloud-sdk 0.90.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-14

Security-Review: PASS
Pentest: PASS
Publication: PENDING

## Overview

v0.90 implements all seven active Hetzner Robot vSwitch operations and closes
the cumulative v0.86-v0.90 public checkpoint. No network transport, runtime,
TLS implementation, or retry policy enters the default provider graph.

## Robot vSwitches

- Added list, create, detail, update, cancellation, server attachment, and
  server detachment with exact source-locked methods, paths, quotas, response
  limits, and provider failure classifications.
- Added protected bounded names, VLAN IDs, canonical positive-number/IP server
  selectors, duplicate-free membership sets, and non-empty update intent.
- VLAN admission enforces Hetzner's documented `4000..=4091` range. Outbound
  protected names use a conservative ASCII profile, and canonical IP
  comparison performs no heap allocation.
- Decoded resources distinguish high-assurance names from bounded protected
  quarantined provider names, preserving inventory availability without
  allowing untrusted names to become outbound request values implicitly.
- Added transactional sensitive form encoding for `name`, `vlan`,
  `cancellation_date`, and repeated `server[]` fields.
- Added strict bounded list/detail/create response decoding with finite server
  status, canonical network and gateway validation, duplicate rejection, and
  exact create-request reconciliation.
- Added request-bound direct/shared mutation and destructive permits. Automatic
  retry remains forbidden for every state change.
- Empty update/cancel/attach/detach responses return no inferred state. A later
  detail read is required when reconciliation matters, and Robot exposes no
  revision binding between those observations.
- Added immutable source evidence, mutation-resistant source checks,
  adversarial unit tests, deterministic fuzz seeds, and a dedicated vSwitch
  response fuzz target.

## Versions

| Crate | Published | v0.90 | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.90.0` | selected after all release gates |
| `cloud-sdk-hetzner` | `0.44.0` | `0.45.0` | selected after all release gates |
| `cloud-sdk-reqwest` | `0.35.2` | `0.35.3` | selected, dependency-only |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.4` | `0.30.5` | selected, dependency-only |

## Evidence

- [`docs/PUBLIC_API_REVIEW.md#v0900`](../docs/PUBLIC_API_REVIEW.md#v0900)
- [`docs/DEPENDENCY_REVIEW.md#v0900`](../docs/DEPENDENCY_REVIEW.md#v0900)
- [`docs/THREAT_MODEL_DELTA.md#v0900`](../docs/THREAT_MODEL_DELTA.md#v0900)
- [`docs/REJECTED_ABSTRACTIONS.md#v0900`](../docs/REJECTED_ABSTRACTIONS.md#v0900)
- [`docs/MIGRATION.md#v0900`](../docs/MIGRATION.md#v0900)

The permanent pentest report records the exact reviewed implementation and
remediation commits. No finding remains open.

## Stop Gate

Run the incremental pentest for the exact committed implementation, publish
the permanent report, run `scripts/release_0_90_gate.sh`, and require green
GitHub CI/CodeQL on the unchanged release-evidence commit before tagging or
publishing the four selected crates.
