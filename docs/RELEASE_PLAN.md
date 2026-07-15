# cloud-sdk Release Plan To 1.0

Status: planning document.

This plan is intentionally granular. `cloud-sdk` manages infrastructure APIs, so
each milestone must be small enough to review, test, pentest, and stop cleanly
before tagging.

The list below is not a maximum. Add patch releases or split a milestone before
implementation if the work no longer fits in one safe review pass.

Tags use:

```text
v0.N.0      milestone release
v0.N.P      patch/fix release for milestone N
v1.0.0      first serious production-ready cloud-sdk foundation and Hetzner provider
v0.33.0+    pre-1.0 Robot Webservice support track
```

## Release Principles

Every release must have:

- a clear definition of done;
- a local verification command;
- security review notes;
- known limitations;
- release notes;
- dependency-policy evidence;
- source-lock evidence for API behavior;
- completed pentest evidence for the exact implementation commit being
  reviewed;
- no hidden dependency on one developer machine.

Every release should prefer:

- one SDK boundary or endpoint family at a time;
- source-lock and drift checks before endpoint work;
- generated or source-derived tests before broad handwritten models;
- first-party no_std domain types for request construction and validation;
- third-party crates only as reviewed optional adapters or test/reference
  tooling unless a default dependency is explicitly accepted;
- negative and adversarial tests for every parser, builder, and validator;
- explicit caller-controlled retry, timeout, and rate-limit behavior;
- no default networking, async runtime, TLS stack, filesystem, clock, token
  storage, or secret-manager dependency.

## Pentest Before Tags

Every version must pass a security review and pentest before it is tagged. This
applies to `v0.N.P` patch tags as well as milestone tags.

A version is not tag-ready until:

- `scripts/checks.sh` passes;
- the version-specific release gate, including required live Hetzner and IANA
  drift checks, passes;
- `cargo deny check` passes;
- `cargo audit` passes;
- `scripts/generate-sbom.sh` succeeds;
- `scripts/check_sbom_freshness.sh` proves both committed SBOMs match their
  current dependency graphs;
- release notes exist at `release-notes/RELEASE_NOTES_X.Y.Z.md`;
- a pentest report exists at `security/pentest/vX.Y.Z.md`;
- the pentest report names the exact full 40-character `Reviewed-Commit:`;
- the pentest report has `Status: PASS`;
- the pentest report has non-blank `Tester:` and `Scope:` fields;
- the pentest report has a `Date: YYYY-MM-DD` field;
- `sbom/cloud-sdk.spdx.json` exists and is non-empty;
- `sbom/reqwest-feature-unification.spdx.json` exists and is non-empty when
  the standalone downstream fixture is present;
- `sbom/fuzz.spdx.json` exists and is non-empty for the excluded fuzz tooling
  graph;
- `scripts/validate-release-readiness.sh vX.Y.Z` proves that the reviewed
  implementation commit is an ancestor of the final release commit;
- shared readiness rejects modified tracked files and all untracked files;
- the version-specific gate snapshots the clean validated `HEAD`, requires it
  to remain unchanged, and reruns readiness after every check;
- GitHub CI and CodeQL default setup are green on the final release commit;
- tagging has been explicitly requested.

`Reviewed-Commit:` records the implementation commit that was reviewed. If
retest, CodeQL, or another release gate causes release-relevant changes, rerun
the review and update `Reviewed-Commit:` to the latest reviewed commit before
tagging.

Normal CI validates release metadata without requiring the still-pending
current report. The versioned release gate requires the report before tagging.
The reviewed implementation commit must be an ancestor of the final release
commit. The permanent report and final release metadata may be committed
together after a green pentest. GitHub validates that complete release commit.
The normal publisher still requires a verifiable signed, annotated `vX.Y.Z`
tag to point at `HEAD` and has no dirty-tree, skipped-check, untagged, or
no-verification bypass flags.

When a version's implementation criteria are done, stop and say:

```text
vX.Y.Z implementation stop reached. Run pentest for this exact commit.
```

No tag is created at that point.

### Pentest Handoff Flow

Use this loop for every version:

1. Complete the implementation, tests, documentation, release metadata, and
   local gates, then commit the exact state handed to pentest.
2. The maintainer runs pentest. Temporary findings may be recorded in root
   `PENTEST.md`, which must never be committed.
3. If pentest finds an issue, fix it, add regression coverage, remove
   `PENTEST.md`, rerun local gates, commit the fix, and repeat pentest.
4. If pentest is green, write `security/pentest/vX.Y.Z.md` with `Status: PASS`
   and the full `Reviewed-Commit`. A no-findings result is valid evidence and
   does not require a redundant retest.
5. Finalize the SBOMs and release metadata, remove root `PENTEST.md`, run local
   gates, and commit the permanent report with the final release state.
6. Wait for GitHub CI and CodeQL default setup on that final release commit.
7. If GitHub finds an issue, fix it, update the pentest report to describe the
   change and latest reviewed state, commit, and wait for GitHub again.
8. When GitHub is green, run the versioned release gate against the unchanged
   commit. Tag and push only when explicitly requested.

Root `PENTEST.md` is temporary scratch input. It must not be committed. The
permanent report is part of the release tag.

## Source Spec Pin Rotation

The Hetzner API drift check fetches upstream OpenAPI specs over HTTPS from exact
official URLs and reports downloaded SHA-256 values against reviewed pins.
That pin is a trust boundary. When `PINNED_SPEC_SHA256` changes:

1. Fetch the new spec manually.
2. Diff the new spec content against the previous pinned spec content, not only
   the hash value.
3. Confirm the diff matches the intended upstream changelog or reviewed API
   documentation change.
4. Update `PINNED_SPEC_SHA256` only in the same reviewed source-lock pass that
   updates fingerprints, release notes, and pentest evidence.

Release fetches reject redirects and documents larger than 32 MiB, enforce
connection and total-time ceilings, and require valid UTF-8 JSON objects. A
new digest may be parsed only to classify maintenance drift; the command still
fails, and fetched content is never accepted, compiled, or packaged
automatically. Caller-supplied local documents are authenticated against the
reviewed digest before parsing.

## Crate Versioning And Publish Order

Provider-neutral domains live in `cloud-sdk`; reusable transport, testkit, and
secret handling belong in `cloud-sdk-reqwest`, `cloud-sdk-testkit`, and
`cloud-sdk-sanitization`. Hetzner endpoint models live in
`cloud-sdk-hetzner`. The default architecture is one primary crate per provider.

Track every release in `release-crates.toml` and
`docs/CRATE_VERSION_MATRIX.md`:

- `code`: the crate received meaningful implementation, API, or documentation
  changes and uses the release version;
- `dependency`: the crate only needs a manifest update because a related crate
  changed outside its current dependency range;
- `metadata`: the crate must be republished with the milestone version to
  correct or publish immutable package metadata;
- `unchanged`: the crate stays on the previous published version and is not
  published.

## Completeness Review Register

Every planning or pentest pass must check this register for implied work that
has not been assigned to a release.

