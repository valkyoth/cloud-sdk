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
| `cloud-sdk-cratesio` | none | `1.1.0` | endpoint-safe candidate provider boundary |

Exact final change classifications are assigned only after the complete train
is implemented.

## Completed Checkpoints

### Commit 1 - Source Lock And Finite Scope

- Locked six bounded official source representations: the public OpenAPI
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
- Made refresh candidates self-contained with bounded source payloads and
  reconstruct every inventory, summary, observation, and provider lock during
  verification, preventing circularly rewritten derivatives from being clean.
- Require all seven stable Cargo contracts to match their public OpenAPI route
  structures, preserving path-parameter identity and position while admitting
  only operation-scoped reviewed aliases such as OpenAPI `{name}` to Cargo
  `{crate_name}`.
- Replaced natural-language policy inference with an exact reviewed-payload
  digest and typed policy, and require the current bytes to equal the policy at
  the reviewed source commit before candidate verification succeeds.
- Reject nonexistent calendar dates in source-review evidence.
- Require every OpenAPI path placeholder to have exactly one direct, required
  path declaration; reject malformed, missing, extra, duplicate, misplaced,
  optional, or referenced path-parameter evidence.
- Bind stable Cargo path parameters to string schemas and exact simple,
  non-exploded, non-reserved, schema-based OpenAPI wire serialization.
- Require the complete reviewed stable path schema, rejecting restrictive JSON
  Schema assertions even when the declared base type remains a string.
- Pin the OpenAPI 3.1 JSON Schema dialect at both the document default and
  nested `$schema` boundaries before deriving Cargo compatibility.
- Restrict nested dialect inspection to real Schema Object positions so
  payload examples and model properties named `$schema` remain valid data.
- Reject Schema Object `$dynamicRef` until dynamic targets and scope can be
  resolved entirely from digest-bound reviewed evidence.
- Scope ordinary `$ref` validation to Schema Objects and typed OpenAPI
  Reference Object positions, preserving local resolution checks without
  rejecting `$ref` properties in arbitrary example payloads.
- Follow each local `$ref` target under its admitting OpenAPI type, bound
  reference cycles, and support strict percent-decoded RFC 6901 object and
  array traversal so hidden external dependencies cannot bypass the source
  lock and valid array pointers do not create false drift failures.
- Apply one exception-safe depth budget to references, inline callbacks, and
  recursive content/header structures so adversarial nesting fails with a
  controlled source-lock error instead of exhausting Python's call stack.

Commit 2 passed its incremental pentest and final remediation retest. The
accepted comparison baseline for Commit 3 is
`17650825a067b0748399ddfbf8cb9f066bd6b66d`; the complete `1.1.0` security
review remains pending until every planned checkpoint and the final
full-service assessment are complete.

### Commit 3 - Crate, Identity, And Module Boundaries

- Added the `cloud-sdk-cratesio` provider crate with empty defaults and a
  `no_std` base.
- Added provider and registry service marker identities without extending a
  closed neutral-core enum.
- Reserved provider-owned catalog, accounts, ownership, publishing, and
  trusted-publishing modules without claiming endpoint support early.
- Defined explicit `alloc`, `serde`, `std`, `blocking`, and `async` feature
  ownership while adding no network, TLS, runtime, filesystem, or clock
  dependency.
- Added exact crate-topology, feature, dependency, provider-isolation,
  platform, package-content, and release-governance checks.
- Disabled Cargo automatic target discovery, declared the reviewed library and
  identity test targets explicitly, and made the boundary gate reject build
  scripts, dependency aliases, source substitutions, and unreviewed dependency
  sections.

Commit 3 passed its incremental pentest, remediation, and final green retest.
The accepted implementation commit is
`5b05e70732f5ffc62949617b22829a613c55388c`; the complete `1.1.0` security
review remains pending until every planned checkpoint and the final
full-service assessment are complete.

### Commit 4 - Endpoint And Request-Target Safety

- Added exact constructors and fixed-origin policies for the production API,
  staging API, and anonymous static package-download authority.
- Added bounded provider-specific wrappers for canonical `/api/v1/` request
  targets and query-free `/crates/{name}/{archive}.crate` targets.
- Added source-correlated download redirect validation that accepts only the
  exact production download route and `static.crates.io` archive destination.
- Added an opaque checked production-response proof minted only by atomic
  execution through the exact production-bound raw executor. The SDK owns the
  bodyless `GET`, empty request headers, and response policy before requiring
  `302` status, empty body, absent content type, one retained `Location`, and
  caller-owned bounded target storage.
- Validate the exact version-download source route before blocking, Send async,
  or local async dispatch, so unrelated generic crates.io API targets fail with
  zero transport calls.
- Kept the structural response constructor private to source execution so safe
  callers cannot combine an unrelated response with a separately verified
  transport to assert production provenance.
- Made redirect following atomic through credential-free blocking, Send async,
  and local async raw executors. The SDK supplies a bodyless `GET` with empty
  headers and exposes neither destination endpoint nor target components.
- Added an explicit HTTPS custom API endpoint that cannot be constructed
  without the provider-neutral trusted-operator acknowledgement.
- Added adversarial host, port, path, query, fragment, user-info, Unicode,
  control-byte, encoded-separator, traversal, downgrade, redirect, archive,
  and authority-confusion coverage.

Commit 4 implementation and pentest remediation are complete. Its green
retest and GitHub checks remain pending before this checkpoint can be accepted.
The comparison baseline is
`716c3ef8dd56a3dcd5881ed70a1ae9011517b3bf`.

### Maintenance Evidence

- Advanced the complete development and compatibility gate to stable Rust
  `1.98.1` and the fuzz compiler to `nightly-2026-09-04` while retaining Rust
  `1.92.0` as the MSRV.
- Verified every workspace and auxiliary-workspace dependency is current,
  every pinned Cargo security/SBOM/fuzz tool matches crates.io, and
  `actions/checkout` remains pinned to the exact latest `v7.0.1` commit.
- Added a manifest-driven live freshness gate for every exact direct library
  pin after confirming that `cargo outdated` resolves inside exact
  requirements and can therefore miss newer crates.io releases.
- Updated `aws-lc-rs` to `1.18.1`, bundled `aws-lc-sys` to `0.45.0`,
  `base64-ng` to `2.0.3`, and `sanitization` to `2.0.4`. These patches retain
  the existing features and MSRVs while adding fail-closed crypto contracts,
  native-build hardening, high-assurance target gating, and protected-storage
  continuity. FIPS remains excluded.
- Refreshed compatible transitive lock entries for `cc`, `find-msvc-tools`,
  `mio`, `smallvec`, `tinyvec`, and `tokio-rustls`; the isolated reqwest
  feature-unification fixture additionally advances its Hickory packages from
  `0.26.1` to `0.26.2`. No default or published dependency capability expands.
- Refreshed crates.io source evidence to upstream commit
  `9ae7f769cea32f38ebc2ea9ec2ce455b47641511` after `find_user` gained an
  optional `include` query and optional `linked_accounts` response data. The
  operation count, authentication, response statuses, media types, Cargo
  compatibility, and data-access policy semantics remain unchanged.
- Re-reviewed the complete 142-entry Hetzner changelog feed after its semantic
  digest changed without a new entry. The latest notice remains the reviewed
  Debian 11 image deprecation, and the machine-readable Hetzner API still
  reports no drift. No SDK behavior or model change is required.
