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
v0.74.0+    pre-1.0 Robot Webservice support track
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
| Safe endpoint models still require callers to assemble transport requests, policy checks, and decoding manually. | Keep preparation/decoding from `v0.29.0-v0.31.0`; add typed operations in `v0.50.0`, the neutral client kernel in `v0.52.0`, and complete Hetzner clients in `v0.69.0-v0.73.0`. No nested client crate is introduced. |
| A custom HTTPS endpoint can receive real credentials when its value is attacker-controlled. | Keep explicit trust from `v0.27.0-v0.28.0`; add endpoint-policy algebra and exact IPv6/DNS/IDNA authority rules in `v0.34.0`, raw/auth separation in `v0.40.0-v0.42.0`, and official or acknowledged destinations in clients from `v0.52.0`. |
| Closed provider/API-family and HTTP-method enums force core edits for later providers. | Add extensible identities in `v0.32.0`; complete methods in `v0.33.0` with uppercase non-aliasing extensions, origin-form-only OPTIONS, and explicit CONNECT/TRACE/upgrade denial; add endpoint-policy algebra in `v0.34.0`. |
| Target validation is stronger in reqwest than core, so custom transports can accept ambiguous targets. | Separate and canonicalize path/query in core in `v0.35.0`; reject malformed percent triplets, encoded separators/controls, dot segments, doubled slashes, fragments, pre-encoded input, and ambiguous absent/empty/value/duplicate/space semantics. |
| The transport request lacks bounded provider headers and general response metadata. | Add ordered sensitive header and response-metadata contracts in `v0.36.0`, reserve framing/auth/proxy headers, bind Host/SNI to endpoint identity, enforce per-field/count/aggregate caps, then complete raw execution/auth separation in `v0.40.0-v0.42.0`. |
| `TransportResponse<'buffer>` does not prove that its body came from the admitted caller buffer. | Replace it with a sealed `ResponseWriter<'buffer>` in `v0.37.0`; only core commits status, bounded metadata, and initialized length into a cleanup-owning guard, so external/static bodies and escaping decoded borrows are unrepresentable. |
| Response sanitization is a non-verifiable transport promise and misses failure, rejection, decode, cancellation, and retained-state transfer paths. | Make core-owned clearing mandatory through one audited non-elidable primitive in `v0.38.0`; guard the complete response workspace; move retained sensitive metadata atomically into another cleanup-owning non-`Copy` type; clear failed, partial, and source storage; keep platform sanitizers additive and document lifecycle exclusions. |
| Fixed-buffer writers can leave partial output or secret tail bytes after failure. | Make every path/query/header/JSON/form writer preflighted and transactional in `v0.39.0`, with checked arithmetic, exact preflight/write equivalence, an immutable snapshot or generation/version-checked input across two passes, and domain-separated collision-resistant digest comparison under `v0.46.0` only when exact comparison is impossible; add aggregate caps, exact sensitive subslices, cleanup guards, and capacity profiles. |
| Transport errors do not state whether a mutation may have reached the provider. | Add `NotSent`, `PossiblySent`, and `ResponseStarted` delivery phases in `v0.40.0`, map unknown to `PossiblySent`, consume the phase in retry policy in `v0.46.0`, and require Robot order reconciliation in `v0.93.0`. |
| Response-head handling and adapter allocation lack explicit wire-level bounds. | In `v0.40.0`, define bounded informational responses, reject 101, enforce header/chunk limits before unbounded accumulation, stream-count actual bytes, distinguish caller-buffer from process-allocation guarantees, drop unadmitted headers, protect transient sensitive headers, and define HEAD/204/304/media/trailer/duplicate behavior. |
| Structured queries cannot safely represent already encoded provider pagination links. | Add a distinct `ValidatedProviderLink` in `v0.44.0` that preserves raw encoded path/query bytes, remains operation-pattern bound, and cannot enter the structured query builder. |
| Core pagination and rate-limit types encode Hetzner conventions. | Split pagination, quota, and retry/idempotency into `v0.44.0-v0.46.0`; distinguish delta, wall-clock, and HTTP-date resets, cap caller delay, resolve conflicting metadata, and retain cursor cleanup, authority, drift, replay, and quota-overflow tests. |
| `AsyncTransport` requires `Send`, and large payloads require one contiguous buffer. | Add `LocalAsyncTransport` in `v0.47.0`, streaming transport contracts in `v0.48.0`, and bounded incremental decoding in `v0.49.0`. |
| Fingerprints, plan confirmations, and idempotency keys lack collision and intent-identity rules. | Add versioned domain-separated canonical inputs and collision-resistant caller-supplied hashing or exact bounded comparison in `v0.46.0` and `v0.51.0`; fresh caller entropy creates each idempotency intent before binding it to a request fingerprint. |
| Retry and streaming contracts do not state whether bodies and dirty sinks are replayable, structurally bounded, or required to make progress. | Give `v0.46.0` one retry owner, explicit body replayability, hard attempt/cumulative-delay budgets, and distinct wall-clock/monotonic inputs; add per-operation byte/chunk/observation and empty-chunk budgets, actual chunk accounting, declared-length checks, source-change invalidation, and transactional/dirty state in `v0.48.0`. |
| Incremental decoding can still amplify token, field, number, exponent, or partial UTF-8 state. | Add total token/field and numeric/exponent limits plus chunk-spanning UTF-8 validation in `v0.49.0`. |
| Query/body compatibility and response selection still depend on runtime operation keys. | Add exhaustive typed associations for service, endpoint, auth, headers, media, statuses, success/error policy, caps, pagination/quota/retry, streaming, and permit class in `v0.50.0`; prove all 208 Hetzner operations in `v0.68.0`. |
| Cost, destructive intent, and retry metadata are inspectable but not enforced. | Add non-`Copy` stateful permits in `v0.51.0`; direct permits are non-`Clone`, while explicitly shareable handles retain one atomic consumption state, budget, and recovery generation; recover only after generation-checked `NotSent`, mark uncertain delivery spent/pending reconciliation, require exact idempotency/reconciliation for repetition, and reject rollback-extended expiry. |
| Credential refresh and Basic authorization can race, encode ambiguous credentials, or cross a same-authority service/tenant boundary. | Add generation/CAS-protected bearer refresh and provider/operation-owned `Required`, `Optional`, or `Forbidden` scope policy in `v0.41.0`; apply the same policy to Basic in `v0.42.0`, fail closed on omitted required fields, source-lock username charset/colon rules, and cap encoded authorization length. |
| Concurrent clients need explicit workspace ownership rather than hidden queues or aliased buffers. | Require caller-owned per-request workspace leases, bounded admission, no mutable alias across await points, and identical blocking/Send-async/local-async cleanup in `v0.52.0`. |
| Pagination/action workflows and diagnostics remain low-level, and action polling lacks a structural observation limit. | Add the client kernel in `v0.52.0`, bounded pager/action drivers and separated control/backoff/progress policy in `v0.53.0`, payload-free diagnostics and opt-in observation in `v0.54.0`, and dynamic testkit scenarios in `v0.55.0`. |
| Drift tooling is Hetzner-specific and historical review evidence depends mainly on fingerprints. | Add a provider-manifest drift engine, canonical reviewed diffs, alert ownership, and compatibility policy in `v0.56.0`. |
| A neutral freeze before Robot would miss Basic auth, repeated forms, lockout, unusual errors, quotas, maintenance, and empty bodies. | Source-lock a narrow credential-free Robot wire fixture in `v0.42.0`; keep the complete 89-operation inventory at `v0.74.0`; require both the Robot fixture and OVHcloud probe before freeze in `v0.62.0`. |
| Code review and synthetic probes cannot prove neutral contracts against complete primary-provider data shapes. | Source-lock and implement the unpublished OVHcloud v2 probe in `v0.57.0-v0.61.0`; before freeze in `v0.62.0`, require full-fidelity Hetzner Cloud, DNS-secret, security-secret, large Storage Box, typed-error, and no-content vertical slices through blocking, Send-async, and local-async execution. |
| Existing Hetzner responses expose common identity rather than complete fields, and timestamps are inconsistent. | Complete Cloud/DNS/security/Console models in `v0.63.0-v0.67.0`, exact bindings in `v0.68.0`, and complete clients in `v0.69.0-v0.73.0`. |
| Robot Webservice has different auth, encoding, and API shape than Cloud/DNS. | Assign source lock, protocol, every active family, ordering, clients, and live evidence to one-purpose milestones `v0.74.0-v0.95.0`. |
| Legacy Robot Storage Box operations are deprecated and overlap the Console API. | The `v0.74.0` matrix marks all 16 legacy operations excluded and no Robot Storage Box module is created. |
| Repeated invalid Robot credentials can temporarily block the caller's source IP. | Separate credentials and lockout policy in `v0.76.0`; classify authentication rejection as structurally non-retryable in `v0.77.0`; require newly supplied or explicitly reconfirmed credentials before `v0.94.0` clients can attempt again; `v0.95.0` live tests never intentionally submit invalid credentials. |
| Robot ordering mutations can create immediate infrastructure costs. | Keep catalogs/transactions read-only in `v0.91.0-v0.92.0`; `v0.93.0` requires cost permits, indeterminate-send reconciliation, and keeps purchases outside CI/live smoke. |
| FIPS configuration flags do not prove certificate, CRL, target, current time, module, or operational readiness. | Add real good/revoked/unknown/expired/wrong-issuer/incomplete-chain handshakes, fail closed without trustworthy current time, and produce bounded readiness evidence in `v0.97.0`; continue to disclaim application or organizational compliance. |
| Release controls do not provide organizationally independent review by themselves. | Add governance limits, signer policy, provenance review, and independent-review disclosure in `v0.98.0`; never claim independence when unavailable. |
| Destructive and billable behavior lacks controlled disposable-account evidence. | Add a manual-only mutation harness with spending ceilings, cleanup ledgers, and empty-inventory verification in `v0.99.0`; CI remains incapable of invoking it. |
| Future providers need proven patterns but are not part of the Hetzner 1.0 claim. | The unpublished OVHcloud probe lands in `v0.57.0-v0.61.0`; post-1.0 publishing starts with a finite source-locked Scaleway inventory in `v1.1.0`, then a finite DigitalOcean inventory in `v1.7.0`, with full OVHcloud considered after `v1.12.0`. |

