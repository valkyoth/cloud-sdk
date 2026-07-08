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
v1.1.0+     post-1.0 Robot Webservice support track
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
- `cargo deny check` passes;
- `cargo audit` passes;
- `scripts/generate-sbom.sh` succeeds;
- release notes exist at `release-notes/RELEASE_NOTES_X.Y.Z.md`;
- a pentest report exists at `security/pentest/vX.Y.Z.md`;
- the pentest report names the exact full 40-character `Reviewed-Commit:`;
- the pentest report has `Status: PASS`;
- the pentest report has non-blank `Tester:` and `Scope:` fields;
- the pentest report has a `Date: YYYY-MM-DD` field;
- `sbom/cloud-sdk.spdx.json` exists and is non-empty;
- GitHub CI and CodeQL default setup are green on the release-report commit;
- tagging has been explicitly requested.

`Reviewed-Commit:` records the implementation commit that was reviewed. If
retest, CodeQL, or another release gate causes release-relevant changes, rerun
the review and update `Reviewed-Commit:` to the latest reviewed commit before
tagging.

When a version's implementation criteria are done, stop and say:

```text
vX.Y.Z implementation stop reached. Run pentest for this exact commit.
```

No tag is created at that point.

### Pentest Handoff Flow

Use this loop for every version:

1. Implementation reaches the version stop point.
2. Local gates pass: `scripts/checks.sh`, `cargo deny check`, and
   `cargo audit`.
3. The maintainer runs pentest and writes temporary findings to root
   `PENTEST.md`.
4. Findings are reviewed and fixed.
5. Documentation, tests, release notes, and drift/source-lock evidence are
   updated for the fixes.
6. `PENTEST.md` is removed after findings are handled.
7. Local gates are run again.
8. GitHub CI and CodeQL default setup are checked after the fix commit.
9. A permanent report is written at `security/pentest/vX.Y.Z.md` only when the
   exact implementation commit has passed with `Status: PASS`.
10. Commit the permanent report and any required release metadata.
11. GitHub CI and CodeQL default setup are checked on the report commit.
12. Tagging and pushing tags happen only when explicitly requested.

Root `PENTEST.md` is temporary scratch input. It must not be committed. The
permanent report is part of the release tag.

## Crate Versioning And Publish Order

Provider-neutral domains live in `cloud-sdk`. Hetzner endpoint models live in
`cloud-sdk-hetzner`. Extra provider-specific crates are versioned only for real
optional boundaries: reqwest transport, testkit fixtures, and secret
sanitization.

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
| Deprecated datacenter endpoints exist in the spec but should not become accidental public commitments. | Tracked as `deferred-deprecated` in `docs/API_MATRIX.md`; final deprecated-endpoint policy lands in `v0.25.0`. |
| Resource-local action lookups are deprecated upstream but still present in the spec. | Tracked as `deferred-deprecated`; action helper policy lands in `v0.18.0`. |
| API drift could otherwise be missed between endpoint implementation passes. | Added operation and schema fingerprints in `v0.2.0`; recurring maintenance hardening lands in `v0.24.0`. |
| Optional serde support can break no_std/default graph expectations. | Scheduled as a dedicated boundary in `v0.14.0`. |
| Transport adapters can accidentally admit runtime, TLS, or secret handling assumptions. | Blocking and async adapters are separated into `v0.16.0` and `v0.17.0`, after model/testkit work. |
| Robot Webservice has different auth, encoding, and API shape than Cloud/DNS. | Deferred to `v1.1.0+` with a separate source lock and implementation track. |
| Future providers such as Cloudflare need patterns but are not part of Hetzner 1.0. | Provider-neutral naming and module guidance are part of `v1.0.0`; no non-Hetzner provider is claimed before 1.0. |

## Milestones

### v0.1.0 - Repository Foundation

Status: tagged.

Goal: initialize the serious Rust workspace and policy baseline.

