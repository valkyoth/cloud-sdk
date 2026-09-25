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
| `cloud-sdk` | `1.0.0` | `1.1.0` | shared incremental JSON decoder; default graph unchanged |
| `cloud-sdk-hetzner` | `1.0.0` | `1.1.0` | stable API; compatibility exports use the shared decoder |
| `cloud-sdk-reqwest` | `1.0.0` | `1.1.0` | reviewed TLS and authentication dependency updates |
| `cloud-sdk-sanitization` | `1.0.0` | `1.1.0` | reviewed sanitization dependency update |
| `cloud-sdk-testkit` | `1.0.0` | `1.1.0` | candidate metadata |
| `cloud-sdk-cratesio` | none | `1.1.0` | endpoint-safe provider, protected credentials, wire, typed query and pagination foundations |

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

Commit 4 passed its incremental pentest, remediation, and final green retest.
The accepted implementation commit is
`b613b1a25b31a85a04b0d79b955364a8e2a65ee9`; the complete `1.1.0` security
review remains pending until every planned checkpoint and the final
full-service assessment are complete.

### Commit 5 - Credentials And Authentication Contexts

- Added five non-cloneable protected credential kinds behind `alloc`, with
  immutable production/staging origins and source-locked operation contexts.
- Reused the admitted neutral sanitization crate for fallible protected storage,
  ownership transfer, local rotation and full caller-buffer cleanup.
- Added bounded raw Authorization, Bearer, OIDC JSON and secret-path formatting
  through an explicit trusted adapter callback. No executable authenticated
  client, token acquisition, remote revocation or automatic retry is claimed.
- Added positive/adversarial tests, compile-fail type/lifetime checks and exact
  API-token route comparison with the source inventory. Default features stay
  empty and no new third-party package is admitted.

Commit 5 passed its incremental pentest and remediation retest from accepted
checkpoint `3c9b2ad6c230f069b75b1138cbbebecbe350aba7` through reviewed commit
`e79273b7f4d4bafa0520e0b7105dd11012b782af`. GitHub checks passed. The train
remains blocked from publication until all numbered checkpoints are accepted.

Commit 5 pentest remediation adds independent fixed-operation method/path
assertions at both constructor and emitted-material boundaries, including
temporary-token revocation. This closes a regression-test gap without changing
the currently correct runtime constructors. Both regression tests reject each
of ten isolated method/path mutations; the restored snapshot passes. The final
user-confirmed retest found no remaining issue.

Checkpoint qualification also corrected the isolated package-feature gate to
patch both unpublished local dependencies of `cloud-sdk-cratesio`, matching the
main check gate. A mock-Cargo regression test asserts the actual complete
package command. This changes only build verification, not SDK behavior.

### Commit 6 - Wire, Error, Rate, And Response Foundations

- Refreshed exact pins to `base64-ng 2.0.4`, `sanitization 2.1.0`, and
  `rustls 0.23.44` after the live freshness gate identified the newer releases.
  Existing feature selection and MSRV remain unchanged; the dependency digest
  records archive checksums and optional TLS behavior changes.
- Reused the bounded incremental JSON decoder in `cloud-sdk/alloc`, retaining
  Hetzner's existing public reexports and moving all parser regression tests
  with the implementation. No new runtime third-party dependency is added.
- Added cleanup-owning crates.io JSON success admission with exact status,
  lowered byte budgets, strict UTF-8 JSON media policy, complete duplicate-key
  and structural validation, and Cargo error-envelope rejection even on 200.
- Added payload-free provider errors, explicit 429/503 classification and
  bounded `Retry-After` parsing through the existing neutral HTTP-date parser.
- Added identifying user-agent validation, clock-free non-burst scheduling and
  a process-wide blocking adapter gate with a full one-second quiet period
  after every attempt. No retries, sleeps, or network calls are implicit.
- Tightened contact validation after pentest: reject malformed email dot-atoms,
  local parts above 64 bytes, and invalid or oversized DNS labels in email and
  HTTPS contacts. Regression tests retain the exact 256-byte header boundary.
