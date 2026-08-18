# cloud-sdk 0.95.0 Release Notes

Status: implementation stop; pentest required.

Release date: pending

Security-Review: PENDING
Pentest: PENDING
Publication: PENDING

## Overview

v0.95 adds carefully isolated read-only Hetzner Robot live evidence and closes
the cumulative v0.91-v0.95 public checkpoint. It publishes ordering catalogs,
transaction snapshots, guarded billable ordering, all 89 active Robot
operations through the official typed client, credential lockout, and the
read-only operator harness without changing any default dependency graph.

## Robot Live Evidence

- Added an ignored typed `GET /server` probe through `RobotClient::official`,
  the existing scoped Basic transport, bounded caller-owned workspace, strict
  server-list decoder, and one-attempt credential generation.
- Added separate bounded private username/password file ingestion with
  symlink, hard-link, permission, identity, size, empty, and same-file
  rejection plus complete source cleanup and redacted diagnostics.
- Extended credential-free staging with a distinct root-owned Robot launcher
  and manifest format 3 binding both launchers, the executable, runner, and
  reviewed commit.
- Made the isolated runner reject mixed Cloud/Robot credentials, destructive
  opt-in, incomplete files, arbitrary arguments, and all operation selection.
- Added source and regression gates proving CI receives no Robot credentials
  and the live path contains no mutation, order, transaction, custom endpoint,
  invalid-login test, or automatic retry.
- Documented least-capability account setup, privileged sealing, private-file
  handling, output policy, lockout risk, revocation, and residual cleanup
  boundaries.

## Cumulative Robot Checkpoint

- Publishes the v0.91 ordering catalogs, v0.92 transaction snapshots, v0.93
  cost-authorized orders, and v0.94 complete typed Robot clients and
  one-generation authentication lockout.
- Retains permit gates for every state change, exact official endpoint policy,
  request-bound checked decoding, bounded single-response lists, no synthetic
  Robot pager/action APIs, no implicit retry, and no default transport.

## Versions

| Crate | Published | v0.95 | Publication |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.95.0` | selected after all release gates |
| `cloud-sdk-hetzner` | `0.45.0` | `0.46.0` | selected, cumulative code |
| `cloud-sdk-reqwest` | `0.35.3` | `0.36.0` | selected, cumulative code |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged and excluded |
| `cloud-sdk-testkit` | `0.30.5` | `0.31.0` | selected, cumulative code |

## Evidence

- [`docs/PUBLIC_API_REVIEW_0.95.0.md`](../docs/PUBLIC_API_REVIEW_0.95.0.md)
- [`docs/DEPENDENCY_REVIEW_0.95.0.md`](../docs/DEPENDENCY_REVIEW_0.95.0.md)
- [`docs/THREAT_MODEL_DELTA_0.95.0.md`](../docs/THREAT_MODEL_DELTA_0.95.0.md)
- [`docs/REJECTED_ABSTRACTIONS_0.95.0.md`](../docs/REJECTED_ABSTRACTIONS_0.95.0.md)
- [`docs/MIGRATION_0.95.0.md`](../docs/MIGRATION_0.95.0.md)

## Stop Gate

Run the pentest for the exact committed implementation, publish the permanent
report, execute `scripts/release_0_95_gate.sh`, and require green GitHub CI and
CodeQL on the unchanged release-evidence commit before tagging or publishing
the four selected crates.
