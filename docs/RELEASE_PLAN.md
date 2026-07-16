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
v0.47.0+    pre-1.0 Robot Webservice support track
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
| Safe endpoint models still require callers to assemble transport requests, status checks, response bounds, content-type checks, and decoding manually. | Add common prepared-request policy in `v0.29.0`, complete preparation in `v0.30.0`, checked decoding in `v0.31.0`, typed operations in `v0.39.0`, and opt-in client workflows inside `cloud-sdk-hetzner` in `v0.41.0`. No nested Hetzner client crate is introduced. |
| A provider-neutral custom HTTPS endpoint receives the configured credential and can exfiltrate it when its value is attacker-controlled. | Make custom endpoint trust explicit in `v0.27.0`, expose immutable credential-bound endpoint identity in `v0.28.0`, add provider endpoint policy in `v0.32.0`, split auth from raw execution in `v0.34.0`, and require official or acknowledged custom destinations in the `v0.41.0` client. |
| Closed provider/API-family enums and a four-method HTTP enum would force breaking core changes for later providers. | Replace central taxonomies with bounded extensible identifiers, add standard method coverage, and introduce provider-owned endpoint policies in `v0.32.0`. |
| The transport request cannot carry bounded provider headers, idempotency keys, conditional requests, signing inputs, or general response metadata. | Add canonical path/query and bounded header/metadata contracts in `v0.33.0`, then split raw HTTP execution from provider authentication in `v0.34.0`. |
| Core pagination and rate-limit types encode Hetzner's numbered-page and three-header conventions. | Introduce explicit pagination strategies, provider-owned quota decoders, bounded quota buckets, and source-locked retry/idempotency policy in `v0.35.0`. |
| `AsyncTransport` requires `Send`, excluding common browser-WASM and single-threaded executor transports. | Add a separate `LocalAsyncTransport` plus opt-in streaming contract boundaries in `v0.36.0`; existing `AsyncTransport` keeps its cross-thread guarantee. |
| Caller capacity prediction and one fixed large JSON profile make constrained environments unnecessarily difficult. | Add exact capacity reporting, standard storage profiles, allocation-backed conveniences, and selectable parser limits in `v0.37.0`; add streaming list/metrics decoding in `v0.38.0`. |
| Query/body compatibility and response selection still depend on runtime operation keys. | Add typed operation/output contracts, sealed query/body associations, typed prepared requests, and forward-compatible informational enums in `v0.39.0`. |
| Cost, destructive intent, and retry metadata are inspectable but not enforced at execution. | Add scoped mutation/destructive/cost permits, plan-confirm fingerprints, idempotency binding, and no-op mutation rejection in `v0.40.0`. |
| The secure end-to-end workflow is still manually assembled. | Move the high-level client to `v0.41.0`, after the neutral transport, metadata, typed-operation, and permit contracts are stable. |
| Pagination/action workflows and diagnostics remain low-level, while the testkit cannot yet model dynamic multi-request scenarios. | Add pure workflow drivers, payload-free structured diagnostics, dynamic responders, fault injection, and request recording in `v0.42.0`. |
| Drift tooling is Hetzner-specific and historical review evidence depends mainly on fingerprints. | Add a provider-manifest drift engine, canonical reviewed diffs, alert ownership, and compatibility policy in `v0.43.0`. |
| Code review alone cannot prove that the provider-neutral core supports materially different providers. | Build an unpublished 5-10-operation OVHcloud API v2 architecture probe and freeze the neutral contracts only after its geographic endpoint, OAuth2, custom-header, cursor-pagination, and asynchronous-task requirements pass conformance in `v0.44.0`. |
| Existing Hetzner response models expose common identity rather than the complete supported field set, and timestamp validation is inconsistent. | Complete Cloud models and shared RFC3339 validation in `v0.45.0`, then complete DNS, security, and Console Storage Box models in `v0.46.0`. |
| Robot Webservice has different auth, encoding, and API shape than Cloud/DNS. | Assigned a separate source lock and twelve pre-1.0 implementation and hardening milestones from `v0.47.0` through `v0.58.0`. |
| Legacy Robot Storage Box operations are deprecated and overlap the supported Console API. | The `v0.47.0` Robot matrix must mark all 16 legacy operations excluded and must not create a Robot Storage Box module. |
| Repeated invalid Robot credentials can temporarily block the caller's source IP. | Basic credentials are type-separated and redacted in `v0.48.0`; `v0.57.0` live tests never submit intentionally invalid credentials. |
| Robot ordering mutations can create immediate infrastructure costs. | Read-only ordering lands separately in `v0.55.0`; `v0.56.0` requires scoped cost permits and keeps billable calls outside CI and normal live smoke tests. |
| Successful decoding does not automatically clear caller-owned wire storage. | Add `decode_and_clear` and guard-based cleanup in `v0.37.0`, with error, success, cancellation, and panic-unwind tests where `std` is available. |
| Release controls do not by themselves provide organizationally independent review or repository-pinned signer authorization. | Add explicit governance limits, signer authorization/rotation policy, protected release guidance, provenance review, and independent pre-1.0 review evidence in `v0.59.0`; do not claim independence when it was not available. |
| Destructive and billable behavior lacks controlled disposable-account integration evidence. | Add an explicitly manual mutation harness with spending ceilings, unique prefixes, cleanup ledgers, and empty-inventory verification in `v0.60.0`; CI remains incapable of invoking it. |
| Future providers need proven patterns but are not part of the Hetzner 1.0 product claim. | The unpublished OVHcloud v2 probe lands in `v0.44.0`. Post-1.0 publishing starts with Scaleway, then DigitalOcean, while a full OVHcloud provider follows only after a dedicated plan for its broader v1/v2, authentication, ordering, and OpenStack boundaries. |