Deliverables:

- Rust stable `1.96.1` pinned.
- Rust `1.90.0` through `1.96.1` compatibility policy.
- One provider-neutral no_std crate, one focused Hetzner provider crate, and
  three optional Hetzner boundary crates.
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
- `scripts/release_0_7_gate.sh` once added.

Stop gate:

```text
v0.7.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.8.0 - Volumes And Floating IPs

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
- `scripts/release_0_8_gate.sh` once added.

Stop gate:

```text
v0.8.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.9.0 - Storage Box Models

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
- `scripts/release_0_9_gate.sh` once added.

Stop gate:

```text
v0.9.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.10.0 - Firewalls And Networks

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
- `scripts/release_0_10_gate.sh` once added.

Stop gate:

```text
v0.10.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.11.0 - Load Balancer Models

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
- `scripts/release_0_11_gate.sh` once added.

Stop gate:

```text
v0.11.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.12.0 - DNS Zones

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
- `scripts/release_0_12_gate.sh` once added.

Stop gate:

```text
v0.12.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.13.0 - DNS RRSets

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
- `scripts/release_0_13_gate.sh` once added.

Stop gate:

```text
v0.13.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.14.0 - Optional Serde Boundary

Goal: admit optional serde request/response support without weakening the
default no_std graph.

Deliverables:

- Non-default serde feature or adapter crate decision.
- Derive coverage for source-locked request and response models implemented so
  far.
- Duplicate/unknown-field, optional-null, and redaction policy.
- Tests proving default features remain empty and no serde dependency appears
  in the default graph.
- JSON fixture tests for representative success and error responses.

Verification:

- `scripts/checks.sh`
- `cargo tree -p cloud-sdk-hetzner --no-default-features`
- `cargo test -p cloud-sdk-hetzner --all-features serde`
- `scripts/release_0_14_gate.sh` once added.

Stop gate:

```text
v0.14.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.15.0 - Mock Transport And Testkit

Goal: implement deterministic mock transport, pagination/action fixtures, and
adversarial response corpus before real transports are admitted.

Deliverables:

- First usable `cloud-sdk-hetzner-testkit` mock transport boundary.
- Fixture builders for success, paginated, action, rate-limit, and error
  responses.
- Adversarial corpus for malformed JSON, unknown fields, missing required
  fields, oversized responses, invalid pagination, and invalid action states.
- Tests proving mock transport does not require network, TLS, filesystem, or
  runtime dependencies by default.

Verification:

- `scripts/checks.sh`
- `cargo test -p cloud-sdk-hetzner-testkit --all-features`
- `cargo test --workspace --all-features`
- `scripts/release_0_15_gate.sh` once added.

Stop gate:

```text
v0.15.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.16.0 - Optional Blocking Transport Adapter

Goal: admit the first reviewed blocking transport adapter outside the default
graph.

Deliverables:

- Blocking transport trait implementation in an optional adapter crate.
- Dependency admission document for HTTP, TLS, URL, and header crates used.
- Explicit timeout, user-agent, authentication header, retry, and redaction
  policy.
- Tests with mock HTTP server or testkit fixtures only; no live network by
  default.
- Default workspace graph remains transport-free.

Verification:

- `scripts/checks.sh`
- `cargo test -p cloud-sdk-hetzner-reqwest --all-features`
- `cargo tree -p cloud-sdk-hetzner --no-default-features`
- `scripts/release_0_16_gate.sh` once added.

Stop gate:

```text
v0.16.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.17.0 - Optional Async Transport Adapter

Goal: add async transport support with explicit runtime neutrality and no
default runtime dependency.

Deliverables:

- Async transport trait or adapter implementation behind non-default features.
- Runtime-neutral future model or documented runtime feature policy.
- Cancellation, timeout, body-size, rate-limit, and retry guidance.
- Tests with deterministic mock transport and no live API by default.
- Dependency review for async, TLS, and HTTP crates.