| Gap | Resolution |
| --- | --- |
| The original prompt omitted Storage Box operations even though Hetzner's current spec includes them. | Added Storage Boxes to `v0.2.0` source lock and scheduled implementation in `v0.9.0`. |
| Five non-deprecated global and certificate action queries remain `planned` after the resource-family implementation passes. | Assigned all five operations and a zero-planned-non-deprecated matrix gate to `v0.26.0`. |
| Deprecated datacenter endpoints exist in the spec but should not become accidental public commitments. | Tracked as `deferred-deprecated` in `docs/API_MATRIX.md`; final deprecated-endpoint policy lands in `v0.27.0`. |
| Resource-local action lookups are deprecated upstream but still present in the spec. | Tracked as `deferred-deprecated`; action helper policy lands in `v0.18.0`. |
| API drift could otherwise be missed between endpoint implementation passes. | Added operation and schema fingerprints in `v0.2.0`; recurring maintenance hardening lands in `v0.25.0`. |
| Optional serde support can break no_std/default graph expectations. | Scheduled as a dedicated boundary in `v0.14.0`. |
| Transport adapters can accidentally admit runtime, TLS, or secret handling assumptions. | Blocking and async adapters are separated into `v0.16.0` and `v0.17.0`, after model/testkit work. |
| Platform trust stores can be attacker-extended and aws-lc introduces native build-script, C, and assembly trust. | Documented for `v0.16.0`; FIPS transport lands in `v0.23.0`, followed by deterministic-root and native-build review in `v0.24.0`. |
| Adding providers could multiply transport, testkit, sanitization, or API-family crates. | Enforced one primary crate per provider and provider-neutral shared boundaries in `v0.12.0`; release automation rejects nested `cloud-sdk-{provider}-{suffix}` packages. |
| Required request fields represented as `Option` permit invalid intermediate states and generic missing-field errors. | Audit all public constructors in `v0.27.0`; required values become direct typed arguments, while `Option` remains only for genuinely optional or tri-state input. |
| Public errors lack safe `Display` and `core::error::Error` integration. | Add static payload-free formatting, field-specific variants, redaction tests, and no_std error-trait coverage in `v0.27.0`. |
| Mutable transport receivers prevent ordinary concurrent requests and encourage mutex guards across `.await`. | Add shared blocking and async transport contracts, caller-bounded concurrency guidance, and concurrent conformance tests in `v0.28.0`. |
| Immutable text token input cannot be cleared and current clients lack an explicit rotation path. | Add mutable-byte and guarded-buffer ingestion with source cleanup plus concurrency-safe credential rotation in `v0.28.0`. |
| Safe endpoint models still require callers to assemble transport requests, status checks, response bounds, content-type checks, and decoding manually. | Add common prepared-request policy in `v0.29.0`, complete prepared operations for the existing Hetzner surface in `v0.30.0`, checked typed decoding in `v0.31.0`, and opt-in client workflows inside `cloud-sdk-hetzner` in `v0.32.0`. No nested Hetzner client crate is introduced. |
| A provider-neutral custom HTTPS endpoint receives the configured credential and can exfiltrate it when its value is attacker-controlled. | Make custom endpoint trust explicit in `v0.27.0`, expose immutable credential-bound endpoint identity in `v0.28.0`, and require exact official Hetzner authority checks or explicit custom-endpoint acknowledgement in `v0.32.0`. |
| Robot Webservice has different auth, encoding, and API shape than Cloud/DNS. | Assigned a separate source lock and twelve pre-1.0 implementation and hardening milestones from `v0.33.0` through `v0.44.0`. |
| Legacy Robot Storage Box operations are deprecated and overlap the supported Console API. | The `v0.33.0` Robot matrix must mark all 16 legacy operations excluded and must not create a Robot Storage Box module. |
| Repeated invalid Robot credentials can temporarily block the caller's source IP. | Basic credentials are type-separated and redacted in `v0.34.0`; `v0.43.0` live tests never submit intentionally invalid credentials. |
| Robot ordering mutations can create immediate infrastructure costs. | Read-only ordering lands separately in `v0.41.0`; `v0.42.0` requires explicit cost-bearing intent and keeps billable calls outside CI and normal live smoke tests. |
| Future providers such as Cloudflare need patterns but are not part of Hetzner 1.0. | Provider-neutral naming and module guidance are part of `v1.0.0`; no non-Hetzner provider is claimed before 1.0. |

## Milestones

### v0.1.0 - Repository Foundation

Status: tagged.

Goal: initialize the serious Rust workspace and policy baseline.

Deliverables:

- Rust stable `1.97.0` pinned.
- Rust `1.90.0` through `1.97.0` compatibility policy.
- Provider-neutral no_std foundation, reqwest, testkit, and sanitization
  boundary crates plus one focused Hetzner provider crate.
- CI, dependency policy, security policy, release notes.
- Fail-closed release gates for pentest evidence, no_std policy, and required
  dependency security tools.
- Implementation, release, API, threat-model, modularity, toolchain, unsafe,
  and supply-chain docs.

Verification:

- `scripts/checks.sh`
- `scripts/release_0_1_gate.sh`

Stop gate:

```text
v0.1.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.2.0 - Official API Source Lock

Status: tagged.

Goal: pin the authoritative current Hetzner API source before endpoint models.

Deliverables:

- Official OpenAPI/spec-source discovery.
- `docs/SPEC_LOCK.md` with retrieval date, source URLs, revision/hash, and
  changelog items considered.
- Complete `docs/API_MATRIX.md` endpoint table with method, path, resource
  owner module, pagination, sorting, action behavior, deprecation status, and
  implementation status.
- Explicit Storage Boxes review because the current spec includes Storage Box
  API operations that were not in the original prompt endpoint list.
- `docs/API_FINGERPRINTS.tsv` and `docs/API_SCHEMA_FINGERPRINTS.tsv`.
- `scripts/check_hetzner_api_drift.py` to report added, removed, and changed
  operations or schemas.
- `scripts/release_0_2_gate.sh`.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_upstream.sh --local-only`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/release_0_2_gate.sh`

Stop gate:

```text
v0.2.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.3.0 - Core Request And Response Policy

Status: tagged.

Goal: implement the no_std request, query, error, rate-limit, and action
foundation before endpoint families depend on it.

Deliverables:

- HTTP method and path domains with base URL policy for Cloud/DNS and Storage
  Box surfaces.
- Bounded query parameter builder with deterministic ordering and percent
  encoding policy.
- Label and label-selector validation with negative tests.
- Pagination, sorting, and `per_page` policy types.
- Error envelope and rate-limit metadata types.
- Action status model with documented terminal and nonterminal states.
- Tests for malformed paths, oversized query values, invalid labels, invalid
  pagination, unknown error codes, and non-panicking parsing.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `cargo test -p cloud-sdk-hetzner --all-features`
- `scripts/release_0_3_gate.sh`

Stop gate:

```text
v0.3.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.4.0 - Read-Only Catalog Resources

Status: release candidate; retest passed and permanent report is included.

Goal: implement low-risk read-only resources first using the v0.3 request
foundation.

Deliverables:

- Locations list/get.
- Pricing get.
- Server types list/get.
- Load balancer types list/get.
- ISOs list/get.
- Public image list/get only; image mutation remains in `v0.7.0`.
- Pagination and sorting tests for every list endpoint that supports them.
- Golden path construction tests from `docs/API_MATRIX.md`.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `cargo test -p cloud-sdk-hetzner --all-features catalog`
- `scripts/release_0_4_gate.sh`

Stop gate:

```text
v0.4.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.5.0 - Security Resources

Status: release candidate; retest passed and permanent report is included.

Goal: model SSH keys and certificates safely before server creation can depend
on them.

Deliverables:

- SSH key list/create/get/update/delete request domains.
- Certificate list/create/get/update/delete request domains.
- Certificate retry action request domain.
- Redacted `Debug` or no `Debug` for secret-adjacent request values.
- Validation for SSH public key input shape, names, labels, and certificate
  create modes.
- Tests for redaction, missing required fields, invalid labels, and action
  request paths.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `cargo test -p cloud-sdk-hetzner --all-features security`
- `scripts/release_0_5_gate.sh`

Stop gate:

```text
v0.5.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.6.0 - Server Resource Models

Status: release candidate; retest passed and permanent report is included.

Goal: implement server CRUD, metrics request domains, and server actions
without adding transport or token storage.

Deliverables:

- Server list/create/get/update/delete request domains.
- Server metrics request domain with time-range validation.
- Server action request domains for power, reboot, reset, shutdown, rebuild,
  rescue, backup, ISO, network, placement group, DNS pointer, protection, type
  change, image creation, console, and password reset operations.
- Explicit handling of deprecated omitted `dns_ptr` behavior by requiring
  caller intent.
- Tests for required create fields, mutual exclusions, action path building,
  metrics time ranges, and deprecated-field policy.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `cargo test -p cloud-sdk-hetzner --all-features servers`
- `scripts/release_0_6_gate.sh`

Stop gate:

```text
v0.6.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.7.0 - Images, Placement Groups, And Primary IPs

Status: release candidate; retest passed and permanent report is included.

Goal: complete remaining server-adjacent resource models.

Deliverables:

- Image update/delete and image protection action request domains.
- Placement group list/create/get/update/delete request domains.
- Primary IP list/create/get/update/delete request domains.
- Primary IP assign, unassign, DNS pointer, and protection action request
  domains.
- Policy for deprecated datacenter fields: no new public request fields for
  removed upstream fields.
- Tests for image type filters, placement group type validation, primary IP
  assignment requirements, DNS pointer explicit-null behavior, and action paths.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `cargo test -p cloud-sdk-hetzner --all-features server_adjacent`
- `scripts/release_0_7_gate.sh`

Stop gate:

```text
v0.7.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.8.0 - Volumes And Floating IPs

Status: release candidate; retest passed and permanent report is included.

Goal: implement volume and floating IP resources plus actions.

Deliverables:

- Volume list/create/get/update/delete request domains.
- Volume attach, detach, resize, and protection action request domains.
- Floating IP list/create/get/update/delete request domains.
- Floating IP assign, unassign, DNS pointer, and protection action request
  domains.
- Tests for size bounds, server/location selection, DNS pointer explicit-null
  behavior, and action path construction.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `cargo test -p cloud-sdk-hetzner --all-features storage_ip`
- `scripts/release_0_8_gate.sh`

Stop gate:

```text
v0.8.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.9.0 - Storage Box Models

Status: release candidate; retest passed and permanent report is included.

Goal: implement Storage Box operations from the Hetzner spec while keeping them
separate from Robot Webservice support.

Deliverables:

- Storage Box list/create/get/update/delete and folder-list request domains.
- Storage Box type list/get request domains.
- Storage Box snapshot list/create/get/update/delete request domains.
- Storage Box subaccount list/create/get/update/delete request domains.
- Storage Box and subaccount action request domains.
- Tests for snapshot paths, subaccount IDs, access setting requests, password
  reset redaction, and deprecated resource-local action lookup policy.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `cargo test -p cloud-sdk-hetzner --all-features storage_boxes`
- `scripts/release_0_9_gate.sh`

Stop gate:

```text
v0.9.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.10.0 - Firewalls And Networks

Status: release candidate; retest passed and permanent report is included.

Goal: implement firewall, firewall action, network, and network action request
domains.

Deliverables:

- Firewall list/create/get/update/delete request domains.
- Firewall apply/remove resources and set-rules action domains.
- Network list/create/get/update/delete request domains.
- Network route, subnet, IP range, and protection action domains.
- Rule validation for direction, protocol, source/destination selectors, ports,
  and descriptions.
- Tests for CIDR validation boundaries, port ranges, firewall rule conflicts,
  subnet/route mutation paths, and labels.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `cargo test -p cloud-sdk-hetzner --all-features networks_firewalls`
- `scripts/release_0_10_gate.sh`

Stop gate:

```text
v0.10.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.11.0 - Load Balancer Models

Status: tagged and published.

Goal: implement load balancers, metrics, services, targets, network attach,
public interface actions, algorithms, and type changes.

Deliverables:

- Load balancer list/create/get/update/delete request domains.
- Load balancer metrics request domain.
- Service add/update/delete request domains.
- Target add/remove request domains.
- Network attach/detach, DNS pointer, protection, type change, algorithm
  change, and public interface action domains.
- Tests for health check validation, port/protocol combinations, target
  selection, metrics time ranges, and DNS pointer explicit-null behavior.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `cargo test -p cloud-sdk-hetzner --all-features load_balancers`
- `scripts/release_0_11_gate.sh`

Stop gate:

```text
v0.11.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.12.0 - DNS Zones

Status: implementation complete; pentest and retest passed.

Goal: implement zones, zonefile import/export, zone actions, TTL policy, and
primary nameserver policy.

Deliverables:

- Zone list/create/get/update/delete request domains.
- Zonefile get and import request domains.
- Zone primary nameserver, TTL, and protection action domains.
- Explicit policy for omitted TTL deprecation.
- Tests for zone name validation, TTL bounds, zonefile body boundaries,
  nameserver lists, and action paths.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `cargo test -p cloud-sdk-hetzner --all-features dns_zones`
- `scripts/release_0_12_gate.sh`.

Stop gate:

```text
v0.12.0 pentest stop passed. Commit only the permanent report, wait for CI,
then run release readiness before tagging.
```

### v0.13.0 - DNS RRSets

Status: implementation complete; pentest and retest passed.

Goal: implement RRSet CRUD, RRSet actions, record mutation helpers, and DNS
record validation.

Deliverables:

- RRSet list/create/get/update/delete request domains.
- RRSet protection, TTL, set-records, add-records, remove-records, and
  update-records action domains.
- Validation for record type, name, TTL, record list shape, and explicit-null
  TTL policy.
- Tests for RRSet path encoding, record-set mutation semantics, duplicate or
  missing record cases, and deprecated omitted TTL behavior.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `cargo test -p cloud-sdk-hetzner --all-features dns_rrsets`
- `scripts/release_0_13_gate.sh` after the permanent pentest report is added.

Stop gate:

```text
v0.13.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.14.0 - Serde And Sanitization Boundaries

Status: release candidate; pentest and retest passed.

Goal: admit optional serde request/response support and provider-neutral
caller-buffer sanitization without weakening the default no_std provider graph.

Deliverables:

- Non-default no_std `serde` feature with optional allocation and no Serde
  `std` feature.
- Explicit checked serialization for all v0.13 RRSet body models and validated
  deserialization for shared action/error response envelopes. Other models
  remain unsupported until individually source-mapped.
- Duplicate/unknown-field, optional-null, and redaction policy.
- Aggregate request-body limits checked before serialization or transport,
  including worst-case bounded RRSet record lists.
- Tests proving default features remain empty and no serde dependency appears
  in the default graph.
- JSON fixture tests for representative success and error responses.
- First usable `cloud-sdk-sanitization` volatile caller-buffer guard through
  the reviewed first-party `sanitization` crate with default features disabled.
- Redacted API errors, no ordinary equality for password/private-key request
  values, and atomic escaped private-key output without raw access.

Verification:

- `scripts/checks.sh`
- `scripts/check_serde_boundary.sh`
- `cargo tree -p cloud-sdk-hetzner --no-default-features`
- `cargo test -p cloud-sdk-hetzner --all-features serde`
- `scripts/release_0_14_gate.sh` after the permanent pentest report is added.

Stop gate:

```text
v0.14.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.15.0 - Testkit Boundary

Status: tagged and published.

Goal: implement deterministic mock transport, pagination/action fixtures, and
an adversarial response corpus before real transports are admitted.

Deliverables:

- First usable `cloud-sdk-testkit` mock transport boundary.
- Fixture builders for success, paginated, action, rate-limit, and error
  responses.
- Adversarial corpus for malformed JSON, unknown fields, missing required
  fields, oversized responses, invalid pagination, and invalid action states.
- Tests proving mock transport does not require network, TLS, filesystem, or
  runtime dependencies by default.
- Provider-neutral blocking transport request/response contract in
  `cloud-sdk`, limited to origin-form targets and caller-owned buffers.
- Hetzner Serde integration proving shared adversarial cases exercise a real
  provider parser without creating a testkit-to-provider dependency.

Verification:

- `scripts/checks.sh`
- `scripts/check_testkit_boundary.sh`
- `scripts/check_rust_version_matrix.sh`
- `cargo test -p cloud-sdk-testkit --all-features`
- `cargo test --workspace --all-features`
- `scripts/release_0_15_gate.sh`.

Stop gate:

```text
v0.15.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.16.0 - Optional Blocking Transport Adapter

Status: tagged and published.

Goal: admit the first reviewed blocking transport adapter outside the default
graph.

Deliverables:

- Blocking transport trait implementation in an optional adapter crate.
- Reqwest 0.13.4 dependency admission document for HTTP, TLS, URL, runtime,
  cryptographic-provider, and header dependencies used.
- Explicit timeout, user-agent, authentication header, retry, and redaction
  policy.
- HTTPS-only rustls client with TLS 1.2 minimum, no redirect, no retry, no
  proxy, no referer, and no response decompression.