- Operation-specific models, asynchronous client integration and complete
  authenticated execution remain assigned to later checkpoints. This is an
  implementation checkpoint, not a service release.

Commit 6 passed its incremental pentest and remediation retest at `d2a71050`.
The [permanent report](../security/pentest/cratesio-commit-6.md) records the
reviewed range. GitHub CI and CodeQL passed at `f079a65c`, the accepted baseline
for Commit 7.

### Commit 7 - Identifiers, Queries, And Pagination

- Added allocation-free borrowed crate names, exact Cargo versions, category
  hierarchies, keywords, user/team logins, owners, numeric IDs and dates.
- Added operation-scoped include, sort and filter values, bounded pages and
  opaque seek state; reject duplicates, shadowed filters and page/seek conflicts.
- Reused atomic snapshot encoding for typed paths, percent-encoded queries,
  repeated array pairs and complete operation-bound targets.
- Added meta next/previous link validation and legacy `more` continuation,
  preserving original filters and page sizes across exact official origins.
  Transport transfer retains the core cleanup-owning link's dispatch checks.
- Reused neutral traversal budgets; rate admission stays explicit and separate.
- Added a source-pinned request-policy checker for implementation limits and
  all public query-name fixtures, with live verification in the final gate.
- Admitted exact `semver 1.0.28` only as a dev-only independent grammar oracle;
  it adds no runtime/default dependency or change to existing provider APIs.
- Refreshed reqwest to `0.13.5`, including its transitive `base64 0.23.1`
  admission and narrowly scoped duplicate-line exception. The SDK's own
  credential encoding remains on `base64-ng`; transport is still opt-in.
- Added boundary, mutation, operation-matrix, SemVer differential, transport
  binding, cleanup and documentation examples. See the
  [request policy](../docs/CRATESIO_REQUEST_POLICY.md).

GitHub passed on `89910ad7`; the user authorized advancing to Commit 8.
The independent Hetzner drift detected on September 9 is resolved below.

### Commit 8 - Taxonomy And Site Discovery

- Implemented all seven category, slug, keyword, site metadata and summary
  operations, including complete nested Crate and CrateLinks response fields.
- Added typed atomic targets, exact source operation metadata, explicit
  numbered continuations, and required/nullable/unknown-field validation.
- Pentest remediation rejects supplied pagination links (including nulls)
  that contradict derived next/previous state. Category and keyword regression
  matrices preserve completion, continuation and page-ceiling distinctions.
- Reused the neutral incremental JSON parser and protected staging; added
  bounded models, calendar timestamps and redacted open-ended badge values.
- Added official anonymous blocking, local-async and Send-async discovery.
  All modes bind origin and identifying user-agent, enforce response policy
  and clear caller storage, including cancelled and unpolled futures.
- Added the neutral `BoundUserAgent` inspection trait and implementations for
  raw reqwest adapters. Transport-owned user-agent cannot be overridden by
  ordinary headers; loopback tests compare configuration with wire bytes.
- The process gate now holds owned admission through async completion without
  holding a mutex across await. Cancellation consumes admission; valid
  Retry-After can only extend the quiet interval. No hidden retry or sleep.
- Added seven pinned-source fixtures, generator mutation tests, complete
  required-field/limit/identity/parity tests and a compile-checked README example.
  Existing `serde_json` is a dev-only fixture oracle, not a runtime dependency.
