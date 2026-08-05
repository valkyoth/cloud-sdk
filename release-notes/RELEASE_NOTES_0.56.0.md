# cloud-sdk 0.56.0 Milestone Notes

Status: implementation stop reached; pentest required.

Release date: pending

Security-Review: PASS
Pentest: REQUIRED
Publication: DEFERRED TO v0.60.0

## Overview

v0.56 adds a provider-neutral drift-evidence engine for future provider probes
and crates. It preserves the complete Hetzner-specific OpenAPI checker while
proving that its existing source locks can be represented by the neutral
model.

This milestone receives an incremental pentest from signed v0.55.0, the full
local and GitHub gates, and a normal signed tag. No crate is published until
the v0.60.0 checkpoint.

## Drift Model

- Added strict, bounded plugin, provider-lock, and observation documents.
- Covered authentication, cost, endpoints, headers, idempotency, operations,
  pagination, retry, and schema evidence.
- Added deterministic field-level reports containing hashes and RFC 6901 JSON
  pointers rather than source or normalized values.
- Added explicit security, provider, and release owners plus per-category
  add/change/remove severity.

## Source Retrieval

- Requires hard-coded provider/source endpoint approval, globally routable DNS
  results, exact credential-free HTTPS URLs, and platform TLS verification.
- Disables ambient proxy configuration and documents the required egress
  control against DNS rebinding on high-assurance release hosts.
- Rejects redirects before following them and validates the final URL.
- Enforces per-source byte/read-time limits and a killable 180-second
  whole-plan deadline.
- Authenticates every complete response before invoking a hard-coded reviewed
  adapter, derives live evidence from those bytes, and independently requires
  the result to match the tracked observation.
- Derives authentication, server, response-header, and pagination evidence
  directly from OpenAPI; reconstructs repository-owned policies from no-follow
  hashed files instead of copying lock contracts.
- Contains fetch, parse, normalization, comparison, and reporting in the
  deadline worker and transfers only a bounded 2 MiB report across IPC.

## Release Process

- Restored incremental pentesting for every tag beginning with v0.56.0.
- Retained five-minor crates.io publication checkpoints; v0.60.0 is next.
- Added an immediate `review_baseline` distinct from the public package
  baseline and fail-closed readiness tests for both release stages.
- Requires exactly one value for every release-note and pentest evidence field,
  rejecting duplicated or contradictory security status.

## Versions

| Crate | Source version | Publication |
| --- | --- | --- |
| `cloud-sdk` | `0.56.0` | deferred to v0.60.0 |
| `cloud-sdk-hetzner` | `0.39.0` | unchanged |
| `cloud-sdk-reqwest` | `0.32.4` | unchanged |
| `cloud-sdk-sanitization` | `0.18.0` | unchanged |
| `cloud-sdk-testkit` | `0.29.0` | unchanged |

## Documentation

- [`docs/PROVIDER_DRIFT.md`](../docs/PROVIDER_DRIFT.md)
- [`docs/MIGRATION_0.56.0.md`](../docs/MIGRATION_0.56.0.md)
- [`docs/PUBLIC_API_REVIEW_0.56.0.md`](../docs/PUBLIC_API_REVIEW_0.56.0.md)
- [`docs/DEPENDENCY_REVIEW_0.56.0.md`](../docs/DEPENDENCY_REVIEW_0.56.0.md)

## Release Gate

After the incremental pentest passes, replace `Pentest: REQUIRED` with
`Pentest: PASS`, add the permanent report, and run
`scripts/release_0_56_gate.sh`. Tag only after that clean gate plus GitHub CI
and CodeQL are green. Do not publish crates for this internal milestone.