- Authority-preserving target composition, bounded response reads, and
  sanitized adapter-owned token and request-body buffers.
- Deterministic loopback tests only; no live network by default.
- Default workspace graph remains transport-free.

Verification:

- `scripts/checks.sh`
- `scripts/check_reqwest_boundary.sh`
- `cargo test -p cloud-sdk-reqwest --all-features`
- fixture-scoped `cargo deny` and `cargo audit` checks;
- production and feature-unification SPDX SBOM generation;
- canonical committed-SBOM freshness comparison;
- `cargo tree -p cloud-sdk-hetzner --no-default-features`
- `scripts/release_0_16_gate.sh`.

Stop gate:

```text
v0.16.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.17.0 - Optional Async Transport Adapter

Status: tagged and published.

Goal: add async transport support with explicit runtime neutrality and no
default runtime dependency.

Deliverables:

- Runtime-neutral `AsyncTransport` future contract in the no_std core.
- No-allocation async mock implementation in `cloud-sdk-testkit` without an
  executor dependency.
- Optional `cloud-sdk-reqwest/async-rustls` implementation requiring a
  caller-provided Tokio executor while leaving all default graphs runtime-free.
- Cancellation-safe, caller-bounded async response accumulation with sanitized
  adapter-owned request and response storage.
- Explicit no-redirect/no-retry policy: rate-limit and retry interpretation
  remains visible to caller-owned provider logic.
- Deterministic loopback coverage for exact requests, timeouts, cancellation,
  overflow, redirects, content types, feature unification, and redaction.
- Updated reqwest, bytes, Tokio, TLS, and HTTP dependency review and graph gates.

Verification:

- `scripts/checks.sh`
- `cargo test -p cloud-sdk-reqwest --all-features`
- `cargo tree -p cloud-sdk-hetzner --no-default-features`
- `scripts/release_0_17_gate.sh`.

Stop gate:

```text
v0.17.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.18.0 - Pagination And Action Polling Helpers

Status: tagged and published.

Goal: provide ergonomic optional helpers over transport traits without hiding
rate-limit, timeout, or retry policy.

Deliverables:

- Pagination helper that exposes page boundaries and rate-limit metadata.
- Action polling helper with caller-supplied delay/backoff policy.
- Terminal action states and failure propagation.
- Strict all-or-none rate-limit header parsing in blocking and async adapters.
- Strict reusable Hetzner `meta.pagination` parsing and conversion into the
  provider-neutral cursor.
- Source-locked correction of Hetzner's default page size to 25 and maximum to
  50 unless an operation documents an exception.
- Tests for stop conditions, timeout/cancel behavior, empty pages, repeated
  pages, action failure, and rate-limit propagation.

Verification:

- `scripts/checks.sh`
- `cargo test --workspace --all-features pagination`
- `cargo test --workspace --all-features action_polling`
- `scripts/release_0_18_gate.sh`.

Stop gate:

```text
v0.18.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.19.0 - Live Smoke Harness

Status: tagged and published.

Goal: add opt-in live tests gated by environment variables and least-privilege
test project guidance.

Deliverables:

- Live smoke harness disabled by default.
- Repository-anchored clean-commit staging with no token present or mounted,
  privileged root-owned sealing, and authenticated open-descriptor execution
  that never invokes Cargo or build tooling.
- Required environment variables and token-scope guidance.
- Read-only smoke tests for locations, server types, load balancer types, ISOs,
  public system images, and pricing.
- A separately documented destructive test plan that requires a dedicated
  project, explicit opt-in, a unique resource prefix, and cleanup verification;
  mutation execution remains disabled in this release.
- Fixed provider origin, bounded responses, private regular token-file input,
  single-allocation source-buffer cleanup, and redaction of tokens, paths,
  response bodies, and IDs in diagnostics.
- Offline tests for request methods and targets, response-envelope shape,
  pagination, token normalization, size bounds, symlinks, Unix permissions,
  and diagnostic redaction.

Verification:

- `scripts/checks.sh`
- `cargo test --workspace --all-features`
- `scripts/smoke_hetzner_live.sh --check`
- `scripts/smoke_hetzner_live.sh --prepare`, privileged system installation,
  and build-environment teardown before credential provisioning.
- Documented manual live-smoke command with no token in shell history examples
  and no Cargo invocation during authenticated execution.
- `scripts/release_0_19_gate.sh`.

Stop gate:

```text
v0.19.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.20.0 - Platform Matrix

Status: tagged and published.

Goal: prove claimed platform support for Linux, Windows, BSD, macOS, Android,
iOS, WASM, and embedded/no_std targets where applicable.

Deliverables:

- Target matrix document with native, portable, best-effort, and unsupported
  transport tiers.
- Allowlisted no_std and alloc/Serde checks for representative Linux, Windows,
  FreeBSD, macOS, Android, iOS, WASM, and bare-metal targets.
- Native all-feature workspace checks on Linux, Windows, macOS ARM64, and macOS
  x86-64 runners.
- Platform-specific reqwest limitations and target-native transport guidance.
- Default dependency-graph rejection for network, TLS, runtime, socket, and OS
  dependencies, with adversarial script regression tests.

Verification:

- `scripts/checks.sh`
- `scripts/test-platform-matrix.py`
- `scripts/check_platform_matrix.sh --all`
- Target-specific commands documented in the release notes and platform guide.
- `scripts/release_0_20_gate.sh`.

Stop gate:

```text
v0.20.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.21.0 - Documentation And Examples Hardening

Status: tagged.

Goal: make docs.rs examples, transport examples, security recipes, and release
runbooks complete enough for real users.

Deliverables:

- Provider-neutral quickstart.
- Hetzner read-only, mutation, pagination, action polling, DNS, and Storage Box
  examples.
- Security recipes for token handling, logging, retries, timeouts, and live
  smoke tests.
- Docs.rs feature documentation for every crate.
- Broken-link and doctest checks where supported.

Verification:

- `scripts/checks.sh`
- `cargo test --workspace --doc --all-features`
- `scripts/check_doc_links.sh`
- `scripts/test-doc-links.py`
- `scripts/release_0_21_gate.sh`.

Stop gate:

```text
v0.21.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.22.0 - Fuzzing And Adversarial Tests

Status: pentest passed after remediation; final release commit pending.

Goal: fuzz request builders, parsers, validators, and response handling.

Deliverables:

- Six isolated libFuzzer targets for fixed-buffer writers, request targets,
  labels and DNS, pagination, action polling, and response envelopes.
- Synthetic named seed corpus derived from source-locked valid and invalid
  examples, with generated corpora and artifacts rejected from version control.
- Pinned nightly Rust, cargo-fuzz, and libfuzzer-sys versions in an excluded,
  non-published package with an independently audited lockfile and SBOM.
- Documented long-run, exact crash replay, minimization, sanitization, and
  deterministic-regression process.
- Dedicated CI and release-gate build plus bounded seed replay, without
  requiring unbounded fuzz campaigns in every CI job.
- Exhaustive fixed-buffer JSON atomicity tests and adversarial Serde tests for
  malformed, deeply nested, oversized, duplicate, overflowing, and
  control-character upstream inputs.

Verification:

- `scripts/checks.sh`
- `scripts/check_fuzz_harness.sh --build`
- `scripts/check_fuzz_harness.sh --smoke`
- `cargo test --workspace --all-features`
- Workspace, downstream fixture, and fuzz lockfile Cargo Deny/RustSec checks.
- `scripts/check_sbom_freshness.sh`
- `scripts/release_0_22_gate.sh`.

Stop gate:

```text
v0.22.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.23.0 - Optional Blocking FIPS Transport

Goal: add a fail-closed blocking rustls FIPS-mode transport without weakening
or silently changing the standard blocking transport, while avoiding a
validation claim the current AWS-LC-FIPS 3.0.x dependency cannot support.

Deliverables:

- Dedicated `blocking-rustls-fips` feature in `cloud-sdk-reqwest`; default and
  `std` graphs remain transport-free.
- Explicit rustls FIPS `CryptoProvider` and `ClientConfig`, with runtime
  `fips()` verification before client construction succeeds.
- Mandatory deployment-managed trust roots and complete CRLs, with chain-wide
  unknown-status denial and CRL-expiration enforcement.
- Exact published requirements for reqwest, rustls, platform-verifier, and all
  AWS-LC packages in the reviewed FIPS graph; applications still own a locked
  or vendored complete dependency resolution.
- FIPS-only dependency graph includes `aws-lc-fips-sys`; the boundary records
  and checks rustls' current compilation of ordinary `aws-lc-sys` alongside
  the FIPS-selected FFI instead of claiming that build dependency is absent.
- Defined additive-feature behavior: the FIPS provider wins safely when both
  blocking transport features are selected, while the FIPS-only graph remains
  independently auditable.
- Existing HTTPS-only, TLS-version, timeout, redirect, retry, proxy,
  decompression, authority, response-bound, redaction, and sanitization policy
  remains enforced.
- Explicit per-client provider construction that is independent of missing,
  conflicting, or preinstalled process-global provider state, plus runtime
  rejection if the provider or complete configuration does not report FIPS.
- FIPS dependency admission covering the exact aws-lc-fips-sys release,
  current NIST validation-status limitation, C/C++ compiler, CMake, Go, Perl,
  bindgen, checksum, system-library discovery, and reproducible-build limits.
- Documentation stating that a crate feature does not make an application or
  deployment FIPS compliant or establish a current validation certificate.
- Dedicated Linux CI/release check for bundled-source graph and runtime FIPS
  status, without presenting the runner as a validated operating environment.

Verification:

- `scripts/checks.sh`
- `scripts/check_reqwest_fips_boundary.sh` once added.
- FIPS-only Cargo feature and dependency-tree checks.
- Runtime `CryptoProvider::fips()` and `ClientConfig::fips()` tests.
- Missing-policy, empty-root, empty-CRL, malformed-CRL, and successful
  verifier-construction tests; rustls' fail-closed unknown-status and CRL
  expiration policies are selected without permissive overrides.
- Generated-crate extraction and locked FIPS test compilation, including the
  public certificate and CRL verifier fixtures.
- Publish-state mutation and exact manifest-constraint tests.
- `cargo deny check`
- `cargo audit`
- `scripts/release_0_23_gate.sh` once added.

Stop gate:

```text
v0.23.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.24.0 - Dependency And Tooling Hardening

Status: pentest and retest passed; final release checks are pending.

Goal: refresh dependency, tool, SBOM, audit, and supply-chain evidence before
release-candidate work.

Deliverables:

- Current dependency review for every default, optional, dev, and tool crate.
- Re-evaluate platform trust-store policy and add a separately reviewed
  deterministic Mozilla root-store feature for reproducible public WebPKI
  trust decisions.
- Re-audit aws-lc-sys build-script, vendored C/assembly, Cargo checksum,
  offline-build, and pinned native-toolchain requirements.
- `cargo-deny` and `cargo-audit` evidence.
- SBOM generation and documentation.
- Toolchain and MSRV review for Rust `1.90.0` through current pinned stable.
- Updated security controls and supply-chain docs.

Verification:

- `scripts/checks.sh`
- `scripts/check_latest_tools.sh --fetch`
- `scripts/check_reqwest_webpki_roots_boundary.sh`
- `scripts/generate-sbom.sh`
- `cargo deny check`
- `cargo audit`
- `scripts/release_0_24_gate.sh` once added.

Stop gate:

```text
v0.24.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.25.0 - API Drift Automation Hardening

Goal: make upstream drift monitoring actionable as a maintenance process, not
only a one-off source lock.

Deliverables:

- Drift detector reports grouped by added, removed, changed, deprecated, and
  schema-only changes.
- Maintenance playbook for accepting, rejecting, or deferring upstream changes.
- Read-only scheduled and manual CI workflow for maintainers.
- Release-note template for upstream drift updates.
- Tests for the drift detector using checked-in fixture specs.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- Drift-detector fixture tests.
- `scripts/release_0_25_gate.sh`.

Stop gate:

```text
v0.25.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.26.0 - Complete Non-Deprecated Endpoint Coverage

Goal: implement every remaining non-deprecated Hetzner Cloud/DNS and Storage
Box request operation so the source-locked API matrix reaches 100% claimed
endpoint coverage without exposing deprecated operations.

Deliverables:

- Global action request support for `GET /actions`, including the required
  action-ID filter and bounded repeated query encoding.
- Global action lookup support for `GET /actions/{id}`.
- Certificate action request support for `GET /certificates/actions` with
  pagination and sorting.
- Certificate action lookup support for `GET /certificates/actions/{id}`.
- Per-certificate action list support for `GET /certificates/{id}/actions`
  with pagination and sorting.
- Focused endpoint, query, pagination, sorting, buffer-boundary, and
  adversarial tests for all five operations.
- API-matrix validation that fails unless every non-deprecated operation is
  implemented and no `planned` non-deprecated row remains.
- README and operation-level documentation updated from partial to complete
  endpoint coverage.
- Deprecated resource-local action lookups and datacenter endpoints remain
  `deferred-deprecated` and are not added to the public API.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- Zero-planned-non-deprecated API-matrix regression check.
- Focused global-action and certificate-action request tests.
- `scripts/release_0_26_gate.sh` once added.

Stop gate:

```text
v0.26.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.27.0 - Existing Hetzner Surface Stabilization

Goal: stabilize the existing Cloud, DNS, and Console Storage Box surface before
adding Robot-specific protocol and endpoint modules.

Deliverables:

- Public API review for existing exported types and feature flags.
- Deprecated upstream endpoint policy.
- Error and versioning policy.
- Provider documentation distinguishes request models, path/query encoding,
  body serialization, success responses, error responses, and end-to-end
  client coverage instead of using an ambiguous `Supported` claim.
- API-matrix and README terminology consistently defines current `implemented`
  status as request-construction coverage, with a checked documentation
  regression test preventing capability claims from drifting ahead of code.
- `cloud-sdk-reqwest` endpoint and client-builder documentation states that the
  configured HTTPS endpoint receives the supplied credential and must never be
  derived from tenant-controlled or otherwise untrusted input.
- Generic endpoint construction and builder APIs receive a naming and semver
  review so arbitrary credential-bearing destinations use a conspicuous custom
  endpoint path; any rename or deprecation includes migration notes.
- Every public request constructor is audited. Required fields become direct
  validated arguments to `new` or `try_new`; `Option` is accepted only for
  genuinely optional, nullable, resettable, or tri-state API semantics.
- Constructors do not create an invalid intermediate request merely to return a
  generic `MissingRequiredField`. Cross-field validation remains fallible, and
  migration notes cover every changed signature.
- All public first-party error enums implement payload-free `Display` and
  `core::error::Error` under the MSRV. Messages are static and never interpolate
  request targets, bodies, credentials, provider payloads, or customer data.
- Missing-input errors that remain possible use field-specific variants such as
  `MissingServerName`; broad variants remain only where no safe, stable field
  distinction exists.
- Pre-Robot semver audit and migration notes.
- Examples and docs.rs output reviewed.
- Release notes for known limitations carried into the Robot implementation
  track.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `cargo public-api` or equivalent if admitted.
- Documentation tests require the custom-endpoint credential warning beside
  every blocking and async construction example.
- Compile tests prove required constructor fields cannot be omitted, while
  optional and tri-state fields retain their intended semantics.
- Error-trait, exact static-message, redaction, and no-sensitive-payload tests
  cover every public error family.
- `scripts/release_0_27_gate.sh` once added.

Stop gate:

```text
v0.27.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.28.0 - Shared Transport And Credential Lifecycle

Goal: make provider-neutral transports safely shareable for caller-bounded
concurrency while binding credentials to an immutable endpoint and providing a
clear secret-ingestion and rotation lifecycle.

Deliverables:

- The primary blocking and executor-neutral async transport contracts send
  through `&self`, so a thread-safe implementation can serve concurrent
  requests without a caller-held mutex across I/O or `.await`.
- Implementations that are not `Sync` remain usable sequentially; concurrency
  is available only when the concrete transport satisfies the caller's `Sync`,
  `Send`, executor, and task-lifetime requirements.