- Resolved [Hetzner API drift](../docs/SPEC_LOCK.md#reviewed-live-drift-2026-09-10):
  no endpoint/parameter inventory change; 730 response-field rows now cover
  required nullable image deprecation, IP name bounds and all target-health
  variants. Regenerated fixtures preserve the existing health cross-field rules.
- Reviewed the September 8 legacy `deprecated` removal announcement and
  accept omission at only the six affected resource paths while retaining
  validation for present values and required replacement fields. The September 9
  Object Storage notice is outside current SDK scope.

Commit 8 passed incremental pentest and remediation retest for
`89910ad7cab7a1549a67f16e7bea7556557c4f3e..2c0746982c927929f63a08ff034bd3d132f0a4d3`,
including the neutral transport and Hetzner maintenance changes. The
[permanent report](../security/pentest/cratesio-commit-8.md) records the resolved
pagination finding and qualification evidence. GitHub CI and CodeQL must pass
on the evidence checkpoint before Commit 9 starts. The user subsequently
confirmed GitHub green on `71e3f972` and authorized Commit 9.
The [discovery contract](../docs/CRATESIO_DISCOVERY_POLICY.md) records limits
and verification commands. No tag or publication is authorized.

### Commit 9 - Crate Search And Metadata

- Added all three source-locked catalog GET operations: crate list/search,
  named metadata, and the literal `new` lookup (never the publish PUT route).
- Separated the stable Cargo minimal search profile from complete crates.io
  web records, including sort/filter/page policy and explicit include selectors.
- Added bounded metadata expansion, full known included-version schema checks,
  protected forward-compatible version fields and default-version binding.
- Validated returned continuation authority, path, filters, size and direction.
  Numbered-page and relevance ceilings report `LimitReached`, not false completion.
- Reused discovery's private checked runner for blocking/local/Send execution,
  exact wire admission, shared one-second rate gate and cancellation cleanup.
- Added optional raw API-token list execution through a checked blocking
  trusted-adapter callback. No Bearer conversion, cookie support, automatic
  credentials, custom origin, retry or bulk traversal was added.
- Added three pinned-source fixtures, a fail-closed included-version schema
  generator, regression tests and compile-checked examples. Dependencies and
  default features are unchanged. Live crates.io and Hetzner drift checks were
  clean on 2026-09-11.

The [catalog contract](../docs/CRATESIO_CATALOG_POLICY.md) records exact scope
and verification. Incremental pentest and remediation retest passed through
`41fd2611`, including the shared discovery runner and pagination helper.
GitHub was confirmed green on evidence checkpoint `38d493a1`; Commit 9 is accepted.

Commit 9 pentest remediation bounds provider-requested delays to 24 hours and
rejects larger values without poisoning the shared gate, including the token
adapter. Schema generation rejects `oneOf` sibling constraints and invalid
branch counts; runtime branch validation preserves allocation failures instead
of reporting them as limit errors. Regression tests reproduce all three
findings and verify bounded scheduling recovery, cleanup and error propagation.
The [remediation record](../docs/CRATESIO_CATALOG_POLICY.md#pentest-remediation)
and [permanent pentest report](../security/pentest/cratesio-commit-9.md) record
the user-confirmed green retest with no open findings. The intentional bounded
24-hour process-wide pause remains an availability consideration, not a new
vulnerability. GitHub approval of the evidence checkpoint was confirmed.
No publication is authorized. Full-service security review and
pentest remain pending until the final planned checkpoint.

### Commit 10 - Version Details

- Pentest remediation exposes closure-scoped `DiscoveryValue::visit_fields`
  so consumers can enumerate dynamic feature names and release tracks.
  External-consumer regression coverage checks enumeration, redaction,
  empty/non-object handling, callback error propagation and buffer cleanup.
  The independent retest closed F1 at `94c004b2` with no new findings; see the
  [checkpoint report](../security/pentest/cratesio-commit-10.md). GitHub approval
  remains required before Commit 11.
- Add checked version list/detail, dependency, deprecated author and README
  location operations with blocking, local-async and Send-async parity.
- Require explicit bounded seek pagination, retain complete version metadata,
  reject identity/duplicate/checksum inconsistencies, and preserve unknown kinds.
- Select the JSON README location profile; never follow its redirect or render
  static HTML. Requirements remain bounded metadata, not a resolver.
- Share the existing schema validator without changing catalog validation;
  add source-derived projections, fixtures and controller byte locks.
- Review the OpenAPI owner-response and policy-page drift without expanding
  runtime ownership scope or changing the request-rate admission policy.

The [version contract](../docs/CRATESIO_VERSION_POLICY.md) records scope and
limits. Pentest must cover `38d493a1..HEAD`. Stop before Commit 11; do not tag or
publish. The version surface adds no dependency. Maintenance also updates the
exact rustls pin from `0.23.44` to `0.23.45` in all three affected lockfiles and
regenerates their SBOMs. This fixes TLS 1.3 encryption-level message alignment
([GHSA-2mjx-qc3c-rqvc](https://github.com/rustls/rustls/security/advisories/GHSA-2mjx-qc3c-rqvc));
default features, TLS provider and native build policy are unchanged. The
transport security update is included in this checkpoint's pentest range.
Local qualification passed at implementation checkpoint `6db21a77`; the
[verification record](../docs/CRATESIO_VERSION_POLICY.md#local-qualification-2026-09-15)
lists repository, compiler/platform, packaging, fuzz and supply-chain evidence.
This is the implementation stop, not an independent pentest result.

### Commit 11 - Downloads And Usage

- Add four checked JSON operations for archive locations, crate/version count
  windows and numbered reverse dependencies, with blocking/local/Send parity.
- Add allocation-free static artifact streaming contracts, mandatory SHA-256
  hooks, explicit bounds, transactional sink commit and cancellation cleanup.
  A caller streaming adapter is required; the bundled raw reqwest executor is
  still a buffered adapter. No hashing/transport dependency was added.
- Retain protected metadata with exact date, count, ID and include correlations;
  distinguish numbered continuation exhaustion from end-of-results.
- Refresh crates.io documentation-only source hashes and byte-lock three more
  controller implementations. Hetzner's newly detected network-members endpoint
  remains unaccepted pending separate SDK coverage; live drift stays visible.
- Replace consumer README version pins with feature-correct `cargo add`
  instructions; preserve exact workspace dependency pins. Combine the Hetzner
  provider row and remove the internal OVHcloud probe from the provider table.

The [download contract](../docs/CRATESIO_DOWNLOAD_POLICY.md) records complete
boundaries and pending drift. The [incremental pentest report](../security/pentest/cratesio-commit-11.md)
records no confirmed security findings for `51a7d946..03301ac5`.
The user confirmed GitHub green on `5c925018` and authorized Commit 12.
No tag or publication is authorized.

### Commit 12 - Public Accounts And Ownership Reads

- Add anonymous user lookup/statistics, qualified team lookup, and combined,
  user-only and team-only owner lists with all three checked execution modes.
- Preserve typed identity namespaces, protected nullable metadata and explicit
  linked-account inclusion. Validate response identity, duplicate owners and
  bounded lists without deriving mutation authority from a read snapshot.
- Pin the public controllers and username canonicalization evidence; add six
  generated response projections, source fixtures and fail-closed regressions.
- Keep the dependency/feature graph unchanged. Live crates.io drift is clean;
  the already detected Hetzner network-members coverage gap remains pending.

The [account contract](../docs/CRATESIO_ACCOUNT_POLICY.md) records the scope and
limits. The [incremental pentest report](../security/pentest/cratesio-commit-12.md)
records no confirmed findings for `5c925018..7d98a087`. The user confirmed GitHub
green on `7d4c147b` and authorized Commit 13. No tag or publication is authorized.

### Commit 13 - Authenticated Personal Workflows

- Add eight personal mutation operations with immutable action-specific intent,
  consumed permits, protected path tokens and a trusted blocking credential
  adapter boundary. No automatic retries or custom credential destinations.
- Bound invitation path/body/response IDs and acceptance state; require explicit
  true acknowledgements; preserve provider errors, response bounds and the
  process-wide admission gate. Clear all scratch on failure and unwinding.
- Separate email from publish-notification updates because the upstream combined
  operation can partially apply. Support the deprecated notification batch using
  its pinned controller, which supplies the body missing from OpenAPI.
- Add source projections, pinned controllers, adversarial response tests and
  scoped-serialization tests. See [the workflow policy](../docs/CRATESIO_PERSONAL_POLICY.md).
- Refresh reviewed direct and transitive dependencies across all four lockfiles
  and the fuzz nightly. Stable Rust remains 1.98.1 and checkout remains 7.0.1.
- Extend freshness checks to isolated tooling/fuzz manifests, which previously
  escaped root pin checks; update Saphyr and syn and remove the obsolete Base64
  duplicate exception without changing the default provider graph.
- Refresh crates.io source evidence after reviewing owner-description/example
  and rendered-policy changes; the operation inventory and request-rate rules
  remain unchanged. The Hetzner network-members coverage gap remains tracked.

Incremental pentest and GitHub passed at `9a1f2020` after verification-control
remediation; see [the report](../security/pentest/cratesio-commit-13.md).

### Commit 14 - API Token Inspection And Revocation

- Implement all three public token-management operations with consumed exact
  permits, fixed origins and single-attempt trusted blocking adapter execution.
- Validate ID-bound protected metadata, required nullable scope/expiry fields,
  known endpoint scopes, timestamps and local limits without granting authority.
- Distinguish revoke-by-ID's JSON acknowledgement from empty 204 self-revocation;
  no implicit retry, rotation, token creation or cookie-based listing.
- Add source-locked schema/status checks in CI and release gates, pinned upstream
  controller evidence, adversarial tests, cleanup and compile-fail authorization
  tests. No manifest, dependency, feature or lockfile changes.
- Document out-of-band replacement provisioning and ambiguous mutation outcomes
  in [the token contract](../docs/CRATESIO_TOKEN_POLICY.md).

Pentest and GitHub passed after header-retention remediation at `fae5b5a1`;
see [the report](../security/pentest/cratesio-commit-14.md).

### Commit 15 - Crate And Version Settings

- Add both public settings PATCH operations with exact target-bound consumed
  permits, bounded JSON and single-attempt trusted blocking adapter execution.
- Make yank-message clearing explicit; reject conflicting unyank/message intent
  and check returned crate/version identity, yank state and message.
- Reuse full bounded metadata validation and document concurrency, no-op and
  ambiguous upstream mutation outcomes without promising CAS or automatic retry.
- Add source-locked request/response drift checks, controller hashes, adversarial
  tests and compile-fail authorization coverage. No dependency/feature changes.
- See [the settings contract](../docs/CRATESIO_SETTINGS_POLICY.md).

Pentest and GitHub passed at `42e534bd` with no findings;
see [the report](../security/pentest/cratesio-commit-15.md).

### Commit 16 - Ownership Mutations

- Add Cargo-compatible owner additions/removals with explicit identity
  namespaces, bounded batches and separate destructive removal confirmation.
- Add optional crate-bound self/last-individual-owner preflight without claiming
  snapshot authority, freshness or compare-and-swap semantics.
- Return protected acknowledgements, not inferred invitation acceptance or
  per-owner completion. No automatic retries or implicit mutation chaining.
- Pin current namespace controller semantics alongside the existing source
  lock, check OpenAPI/Cargo compatibility and add adversarial/gate regressions.
- See [the ownership contract](../docs/CRATESIO_OWNERSHIP_POLICY.md).

### Commit 17 - Cargo Yank And Unyank

- Add bodyless DELETE/PUT intents for exact crate/version identities with
  consumed, token-bound consent and no automatic retries.
- Check exact-200 Cargo acknowledgements, retaining the distinction between
  intended state and a separately fetched, identity-checked version snapshot.
- Reuse official-origin authentication, shared admission and bounded response
  cleanup through the trusted blocking adapter. Bundled integration remains
  Commit 20; no hidden polling or live mutation tests are introduced.
- Source-lock the controller's message-clearing side effect and asynchronous
  index propagation; add wire/schema, replay, stale-state and failure regressions.
- See [the yank contract](../docs/CRATESIO_YANK_POLICY.md). Incremental pentest
  baseline is `9fabe832`; stop before Commit 18, without tagging or publishing.

### Maintenance Evidence (Earlier Checkpoints)

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
