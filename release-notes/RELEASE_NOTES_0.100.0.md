# cloud-sdk 0.100.0 Release Notes

Status: implementation candidate; fresh pentest required.

Release date: 2026-08-19

Security-Review: PENDING
Pentest: PENDING
Publication: PENDING

## Overview

v0.100 is the final development checkpoint before v1.0. It publishes the
cumulative request-fidelity, Server Metadata, platform/MSRV, release-
governance, provenance, and mutation safety work from v0.96-v0.100 while
freezing the public API candidate.

## Mutation Safety

- Automated checks select exact typed Cloud, DNS, Security, Console Storage,
  Robot mutation, and Robot billable-order reconciliation paths without
  accepting credentials or performing network I/O.
- The Robot order path proves price, cost-permit, delivery, and reconciliation
  controls without dispatch. The release process never purchases a server.
- Live provider mutation and mandatory operational attestation are deferred to
  a separately reviewed future milestone and do not block this release.

## Cumulative Public Surface

- `cloud-sdk-hetzner` adds complete source-locked query arguments, corrected
  Server metrics, canonical Server Metadata models and fixed-origin execution,
  and current response semantics while retaining all 304 active Cloud-family
  and 89 active Robot operation claims.
- `cloud-sdk-reqwest` adds credential-free IPv4 link-local raw HTTP builders
  and makes its native transport target set explicit: Linux, Windows, macOS,
  and FreeBSD.
- `cloud-sdk` adds plain-text and YAML media-type constants. Default graphs
  remain `no_std`, transport-free, runtime-free, and FIPS-free.
- The cumulative workspace API passes semver comparison against v0.95. The
  removed Primary IP `type` filter is retained as a deprecated fail-closed
  compatibility shim; Image name/label filtering uses `SourceLockedQuery`
  without changing the established `ImageListRequest` generic arity.
- Every selected package's README Cargo examples use `=version` requirements
  for exact planned first-party releases. The release gate detects dependency
  tables regardless of fence labeling and rejects dependency skew or malformed
  TOML across CommonMark backtick and tilde fence variants.

## Versions

| Crate | Published | v0.100 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.95.0` | `0.100.0` | pending |
| `cloud-sdk-hetzner` | `0.46.0` | `0.47.0` | pending |
| `cloud-sdk-reqwest` | `0.36.0` | `0.37.0` | pending |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.31.0` | `0.31.1` | pending dependency update |

## Remaining Gates

Run the fresh incremental pentest from signed `v0.99.0`, commit the permanent
report, and run `scripts/release_0_100_gate.sh`. Tag and publish only after the
exact final commit passes the full gate, GitHub CI, and CodeQL.