- `cloud-sdk-reqwest` blocking and async clients are safely cloneable or
  shareable handles over bounded internal state. Request-local bodies and
  response buffers are never shared implicitly.
- Concurrency remains caller-bounded. The SDK creates no unbounded task set,
  semaphore, queue, retry fan-out, or background runtime.
- A provider-neutral bound-endpoint identity reports the transport's immutable,
  normalized scheme, host, effective port, and base path without exposing
  credentials. The Hetzner provider exposes an exact verifier for both official
  v1 endpoint families before permitting execution.
- Endpoint identity cannot be replaced after credential binding. Custom
  endpoints remain explicit and cannot be populated from environment proxy
  configuration or redirected at request time.
- `BearerToken` accepts validated mutable bytes and guarded secret storage in
  addition to the compatibility text constructor. Consuming mutable input
  clears the admitted source buffer on both success and failure.
- Blocking and async transports expose a documented token-rotation operation.
  Rotation is atomic for newly started requests, does not hold a lock across
  network I/O or `.await`, leaves the previous token active on rejected input,
  recovers structurally complete state after lock poisoning, and sanitizes
  retired token storage after the last in-flight use.
- Token construction, rotation, debug output, and errors never expose secret
  bytes; caller-owned immutable strings remain a documented cleanup boundary.

Verification:

- `scripts/checks.sh`
- Default/no_std and optional transport dependency-boundary checks.
- Shared blocking and async conformance tests issue overlapping requests and
  prove request/response buffers and failures remain isolated.
- Tests prove concurrency requires caller-selected bounds and no SDK path
  spawns tasks, sleeps, retries, or owns an executor.
- Bound-endpoint identity tests cover host, subdomain, port, base-path, and
  normalization mismatches and prove identity cannot be replaced after
  credential binding.
- Mutable-byte and guarded-token tests cover cleanup on every success/error
  path, concurrent rotation, in-flight token snapshots, and retired-token
  sanitization. A deliberate poisoning test proves all cloned clients recover.
- `scripts/release_0_28_gate.sh` once added.

Stop gate:

```text
v0.28.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.29.0 - Prepared Request And Response Policy

Goal: define one provider-neutral contract that turns a validated operation
into method, target, query, body, endpoint, retry metadata, and checked response
policy without caller-specific assembly.

Deliverables:

- A common no_std operation/preparation trait in `cloud-sdk` produces a
  `PreparedRequest` from typed input and caller-owned target/body storage.
- `PreparedRequest` binds one validated `TransportRequest` to its provider
  service/base family, expected success statuses, accepted response media
  types, maximum response-body length, and empty-body policy.
- Operation metadata distinguishes `ReadOnly`, `Mutation`, and `Destructive`
  impact, with a separate safe/idempotent/non-idempotent classification and
  explicit retry eligibility. Cost-bearing intent remains an orthogonal marker.
- Metadata has no convenience default that can classify an unknown mutation as
  read-only, idempotent, retryable, or non-destructive.
- Validated response content-type metadata is added to `TransportResponse`;
  concrete transports reject malformed header values while response policy
  distinguishes missing, unexpected, and valid content types.
- Provider-neutral response-policy validation classifies endpoint mismatch,
  unexpected status, missing or incorrect content type, forbidden body, and
  oversized body before provider decoding starts.
- Caller-owned response storage remains structural: execution lends only the
  operation's admitted capacity to the transport and never trusts a numeric
  length beyond the initialized slice.
- Preparation remains allocation-free and adds no network, TLS, runtime,
  filesystem, clock, credential storage, automatic retry, delay, jitter, or
  sleep to the default graph.
- `cloud-sdk-testkit` records prepared requests and operation metadata and can
  model endpoint mismatch, content-type failures, status mismatch, empty bodies,
  oversized responses, and retry-classification mistakes.

Verification:

- `scripts/checks.sh`
- Default/no_std dependency-boundary checks.
- Compile tests require complete operation metadata and caller-owned storage.
- Adversarial tests prove mutations and destructive operations cannot acquire
  read-only, idempotent, or retryable behavior through defaults or conversion.
- Prepared-request, endpoint-family, response-policy, and testkit conformance
  tests cover both shared blocking and async transports.
- `scripts/release_0_29_gate.sh` once added.

Stop gate:

```text
v0.29.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.30.0 - Existing Hetzner Prepared Operations

Goal: make every source-locked non-deprecated Cloud, DNS, and Console Storage
Box operation produce one complete prepared request without requiring callers
to combine paths, queries, bodies, methods, or response expectations manually.

Deliverables:

- Operation descriptors inside `cloud-sdk-hetzner` reuse the existing typed
  endpoint constructors, implement the common preparation contract, and bind
  the official method, origin-form target, request content type, expected
  success statuses, response media type, body policy, and response bound.
- Caller-supplied target and body buffers are written atomically. Insufficient
  capacity returns a typed preparation error without exposing a partial request
  as executable.
- List filters, pagination, sorting, label selectors, resource identifiers,
  JSON request bodies, zonefiles, metrics queries, actions, and empty-body
  operations receive complete atomic wire serialization while retaining their
  existing validation and encoding rules.
- Preparation covers every non-deprecated operation claimed in
  `docs/API_MATRIX.md`; a release check fails when an implemented operation has
  no prepared-request path or when prepared metadata disagrees with the source
  lock.
- Read-only, mutating, destructive, and cost-bearing operations remain
  source-locked in operation metadata together with idempotency and retry
  classification so later execution policy cannot treat them as interchangeable.
- The prepared-operation layer remains no_std and transport-independent.
  `cloud-sdk-hetzner` does not depend on `cloud-sdk-reqwest`, and no
  `cloud-sdk-hetzner-client` package is introduced.
- Compile-checked examples show preparation with caller-owned storage while
  retaining direct access to the lower-level endpoint APIs.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- Zero-missing-prepared-operation API-matrix gate.
- Per-family golden request and insufficient-buffer tests.
- Mutation-classification and source-locked response-policy tests.
- `scripts/release_0_30_gate.sh` once added.

Stop gate:

```text
v0.30.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.31.0 - Checked Hetzner Response Decoding

Goal: provide one checked decoding path that consumes a transport response,
enforces every prepared response policy, and returns typed provider success or
error data without requiring callers to remember security steps.

Deliverables:

- Resource-specific success response models cover every non-deprecated Cloud,
  DNS, and Console Storage Box operation, including list envelopes, nullable
  fields, action results, empty success bodies, metrics, and zonefiles.
- A checked decoder consumes `TransportResponse` together with the operation's
  prepared metadata; callers cannot pass a raw body while bypassing its status,
  content-type, empty-body, or maximum-size policy. Endpoint/service mismatch
  is rejected before transport execution by the prepared-request path.
- The decoder applies the bounded `ResponseBytes` boundary before parser use,
  then returns either the operation's typed success value or typed Hetzner API
  error envelope according to source-locked status semantics.
- Unexpected status, malformed or missing content type, oversized body,
  malformed payload, duplicate fields, invalid identifiers, unknown enum values,
  and typed provider errors remain distinct payload-free error cases.
- Response models validate security-relevant fields after parsing, tolerate
  only documented additive compatibility, and never expose unvalidated wire
  structs publicly.
- The decoder remains transport-independent and performs no request, retry,
  sleep, allocation beyond its admitted feature contract, logging, or implicit
  sanitization of caller-owned response storage.
- Optional parser dependencies and alloc use receive explicit no_std, license,
  feature, malformed-input, and supply-chain review; default features stay empty.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- Zero-missing-success-model and zero-missing-decoder operation-matrix gates.
- Golden and adversarial decoding fixtures for every response family and every
  documented success/error status shape.
- Fuzz coverage for shared envelopes and representative resource, list,
  metrics, zonefile, nullable, empty-body, and malformed response paths.
- Default/no_std and optional decoder feature-matrix checks.
- `scripts/release_0_31_gate.sh` once added.

Stop gate:

```text
v0.31.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.32.0 - Secure High-Level Hetzner Client Workflows

Goal: make the secure end-to-end path the shortest supported path for existing
Hetzner Cloud, DNS, and Console Storage Box operations while preserving the
transport-free default provider graph.

