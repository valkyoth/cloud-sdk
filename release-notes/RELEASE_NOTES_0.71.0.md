# cloud-sdk 0.71.0 Milestone Notes

Status: implementation stop reached; pentest and retest required before tagging.

Release date: 2026-08-09

Security-Review: PENDING
Pentest: PENDING
Publication: DEFERRED TO v0.75.0

## Overview

v0.71 completes named Hetzner DNS client workflows for all 24 active DNS
operations and retires the experimental FIPS transport. This is an internal
milestone; no crate is selected for crates.io publication.

## DNS Client Methods

- Added generated named methods for eight read-only, nine mutation, and seven
  destructive DNS operations.
- Preserved blocking, `Send` async, and local-async parity, four numbered list
  policies, checked actions, zonefile responses, and exact service identity.
- Kept state-changing requests behind cleanup-owning preparation and exact
  plan-confirm permit attempts, with no client-owned retries or authority.
- Added deterministic paginated reads, permit-authorized action execution,
  TSIG redaction and cleanup, unpolled cancellation cleanup, and an ignored
  named read-only live smoke path.
- Added `DNS_CLIENT_METHODS` and generator checks tied to the source-locked
  operation manifest.

## FIPS Deferment

- Removed `blocking-rustls-fips`, `FipsTlsPolicy`, FIPS-specific builder and
  error APIs, and `aws-lc-fips-sys` from active source and dependency graphs.
- Retained ordinary blocking, deterministic-root, and async rustls transports.
- Added a fail-closed CI policy preventing accidental FIPS reintroduction.
- Deferred any future optional FIPS integration until Brynja meets exact
  module, certificate, environment, build, runtime, and review conditions.

## Versions

| Crate | Published | v0.71 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.70.0` | `0.71.0` | deferred to v0.75.0 |
| `cloud-sdk-hetzner` | `0.41.0` | `0.41.0` | code accumulated, no publication |
| `cloud-sdk-reqwest` | `0.34.1` | `0.34.1` | code accumulated, no publication |
| `cloud-sdk-sanitization` | `0.18.0` | `0.18.0` | unchanged |
| `cloud-sdk-testkit` | `0.30.1` | `0.30.1` | unchanged |

## Release Evidence

- [`docs/PUBLIC_API_REVIEW_0.71.0.md`](../docs/PUBLIC_API_REVIEW_0.71.0.md)
- [`docs/DEPENDENCY_REVIEW_0.71.0.md`](../docs/DEPENDENCY_REVIEW_0.71.0.md)
- [`docs/THREAT_MODEL_DELTA_0.71.0.md`](../docs/THREAT_MODEL_DELTA_0.71.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.71.0.md`](../docs/REJECTED_ABSTRACTIONS_0.71.0.md)
- [`docs/MIGRATION_0.71.0.md`](../docs/MIGRATION_0.71.0.md)

## Release Gate

Run the incremental pentest from signed v0.70.0 after this implementation
commit. After remediation and a green retest, add the pentest report, run
`scripts/release_0_71_gate.sh` on the clean final evidence commit, and require
green GitHub CI and CodeQL before the signed internal tag. Do not publish
crates.