Verification:

- `scripts/checks.sh`
- `cargo test -p cloud-sdk-hetzner-reqwest --all-features`
- `cargo tree -p cloud-sdk-hetzner --no-default-features`
- `scripts/release_0_17_gate.sh` once added.

Stop gate:

```text
v0.17.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.18.0 - Pagination And Action Polling Helpers

Goal: provide ergonomic optional helpers over transport traits without hiding
rate-limit, timeout, or retry policy.

Deliverables:

- Pagination helper that exposes page boundaries and rate-limit metadata.
- Action polling helper with caller-supplied delay/backoff policy.
- Terminal action states and failure propagation.
- Tests for stop conditions, timeout/cancel behavior, empty pages, repeated
  pages, action failure, and rate-limit propagation.

Verification:

- `scripts/checks.sh`
- `cargo test --workspace --all-features pagination action_polling`
- `scripts/release_0_18_gate.sh` once added.

Stop gate:

```text
v0.18.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.19.0 - Live Smoke Harness

Goal: add opt-in live tests gated by environment variables and least-privilege
test project guidance.

Deliverables:

- Live smoke harness disabled by default.
- Required environment variables and token-scope guidance.
- Read-only smoke tests for catalog resources.
- Optional destructive test plan that requires explicit opt-in and resource
  naming prefix.
- Redaction of tokens and IDs in logs.

Verification:

- `scripts/checks.sh`
- `cargo test --workspace --all-features`
- Documented manual live-smoke command with no token in shell history examples.
- `scripts/release_0_19_gate.sh` once added.

Stop gate:

```text
v0.19.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.20.0 - Platform Matrix

Goal: prove claimed platform support for Linux, Windows, BSD, macOS, Android,
iOS, WASM, and embedded/no_std targets where applicable.

Deliverables:

- Target matrix document with supported, best-effort, and unsupported targets.
- `cargo check` evidence for no_std and std feature combinations.
- CI jobs or documented local commands for representative targets.
- Platform-specific transport limitations documented.
- Tests proving default crates do not require OS services.

Verification:

- `scripts/checks.sh`
- Target-specific `cargo check` commands documented in release notes.
- `scripts/release_0_20_gate.sh` once added.

Stop gate:

```text
v0.20.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.21.0 - Documentation And Examples Hardening

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
- Documentation link check if available.
- `scripts/release_0_21_gate.sh` once added.

Stop gate:

```text
v0.21.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.22.0 - Fuzzing And Adversarial Tests

Goal: fuzz request builders, parsers, validators, and response handling.

Deliverables:

- Fuzz targets for query/path builders, label selectors, DNS records,
  pagination metadata, error envelopes, and action states.
- Seed corpus from source-locked valid and invalid examples.
- Crash reproduction process.
- CI or release-gate build check for fuzz targets, without requiring long fuzz
  runs in every CI job.
- Adversarial tests for malformed upstream responses and oversized inputs.

Verification:

- `scripts/checks.sh`
- Fuzz target build command documented in release notes.
- `cargo test --workspace --all-features`
- `scripts/release_0_22_gate.sh` once added.

Stop gate:

```text
v0.22.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.23.0 - Dependency And Tooling Hardening

Goal: refresh dependency, tool, SBOM, audit, and supply-chain evidence before
release-candidate work.

Deliverables:

- Current dependency review for every default, optional, dev, and tool crate.
- `cargo-deny` and `cargo-audit` evidence.
- SBOM generation and documentation.
- Toolchain and MSRV review for Rust `1.90.0` through current pinned stable.
- Updated security controls and supply-chain docs.

Verification:

- `scripts/checks.sh`
- `scripts/check_latest_tools.sh`
- `scripts/generate-sbom.sh`
- `cargo deny check`
- `cargo audit`
- `scripts/release_0_23_gate.sh` once added.

Stop gate:

```text
v0.23.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.24.0 - API Drift Automation Hardening

Goal: make upstream drift monitoring actionable as a maintenance process, not
only a one-off source lock.

Deliverables:

- Drift detector reports grouped by added, removed, changed, deprecated, and
  schema-only changes.
- Maintenance playbook for accepting, rejecting, or deferring upstream changes.
- Optional scheduled CI workflow or documented manual check for maintainers.
- Release-note template for upstream drift updates.
- Tests for the drift detector using checked-in fixture specs.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- Drift-detector fixture tests.
- `scripts/release_0_24_gate.sh` once added.

Stop gate:

```text
v0.24.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.25.0 - Release Candidate Cleanup

Goal: finish public API review, deprecation policy, examples, docs, and
semver-readiness audit before 1.0.

Deliverables:

- Public API review for all exported types and feature flags.
- Deprecated upstream endpoint policy.
- Error and versioning policy.
- Semver-readiness audit and migration notes.
- Examples and docs.rs output reviewed.
- Release notes for known limitations and 1.0 blockers.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `cargo public-api` or equivalent if admitted.
- `scripts/release_0_25_gate.sh` once added.

Stop gate:

```text
v0.25.0 implementation stop reached. Run pentest for this exact commit.
```

### v1.0.0 - Production SDK

Goal: first serious production-ready `cloud-sdk` foundation and Hetzner
Cloud/DNS provider.

Deliverables:

- Complete claimed endpoint coverage for non-deprecated Cloud/DNS and Storage
  Box operations.
- Default graph remains no_std and transport-free.
- Optional transport adapters have security and dependency evidence.
- API drift process is documented and tested.
- Live and mock tests cover critical workflows.
- Platform matrix and MSRV evidence are current.
- Provider-neutral naming and module patterns are documented for later crates
  such as `cloud-sdk-cloudflare`.

Verification:

- `scripts/checks.sh`
- `scripts/check_hetzner_api_drift.py --fetch`
- `scripts/generate-sbom.sh`
- `cargo deny check`
- `cargo audit`
- Full release gate script for `v1.0.0` once added.

Stop gate:

```text
v1.0.0 implementation stop reached. Run pentest for this exact commit.
```

### v1.1.0 - Robot Webservice Source Lock

Goal: start Robot Webservice support without changing the 1.0 Cloud/DNS
contract.

Deliverables:

- Source-lock <https://robot.hetzner.com/doc/webservice/en.html>.
- Document Robot as a distinct API surface with HTTP Basic Auth, HTTPS-only
  transport, form-encoded POST parameters, JSON/YAML response modes, and
  Robot-specific errors/rate limits.
- Add a `robot` module plan for server, IP, subnet, reset, failover,
  wake-on-LAN, boot configuration, reverse DNS, traffic, SSH keys, server
  ordering, Robot storage box, firewall, and vSwitch operations.
- Decide whether Robot implementation lives in the main SDK crate modules only
  or also gets an optional adapter/helper crate.

Verification:

- `scripts/checks.sh`
- Robot source-lock drift check once added.
- `scripts/release_1_1_gate.sh` once added.

Stop gate:

```text
v1.1.0 implementation stop reached. Run pentest for this exact commit.
```

### v1.2.0+ - Robot Webservice Implementation

Goal: implement Robot Webservice operations in small reviewed passes and expose
them through `cloud-sdk-hetzner`.

Deliverables:

- Separate Robot auth, encoding, error, rate-limit, fixture, and live-test
  evidence.
- Robot operations implemented in small endpoint-family releases.
- Cloud/DNS default behavior remains unchanged.
- Robot docs clearly separate Cloud API tokens from Robot webservice users.

Verification:

- `scripts/checks.sh`
- Robot source-lock drift check.
- Version-specific release gate scripts.

Stop gate:

```text
v1.2.0 implementation stop reached. Run pentest for this exact commit.
```
