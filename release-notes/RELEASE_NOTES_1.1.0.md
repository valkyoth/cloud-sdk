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
