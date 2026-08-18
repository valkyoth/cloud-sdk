# cloud-sdk 0.85.0 Release Notes

Status: released and published.

Release date: 2026-08-13

Security-Review: PASS
Pentest: PASS
Publication: COMPLETE

## Overview

v0.85 completes all 15 active Hetzner Robot boot-configuration operations and
closes the v0.81-v0.85 cumulative train. After pentest, final release gates,
and green GitHub CI/CodeQL, this checkpoint will publish only the changed
facade, Hetzner provider, reqwest adapter, and testkit crates.

## Robot Boot Configuration

- Added exact official overview and Rescue/Linux/VNC/Windows get, activate,
  deactivate, and supported last-operation request types.
- Added bounded selectors, keyboard layouts, languages, and duplicate-free
  repeated authorized-key fingerprints with atomic sensitive forms.
- Classified all mutations as non-idempotent and retry-denied; Linux, VNC,
  and Windows activation is destructive.
- Added protected typed overview/family models and strict exact-request
  decoding for identity, selector, language, active state, passwords, and
  generated key material.
- Bound decoding to exact overview/current/last/mutation response shapes,
  admitted the documented inactive Windows overview null language, and
  rejected contradictory state or multiple active overview families.
- Added operation-specific boot/Windows error narrowing and explicit handling
  of source-locked deprecated response fields without exposing them.
- Added a 15-operation immutable source fixture, mutation-resistant checker,
  focused compiled policy tests, compile-fail provenance, deterministic seeds,
  and a direct bounded response fuzz target.
- Renamed optional generated-field controls in test fixtures so CodeQL does
  not misclassify fixture-state booleans as hard-coded passwords.

## Cumulative Checkpoint

The publication includes reviewed v0.81 subnet, v0.82 reset, v0.83 failover,
and v0.84 Wake-on-LAN support. No external dependency version or feature
changes. Reqwest and testkit move only because their exact provider-neutral
core dependency requirement changes.

## Versions

| Crate | Previous published | v0.85 | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.80.0` | `0.85.0` | selected |
| `cloud-sdk-hetzner` | `0.43.0` | `0.44.0` | selected |
| `cloud-sdk-reqwest` | `0.35.1` | `0.35.2` | selected, dependency-only |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged, not selected |
| `cloud-sdk-testkit` | `0.30.3` | `0.30.4` | selected, dependency-only |

## Evidence

- [`docs/PUBLIC_API_REVIEW.md#v0850`](../docs/PUBLIC_API_REVIEW.md#v0850)
- [`docs/DEPENDENCY_REVIEW.md#v0850`](../docs/DEPENDENCY_REVIEW.md#v0850)
- [`docs/THREAT_MODEL_DELTA.md#v0850`](../docs/THREAT_MODEL_DELTA.md#v0850)
- [`docs/REJECTED_ABSTRACTIONS.md#v0850`](../docs/REJECTED_ABSTRACTIONS.md#v0850)
- [`docs/MIGRATION.md#v0850`](../docs/MIGRATION.md#v0850)
- [`security/pentest/v0.85.0.md`](../security/pentest/v0.85.0.md)

## Release Gate

Run `scripts/release_0_85_gate.sh` on the clean final evidence commit. GitHub
CI and CodeQL must be green on that unchanged commit before signing the tag
and publishing the four selected crates.
