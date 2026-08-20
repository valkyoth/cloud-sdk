# cloud-sdk 1.0.0 Release Notes

Status: stable release candidate; full-project pentest passed.

Release date: 2026-08-20

Security-Review: PASS
Pentest: PASS
Publication: PENDING

## Overview

v1.0.0 promotes the published v0.100.0 candidate to the stable SemVer
contract. It adds no runtime API, behavior, feature, target, provider scope, or
third-party dependency change. A repository gate compares normalized manifests,
lockfiles, and public package trees directly with signed tag `v0.100.0`.

## Stable Scope

- `cloud-sdk` provides the provider-neutral no_std-first operation, transport,
  authentication, execution, pagination, retry, cleanup, and policy contracts.
- `cloud-sdk-hetzner` covers the complete claimed non-deprecated Hetzner Cloud,
  DNS, Security, Console Storage Box, Robot, and canonical Server Metadata API
  scope recorded in `docs/HETZNER_1_0_SCOPE.md`.
- `cloud-sdk-reqwest` provides optional blocking and async native transports on
  Linux, Windows, macOS, and FreeBSD. Portable targets use the neutral traits.
- `cloud-sdk-sanitization` and `cloud-sdk-testkit` provide the reviewed secret
  cleanup and deterministic test boundaries used by the workspace.
- FIPS is excluded from v1.0 and remains deferred until the separately reviewed
  Brynja integration is ready.

## Versions

| Crate | Previous | v1.0 | Change |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.100.0` | `1.0.0` | stable metadata promotion |
| `cloud-sdk-hetzner` | `0.47.0` | `1.0.0` | stable metadata promotion |
| `cloud-sdk-reqwest` | `0.37.0` | `1.0.0` | stable metadata promotion |
| `cloud-sdk-sanitization` | `0.19.0` | `1.0.0` | stable metadata promotion |
| `cloud-sdk-testkit` | `0.31.1` | `1.0.0` | stable metadata promotion |

## Security Review

The independent full-project assessment reviewed the exact stable candidate
against `v0.100.0` and found no Critical, High, Medium, or Low issue. The
permanent report is recorded in `security/pentest/v1.0.0.md`.

## Remaining Gates

Run `scripts/release_1_0_gate.sh` on the exact final evidence commit. Tag and
publish only after that unchanged commit passes the complete local gate,
GitHub CI, and CodeQL.