Deliverables:

- An opt-in `client` feature inside `cloud-sdk-hetzner`, generic over
  the shared blocking or async transport contract; provider code never selects
  reqwest, TLS roots, an async executor, timeouts, credentials, or secret
  storage.
- Safe Cloud/DNS and Console Storage Box constructors verify that the supplied
  credential-bound transport exactly matches the corresponding official
  Hetzner scheme, host, effective port, and base path before accepting it.
- Arbitrary routing is available only through a visibly named
  `with_custom_endpoint` path requiring explicit caller acknowledgement that
  credentials will be sent to that endpoint; documentation forbids endpoint
  values derived from tenant-controlled or otherwise untrusted input.
- High-level operation methods prepare into bounded caller-owned storage, send
  exactly one request, invoke only the checked decoder, and return the
  operation's typed success or Hetzner error value.
- Typed errors preserve preparation, transport, HTTP/API, policy, and decoding
  failures as distinct cases without logging or embedding request bodies,
  credentials, zonefiles, user data, or response secrets in diagnostics.
- Shared client handles support caller-bounded concurrent operations without a
  mutex guard across network I/O or `.await`; each call owns distinct bounded
  target, body, and response storage.
- No request is retried by default. Any admitted retry policy must be supplied
  explicitly by the caller, receive method and operation classification, keep
  delay/sleep execution caller-controlled, and require separate explicit intent
  before a mutation can be attempted again.
- Existing pagination, action polling, and rate-limit state machines integrate
  with prepared operations without automatically issuing requests or sleeping.
- `cloud-sdk-reqwest` examples demonstrate an optional blocking and async happy
  path through the generic client without introducing a provider dependency in
  the transport crate.
- Mock and live-smoke workflows cover successful decoding, typed API errors,
  unexpected status, missing/wrong content type, malformed JSON, empty bodies,
  oversized responses, transport failure, and proof of one-send behavior.
- The main and provider READMEs lead with concise compile-checked client
  examples and retain lower-level preparation examples for no-client users.

Verification:

- `scripts/checks.sh`
- Default/no_std, `client`, blocking, and async feature-matrix checks.
- Complete mock happy-path, checked-decoder, and adversarial response tests for
  every endpoint response family.
- Official-constructor tests reject lookalike hosts, subdomains, non-default
  ports, altered base paths, user information, and custom endpoints before any
  credential-bearing request is sent.
- Custom-endpoint tests prove acknowledgement is mandatory and redirects,
  cross-origin forwarding, proxies, and environment-derived routing stay
  disabled.
- Compile-checked blocking and executor-neutral async examples.
- Credential-free live staging plus the existing opt-in read-only smoke suite.
- Concurrent client tests use explicit caller bounds and cover token rotation,
  cancellation, independent buffers, and failure isolation.
- `scripts/release_0_32_gate.sh` once added.

Stop gate:

```text
v0.32.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.33.0 - Robot Source Lock And Operation Matrix

Goal: establish a reproducible source of truth for every active and deprecated
Robot Webservice operation before adding public Robot types.

Deliverables:

- Pin <https://robot.hetzner.com/doc/webservice/en.html> with digest, retrieval
  metadata, normalized operation inventory, and a tested drift detector.
- Confirm the current expected inventory of 105 documented operation headings,
  16 deprecated Robot Storage Box operations, and 89 active target operations.
- Record methods, paths, inputs, response shapes, errors, request limits, and
  deprecations in a separate Robot API matrix.
- Document HTTPS-only HTTP Basic Auth, form-encoded inputs, JSON responses,
  optional YAML paths, authentication lockout behavior, and maintenance errors.
- Exclude every deprecated Robot Storage Box endpoint and point users to the
  already supported Console Storage Box API.
- Keep all future Robot modules inside `cloud-sdk-hetzner`; no nested provider
  crate is introduced.

Verification:

- `scripts/checks.sh`
- Robot source-lock fixture and parser tests.
- `scripts/check_hetzner_robot_drift.py --fetch` once added.
- `scripts/release_0_33_gate.sh` once added.

Stop gate:

```text
v0.33.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.34.0 - Robot Protocol And Credential Foundation

Goal: add the shared Robot protocol, credential, encoding, error, and transport
boundaries without implementing resource operations prematurely.

Deliverables:

- `cloud_sdk_hetzner::robot` foundation with the fixed Robot base URL,
  bounded identifiers, origin-form paths, methods, and request metadata.
- Allocation-free `application/x-www-form-urlencoded` writers with atomic
  caller-buffer failure and repeated-field support.
- Bounded Robot error envelopes, invalid-input detail, maintenance handling,
  and per-operation request-limit policy.
- Provider-neutral Basic Auth credential support in `cloud-sdk-reqwest`, with
  redacted diagnostics, sanitization-backed cleanup, HTTPS-only enforcement,
  and no accidental bearer/Basic cross-use.
- Blocking and async adapters support Robot without changing default features
  or weakening standard, deterministic-root, or FIPS TLS modes.
- Robot request/response fixtures and mock transport helpers in the existing
  provider-neutral testkit.

Verification:

- `scripts/checks.sh`
- Robot auth, form-encoding, error, rate-limit, redaction, and transport tests.
- Default/no_std and optional transport dependency-boundary checks.
- Robot source-lock drift check.
- `scripts/release_0_34_gate.sh` once added.

Stop gate:

```text
v0.34.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.35.0 - Robot Servers And Cancellation

Goal: implement the six active Robot server and cancellation operations.

Deliverables:

- Server list, get, and update request and response domains.
- Cancellation get, create, and revoke domains with explicit date, reason, and
  location-reservation validation.
- Server-number identifiers are canonical; deprecated server-IP path aliases
  remain excluded.
- Destructive cancellation requests require explicit typed intent and never
  appear in live read-only smoke coverage.
- Focused fixtures and adversarial tests for nullable subnets, capability
  flags, dates, status, cancellation conflicts, and empty responses.

Verification:

- `scripts/checks.sh`
- Focused Robot server and cancellation tests.
- Robot source-lock drift check.
- `scripts/release_0_35_gate.sh` once added.

Stop gate:

```text
v0.35.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.36.0 - Robot IP And Subnet Management

Goal: implement all 18 active Robot IP and subnet operations.

Deliverables:

- IP and subnet list, get, traffic-warning update, MAC get/set/delete, and
  cancellation get/create/revoke domains.
- Canonical IPv4, IPv6, subnet, MAC, gateway, mask, and broadcast validation.
- Bounded repeated form fields and explicit cancellation dates.
- Response models cover nullable MACs, traffic thresholds, lock state, and
  assignment metadata without exposing unvalidated network identities.
- Focused malformed-address, host-bit, encoding, boundary, and conflict tests.

Verification:

- `scripts/checks.sh`
- Focused Robot IP, subnet, MAC, and cancellation tests.
- Robot source-lock drift check.
- `scripts/release_0_36_gate.sh` once added.

Stop gate:

```text
v0.36.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.37.0 - Robot Reset, Failover, And Wake-On-LAN

Goal: implement the nine active reset, failover, and Wake-on-LAN operations.

Deliverables:

- Reset list/get/execute domains with source-locked reset-type validation.
- Failover list/get/route/delete domains with canonical IP handling.
- Wake-on-LAN get/execute domains keyed only by server number.
- Mutation types make reboot, rerouting, and wake intent explicit.
- Tests cover unsupported capabilities, invalid reset types, route targets,
  no-output responses, and deprecated server-IP aliases remaining absent.

Verification:

- `scripts/checks.sh`
- Focused Robot reset, failover, and Wake-on-LAN tests.
- Robot source-lock drift check.
- `scripts/release_0_37_gate.sh` once added.

Stop gate:

```text
v0.37.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.38.0 - Robot Boot Configuration

Goal: implement all 15 active Robot boot, rescue, Linux, VNC, and Windows
configuration operations.

Deliverables:

- Boot overview and rescue get/activate/deactivate/last domains.
- Linux get/activate/deactivate/last domains.
- VNC and Windows get/activate/deactivate domains.
- Passwords, authorized keys, host keys, language, distribution, architecture,
  and license-related fields receive explicit validation and redaction policy.
