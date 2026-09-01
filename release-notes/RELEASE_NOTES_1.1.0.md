# cloud-sdk 1.1.0 Release Notes

Status: unreleased crates.io API implementation candidate.

Release date: pending

Security-Review: PENDING
Pentest: PENDING
Publication: BLOCKED DURING CANDIDATE TRAIN

## Overview

The `1.1.0` development line adds a complete source-locked crates.io provider
while preserving the stable Hetzner provider and provider-neutral execution
boundaries introduced in `1.0.0`. Work follows the numbered checkpoints in
`docs/cratesio-commit-plan.md`. Each checkpoint is committed, incrementally
pentested against the preceding accepted checkpoint, and kept untagged.

No crate may be published while `release-crates.toml` uses
`stage = "candidate"`. The final checkpoint will replace this draft with exact
API, dependency, security, migration, and package-change evidence before the
plan can become `public`.

## Candidate Scope

- source-lock all public crates.io OpenAPI operations and stable Cargo Registry
  Web API overlaps;
- add one `cloud-sdk-cratesio` provider crate with empty default features;
- implement checked request, response, authentication, pagination, publishing,
  ownership, trusted-publishing, download, and unified-client paths;
- enforce crates.io access-policy, rate, user-agent, retry, mutation, and
  credential boundaries; and
- qualify the complete provider through drift checks, adversarial tests,
  fuzzing, platform evidence, pentest, CI, and CodeQL.

## Candidate Versions

| Crate | Published | Candidate | Current state |
| --- | --- | --- | --- |
| `cloud-sdk` | `1.0.0` | `1.1.0` | candidate metadata |
| `cloud-sdk-hetzner` | `1.0.0` | `1.1.0` | candidate metadata; stable behavior unchanged |
| `cloud-sdk-reqwest` | `1.0.0` | `1.1.0` | candidate metadata |
| `cloud-sdk-sanitization` | `1.0.0` | `1.1.0` | candidate metadata |
| `cloud-sdk-testkit` | `1.0.0` | `1.1.0` | candidate metadata |

`cloud-sdk-cratesio` will enter this table when Commit 3 creates the provider
crate. Exact final change classifications are assigned only after the complete
train is implemented.

## Completed Checkpoints

### Commit 1 - Source Lock And Finite Scope

- Locked five bounded official source representations: the public OpenAPI
  document, stable Cargo Registry Web API contract, deployed data-access route,
  and commit-pinned upstream OpenAPI and policy implementations.
- Classified all 51 public operations across 40 paths and retained the two
  upstream-deprecated operations as explicit rows.
- Mapped the seven stable Cargo operations to their OpenAPI rows and excluded
  Cargo's `/me` browser instruction from API coverage.
- Recorded observed and admitted authentication, request and response schema
  fingerprints, media types, statuses, and policy classifications.
- Added offline validation, explicit live reconstruction, and adversarial
  regression tests for redirects, bounds, malformed evidence, unresolved
  references, unknown authentication, and incomplete classification.
- Bound every requested and final source URL to its exact official authority
  and path, bound committed inventories by SHA-256, and required live
  reconstruction in the final release gate.
- Made path-token and OIDC-body authentication conditional on exact upstream
  structures, and rejected TRACE explicitly rather than omitting it from
  coverage.

Commit 1 passed its incremental pentest, remediation retest, and GitHub checks.

### Commit 2 - Drift, Policy, And Compatibility Detection

- Added a crates.io adapter for the provider-neutral drift engine with
  operation, parameter, schema, authentication, content-type, response-status,
  stability, Cargo-contract, and policy fingerprints.
- Added an exact current-policy observation alongside the commit-pinned policy
  provenance so rate, identifying `User-Agent`, fallback, contact, and
  preferred-data-source changes are visible.
- Added canonical payload-free reporting for additions, removals, renames,
  changed requiredness and schemas, authentication changes, status/media
  changes, and stable Cargo/OpenAPI conflicts.
- Added a bounded candidate-only refresh workflow that validates every source
  and artifact before one non-overwriting atomic publication and never mutates
  accepted repository evidence.
- Added semantic fixtures for every drift family plus incomplete policy,
  unavailable refresh, and candidate-overwrite rejection.

Security review remains pending until the Commit 2 incremental pentest is
complete.
