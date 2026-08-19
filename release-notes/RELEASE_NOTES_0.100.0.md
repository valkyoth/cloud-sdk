# cloud-sdk 0.100.0 Release Notes

Status: pentest passed; controlled mutation required.

Release date: 2026-08-19

Security-Review: PASS
Pentest: PASS
Publication: PENDING

## Overview

v0.100 is the final development checkpoint before v1.0. It publishes the
cumulative request-fidelity, Server Metadata, platform/MSRV, release-
governance, provenance, and controlled-mutation work from v0.96-v0.100 while
freezing the public API candidate.

## Controlled Mutation

- Automated checks select exact typed Cloud, DNS, Security, Console Storage,
  Robot mutation, and Robot billable-order reconciliation paths without
  accepting credentials or performing network I/O.
- Manual evidence requires a disposable scope, explicit approval and EUR
  ceiling, a unique run prefix, one attempt per reversible scenario, exact
  plan and cleanup fingerprints, resolved delivery, unique opaque resource
  references, one cleanup ledger, independent zero inventories, billing
  review, and credential revocation.
- The Robot order path proves price, cost-permit, delivery, and reconciliation
  controls without dispatch. The release process never purchases a server.
- Release binding rejects evidence that predates its controls or any code,
  manifest, lockfile, workflow, or documentation change after live
  qualification.
- Evidence parsing uses bounded no-follow regular-file reads, rejects duplicate
  fields and boolean integer aliases, and validates the exact committed Git
  blob and reviewed executable mode rather than a worktree path or symlink
  target. Malformed scalar types produce static diagnostics without tracebacks.

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

## Versions

| Crate | Published | v0.100 source | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.95.0` | `0.100.0` | pending |
| `cloud-sdk-hetzner` | `0.46.0` | `0.47.0` | pending |
| `cloud-sdk-reqwest` | `0.36.0` | `0.37.0` | pending |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged |
| `cloud-sdk-testkit` | `0.31.0` | `0.31.1` | pending dependency update |

## Remaining Gates

Complete and commit `security/mutation/v0.100.0.json` against the exact frozen
source commit, then run `scripts/release_0_100_gate.sh`. The incremental pentest
from signed `v0.99.0` and remediation retest passed. Tag and publish only after
the exact final commit passes the full gate, GitHub CI, and CodeQL.
