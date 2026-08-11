# cloud-sdk 0.77.0 Milestone Notes

Status: implementation complete; pentest required.

Release date: 2026-08-11

Security-Review: PENDING
Pentest: REQUIRED
Publication: DEFERRED TO v0.80.0

## Overview

v0.77 adds strict bounded Hetzner Robot error, quota, authentication, and
maintenance decoding. This is an internal milestone; no crate is selected for
crates.io publication.

## Robot Protocol

- Added an admitted-response decoder for bodyless 401 authentication rejection,
  400 `INVALID_INPUT`, 403 `RATE_LIMIT_EXCEEDED`, 404 `SERVER_NOT_FOUND`, and
  bodyless 503 maintenance.
- Enforced JSON media type, a 64 KiB body limit, exact envelopes, status
  agreement, duplicate rejection, array and string bounds, and nonzero quota
  values.
- Protected provider messages and input names in cleanup-owning storage with
  closure-scoped access and redacted diagnostics.
- Added finite retry dispositions. Authentication rejection is never retryable;
  maintenance, quota, and explicitly classified transport failures remain
  caller-policy decisions.
- Kept transient transport construction separate from provider bytes so
  unknown status and code values fail closed.
- Added an adversarial fuzz target plus malformed, oversized, duplicate,
  unknown, mismatch, quota, redaction, and retry-denial tests. Its release
  smoke admits the selector byte plus the complete 65,536-byte body boundary.
- Preserved local parser allocation failures as a distinct payload-free
  `RobotDecodeError::Allocation` instead of classifying them as hostile input.

## Versions

| Crate | Published | v0.77 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.75.0` | `0.77.0` | deferred to v0.80.0 |
| `cloud-sdk-hetzner` | `0.42.0` | `0.42.0` | code accumulated, no publication |
| `cloud-sdk-reqwest` | `0.35.0` | `0.35.0` | unchanged |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.2` | `0.30.2` | unchanged |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.77.0.md`](../docs/PUBLIC_API_REVIEW_0.77.0.md)
- [`docs/DEPENDENCY_REVIEW_0.77.0.md`](../docs/DEPENDENCY_REVIEW_0.77.0.md)
- [`docs/THREAT_MODEL_DELTA_0.77.0.md`](../docs/THREAT_MODEL_DELTA_0.77.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.77.0.md`](../docs/REJECTED_ABSTRACTIONS_0.77.0.md)
- [`docs/MIGRATION_0.77.0.md`](../docs/MIGRATION_0.77.0.md)

## Release Gate

After the pentest report is committed, run `scripts/release_0_77_gate.sh` on
the clean final evidence commit. GitHub CI and CodeQL must be green on that
unchanged commit before the signed internal tag. Do not publish crates.