## Post-1.0 Provider Sequence

The pre-1.0
[OVHcloud API v2](https://docs.ovhcloud.com/en/guides/manage-and-operate/api/apiv2/)
probe is architecture evidence, not a provider release. It stays in an excluded
package or fixture, is absent from the publish sequence, carries no support
claim, and must not become `cloud-sdk-ovhcloud` by accident. Its purpose is to
test contracts that differ materially from Hetzner: geographic API
authorities, OAuth2 service-account authentication, versioned response
headers, cursor pagination in headers, and asynchronous task or event
resources.

Published provider work starts only after the Hetzner `v1.0.0` release:

1. `cloud-sdk-scaleway` is the first published provider. Its source lock and
   release plan must cover
   [Scaleway's APIs](https://www.scaleway.com/en/developers/api/), including
   global, regional, and zonal endpoints, `X-Auth-Token`, PATCH operations,
   product-specific schemas, page-based pagination, and response quota
   metadata. Stable GA API versions form the supported completeness claim.
   Alpha and beta APIs require explicit experimental modules or features and
   are excluded from stable coverage.
2. `cloud-sdk-digitalocean` is the second published provider. It must use
   DigitalOcean's
   [official OpenAPI source](https://github.com/digitalocean/openapi) and prove
   the conventional bearer-auth, `/v2`, link-pagination, request-ID,
   rate-limit, and `Retry-After` path without weakening bounded decoding or
   retry policy. Large adjacent surfaces such as Spaces, metadata, OAuth
   applications, and AI services require explicit scope decisions rather than
   entering the initial claim automatically.
3. `cloud-sdk-ovhcloud` follows later as a full provider. Its dedicated plan
   must separate API v2, required API v1 compatibility, OAuth2 and any retained
   legacy authentication, geographic endpoints, asynchronous tasks, ordering
   and other billable operations, and OpenStack-based services. The v0.44 probe
   does not pre-approve those product or security boundaries.

Every published provider keeps one primary crate, a separate official source
lock, threat model, API matrix, live-test policy, release plan, and pentest
stop gates. Exact post-1.0 versions are assigned when the preceding provider is
stable enough that the next provider will not dilute maintenance or security
review.

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

Status: release candidate; pentest and retest passed.

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
  no AST-bound prepared endpoint or required body adapter. Dedicated Rust
  tests lock security-sensitive metadata and response policy.
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
- Excluded, locked `syn` checker and adversarial `cfg_attr`, nested-comment,
  raw-string, discarded-literal, helper-expression, namespaced-macro,
  shadowing, inline-fake-trait, inner-attribute, duplicate-definition,
  definition-replacement, orphan-module, path-redirection, inline-module,
  parent-edge substitution, Cargo-library-target substitution, preceding
  evidence statements, attributed-expression erasure, procedural-attribute
  erasure, operation-scrutinee substitution, manual-query compatibility,
  parent-defined and `include!` implementation macros, attributed associated
  functions/constants/types, module-item attributes and derives, nested
  implementations and statement macros in functions/constants/wire methods and
  adapter arguments, anonymous-constant implementations in every adapter type
  and writer path, opaque expression/type/pattern macros, parent-defined and
  imported aliases named `matches`, substituted explicit-match scrutinees,
  unparsed endpoint-prepare type lists, generated-shadow, and duplicate mapping
  mutations.
- Per-family golden request and insufficient-buffer tests.
- Mutation-classification and source-locked response-policy tests.
- `scripts/release_0_30_gate.sh` once added.

Stop gate:

```text
v0.30.0 pentest stop passed. Commit the permanent report, run the clean release
gate, and wait for GitHub before tagging.
```

### v0.31.0 - Checked Hetzner Response Decoding

Status: tagged.

Goal: provide one checked decoding path that consumes a transport response,
enforces every prepared response policy, and returns typed provider success or
error data without requiring callers to remember security steps.

Deliverables:

- Source-locked success response bindings cover every non-deprecated Cloud,
  DNS, and Console Storage Box operation, including resource identity and list
  envelopes, pagination, action results, empty success bodies, metrics,
  zonefiles, pricing, folders, and composite secret-bearing results.
- A checked decoder consumes `TransportResponse` together with the operation's
  prepared metadata; callers cannot pass a raw body while bypassing its status,
  content-type, empty-body, or maximum-size policy. Endpoint/service mismatch
  is rejected before transport execution by the prepared-request path.
- The decoder applies the bounded `ResponseBytes` boundary before parser use,
  then uses a direct protected string decoder and shared aggregate JSON-node
  budget before returning either the operation's typed success value or typed
  Hetzner API error envelope according to source-locked status semantics.
- Unexpected status, malformed or missing content type, oversized body,
  malformed payload, duplicate fields, invalid identifiers, unknown enum values,
  and typed provider errors remain distinct payload-free error cases.
- Response models validate security-relevant fields after parsing, tolerate
  only documented additive compatibility, and never expose unvalidated wire
  structs publicly. Provider-complete resource field models remain scheduled
  before `1.0.0` and are not claimed by this release.
- Operator-facing decoded text rejects Unicode control, bidi, isolate,
  zero-width, and related invisible formatting characters. Source-locked
  secrets, provider errors, and action errors decode escaped and unescaped text
  directly into first-party volatile-clearing owned storage, move into public
  sensitive models without another plaintext allocation, and remain protected
  across parser and model-validation error paths.
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
  documented success/error status shape, including aggregate heap amplification
  plus escaped credential and error-message parser/model paths.
- Fuzz coverage for shared envelopes and representative resource, list,
  metrics, zonefile, nullable, empty-body, and malformed response paths.
- Default/no_std and optional decoder feature-matrix checks.
- `scripts/release_0_31_gate.sh` once added.

Stop gate:

```text
v0.31.0 tagged.
```

### v0.32.0 - Extensible Core Identity And Endpoint Policy

Goal: remove Hetzner-shaped closed taxonomies before another provider or Robot
depends on them.

Deliverables:

- Replace central closed provider and API-family enums with bounded validated
  identifiers or provider-owned marker types that permit additive providers and
  services without breaking exhaustive downstream matches.
- Extend method support for `PATCH`, `HEAD`, and `OPTIONS`; document how unknown
  extension methods are represented or rejected.
- Add provider-owned endpoint policies for one official endpoint, a static
  official set, region-derived authorities, and explicitly acknowledged custom
  endpoints.
- Migrate every existing prepared Hetzner operation without weakening exact
  authority, scheme, port, base-path, or credential-destination checks.
- Publish a migration guide and compile-fail tests for accidental custom
  endpoint trust.

Verification: `scripts/checks.sh`, default/no_std checks, endpoint-policy
adversarial tests, all prepared-operation coverage, and
`scripts/release_0_32_gate.sh`.

Stop gate: `v0.32.0 implementation stop reached. Run pentest for this exact commit.`

### v0.33.0 - Canonical Targets And Bounded HTTP Metadata

Goal: make request-target and header security consistent across every adapter.

Deliverables:

- Represent path and query separately until final origin-form assembly.
- Validate every percent triplet; reject encoded controls, encoded path
  separators, dot-segment ambiguity, fragments, backslashes, and raw non-ASCII
  path input at the core boundary.
- Add bounded header names and values, duplicate policy, sensitivity markers,
  deterministic ordering, and atomic caller-buffer encoding.
- Add typed common headers for idempotency, conditional requests, API version,
  correlation IDs, and provider signing inputs without logging their values.
- Add bounded response metadata for request ID, `Retry-After`, `ETag`,
  `Location`, and provider extension fields.

Verification: target/header fuzzing, cross-adapter conformance, malformed
percent and duplicate-header tests, `scripts/checks.sh`, and
`scripts/release_0_33_gate.sh`.

Stop gate: `v0.33.0 implementation stop reached. Run pentest for this exact commit.`

### v0.34.0 - Raw HTTP Execution And Authentication Pipeline

Goal: separate provider-neutral HTTP execution from authentication and signing.

Deliverables:

- Add a raw bounded HTTP executor contract that sends already validated method,
  target, headers, and body and returns bounded status, headers, and body.
- Stop requiring the neutral reqwest adapter to inject bearer auth or JSON
  `Accept`; provider/client layers construct complete authenticated requests.
- Provide optional reviewed bearer and Basic policies with type-separated
  credentials, HTTPS-only enforcement, endpoint binding, rotation, redaction,
  and sanitization-backed cleanup.
- Define provider signing middleware inputs without adding a clock, randomness,
  filesystem, or secret store to core.
- Keep redirects, proxy discovery, implicit decompression, and cross-origin
  credential forwarding disabled.

Verification: auth-confusion, header-smuggling, endpoint-exfiltration, rotation,
blocking/async/FIPS, and dependency-boundary tests plus
`scripts/release_0_34_gate.sh`.

Stop gate: `v0.34.0 implementation stop reached. Run pentest for this exact commit.`

### v0.35.0 - Pagination, Quota, Retry, And Idempotency Strategies

Goal: replace provider conventions with explicit strategies and source-locked
operation policy.

Deliverables:

- Keep numbered pagination and add opaque cursor, offset, marker, and bounded
  link-navigation contracts.
- Move rate-limit decoding out of transports into provider policy; support
  bounded multiple quota buckets and pure reset/delay calculations.
- Record retry, idempotency, mutation, destructive, and cost classification per
  source-locked operation instead of deriving it only from HTTP method.
- Add typed idempotency keys bound to operation and request fingerprints;
  reject reuse with a different operation or body.
- Preserve unknown informational quota fields while rejecting unknown states
  that affect safety.

Verification: traversal, cursor-cycle, quota-overflow, policy-table completeness,
idempotency replay, and adversarial metadata tests plus
`scripts/release_0_35_gate.sh`.

Stop gate: `v0.35.0 implementation stop reached. Run pentest for this exact commit.`

### v0.36.0 - Local Async And Streaming Contract Boundaries

Goal: support single-threaded/WASM transports and future large-body APIs
without complicating the existing buffered contract.

Deliverables:

- Add `LocalAsyncTransport` for `!Send` futures while retaining
  `AsyncTransport` for cross-thread-capable futures.
- Add separate opt-in streaming upload, download, and event contracts with
  explicit cancellation and partial-I/O semantics.
- Do not add streaming to `TransportRequest`; buffered and streaming contracts
  remain distinct and feature-gated.
- Add compile evidence for browser-WASM, embedded-style local executors, and
  existing native adapters where toolchains permit.
- Document that WebSocket/SSE implementations are provider-driven and may
  remain post-1.0 when no claimed Hetzner operation requires them.

Verification: local-future compile tests, cancellation/drop tests, default graph
checks, platform matrix, and `scripts/release_0_36_gate.sh`.

Stop gate: `v0.36.0 implementation stop reached. Run pentest for this exact commit.`

### v0.37.0 - Capacity Profiles And Automatic Response Cleanup

Goal: make bounded storage practical on constrained and allocation-backed
systems while clearing wire data reliably.

Deliverables:

- Add exact or conservative `required_capacity`/`encoded_len` reporting for
  targets, headers, bodies, and known response profiles without exposing
  payload content in errors.
- Define reviewed embedded, default, and large-response storage/parser profiles;
  callers may select stricter limits than provider maxima.
- Add `alloc` convenience storage methods without changing empty default
  features or removing caller-owned alternatives.
- Add `decode_and_clear` and guard-based response cleanup that clears the full
  caller buffer on success and every error path.
- Ensure temporary growth in owned secret storage clears replaced allocations.

Verification: boundary capacities, embedded profiles, allocation failures,
success/error/cancellation cleanup, Miri where applicable, and
`scripts/release_0_37_gate.sh`.

Stop gate: `v0.37.0 implementation stop reached. Run pentest for this exact commit.`

### v0.38.0 - Streaming And Incremental Provider Decoding

Goal: avoid requiring one large contiguous JSON tree for large lists, metrics,
zonefiles, logs, and future provider data streams.

Deliverables:

- Add bounded incremental/SAX-style decoding for representative list and
  metrics responses with caller-owned visitor/state storage.
- Preserve duplicate-field, nesting, aggregate-node, string, and secret cleanup
  protections from the checked decoder.
- Provide deterministic backpressure and early-stop semantics without hidden
  allocation, threads, requests, or retries.
- Keep the existing checked contiguous decoder as the simple path for bounded
  responses.

Verification: chunk-boundary corpus, malformed/truncated streams, early-stop
cleanup, differential fixtures, fuzzing, and `scripts/release_0_38_gate.sh`.

Stop gate: `v0.38.0 implementation stop reached. Run pentest for this exact commit.`

### v0.39.0 - Typed Operations And Compile-Time Component Binding

Goal: make endpoint, query, body, and response associations unrepresentable
when mismatched.

Deliverables:

- Add a sealed provider operation trait with associated response, provider
  error, service, request shape, and policy metadata.
- Replace runtime operation-key compatibility checks with typed `QueryFor<O>`
  and `BodyFor<O>` associations.
- Make `Prepared<O>` retain the operation type so checked execution returns
  `O::Response` rather than a broad runtime success enum.
- Generate or source-derive complete bindings for all 208 active Hetzner
  operations and prove no fallback runtime binding remains.
- Adopt a documented forward-compatible enum policy: preserve unknown
  informational values and reject unknown safety-critical states.

Verification: compile-fail mismatch tests, complete binding gates, semver review,
fuzzing, and `scripts/release_0_39_gate.sh`.

Stop gate: `v0.39.0 implementation stop reached. Run pentest for this exact commit.`

### v0.40.0 - Enforced Mutation, Destructive, And Cost Intent

Goal: turn operation safety metadata into execution requirements.

Deliverables:

- Add scoped `MutationPermit`, `DestructivePermit`, and `CostPermit` types bound
  to provider, account/project scope, operation fingerprint, expiry input, and
  optional caller-selected spending ceiling.
- Require a stable plan/confirm hash for cost-bearing operations and reject
  changed operation inputs, product identity, or observed price.
- Reject update requests that contain no actual change; use typestate or
  `try_build` where it improves invalid-state prevention.
- Require explicit retry authorization for state-changing operations and bind
  any idempotency key to the confirmed request fingerprint.
- Keep permit creation caller-controlled; core owns no clock, price lookup,
  random generator, or persistent replay database.

Verification: stale/mismatched permit, no-op mutation, replay, expiry-input,
budget, redaction, and compile-fail tests plus
`scripts/release_0_40_gate.sh`.

Stop gate: `v0.40.0 implementation stop reached. Run pentest for this exact commit.`

### v0.41.0 - Secure High-Level Hetzner Client Workflows

Goal: make the secure typed end-to-end path the shortest supported path.

Deliverables:

- Add an opt-in client inside `cloud-sdk-hetzner`, generic over blocking,
  `Send` async, or local async execution; no nested Hetzner client crate.
- Official constructors bind Cloud/DNS and Console Storage Box credentials to
  exact endpoint policies; custom endpoints require explicit acknowledgement.
- Typed methods prepare bounded storage, authenticate, send exactly once,
  validate response policy, decode `O::Response`, and clear wire storage.
- No retries, sleeps, clocks, executors, TLS roots, or credential files are
  selected implicitly.
- Concurrent calls use independent storage and shared transports without a
  mutex guard across I/O or `.await`.
- READMEs and examples lead with compile-checked blocking and async workflows.

Verification: complete mock response-family workflows, official/custom endpoint
adversarial tests, concurrency/cancellation/rotation, live read-only smoke, and
`scripts/release_0_41_gate.sh`.

Stop gate: `v0.41.0 implementation stop reached. Run pentest for this exact commit.`

### v0.42.0 - Workflow Drivers, Diagnostics, And Testkit Scenarios

Goal: improve repeated workflows and operations without owning runtime policy.

Deliverables:

- Add pure pagers and action drivers that return the next typed request,
  `PollAgain` state, and optional delay hints while callers own sleep and clock.
- Add blocking iterators and opt-in async streams only in appropriate adapters.
- Add payload-free structured diagnostics containing provider/service,
  operation ID, status, request ID, retry category, and error category.
- Expand testkit with dynamic responders, bounded request recording, fault
  injection, pagination/action scenarios, and provider fixture builders.
- Never include credentials, secret query values, request/response bodies, or
  untrusted provider messages in diagnostics.

Verification: multi-page/action scenarios, injected failures, request recording
bounds, diagnostic redaction, and `scripts/release_0_42_gate.sh`.

Stop gate: `v0.42.0 implementation stop reached. Run pentest for this exact commit.`

### v0.43.0 - Provider-Generic Drift And Historical Evidence

Goal: make source locking reusable for later provider crates and auditable over
time.

Deliverables:

- Add a manifest-driven drift engine for source URLs/digests, operation IDs,
  auth, endpoints, pagination, headers, retry/idempotency, cost classification,
  and schema compatibility.
- Keep provider-specific parsers as plugins; fetched content remains untrusted,
  read-only, bounded, redirect-free, and incapable of generating trusted code.
- Store canonical reviewed old/new diffs or references to immutable evidence in
  addition to fingerprints.
- Define alert ownership and response expectations for scheduled drift failures.
- Migrate Hetzner Cloud/DNS/Console checks without reducing current source-lock
  guarantees.

Verification: plugin fixtures, malicious source documents, digest rotation,
canonical-diff reproducibility, scheduled workflow tests, and
`scripts/release_0_43_gate.sh`.

Stop gate: `v0.43.0 implementation stop reached. Run pentest for this exact commit.`

### v0.44.0 - OVHcloud API v2 Architecture Probe And Core Freeze

Goal: prove the neutral contracts against a materially different provider
before freezing them for 1.0.

Deliverables:

- Source-lock the official
  [OVHcloud API v2](https://docs.ovhcloud.com/en/guides/manage-and-operate/api/apiv2/)
  and
  [OAuth2 service-account](https://docs.ovhcloud.com/en/guides/account-and-service-management/account-information/authenticate-api-with-service-account/)
  documentation used by the probe.
- Implement 5-10 read-only OVHcloud API v2 operations in an unpublished,
  excluded conformance package or fixture. It is not a supported/public
  provider release.
- Exercise trusted geographic API-authority selection, provider-owned OAuth2
  bearer policy, bounded `X-Schemas-Version` metadata, cursor pagination from
  `X-Pagination-*` headers, and asynchronous task or event response models.
- Keep OAuth2 token acquisition outside the no_std core while proving that
  acquired credentials can be bound to the selected official OVHcloud
  authority without accepting tenant-controlled destinations.
- Exclude API v1 compatibility, legacy authentication, ordering, mutation,
  OpenStack product APIs, and all billable behavior from the probe.
- Require the probe to use core contracts without adding provider-specific
  exceptions to `cloud-sdk`.
- Record every abstraction change caused by the probe and complete a public
  API/semver freeze review for neutral 1.0 contracts.
- Preserve the one-primary-crate-per-published-provider rule.

Verification: official source lock for the probe, authority/auth/header/cursor
and asynchronous-resource conformance tests, no-publish and dependency-boundary
gates, API review, and
`scripts/release_0_44_gate.sh`.

Stop gate: `v0.44.0 implementation stop reached. Run pentest for this exact commit.`

### v0.45.0 - Complete Hetzner Cloud Response Models

Goal: replace common-identity placeholders with complete validated Cloud API
models.

Deliverables:

- Model the complete supported non-deprecated fields for compute, network,
  volume/IP storage, pricing, catalog, action, and metrics responses.
- Replace weak timestamp shape checks with one calendar-valid UTC RFC3339 type,
  including leap-year, day, and time-range validation.
- Apply the forward-compatible enum policy and explicit nullable/omitted field
  semantics across every model.
- Keep secrets protected and diagnostics payload-free.

Verification: schema-derived completeness gates, boundary dates, unknown values,
golden/adversarial fixtures, fuzzing, and `scripts/release_0_45_gate.sh`.

Stop gate: `v0.45.0 implementation stop reached. Run pentest for this exact commit.`

### v0.46.0 - Complete DNS, Security, And Console Storage Models

Goal: finish complete validated response models outside the main Cloud resource
families.

Deliverables:

- Complete DNS zone/RRSet, certificate/SSH-key, and Console Storage Box,
  snapshot, subaccount, folder, and action fields.
- Preserve bounded zonefile and secret-bearing response handling through
  contiguous and incremental decoders.
- Prove every active pre-Robot operation has typed complete response coverage
  and no common-identity fallback.
- Update capability tables and examples to state practical end-to-end support.

Verification: schema completeness, secrets, large zonefile/storage fixtures,
fuzzing, live read-only smoke, and `scripts/release_0_46_gate.sh`.

Stop gate: `v0.46.0 implementation stop reached. Run pentest for this exact commit.`

### v0.47.0 - Robot Source Lock And Operation Matrix

Goal: establish a reproducible source of truth for Robot before public types.

Deliverables:

- Pin the official Robot reference with digest, retrieval metadata, normalized
  inventory, tested drift detection, and canonical reviewed evidence.
- Confirm the expected 105 documented headings, 89 active operations, and 16
  deprecated Robot Storage Box operations, then record methods, paths, forms,
  response shapes, errors, limits, auth lockout, and maintenance semantics.
- Exclude all deprecated Robot Storage Box operations and legacy aliases.
- Keep Robot modules inside `cloud-sdk-hetzner`.

Verification: source fixtures, provider-generic drift engine, fetch check, and
`scripts/release_0_47_gate.sh`.

Stop gate: `v0.47.0 implementation stop reached. Run pentest for this exact commit.`

### v0.48.0 - Robot Protocol And Credential Foundation

Goal: add Robot form, auth, error, quota, and transport policy.

Deliverables:

- Fixed official endpoint policy, typed service identity, bounded identifiers,
  atomic form encoding, and complete request metadata.
- Type-separated Basic credentials with protected storage, rotation, endpoint
  binding, and no bearer/Basic cross-use.
- Bounded Robot errors, invalid-input details, maintenance handling, quotas,
  and fixtures for blocking, async, local async, and testkit paths.

Verification: auth, form, error, quota, redaction, transport, source drift, and
`scripts/release_0_48_gate.sh`.

Stop gate: `v0.48.0 implementation stop reached. Run pentest for this exact commit.`

### v0.49.0 - Robot Servers And Cancellation

Goal: implement active server and cancellation operations with complete typed
responses and explicit destructive permits.

Deliverables:

- Server list, get, and update operations.
- Cancellation get, create, and revoke operations with explicit date, reason,
  and location-reservation validation.
- Canonical server-number identity; deprecated server-IP aliases stay absent.
- Complete capability, nullable subnet, status, cancellation-conflict, and
  empty-response models.

Verification: focused fixtures, conflicts, dates, nullable fields, source drift,
and `scripts/release_0_49_gate.sh`.

Stop gate: `v0.49.0 implementation stop reached. Run pentest for this exact commit.`

### v0.50.0 - Robot IP And Subnet Management

Goal: implement active IP, subnet, MAC, traffic-warning, and cancellation
operations with canonical network validation and complete models.

Deliverables:

- All 18 active IP/subnet operations: list/get, traffic-warning updates, MAC
  get/set/delete, and cancellation get/create/revoke.
- Canonical IPv4, IPv6, subnet, gateway, mask, broadcast, and MAC validation.
- Bounded repeated form fields, explicit cancellation dates, nullable MACs,
  traffic thresholds, lock state, and assignment metadata.

Verification: address/host-bit/form/conflict tests, source drift, and
`scripts/release_0_50_gate.sh`.

Stop gate: `v0.50.0 implementation stop reached. Run pentest for this exact commit.`

### v0.51.0 - Robot Reset, Failover, And Wake-On-LAN

Goal: implement reset, failover, and Wake-on-LAN operations with typed mutation
permits and source-locked capability validation.

Deliverables:

- All nine active reset, failover, and Wake-on-LAN operations.
- Source-locked reset types, canonical failover routes, and server-number-only
  Wake-on-LAN identity.
- Explicit reboot, reroute, delete, and wake intent with complete no-content
  response policy.

Verification: unsupported capability, route, empty-response, legacy alias
absence, source drift, and `scripts/release_0_51_gate.sh`.

Stop gate: `v0.51.0 implementation stop reached. Run pentest for this exact commit.`

### v0.52.0 - Robot Boot Configuration

Goal: implement rescue, Linux, VNC, and Windows boot configuration with
protected password/key output and complete form/response models.

Deliverables:

- All 15 active boot overview, rescue, Linux, VNC, and Windows
  get/activate/deactivate/last operations.
- Validated language, distribution, architecture, license, authorized-key, and
  host-key fields.
- Protected generated passwords and key material with atomic form encoding.
- Deprecated server-IP aliases and fields stay excluded or cross a documented
  compatibility boundary.

Verification: secret cleanup, atomic forms, compatibility fields, source drift,
and `scripts/release_0_52_gate.sh`.

Stop gate: `v0.52.0 implementation stop reached. Run pentest for this exact commit.`

### v0.53.0 - Robot Reverse DNS, Traffic, And SSH Keys

Goal: implement active reverse-DNS, traffic, and SSH-key operations with
canonical addresses, bounded ranges, and protected key material.

Deliverables:

- All 11 active reverse-DNS, traffic, and SSH-key operations.
- Bounded DNS names, canonical addresses, time ranges, intervals, repeated
  address inputs, and numeric response limits.
- Complete SSH fingerprint, algorithm, key, name, and redaction policy.
- Form encoding safely handles repeated traffic input and SSH key material.

Verification: DNS/date/numeric/key/streaming tests, source drift, and
`scripts/release_0_53_gate.sh`.

Stop gate: `v0.53.0 implementation stop reached. Run pentest for this exact commit.`

### v0.54.0 - Robot Firewalls And vSwitches

Goal: implement active firewall/template and vSwitch operations with bounded
rules, canonical CIDRs, explicit permits, and complete models.

Deliverables:

- All 15 active server firewall, firewall-template, and vSwitch operations.
- Bounded ordered rules, addresses, ports, protocols, VLANs, server lists, and
  cancellation dates.
- Explicit replacement, cancellation, attach, and detach intent.
- Complete in-progress/conflict and empty-response handling.

Verification: ordering, duplicate/conflict, repeated-form, source drift, and
`scripts/release_0_54_gate.sh`.

Stop gate: `v0.54.0 implementation stop reached. Run pentest for this exact commit.`

### v0.55.0 - Robot Ordering Catalogs And Read-Only Transactions

Goal: implement read-only product, price, currency, addon, and transaction
operations without admitting purchase execution.

Deliverables:

- All 12 read-only standard-server, Server Auction, addon, transaction,
  per-server addon, and account-currency operations.
- Exact decimal setup/hourly/monthly prices, locations, distributions, addons,
  product limits, and complete transaction states.
- Current-price warnings and typed plan inputs reusable by the next milestone.
- Deprecated product fields stay outside stable models unless a reviewed
  compatibility boundary requires them.

Verification includes catalog/price/transaction fixtures, source drift, and
`scripts/release_0_55_gate.sh`.

Stop gate: `v0.55.0 implementation stop reached. Run pentest for this exact commit.`

### v0.56.0 - Robot Billable Ordering Mutations

Goal: implement server, auction, and addon ordering through scoped cost permits
and stable plan-confirm fingerprints.

Deliverables:

- All three standard-server, Server Auction, and addon order-creation
  operations.
- Bind product, observed price, currency, quantity, account scope, and expiry
  input to the confirmed order fingerprint.
- Keep billable execution impossible in CI and ordinary live smoke.
- Reject stale prices, mismatched products, duplicate addons, missing permits,
  replay misuse, and partial buffers.

Verification: cost-policy tests, credential-free non-execution proof, source
drift, and `scripts/release_0_56_gate.sh`.

Stop gate: `v0.56.0 implementation stop reached. Run pentest for this exact commit.`

### v0.57.0 - Robot Client Integration And Live Evidence

Goal: combine all active Robot operations into typed blocking, async, and local
async workflows without weakening auth lockout or mutation controls.

Deliverables include complete mock scenarios and an opt-in read-only live
harness that never submits invalid credentials, destructive calls, or orders.

Verification: workflow/testkit tests, credential-free staging, explicit
operator live smoke, source drift, and `scripts/release_0_57_gate.sh`.

Stop gate: `v0.57.0 implementation stop reached. Run pentest for this exact commit.`

### v0.58.0 - Complete Hetzner SDK Hardening

Goal: complete pre-1.0 Cloud, DNS, Console Storage Box, and Robot hardening.

Deliverables:

- Every active operation has complete typed request/response/client coverage;
  every deprecated operation is explicitly excluded.
- Fuzz and adversarial coverage includes targets, headers, auth, JSON/form
  parsing, incremental decoding, permits, ordering, and cleanup.
- Public API, semver, platform, dependency, docs, examples, threat model,
  source locks, SBOMs, and migration notes are complete.
- Full pentest covers credential separation, lockout, SSRF, encoding ambiguity,
  replay, cost confirmation, redaction, bounds, and release evidence.

Verification: all checks, all source drift, matrices, fuzz smoke, SBOM, deny,
audit, and `scripts/release_0_58_gate.sh`.

Stop gate: `v0.58.0 implementation stop reached. Run pentest for this exact commit.`

### v0.59.0 - Release Provenance And Governance Review

Goal: make release trust claims precise and prepare independent 1.0 review.

Deliverables:

- Document authorized release signers, rotation/revocation, branch and release
  protection, and the limits of local Git trust.
- Evaluate current registry trusted-publishing/provenance options from official
  sources; admit only a reviewed flow with rollback and recovery procedures.
- Require two-person approval where maintainership permits it and record an
  independent pre-1.0 security review; explicitly disclose when organizational
  independence is unavailable rather than overstating evidence.
- Produce reproducible package/SBOM provenance evidence without reintroducing
  pentest-report signing or other operator burden that is not part of policy.

Verification: release-runbook tests, signer fixtures, provenance dry runs,
reproducible package comparison, and `scripts/release_0_59_gate.sh`.

Stop gate: `v0.59.0 implementation stop reached. Run pentest for this exact commit.`

### v0.60.0 - Controlled Mutation Evidence And 1.0 Release Candidate

Goal: validate carefully bounded real mutation workflows and freeze the final
1.0 candidate.

Deliverables:

- A separate manual-only disposable-project harness with explicit approval,
  spending ceilings, unique resource prefixes, cleanup ledger, and mandatory
  post-run empty-inventory verification.
- CI, ordinary live smoke, and read-only credentials cannot invoke mutations.
- Failed cleanup is a visible release blocker and preserves a redacted recovery
  ledger for manual remediation.
- Final core/provider API freeze, migration audit, documentation review, and
  release-candidate pentest.

Verification: dry-run/fake-provider mutation scenarios, manual controlled
evidence when an approved disposable account is available, all release gates,
and `scripts/release_0_60_gate.sh`.

Stop gate: `v0.60.0 implementation stop reached. Run pentest for this exact commit.`

### v1.0.0 - Full Hetzner Production SDK

Goal: first serious production-ready provider-neutral foundation and complete
Hetzner Cloud, DNS, Console Storage Box, and Robot provider.

Deliverables:

- Complete non-deprecated typed endpoint, request, response, client, pagination,
  action, quota, auth, and policy coverage for all claimed Hetzner services.
- Extensible neutral identities, canonical targets/headers, raw execution,
  typed operations, local async, bounded streaming, resource profiles, permits,
  diagnostics, and drift contracts are stable.
- Default graphs remain no_std and transport-free; optional blocking, async,
  local async, deterministic-root, and FIPS boundaries have current evidence.
- Deprecated Robot Storage Box and legacy aliases remain excluded.
- Provider-neutral contracts have passed the unpublished OVHcloud API v2 probe.
- Platform, MSRV, documentation, provenance, SBOM, audit, fuzz, controlled
  mutation, independent-review disclosure, and pentest evidence are current.

Verification: `scripts/checks.sh`, all provider source-lock checks, complete
matrices, SBOM, deny, audit, full `v1.0.0` gate, and green GitHub/CodeQL.

Stop gate: `v1.0.0 implementation stop reached. Run pentest for this exact commit.`
