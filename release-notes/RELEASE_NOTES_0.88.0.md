# cloud-sdk 0.88.0 Release Notes

Status: release candidate; pentest and final retest passed.

Release date: 2026-08-13

Security-Review: PASS
Pentest: PASS
Publication: DEFERRED TO v0.90.0

## Overview

v0.88 implements all five active Hetzner Robot SSH-key operations and
continues the v0.86-v0.90 cumulative train. This internal milestone will be
tagged only after its incremental pentest and green CI/CodeQL; it publishes no
crate.

## Robot SSH Keys

- Added list, create, get, rename, and delete with exact source-locked methods,
  paths, forms, quotas, statuses, empty-body behavior, and failures.
- Added protected bounded names and canonical MD5 path fingerprints plus
  bounded OpenSSH and RFC 4716 SSH2 create input.
- Added strict owned key and list models with redacted key data, validated
  timestamps, duplicate rejection, and exact request/response association.
- Parse normalized OpenSSH responses as RFC 4253 key wire, require algorithm
  and size coherence, verify provider MD5, and compute SHA-256 independently.
- Bind successful create to the requested name and normalized key identity,
  rename to name plus fingerprint, and get/delete to exact path identity.
- Require strong-digest mutation authority for sensitive forms and separate
  destructive authority for delete; automatic retry remains forbidden.
- Added immutable source evidence, checker mutation tests, focused hostile
  fixtures, deterministic exact-limit and limit-plus-one response tests, and
  get/list/RFC 4716 create decoder fuzzing that admits the complete 2 MiB list
  range.
- Hardened key names, OpenSSH comments, and SSH2 headers against Unicode
  controls and directional formatting; made fingerprint intermediates
  non-copying cleanup owners; and moved prepared-policy validation before
  request-storage borrowing.

## Versions

| Crate | Published | v0.88 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.85.0` | `0.88.0` | deferred |
| `cloud-sdk-hetzner` | `0.44.0` | `0.44.0` | deferred |
| `cloud-sdk-reqwest` | `0.35.2` | `0.35.2` | unchanged |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.4` | `0.30.4` | unchanged |

## Evidence

- [`docs/PUBLIC_API_REVIEW_0.88.0.md`](../docs/PUBLIC_API_REVIEW_0.88.0.md)
- [`docs/DEPENDENCY_REVIEW_0.88.0.md`](../docs/DEPENDENCY_REVIEW_0.88.0.md)
- [`docs/THREAT_MODEL_DELTA_0.88.0.md`](../docs/THREAT_MODEL_DELTA_0.88.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.88.0.md`](../docs/REJECTED_ABSTRACTIONS_0.88.0.md)
- [`docs/MIGRATION_0.88.0.md`](../docs/MIGRATION_0.88.0.md)
- [`security/pentest/v0.88.0.md`](../security/pentest/v0.88.0.md)

## Stop Gate

The incremental pentest and final remediation retest passed. Run
`scripts/release_0_88_gate.sh` and require green GitHub CI/CodeQL on the
unchanged evidence commit before tagging. Do not publish crates for v0.88.