## Post-1.0 Provider Sequence

The pre-1.0
[OVHcloud API v2](https://docs.ovhcloud.com/en/guides/manage-and-operate/api/apiv2/)
probe is architecture evidence, not a provider release. It stays in an excluded
package or fixture, is absent from the publish sequence, carries no support
claim, and must not become `cloud-sdk-ovhcloud` by accident. Its purpose is to
test contracts that differ materially from Hetzner: geographic API
authorities, OAuth2 service-account authentication, schema-version request
overrides, cursor pagination in headers, and asynchronous task or event
resources.

Published provider work starts only after the Hetzner `v1.0.0` release:

1. `cloud-sdk-scaleway` is the first published provider. Its source lock and
   release plan must select a finite product list and exact stable GA API
   versions from
   [Scaleway's APIs](https://www.scaleway.com/en/developers/api/), including
   global, regional, and zonal endpoints, `X-Auth-Token`, PATCH operations,
   product-specific schemas, and product-specific pagination/count conventions
   such as `per_page`, `page_size`, `X-Total-Count`, or body `total_count`.
   Only matrix rows in that immutable inventory form the supported completeness
   claim. Alpha, beta, unselected GA versions, and unselected products remain
   explicit exclusions until a later source-lock milestone adds them.
2. `cloud-sdk-digitalocean` is the second published provider. It must use
   DigitalOcean's
   [official OpenAPI source](https://github.com/digitalocean/openapi), select a
   finite product/operation inventory at an exact revision, and prove the
   conventional bearer-auth, `/v2`, same-authority link pagination, optional
   error `request_id`, rate-limit, and `Retry-After` path without weakening
   bounded decoding or retry policy. Spaces, metadata, OAuth applications, AI,
   and every unselected surface remain explicit exclusions.
3. `cloud-sdk-ovhcloud` follows later as a full provider. Its dedicated plan
   must separate API v2, required API v1 compatibility, OAuth2 and any retained
   legacy authentication, geographic endpoints, asynchronous tasks, ordering
   and other billable operations, and OpenStack-based services. The
   `v0.57.0-v0.61.0` probe
   does not pre-approve those product or security boundaries.

Every published provider keeps one primary crate, a separate official source
lock, threat model, API matrix, live-test policy, release plan, and pentest
stop gates. Scaleway owns workspace milestones `v1.1.0-v1.6.0`, DigitalOcean
owns `v1.7.0-v1.12.0`, and full OVHcloud publication receives a separate plan
only after the three-provider conformance milestone.

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

## Tier A - Neutral Wire And Isolation Kernel

### v0.32.0 - Extensible Provider And Service Identities

Goal: remove closed Hetzner-shaped core taxonomies.

Deliverables: bounded `ProviderId`/`ServiceId`, provider-owned markers, migration guidance, and proof that adding a provider requires no central enum edit.

Verification: identifier boundaries, compile-fail compatibility tests, public API review, and `scripts/release_0_32_gate.sh`.

Stop gate: `v0.32.0 implementation stop reached. Run pentest for this exact commit.`

### v0.33.0 - Complete HTTP Method Domain

Goal: support provider HTTP methods without repeated core changes.

Deliverables: GET, POST, PUT, DELETE, PATCH, HEAD, and origin-form-only OPTIONS; bounded extension methods must be uppercase canonical tokens and must not alias known methods; CONNECT and TRACE are denied; `OPTIONS *`, protocol upgrade, and tunnelling require a separate future transport contract; safety classification remains operation-owned rather than method-inferred.

Verification: casing/alias/token corpus, CONNECT/TRACE/OPTIONS-star/upgrade/tunnel rejection, operation migration, API review, and `scripts/release_0_33_gate.sh`.

Stop gate: `v0.33.0 implementation stop reached. Run pentest for this exact commit.`

### v0.34.0 - Endpoint Policy Algebra

Goal: make credential destinations provider-owned and explicit.

Deliverables: fixed, finite official-set, region-derived, and acknowledged-custom endpoint policies with non-static identities and exact scheme/authority/port/base-path checks. Authority rules canonicalize bracketed IPv6 literals, reject IPv6 zone identifiers, trailing DNS dots, userinfo, percent-encoded hosts, and Unicode host input, and accept only canonical lowercase ASCII DNS/A-label IDNA. Resolved-address and egress filtering remain optional transport/environment policy, never DNS logic in core.

Verification: SSRF, IPv6/zone/trailing-dot/userinfo/percent-host/IDNA normalization, redirect, credential binding, optional egress-hook isolation, compile-fail trust tests, and `scripts/release_0_34_gate.sh`.

Stop gate: `v0.34.0 implementation stop reached. Run pentest for this exact commit.`

### v0.35.0 - Canonical Path And Query Separation

Goal: give every transport the same request-target security meaning.

Deliverables: separate validated path/query components; distinct absent and present-empty query states; stable pair ordering with duplicate-key preservation; distinct missing and empty values; uppercase percent hex; `%20` for query spaces while `+` remains a separate provider-dialect type; rejection of pre-encoded fragments; exact final query bytes reused by signing and request fingerprints; core rejection of malformed percent triplets, encoded separators/controls, dot segments, doubled slashes, fragments, backslashes, non-ASCII, and ambiguous assembly.

Verification: absent/empty/missing/empty-value/duplicate/order/hex-case/space/pre-encoded vectors, signing/fingerprint byte identity, cross-adapter corpus, fuzzing, atomic assembly tests, and `scripts/release_0_35_gate.sh`.

Stop gate: `v0.35.0 implementation stop reached. Run pentest for this exact commit.`

### v0.36.0 - Bounded HTTP Header Model

Goal: represent complete provider requests without adapter-injected policy.

Deliverables: bounded ordered names/values, sensitivity markers, atomic encoding, typed common headers, and bounded response metadata; reserve `Host`, `Content-Length`, `Transfer-Encoding`, hop-by-hop, proxy, and `Authorization` ownership; bind Host/SNI to `EndpointIdentity`; enforce per-header, header-count, and aggregate-byte request/response limits.

Verification: smuggling, conflicting/identical duplicates, reserved ownership, Host/SNI mismatch, controls, redaction, every capacity boundary, adapter conformance, and `scripts/release_0_36_gate.sh`.

Stop gate: `v0.36.0 implementation stop reached. Run pentest for this exact commit.`

### v0.37.0 - Response Buffer Provenance

Goal: make it impossible for a transport to return bytes outside the admitted caller buffer.

Deliverables: one sealed `ResponseWriter<'buffer>` model with exclusive admitted-buffer access. Transports may write bytes and commit only status, bounded metadata, and initialized length; core validates the length, constructs the response slice internally, and returns a cleanup-owning `CheckedResponseGuard`. Owned decoding clears before return; borrowed decoding is closure-scoped and cannot outlive the guard.

Verification: malicious safe transports attempting static/external/oversized bodies or forged lengths, escaping-borrow compile-fail cases, owned/borrowed decode cleanup, blocking/async parity, and `scripts/release_0_37_gate.sh`.

Stop gate: `v0.37.0 implementation stop reached. Run pentest for this exact commit.`

### v0.38.0 - Bounded Response Cleanup Guarantees

Goal: define and enforce the strongest cleanup guarantee Rust and the platform can actually provide.

Deliverables: all core-owned clearing routes through one audited volatile/non-elidable primitive in the provider-neutral sanitization boundary, never scattered `fill(0)` implementations; additive platform sanitizers remain optional. The `CheckedResponseGuard` from `v0.37.0` owns the complete response workspace, including body storage, temporarily retained headers/metadata, cursor or provider-link bytes, request identifiers, and decoder scratch, and clears it on normal return, transport/policy/decode error, cancellation, and unwind where supported. Retained sensitive state moves atomically into a distinct cleanup-owning non-`Copy` type; successful transfer clears the source immediately, and failed or partial transfer clears both source and destination. Provider/operation metadata explicitly classifies whether request identifiers may be retained, must stay protected, or must be discarded. Borrowed decoding remains guard-scoped; a deliberately named explicit escape exists only if required; process abort, `mem::forget`/leaked guards, and unavoidable TLS, allocator, kernel, or device copies are explicit exclusions. Zero read-back is an integrity check, not proof that an additive sanitizer executed.

Verification: primitive-usage enforcement, body/header/cursor/request-ID/scratch dirty prefill, successful and failed/partial transfer cleanup, non-`Copy` retained-state compile failures, every request-ID sensitivity policy, no-op additive sanitizer, rejection, cancellation, panic-unwind, success/decode cleanup, documented non-guarantees, Miri where applicable, and `scripts/release_0_38_gate.sh`.

Stop gate: `v0.38.0 implementation stop reached. Run pentest for this exact commit.`

### v0.39.0 - Atomic Encoders And Capacity Profiles

Goal: eliminate partial writes, state drift, and secret tails.

Deliverables: checked arithmetic inside every `encoded_len`; exact preflight/write equivalence; an immutable snapshot or exact generation/version-checked input across both passes. If digest comparison is required because exact bounded comparison is impossible, it follows the collision-resistant, versioned, domain-separated rules from `v0.46.0` and never uses ordinary `Hash` or another non-cryptographic digest. Include transactional cursors, aggregate caps, exact sensitive subslices, preparation cleanup guards, request-side ownership/cleanup contracts, and embedded/default/large storage profiles with optional alloc convenience.

Verification: arithmetic boundaries, preflight/write differential tests, generation/version and snapshot mismatch, version/domain-separated collision-resistant digest vectors, non-cryptographic hash prohibition, mutated/nondeterministic two-pass input rejection, every undersized capacity, unchanged-storage assertions, secret-tail cleanup, allocation failure, fuzzing, and `scripts/release_0_39_gate.sh`.

Stop gate: `v0.39.0 implementation stop reached. Run pentest for this exact commit.`

### v0.40.0 - Raw Bounded HTTP Executor

Goal: make transports execute complete validated HTTP requests without provider policy.

Deliverables: method/target/header/body execution through `ResponseWriter`; conservative delivery phase (`NotSent`, `PossiblySent`, `ResponseStarted`) with unknown mapped to `PossiblySent`; a bounded informational-response count and final-response selection; rejection of 101; HEAD and forbidden-body rules for 204/304; wire-level header count/byte limits before unbounded accumulation; actual streamed-byte enforcement when `Content-Length` is absent, false, or oversized; separate success/error body caps and media policies; duplicate response-head rejection; explicit trailer policy; unknown response headers dropped unless admitted by the operation; cookies and transient sensitive headers redacted and cleared; documented upper bounds for unavoidable HTTP/TLS buffers and a precise distinction between caller-buffer and complete process-allocation guarantees; adapter-owned request body/header/staging cleanup on every ordinary exit; and removal of implicit auth, JSON Accept, redirects, proxies, content decoding, retries, and cross-origin forwarding.

Verification: raw blocking/async/FIPS conformance, send-phase fault injection, informational-count/101/HEAD/204/304/duplicate/trailer/media cases, hostile header accumulation, missing/false length, streamed overflow, unknown/cookie headers, documented allocation bounds, auth confusion, cleanup, dependency checks, and `scripts/release_0_40_gate.sh`.

Stop gate: `v0.40.0 implementation stop reached. Run pentest for this exact commit.`

### v0.41.0 - Bearer Authentication Policy

Goal: provide reusable bearer auth outside neutral execution.

Deliverables: HTTPS and exactly one owner of `Authorization`; immutable authentication scope binds provider, service, endpoint identity, and bounded/redacted provider-owned audience/account/tenant fields so credentials cannot cross same-authority boundaries. A provider/operation-owned policy marks each scope field `Required`, `Optional`, or `Forbidden`; omitted required fields, supplied forbidden fields, and every mismatch fail before header construction, so constructing an unscoped credential cannot bypass a required binding. Include mutable/guarded ingestion, atomic rotation, credential generations and compare-and-swap refresh so a slow refresh cannot overwrite a newer token, in-flight snapshots, executor-neutral snapshot/refresh handoff for expiring credentials, redaction, retired-token and adapter-header cleanup, and no acquisition, clock, executor, or secret store in core.

Verification: every provider/service/endpoint/audience/account/tenant required/optional/forbidden omission, presence, and mismatch case; unscoped-credential bypass attempts; rotation/refresh races; stale-generation CAS rejection; poisoned state; cleanup; blocking/async parity; and `scripts/release_0_41_gate.sh`.

Stop gate: `v0.41.0 implementation stop reached. Run pentest for this exact commit.`

### v0.42.0 - Basic And Signing Input Policies

Goal: support non-bearer providers without embedding provider signing in transports.

Deliverables: type-separated Basic credentials with the same provider/operation-owned `Required`, `Optional`, or `Forbidden` provider/service/endpoint/audience/account/tenant scope policy and fail-closed omission/presence/mismatch behavior; source-locked username charset/colon rules and an encoded authorization-header cap; bounded canonical signing inputs covering selected headers, request-body digest, nonce, and caller-provided time; caller-provided hashing/signing; adapter-auth cleanup; and no implicit clock, randomness, filesystem, or key store. Source-lock a narrow Robot wire fixture covering one read request, one non-executed repeated-form mutation fixture, errors, quotas, maintenance, lockout policy, and empty-body behavior without claiming operation coverage.

Verification: auth ownership and every scope mismatch, Basic colon/charset/encoded-length boundaries, canonical-header and body-digest vectors, nonce/time replay, redaction/cleanup, Robot credential-free conformance and no-publish gates, and `scripts/release_0_42_gate.sh`.

Stop gate: `v0.42.0 implementation stop reached. Run pentest for this exact commit.`

### v0.43.0 - Existing Hetzner Wire Migration

Goal: move every current Hetzner operation onto the neutral wire kernel.

Deliverables: all 208 active operations use new identities, endpoint policy, canonical target, bounded headers, raw execution, provenance, and cleanup with no compatibility fallback.

Verification: zero-fallback coverage gate, all operation fixtures, adapters, live read-only smoke, and `scripts/release_0_43_gate.sh`.

Stop gate: `v0.43.0 implementation stop reached. Run pentest for this exact commit.`

## Tier B - Control Plane, Execution, And Typed Workflows

### v0.44.0 - Pagination Strategy Family

Goal: model numbered, cursor, offset, marker, and link pagination separately.

Deliverables: hard budgets, bounded opaque state cleared on drop, and cursor-digest cycle checks where collisions fail closed rather than bypass correctness. `PaginationCursor` and `ValidatedProviderLink` are cleanup-owning non-`Copy` state types populated only through the atomic retained-state transfer contract from `v0.38.0`. A distinct `ValidatedProviderLink` preserves already encoded raw path/query bytes without decode/re-encode, cannot mix with structured query components, preserves duplicate ordering and percent encoding, binds to the original operation's admitted path pattern rather than only the provider base path, preserves the original method, and rejects authority, scheme, method, and operation-target changes plus all fragments and userinfo. Snapshot/drift policy and state updates remain explicit and transactional; diagnostics never contain cursors or links.

Verification: omission/drift/repetition/cycle/digest collision, structured/link type separation, atomic response-to-state transfer and source/destination cleanup, non-`Copy` state compile failures, raw-byte/duplicate/percent preservation, authority/path-pattern/method/operation/userinfo/fragment/downgrade rejection, budget/cleanup tests, DigitalOcean-style fixtures, fuzzing, and `scripts/release_0_44_gate.sh`.

Stop gate: `v0.44.0 implementation stop reached. Run pentest for this exact commit.`

### v0.45.0 - Provider Quota And Rate-Limit Strategies

Goal: move quota decoding out of transports.

Deliverables: provider-owned decoders and bounded multiple buckets; type-separated delta durations, absolute wall-clock reset timestamps, and HTTP-date `Retry-After`; explicit invalid/past timestamp handling; caller-imposed maximum delay; deterministic conflict policy between `Retry-After` and provider quota metadata; pure decision outputs only, with sleep and clock acquisition external; duplicate/partial-header policy and preserved informational extensions.

Verification: duration/timestamp/HTTP-date boundaries, invalid/past/rollback inputs, maximum-delay clamping/rejection, conflicting metadata, overflow, duplicate, incomplete, unknown-state, multi-bucket tests, and `scripts/release_0_45_gate.sh`.

Stop gate: `v0.45.0 implementation stop reached. Run pentest for this exact commit.`

### v0.46.0 - Retry And Idempotency Strategies

Goal: source-lock retry behavior per operation instead of inferring it from methods.

Deliverables: retry/idempotency tables, one explicit retry owner, request-body replayability capability, a non-bypassable nonzero `max_attempts`, maximum cumulative requested delay, mutation authorization, delivery-phase consumption with unknown treated as possibly sent, replay/mismatch rejection, and caller-owned delay/jitter inputs. Request fingerprints use a versioned domain-separated canonical format over provider, service, operation, endpoint identity, exact query bytes, selected headers, body, and applicable account/scope; comparison uses exact bounded canonical bytes or a collision-resistant caller-supplied digest, never `Hash` or another non-cryptographic digest; temporary canonical inputs are redacted and cleared. Each idempotency key begins as a fresh intent identifier from caller-provided entropy and is then bound to the fingerprint, so separate intentional identical operations cannot collide by construction. Wall-clock observations are typed separately from monotonic durations used for delay, timeout, and elapsed budgets; rollback cannot extend any budget. Non-replayable bodies and indeterminate mutations never retry automatically.

Verification: canonical version/domain/field separation, exact-byte and cryptographic digest vectors, weak/non-cryptographic digest rejection, fingerprint-input cleanup, fresh-intent uniqueness, identical-intent binding, policy completeness, zero/maximum attempts, cumulative-delay overflow/exhaustion, endless 429/transient responses, wall-clock rollback, monotonic-budget exhaustion, delivery phases, stale/reused keys, body mismatch, replayable/non-replayable bodies, competing retry owners, retry fan-out, and `scripts/release_0_46_gate.sh`.

Stop gate: `v0.46.0 implementation stop reached. Run pentest for this exact commit.`

### v0.47.0 - Local Async Contract

Goal: support `!Send` browser-WASM, embedded, and single-threaded executors.

Deliverables: `LocalAsyncTransport` beside cross-thread `AsyncTransport`, explicit cancellation semantics, and no executor ownership.

Verification: portable compile matrix, dropped-future cleanup, sequential/concurrent conformance, and `scripts/release_0_47_gate.sh`.

Stop gate: `v0.47.0 implementation stop reached. Run pentest for this exact commit.`

### v0.48.0 - Streaming Transport Contracts

Goal: support bounded upload/download/event I/O without changing buffered requests.

Deliverables: separate opt-in streaming boundaries; explicit body replayability; per-operation maximum upload bytes plus hard chunk and observation budgets; checked actual-byte accounting across chunks; declared-length mismatch detection; overflow rejection before accepting the next chunk; rejection or explicitly bounded tolerance of consecutive empty chunks; a progress requirement for finite upload/download streams; explicit unknown-length and executor-owned framing policy; replayability invalidation when a source changes between attempts; transactional versus dirty partial upload/download sink states; partial-I/O and cancellation semantics; deterministic backpressure; no automatic retry for non-replayable mutations; and no implicit buffering/runtime. Long-lived event streams may remain open-ended only with caller-owned cancellation and an observation policy that still bounds zero-progress behavior.

Verification: exact/under/over/unknown declared lengths, every byte/chunk/observation cap boundary, endless empty chunks, alternating empty/data chunks, consecutive-empty tolerance exhaustion, finite-stream zero progress, event cancellation/observation exhaustion, source mutation between attempts, executor framing, chunk boundaries, short I/O, cancellation, transactional/dirty sinks, replay attempts, backpressure, cleanup, platform checks, and `scripts/release_0_48_gate.sh`.

Stop gate: `v0.48.0 implementation stop reached. Run pentest for this exact commit.`

### v0.49.0 - Incremental Provider Decoding

Goal: decode large lists, metrics, zonefiles, logs, and streams without one large JSON tree.

Deliverables: bounded visitor/state APIs preserving duplicate, nesting, aggregate, string, secret, early-stop, and cleanup protections; explicit total token and field limits; bounded numeric token and exponent lengths; validated partial UTF-8 state across chunk boundaries.

Verification: differential fixtures, token/field/numeric/exponent exhaustion, every partial UTF-8 and general chunk boundary, truncation, amplification, early stop, fuzzing, and `scripts/release_0_49_gate.sh`.

Stop gate: `v0.49.0 implementation stop reached. Run pentest for this exact commit.`

### v0.50.0 - Compile-Time Operation Associations

Goal: make endpoint/query/body/response mismatches unrepresentable.

Deliverables: sealed operation traits, `QueryFor<O>`, `BodyFor<O>`, and typed `Prepared<O>` with exhaustive associated service/endpoint policy, auth class and authentication-scope requirements, request headers/media, admitted statuses, success/error body and media policies, response caps, pagination/quota/retry strategy, streaming mode, response/error types, and required permit class; plus forward-compatible enum rules.

Verification: compile-fail mismatch for every association, source-derived binding coverage, semver review, and `scripts/release_0_50_gate.sh`.

Stop gate: `v0.50.0 implementation stop reached. Run pentest for this exact commit.`

### v0.51.0 - Mutation, Destructive, And Cost Permits

Goal: enforce safety intent at execution.

Deliverables: scope/fingerprint/expiry-bound permits are non-`Copy`, and direct permits are non-`Clone`. If an explicitly shareable handle is needed, every clone references the same atomic consumption state, remaining budget, and recovery generation; cloning never creates independent authority, dropping one handle never restores authority consumed by another, and `NotSent` recovery is generation-checked before the shared state becomes usable again. `PossiblySent` or `ResponseStarted` consumes the shared authority into spent/pending-reconciliation state; repetition after uncertain delivery requires the exact idempotency identity and operation-specific reconciliation. Plan-confirm fingerprints use a distinct versioned domain over provider, service, operation, endpoint identity, exact query bytes, selected headers, body, account/tenant, currency, observed price, spending ceiling, and permit scope; use exact bounded comparison or caller-supplied collision-resistant hashing, never non-cryptographic `Hash`; clear/redact canonical inputs. Include no-op rejection, replay policy, and caller-owned time/price/entropy. Backward wall-clock observations cannot extend permit validity.

Verification: plan-confirm version/domain/field separation, weak digest rejection, canonical-input cleanup, compile-fail direct copy/clone misuse, shared-handle clone/drop interleavings, shared-budget exhaustion, stale-generation `NotSent` recovery rejection, concurrent double-spend, every delivery-state transition, recover/reconcile/repeat rules, exact idempotency mismatch, wall-clock rollback, stale/mismatch/replay/budget/price/no-op/redaction tests, Loom where supported, and `scripts/release_0_51_gate.sh`.

Stop gate: `v0.51.0 implementation stop reached. Run pentest for this exact commit.`

### v0.52.0 - Provider-Generic Client Kernel

Goal: make secure typed execution reusable without publishing extra client crates.

Deliverables: one policy path for blocking, Send-async, and local-async preparation, authentication, send-once execution, checked decoding, and cleanup; caller-owned workspace leases per in-flight request; bounded admission with no hidden unbounded queue; no aliased mutable storage across await points; identical cleanup semantics in every execution mode.

Verification: fake-provider conformance, endpoint/auth mismatch, lease exhaustion/reuse, alias compile-fail cases, cancellation, bounded concurrency, cross-mode cleanup, and `scripts/release_0_52_gate.sh`.

Stop gate: `v0.52.0 implementation stop reached. Run pentest for this exact commit.`

### v0.53.0 - Pager And Action Workflow Drivers

Goal: provide ergonomic workflows without clocks, sleep, or executors in core.

Deliverables: pure next-request/delay drivers, unconditional observation limits, provider progress policies, separate `PollControl` and bounded backoff, redacted policy errors, typed wall-clock observations only for provider timestamps/expiry, and monotonic durations for delay/timeout/elapsed budgets so clock rollback cannot extend execution.

Verification: busy-loop, progress reset/regression, terminal bounds, cancellation, wall-clock rollback, monotonic timeout/budget exhaustion, page/action scenarios, and `scripts/release_0_53_gate.sh`.

Stop gate: `v0.53.0 implementation stop reached. Run pentest for this exact commit.`

### v0.54.0 - Structured Payload-Free Diagnostics

Goal: make failures actionable without leaking provider or customer data.

Deliverables: bounded provider/service/operation/status/request-ID/retry/error categories with no credentials, cursors, targets, bodies, messages, or generic payload-bearing `Debug` path; request-ID observation follows the provider/operation sensitivity and retention policy established in `v0.38.0`; an opt-in observer receives structured lifecycle events, while core never logs automatically.

Verification: redaction corpus, maximum lengths, observer disabled/enabled behavior, reentrancy/error isolation, downstream error types, snapshots, and `scripts/release_0_54_gate.sh`.

Stop gate: `v0.54.0 implementation stop reached. Run pentest for this exact commit.`

### v0.55.0 - Dynamic Testkit

Goal: test realistic multi-request behavior deterministically.

Deliverables: bounded recording, dynamic responders, fault injection including endless-empty and alternating-empty/data stream sources, pagination/action scripts, cancellation, partial I/O, and provider fixture builders.

Verification: exhaustion, mismatch non-consumption, recording caps, injected failures, no_std checks, and `scripts/release_0_55_gate.sh`.

Stop gate: `v0.55.0 implementation stop reached. Run pentest for this exact commit.`

### v0.56.0 - Provider-Generic Drift Engine

Goal: source-lock future providers with auditable historical evidence.

Deliverables: manifests/plugins for sources, auth, endpoints, operations, schemas, pagination, headers, retry/idempotency, cost policy, canonical diffs, and alert ownership.

Verification: malicious documents, redirect denial, digest rotation, reproducibility, plugin fixtures, and `scripts/release_0_56_gate.sh`.

Stop gate: `v0.56.0 implementation stop reached. Run pentest for this exact commit.`

## Tier C - OVHcloud API v2 Probe And Neutral Freeze

### v0.57.0 - OVHcloud Probe Source Lock

Goal: select and immutably record the unpublished probe surface.

Deliverables: official documents, console schema fingerprints, 5-10 read-only candidates, authorities, token endpoints, schema versions, task/event evidence, threat note, and no-publish gate.

Verification: drift fixtures, operation inventory, source reproducibility, and `scripts/release_0_57_gate.sh`.

Stop gate: `v0.57.0 implementation stop reached. Run pentest for this exact commit.`

### v0.58.0 - OVHcloud Authority And OAuth Conformance

Goal: challenge endpoint/auth contracts with geographic API/token pairs and expiring OAuth2.

Deliverables: source-locked authority/alias policy, no credentialed redirects, region-bound token authority, `expires_in` handoff, atomic rotation, and least-privilege guidance.

Verification: pair mismatch, alias, redirect, expiry, rotation, redaction, and `scripts/release_0_58_gate.sh`.

Stop gate: `v0.58.0 implementation stop reached. Run pentest for this exact commit.`

### v0.59.0 - OVHcloud Cursor And Header Conformance

Goal: prove opaque pagination and schema-validation headers need no core exception.

Deliverables: bounded/redacted cursor headers, terminal-page semantics, validation-only `X-Schemas-Version`, reviewed schema-major evidence, and raw metadata decoding.

Verification: cursor cycles/controls/oversize, missing-next, duplicate headers, schema drift, and `scripts/release_0_59_gate.sh`.

Stop gate: `v0.59.0 implementation stop reached. Run pentest for this exact commit.`

### v0.60.0 - OVHcloud Task And Event Conformance

Goal: prove asynchronous resource models against real source-locked read routes.

Deliverables: actual `/task` or `/event` operation coverage where available, bounded task/progress/error/event models, and generic examples kept as fixtures rather than endpoint claims.

Verification: state/progress/timestamp/link/message adversarial fixtures and `scripts/release_0_60_gate.sh`.

Stop gate: `v0.60.0 implementation stop reached. Run pentest for this exact commit.`

### v0.61.0 - OVHcloud End-To-End Probe

Goal: execute the complete unpublished probe through unchanged neutral contracts.

Deliverables: 5-10 read-only operations across blocking/async/local-async/testkit, credential-free fixtures, optional ignored least-privilege live smoke, and zero provider exceptions in core.

Verification: conformance matrix, no-publish/dependency gates, optional live smoke, and `scripts/release_0_61_gate.sh`.

Stop gate: `v0.61.0 implementation stop reached. Run pentest for this exact commit.`

### v0.62.0 - Neutral API Freeze

Goal: freeze provider-neutral contracts only after materially different probes and complete primary-provider vertical slices.

Deliverables: OVHcloud probe-driven changes complete and the `v0.42.0` Robot Basic/form/error/quota/maintenance/empty-body fixture passes unchanged. Implement full-fidelity slices in the real `cloud-sdk-hetzner` provider for: a paginated Cloud read plus one mutation/action; DNS zonefile or TSIG secret output; certificate or SSH-key secret output; a large Storage Box response through incremental decoding; a typed provider error; and an empty/no-content response. Every slice uses complete source fields for its selected operation, typed associations, `CheckedResponseGuard`, secret ownership and cleanup, and executes through blocking, Send-async, local-async, and testkit paths. The remaining 208-operation model/binding/client completion stays in `v0.63.0-v0.73.0`. Finish public API/semver review, compile-fail contract suite, migration guide, threat-model delta, and rejected-abstraction record.

Verification: public API diff; OVHcloud and Robot conformance fixtures; vertical-slice source-field, association, guard, secret, large-response, typed-error, no-content, and cross-executor matrices; downstream fixtures; no_std/platform matrix; and `scripts/release_0_62_gate.sh`.

Stop gate: `v0.62.0 implementation stop reached. Run pentest for this exact commit.`

## Tier D - Complete Hetzner Models And Clients

### v0.63.0 - Complete Cloud Resource Models

Goal: complete compute, network, IP, volume, pricing, and catalog fields.

Deliverables: source-complete validated models, nullability, unknown-value policy, and no common-identity fallback.

Verification: schema gates, golden/adversarial fixtures, fuzzing, and `scripts/release_0_63_gate.sh`.

Stop gate: `v0.63.0 implementation stop reached. Run pentest for this exact commit.`

### v0.64.0 - Cloud Actions, Metrics, And Special Models

Goal: complete action, metrics, composite, decimal, and timestamp responses.

Deliverables: calendar-valid UTC RFC3339, exact decimals, bounded metrics, nullable results, and complete action/error fields.

Verification: boundary dates/decimals, metrics limits, action states, fuzzing, and `scripts/release_0_64_gate.sh`.

Stop gate: `v0.64.0 implementation stop reached. Run pentest for this exact commit.`

### v0.65.0 - Complete DNS Models

Goal: complete zones, RRSets, zonefiles, actions, and secret-bearing DNS results.

Deliverables: exact request/response fields, bounded zonefiles, TSIG policy, unknown record handling, and incremental decode coverage.

Verification: schema/secret/zonefile/adversarial/live-read tests and `scripts/release_0_65_gate.sh`.

Stop gate: `v0.65.0 implementation stop reached. Run pentest for this exact commit.`

### v0.66.0 - Complete Security Models

Goal: complete certificate and SSH-key typed coverage.

Deliverables: all fields/actions, protected private/key material, redacted diagnostics, and rotation-compatible responses.

Verification: schema, PEM/key, secret cleanup, unknown-state, fuzzing, and `scripts/release_0_66_gate.sh`.

Stop gate: `v0.66.0 implementation stop reached. Run pentest for this exact commit.`

### v0.67.0 - Complete Console Storage Box Models

Goal: complete boxes, types, snapshots, folders, subaccounts, and actions.

Deliverables: all source fields, secret outputs, bounded large responses, nullability, and contiguous/incremental decoding.

Verification: schema/secret/large-response/live-read tests and `scripts/release_0_67_gate.sh`.

Stop gate: `v0.67.0 implementation stop reached. Run pentest for this exact commit.`

### v0.68.0 - Complete Hetzner Typed Binding Gate

Goal: prove exact associations for all 208 active pre-Robot operations.

Deliverables: zero missing request/query/body/response/error/policy bindings and explicit exclusion of all deprecated operations.

Verification: generated/source-derived matrix gate, compile-fail mismatches, and `scripts/release_0_68_gate.sh`.

Stop gate: `v0.68.0 implementation stop reached. Run pentest for this exact commit.`

### v0.69.0 - Hetzner Client Foundation

Goal: stabilize official/custom construction and storage lifecycle.

Deliverables: Cloud/DNS/Console endpoint and credential separation, explicit custom trust, caller storage profiles, concurrency, and no implicit retry/runtime policy.

Verification: endpoint/auth confusion, cleanup/cancellation/rotation, examples, and `scripts/release_0_69_gate.sh`.

Stop gate: `v0.69.0 implementation stop reached. Run pentest for this exact commit.`

### v0.70.0 - Cloud Client Methods

Goal: expose every claimed Cloud operation through typed workflows.

Deliverables: complete read/mutation/action/metrics methods, permits, pagination, quotas, decoding, and blocking/async/local-async parity.

Verification: operation-client coverage, scenarios, live read-only smoke, and `scripts/release_0_70_gate.sh`.

Stop gate: `v0.70.0 implementation stop reached. Run pentest for this exact commit.`

### v0.71.0 - DNS Client Methods

Goal: expose every claimed DNS operation through typed workflows.

Deliverables: zone/RRSet CRUD, actions, zonefiles, TSIG, permits, pagination, and cleanup across all execution modes.

Verification: client coverage, secret/cancellation scenarios, live read-only smoke, and `scripts/release_0_71_gate.sh`.

Stop gate: `v0.71.0 implementation stop reached. Run pentest for this exact commit.`

### v0.72.0 - Security Client Methods

Goal: complete certificate and SSH-key workflows.

Deliverables: typed CRUD/actions, key/private-material lifecycle, rotation, permits, and cleanup across all execution modes.

Verification: client coverage, secret/error/cancellation scenarios, live read-only smoke, and `scripts/release_0_72_gate.sh`.

Stop gate: `v0.72.0 implementation stop reached. Run pentest for this exact commit.`

### v0.73.0 - Console Storage Box Client Methods

Goal: expose every claimed Console Storage Box operation through typed workflows.

Deliverables: boxes/types/snapshots/folders/subaccounts/actions, permits, pagination, secret handling, and streaming where required.

Verification: client coverage, large/secret scenarios, live read-only smoke, and `scripts/release_0_73_gate.sh`.

Stop gate: `v0.73.0 implementation stop reached. Run pentest for this exact commit.`

## Tier E - Hetzner Robot

### v0.74.0 - Robot Source Lock And Matrix

Goal: establish the reproducible Robot source of truth.

Deliverables: active/deprecated inventory, auth/lockout/forms/errors/limits/maintenance semantics, and explicit exclusion of all 16 deprecated Storage Box operations.

Verification: source fixtures, drift fetch, inventory gate, and `scripts/release_0_74_gate.sh`.

Stop gate: `v0.74.0 implementation stop reached. Run pentest for this exact commit.`

### v0.75.0 - Robot Form Codec

Goal: implement bounded atomic form encoding.

Deliverables: repeated fields, percent rules, exact preflight, transactional state, aggregate caps, and secret-tail cleanup.

Verification: every capacity, repeats, controls, fuzzing, and `scripts/release_0_75_gate.sh`.

Stop gate: `v0.75.0 implementation stop reached. Run pentest for this exact commit.`

### v0.76.0 - Robot Credentials And Lockout Policy

Goal: type-separate Robot Basic credentials and prevent unsafe authentication testing.

Deliverables: protected ingestion/rotation/cleanup, endpoint and Robot-service scope binding, and a lockout-aware credential-attempt generation. Authentication rejection closes that generation for execution; only newly supplied credentials or an explicit caller reconfirmation creates a new generation. No automatic policy, pager, action, or client path can reopen or repeat a rejected generation, and live evidence never intentionally uses invalid credentials.

Verification: auth cross-use, redaction, rotation, rejection-state transition, stale/rejected generation reuse, explicit reconfirmation, concurrent attempts sharing one generation, lockout gate, and `scripts/release_0_76_gate.sh`.

Stop gate: `v0.76.0 implementation stop reached. Run pentest for this exact commit.`

### v0.77.0 - Robot Error And Quota Protocol

Goal: type Robot errors, maintenance, invalid input, and quota behavior.

Deliverables: bounded envelopes, payload-free diagnostics, provider quota decoder, and structurally distinct authentication-rejection, quota, maintenance, invalid-input, and transient transport classifications. Authentication rejection is never automatically retryable and cannot be converted into a generic transient category by unknown-code fallback; fixtures bind this rule to the source-locked Robot protocol.

Verification: malformed/unknown/oversized/duplicate/quota tests, auth-versus-quota/maintenance/transient separation, unknown-code fail-closed behavior, authentication retry denial, and `scripts/release_0_77_gate.sh`.

Stop gate: `v0.77.0 implementation stop reached. Run pentest for this exact commit.`

### v0.78.0 - Robot Servers

Goal: complete server list/get/update operations and models.

Deliverables: canonical server identity, capabilities, statuses, nullable subnets, explicit update intent, and no legacy IP aliases.

Verification: source coverage, field/conflict/boundary tests, and `scripts/release_0_78_gate.sh`.

Stop gate: `v0.78.0 implementation stop reached. Run pentest for this exact commit.`

### v0.79.0 - Robot Cancellations

Goal: complete cancellation get/create/revoke workflows.

Deliverables: dates, reasons, location reservation, conflicts, destructive permits, and exact response policy.

Verification: date/conflict/permit/source tests and `scripts/release_0_79_gate.sh`.

Stop gate: `v0.79.0 implementation stop reached. Run pentest for this exact commit.`

### v0.80.0 - Robot IP Management

Goal: complete active IP, MAC, warning, and cancellation behavior.

Deliverables: canonical addresses/MACs, traffic thresholds, lock/assignment state, repeated forms, and permits.

Verification: address/form/conflict/source tests and `scripts/release_0_80_gate.sh`.

Stop gate: `v0.80.0 implementation stop reached. Run pentest for this exact commit.`

### v0.81.0 - Robot Subnet Management

Goal: complete subnet and subnet-cancellation behavior.

Deliverables: canonical network/gateway/mask/broadcast semantics, assignment metadata, forms, conflicts, and permits.

Verification: host-bit/family/boundary/source tests and `scripts/release_0_81_gate.sh`.

Stop gate: `v0.81.0 implementation stop reached. Run pentest for this exact commit.`

### v0.82.0 - Robot Reset

Goal: complete reset capabilities and mutations.

Deliverables: source-locked reset types, typed intent, permits, action responses, and unsupported-capability rejection.

Verification: capability/permit/source tests and `scripts/release_0_82_gate.sh`.

Stop gate: `v0.82.0 implementation stop reached. Run pentest for this exact commit.`

### v0.83.0 - Robot Failover

Goal: complete failover route management.

Deliverables: canonical routes, reroute/delete intent, permits, conflicts, and no-content policy.

Verification: route/family/permit/source tests and `scripts/release_0_83_gate.sh`.

Stop gate: `v0.83.0 implementation stop reached. Run pentest for this exact commit.`

### v0.84.0 - Robot Wake-On-LAN

Goal: complete server-number-only Wake-on-LAN behavior.

Deliverables: capability checks, explicit wake intent, mutation permit, response policy, and legacy alias absence.

Verification: identity/capability/permit/source tests and `scripts/release_0_84_gate.sh`.

Stop gate: `v0.84.0 implementation stop reached. Run pentest for this exact commit.`

### v0.85.0 - Robot Boot Configuration

Goal: complete rescue, Linux, VNC, and Windows boot operations.

Deliverables: overview/get/activate/deactivate/last operations, validated configuration fields, generated passwords/keys, and protected cleanup.

Verification: secret/form/compatibility/source tests and `scripts/release_0_85_gate.sh`.

Stop gate: `v0.85.0 implementation stop reached. Run pentest for this exact commit.`

### v0.86.0 - Robot Reverse DNS

Goal: complete reverse-DNS operations.

Deliverables: canonical addresses, bounded DNS names, forms, conflicts, permits, and exact models.

Verification: DNS/address/source tests and `scripts/release_0_86_gate.sh`.

Stop gate: `v0.86.0 implementation stop reached. Run pentest for this exact commit.`

### v0.87.0 - Robot Traffic

Goal: complete traffic queries and large response handling.

Deliverables: bounded ranges/intervals/repeated addresses/numeric limits and incremental decoding.

Verification: date/range/repeat/stream/source tests and `scripts/release_0_87_gate.sh`.

Stop gate: `v0.87.0 implementation stop reached. Run pentest for this exact commit.`

### v0.88.0 - Robot SSH Keys

Goal: complete SSH-key operations and protected material handling.

Deliverables: algorithms, fingerprints, names, keys, atomic forms, redaction, and cleanup.

Verification: key/form/secret/source tests and `scripts/release_0_88_gate.sh`.

Stop gate: `v0.88.0 implementation stop reached. Run pentest for this exact commit.`

### v0.89.0 - Robot Firewalls And Templates

Goal: complete firewall and template operations.

Deliverables: bounded ordered rules, CIDRs, ports, protocols, replacement intent, conflicts, and permits.

Verification: ordering/duplicate/rule/form/source tests and `scripts/release_0_89_gate.sh`.

Stop gate: `v0.89.0 implementation stop reached. Run pentest for this exact commit.`

### v0.90.0 - Robot vSwitches

Goal: complete vSwitch membership and cancellation operations.

Deliverables: VLANs, server lists, attach/detach/cancel intent, conflicts, repeated forms, and permits.

Verification: VLAN/membership/form/source tests and `scripts/release_0_90_gate.sh`.

Stop gate: `v0.90.0 implementation stop reached. Run pentest for this exact commit.`

### v0.91.0 - Robot Ordering Catalogs

Goal: complete read-only products, auctions, prices, currencies, and addons.

Deliverables: exact decimals, locations, distributions, limits, current-price warnings, and typed plan inputs without purchase execution.

Verification: catalog/price/decimal/source tests and `scripts/release_0_91_gate.sh`.

Stop gate: `v0.91.0 implementation stop reached. Run pentest for this exact commit.`

### v0.92.0 - Robot Transactions

Goal: complete transaction and per-server addon read models.

Deliverables: all states, identifiers, prices, timestamps, nullability, pagination, and read-only workflows.

Verification: state/decimal/date/source tests and `scripts/release_0_92_gate.sh`.

Stop gate: `v0.92.0 implementation stop reached. Run pentest for this exact commit.`

### v0.93.0 - Robot Ordering Mutations

Goal: gate every billable server, auction, and addon order.

Deliverables: cost permits and plan-confirm fingerprints bound to product, observed price, currency, quantity, account, expiry input, and replay policy; delivery-phase-aware indeterminate-send handling; mandatory transaction reconciliation before any repeat after a possibly sent order; CI cannot purchase.

Verification: stale-price/mismatch/replay/budget, not-sent/possibly-sent/response-started faults, reconciliation-before-repeat, non-execution/source tests, and `scripts/release_0_93_gate.sh`.

Stop gate: `v0.93.0 implementation stop reached. Run pentest for this exact commit.`

### v0.94.0 - Robot Client Integration

Goal: expose every active Robot operation through typed clients.

Deliverables: blocking, Send-async, local-async, pager/action workflows, endpoint/auth separation, permits, cleanup, and complete mock scenarios. All layers delegate retry ownership to the `v0.46.0` policy, propagate Robot authentication rejection without repetition, and require a newly supplied or explicitly reconfirmed credential-attempt generation before another call.

Verification: client coverage, authentication rejection through direct/pager/action/workflow paths with exactly one wire attempt, rejected-generation reuse denial, explicit reconfirmation, lockout/cancellation/concurrency scenarios, and `scripts/release_0_94_gate.sh`.

Stop gate: `v0.94.0 implementation stop reached. Run pentest for this exact commit.`

### v0.95.0 - Robot Live Evidence

Goal: validate least-privilege read-only Robot behavior without lockout or cost risk.

Deliverables: credential-free staging, ignored operator harness, private token files, no invalid credentials, mutations, orders, or destructive calls.

Verification: staging/runner tests, explicit operator smoke, source drift, and `scripts/release_0_95_gate.sh`.

Stop gate: `v0.95.0 implementation stop reached. Run pentest for this exact commit.`

## Tier F - Whole-Platform Qualification

### v0.96.0 - Complete Adversarial And Fuzz Qualification

Goal: close the full wire/auth/decoder/permit/cleanup/Robot adversarial matrix.

Deliverables: zero unclassified claimed operations, maintained corpora, cross-adapter differential tests, and current fuzz evidence.

Verification: all corpora/fuzz smoke/matrices/SBOM/deny/audit and `scripts/release_0_96_gate.sh`.

Stop gate: `v0.96.0 implementation stop reached. Run pentest for this exact commit.`

### v0.97.0 - Platform, MSRV, And FIPS Qualification

Goal: produce current evidence for every supported target, compiler, feature graph, and FIPS boundary.

Deliverables: good/revoked/unknown/expired-CRL/wrong-issuer/incomplete-chain handshakes; readiness evidence with module/target/root/CRL/config/policy fingerprints; unsupported-target rejection; authenticated CRL metadata; fail-closed construction/validation when trustworthy current time is unavailable; current NIST review; no compliance overclaim.

Verification: full platform/MSRV matrix, packaged FIPS tests including missing/untrusted time, native-build review, dependency freshness, and `scripts/release_0_97_gate.sh`.

Stop gate: `v0.97.0 implementation stop reached. Run pentest for this exact commit.`

### v0.98.0 - Provenance And Governance Review

Goal: make release trust and independent-review claims exact.

Deliverables: signer rotation/revocation, branch/release protection, trusted-publishing evaluation, reproducible packages/SBOMs, recovery procedures, and explicit independence disclosure without report-signing burden.

Verification: runbook/signer/provenance/reproducibility tests and `scripts/release_0_98_gate.sh`.

Stop gate: `v0.98.0 implementation stop reached. Run pentest for this exact commit.`

### v0.99.0 - Controlled Mutation Release Candidate

Goal: finish real mutation evidence and freeze the exact 1.0 candidate.

Deliverables: manual-only disposable project, approval, spending ceilings, unique prefixes, cleanup ledger, empty-inventory verification, final API/docs/migration review, and no CI mutation capability.

Verification: fake-provider dry runs, approved manual evidence when available, every release gate, and `scripts/release_0_99_gate.sh`.

Stop gate: `v0.99.0 implementation stop reached. Run pentest for this exact commit.`

### v1.0.0 - Full Hetzner Production SDK

Goal: release the qualified candidate without adding features.

Deliverables: complete non-deprecated Hetzner Cloud, DNS, security, Console Storage Box, and Robot typed SDK; frozen neutral contracts; current docs, provenance, platform, SBOM, audit, fuzz, mutation, independent-review disclosure, and pentest evidence.

Verification: exact `v0.99.0` candidate ancestry, `scripts/checks.sh`, all source locks/matrices, `scripts/release_1_0_gate.sh`, and green GitHub/CodeQL.

Stop gate: `v1.0.0 implementation stop reached. Run pentest for this exact commit.`

## Post-1.0 Provider Blueprint

Provider crates start their own pre-1.0 package histories even when the
workspace facade is stable. Every row requires its own source lock,
threat-model delta, release notes, release gate, and exact-commit pentest stop.

| Workspace milestone | Provider deliverable |
| --- | --- |
| `v1.1.0` | Select and source-lock a finite list of Scaleway products and exact stable GA API versions; create its threat model, operation matrix, and explicit product/version exclusions. No later milestone may silently widen this inventory. |
| `v1.2.0` | Publish the initial `cloud-sdk-scaleway` preview with regional/zonal endpoints and `X-Auth-Token`. |
| `v1.3.0` | Complete read-only rows classified as compute/catalog in the finite `v1.1.0` inventory. |
| `v1.4.0` | Complete read-only rows classified as network/storage in the finite `v1.1.0` inventory. |
| `v1.5.0` | Complete the explicitly admitted mutation/action rows, pagination variants, and cost permits from the finite `v1.1.0` inventory. |
| `v1.6.0` | Reach zero unclassified rows and stabilize/pentest only the selected Scaleway inventory; alpha, beta, and all excluded products remain outside the claim. |
| `v1.7.0` | Select and source-lock a finite DigitalOcean product/operation inventory from exact OpenAPI revisions; create its threat model, matrix, and explicit adjacent-service exclusions. No later milestone may silently widen this inventory. |
| `v1.8.0` | Publish the initial `cloud-sdk-digitalocean` preview with bearer auth, `/v2`, and same-authority link pagination. |
| `v1.9.0` | Complete read-only rows in the finite `v1.7.0` inventory plus its rate-limit and `Retry-After` policy. |
| `v1.10.0` | Complete explicitly admitted DigitalOcean mutation/action rows and idempotency behavior from the finite inventory. |
| `v1.11.0` | Reach zero unclassified rows and stabilize/pentest only the selected DigitalOcean inventory; Spaces, metadata, OAuth apps, AI, and all other exclusions remain separately scoped. |
| `v1.12.0` | Run three-provider conformance against frozen neutral contracts before planning full OVHcloud publication. |

Full `cloud-sdk-ovhcloud` publication receives a separate version plan after `v1.12.0`; the unpublished pre-1.0 probe never becomes its package history. The one-primary-crate-per-provider rule remains mandatory.
