# cloud-sdk 0.80.0 Release Notes

Status: implementation stop reached; pentest required.

Release date: pending

Security-Review: PENDING
Pentest: REQUIRED
Publication: PENDING

## Overview

v0.80 implements every active Hetzner Robot single-IP and separate-MAC
operation and closes the v0.76-v0.80 cumulative train. After pentest and the
unchanged final release gate, this checkpoint publishes the accumulated Robot
foundation and independently versioned neutral crates.

## Robot IP Management

- Added named list, detail, traffic-update, MAC-get, MAC-generate, and
  MAC-delete requests with exact official endpoint, Basic scope, method, path,
  query/form, operation metadata, and `200` JSON policy.
- Added an optional canonical IPv4 server-address list filter that rejects
  IPv6 before transport, plus non-empty partial traffic updates for warning
  state and hourly/daily megabyte and monthly gigabyte thresholds.
- Added protected canonical lowercase EUI-48 values and bounded typed models
  for assignment, server, lock, traffic, network, and nullable separate-MAC
  state.
- Added strict decoders rejecting unknown/duplicate fields, oversized or
  duplicate lists, identity/filter mismatch, inconsistent networks,
  contradictory traffic acknowledgement, and wrong nullable-MAC outcomes.
- Duplicate list identities use fallible `O(n log n)` sorted-index scratch,
  and protected MAC equality is constant-time.
- Added request-bound prepared and checked responses plus exact/digest plan,
  direct/shared mutation/destructive permit, and blocking/Send-async/local-
  async execution support.
- Sensitive update forms require digest fingerprints. MAC generation is
  non-idempotent and MAC deletion is destructive; both deny automatic retry.
- Added preparation-failure and unpolled-attempt cleanup evidence, six-row
  source fixtures/checkers, a compiled six-operation security-policy matrix,
  and a direct checked-response fuzz target with deterministic
  list/detail/MAC/delete selectors.

## Cumulative Checkpoint

The publication includes v0.76 protected Robot credentials and lockout-aware
attempts, v0.77 strict error/quota handling, v0.78 server operations, and v0.79
cancellation operations. No external dependency version or feature changes.

## Versions

| Crate | Previous published | v0.80 | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.75.0` | `0.80.0` | selected |
| `cloud-sdk-hetzner` | `0.42.0` | `0.43.0` | selected |
| `cloud-sdk-reqwest` | `0.35.0` | `0.35.1` | selected, dependency-only |
| `cloud-sdk-sanitization` | `0.18.0` | `0.19.0` | selected |
| `cloud-sdk-testkit` | `0.30.2` | `0.30.3` | selected, dependency-only |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.80.0.md`](../docs/PUBLIC_API_REVIEW_0.80.0.md)
- [`docs/DEPENDENCY_REVIEW_0.80.0.md`](../docs/DEPENDENCY_REVIEW_0.80.0.md)
- [`docs/THREAT_MODEL_DELTA_0.80.0.md`](../docs/THREAT_MODEL_DELTA_0.80.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.80.0.md`](../docs/REJECTED_ABSTRACTIONS_0.80.0.md)
- [`docs/MIGRATION_0.80.0.md`](../docs/MIGRATION_0.80.0.md)
- `security/pentest/v0.80.0.md` after the final pentest and retest

## Release Gate

After the pentest report is committed, run `scripts/release_0_80_gate.sh`.
GitHub CI and CodeQL must be green on the unchanged final evidence commit
before signing the tag and publishing the selected crates.