- Deprecated server-IP aliases and deprecated response fields are excluded or
  represented only through a documented compatibility boundary.
- Atomic form encoding and adversarial secret-output tests cover every mode.

Verification:

- `scripts/checks.sh`
- Focused Robot boot configuration and secret-handling tests.
- Robot source-lock drift check.
- `scripts/release_0_38_gate.sh` once added.

Stop gate:

```text
v0.38.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.39.0 - Robot Reverse DNS, Traffic, And SSH Keys

Goal: implement the 11 active reverse DNS, traffic, and SSH-key operations.

Deliverables:

- Reverse-DNS list/get/set/update/delete domains with canonical addresses and
  bounded DNS names.
- Traffic query domain with bounded time ranges, intervals, repeated addresses,
  and numeric response limits.
- SSH-key list/create/get/update/delete domains with fingerprint, key, name,
  algorithm, and response redaction policy.
- Form encoding safely handles SSH key material and repeated traffic inputs.
- Focused DNS, date-range, numeric-boundary, key-format, and secret-output tests.

Verification:

- `scripts/checks.sh`
- Focused Robot reverse-DNS, traffic, and SSH-key tests.
- Robot source-lock drift check.
- `scripts/release_0_39_gate.sh` once added.

Stop gate:

```text
v0.39.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.40.0 - Robot Firewalls And vSwitches

Goal: implement the 15 active Robot firewall and vSwitch operations.

Deliverables:

- Server firewall get/update/delete and template list/create/get/update/delete
  domains.
- vSwitch list/create/get/update/cancel and server attach/detach domains.
- Bounded firewall rule collections, canonical addresses, ports, protocols,
  VLANs, server lists, and cancellation dates.
- Explicit mutation intent for firewall replacement, vSwitch cancellation, and
  server membership changes.
- Tests cover rule ordering, duplicate/conflicting entries, CIDR policy,
  repeated form fields, in-progress conflicts, and empty responses.

Verification:

- `scripts/checks.sh`
- Focused Robot firewall and vSwitch tests.
- Robot source-lock drift check.
- `scripts/release_0_40_gate.sh` once added.

Stop gate:

```text
v0.40.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.41.0 - Robot Ordering Catalogs And Read-Only Transactions

Goal: implement the 12 read-only Robot ordering operations without admitting
billable order creation yet.

Deliverables:

- Standard-server and Server Auction product list/get domains.
- Standard-server, Server Auction, and addon transaction list/get domains.
- Per-server addon product list and account currency lookup.
- Exact decimal price strings, setup/hourly/monthly prices, locations,
  distributions, addons, product limits, and transaction states are bounded.
- Deprecated product response fields remain outside the stable model unless a
  reviewed compatibility boundary requires them.
- Read-only ordering examples state that displayed prices can change and must
  be revalidated before any later purchase request.

Verification:

- `scripts/checks.sh`
- Focused Robot ordering catalog, price, and transaction-response tests.
- Robot source-lock drift check.
- `scripts/release_0_41_gate.sh` once added.

Stop gate:

```text
v0.41.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.42.0 - Robot Billable Ordering Mutations

Goal: implement the three server, Server Auction, and addon order-creation
operations with explicit cost-bearing intent.

Deliverables:

- Typed order requests for standard servers, Server Auction products, and
  server addons.
- Required product, location, distribution, SSH-key, addon, quantity, and
  acceptance fields are source-locked and atomically form encoded.
- Callers must supply an explicit cost-acknowledgement marker bound to the
  selected product and observed price; no convenience default can place an
  order.
- Order requests and responses redact credentials and customer identifiers.
- CI and normal live smoke tests can never submit billable orders; only a
  separate manual destructive harness may exercise them.
- Adversarial tests prove missing acknowledgement, stale/mismatched product
  identity, malformed prices, duplicate addons, and partial buffers fail closed.

Verification:

- `scripts/checks.sh`
- Focused Robot billable-order policy and form-encoding tests.
- Credential-free tests proving normal live paths cannot submit orders.
- Robot source-lock drift check.
- `scripts/release_0_42_gate.sh` once added.

Stop gate:

```text
v0.42.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.43.0 - Robot Client Integration And Live Evidence

Goal: combine Robot request, transport, response, and policy layers into usable
blocking and async workflows without weakening destructive-operation controls.

Deliverables:

- High-level Robot client workflows for all active endpoint families.
- Blocking and executor-neutral async examples with Basic credentials kept
  distinct from Cloud API bearer tokens.
- Mock fixtures cover success, invalid input, auth, rate-limit, maintenance,
  conflict, empty-body, and oversized-response behavior.
- Opt-in read-only live smoke catalog uses a private credential file, bounded
  responses, redacted diagnostics, and source-locked Robot authority.
- Live tests never attempt invalid credentials because Robot can temporarily
  block the source IP after repeated authentication failures.
- Destructive and billable operations remain absent from normal live smoke
  catalogs and CI.

Verification:

- `scripts/checks.sh`
- Robot mock and high-level workflow tests.
- Credential-free live staging and `--check` tests.
- Explicit operator-run read-only live smoke test.
- Robot source-lock drift check.
- `scripts/release_0_43_gate.sh` once added.

Stop gate:

```text
v0.43.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.44.0 - Complete Hetzner SDK Hardening

Goal: finish the complete pre-1.0 Hetzner surface and produce release-candidate
evidence for Cloud, DNS, Console Storage Box, and Robot together.

Deliverables:

- Robot matrix reports every active operation implemented and every deprecated
  operation explicitly excluded; no unexplained planned row remains.
- Combined high-level client, response, error, rate-limit, auth-separation, and
  transport behavior receives public API and semver review.
- Robot form writers, paths, parsers, errors, ordering policy, and response
  envelopes receive fuzz and adversarial coverage.
- Complete Hetzner examples, security recipes, threat model, API matrices,
  source locks, platform evidence, SBOMs, dependency review, and migration notes.
- Final deprecation policy covers Cloud/DNS and Robot without exporting legacy
  aliases accidentally.
- Full pentest focuses on credential separation, authentication lockout,
  destructive calls, billable ordering, SSRF, encoding ambiguity, redaction,
  response bounds, and release evidence.

Verification:

- `scripts/checks.sh`
- Cloud/DNS, Console Storage Box, and Robot live drift checks.
- Complete active-operation matrix checks.
- Robot and existing fuzz build/smoke suites.
- `scripts/generate-sbom.sh`
- `cargo deny check`
- `cargo audit`
- `scripts/release_0_44_gate.sh` once added.

Stop gate:

```text
v0.44.0 implementation stop reached. Run pentest for this exact commit.
```

### v1.0.0 - Full Hetzner Production SDK

Goal: first serious production-ready `cloud-sdk` foundation and complete
Hetzner Cloud, DNS, Console Storage Box, and Robot provider.

Deliverables:

- Complete claimed endpoint coverage for every non-deprecated Cloud, DNS,
  Console Storage Box, and Robot operation.
- Deprecated Robot Storage Box and legacy alias operations remain excluded.
- Default graphs remain no_std and transport-free.
- Optional blocking, async, deterministic-root, and FIPS transport adapters
  have current security and dependency evidence.
- Bearer and Basic credentials remain type-separated, redacted, sanitized, and
  bound to fixed HTTPS authorities.
- API drift processes are documented and tested for every supported Hetzner
  source.
- Live and mock tests cover critical read-only workflows while destructive and
  billable workflows require explicit manual policy gates.
- Platform matrix, MSRV, documentation, SBOM, audit, fuzz, and pentest evidence
  are current.
- Provider-neutral naming and module patterns are documented for later focused
  provider crates.

Verification:

- `scripts/checks.sh`
- All Hetzner source-lock drift checks.
- Complete active-operation matrix checks.
- `scripts/generate-sbom.sh`
- `cargo deny check`
- `cargo audit`
- Full release gate script for `v1.0.0` once added.

Stop gate:

```text
v1.0.0 implementation stop reached. Run pentest for this exact commit.
```
