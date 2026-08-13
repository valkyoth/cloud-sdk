# cloud-sdk Release Plan To 1.0

Status: planning document.

This plan is intentionally granular. `cloud-sdk` manages infrastructure APIs,
so each milestone must be small enough to review, test, and stop cleanly before
tagging. Every tag receives an incremental pentest; crates.io publication is
batched at five-minor checkpoints.

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
- the security review and pentest evidence required by its release class;
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

## Assurance And Release Classes

Tags retain the ordinary `vX.Y.Z` names. Through v0.50.0 every version received
an individual pentest and crates.io publication. v0.51.0 through v0.55.0 used
one cumulative transition train. Beginning after the published v0.55.0
baseline, pre-1.0 releases use two publication classes while every tag is
individually assessed:

- a scheduled cumulative checkpoint is `v0.N.0` when `N` is divisible by five;
- every other `v0.N.0` and `v0.N.P` is an intermediate signed tag unless an
  exceptional security or publication trigger applies.

Every intermediate version requires the complete repository and
version-specific gates, current release notes and SBOMs, an explicit
`Security-Review: PASS`, an incremental pentest against the immediately
preceding tag, permanent `security/pentest/vX.Y.Z.md` evidence, and green
GitHub CI and CodeQL on the exact final commit. Its release notes name the next
checkpoint for crates.io publication, and it selects no crate for publication.

Each scheduled checkpoint receives the same incremental pentest against its
immediately preceding tag. The release gate additionally proves that every
intervening minor and patch tag contains permanent passing pentest evidence.
Its report records `Assessment: INCREMENTAL`, the preceding tag as `Baseline`,
and itself as `Range-End`. An exceptional crates.io checkpoint follows the same
chain and becomes the new public baseline so no unassessed package change is
published.

All material credential, trust, transport, parsing, destructive-operation,
unsafe/native, release-control, incident, security-fix, and production-claim
changes are covered by the mandatory per-tag pentest. The maintainer may
request an additional targeted or full assessment at any milestone. v1.0.0
always requires an independent full-project pentest and public release gate.

A version is not tag-ready until its applicable assurance class passes:

- `scripts/checks.sh` passes;
- the version-specific release gate, including required live Hetzner and IANA
  drift checks, passes;
- `cargo deny check` passes;
- `cargo audit` passes;
- `scripts/generate-sbom.sh` succeeds;
- `scripts/check_sbom_freshness.sh` proves all committed SBOMs match their
  current dependency graphs;
- release notes exist at `release-notes/RELEASE_NOTES_X.Y.Z.md`;
- a pentest report exists at `security/pentest/vX.Y.Z.md` for every release;
- every applicable pentest report names the exact full 40-character
  `Reviewed-Commit:`, has `Status: PASS`, has non-blank `Tester:` and `Scope:`
  fields, and has a `Date: YYYY-MM-DD` field;
- `sbom/cloud-sdk.spdx.json` exists and is non-empty;
- `sbom/reqwest-feature-unification.spdx.json` exists and is non-empty when
  the standalone downstream fixture is present;
- `sbom/fuzz.spdx.json` exists and is non-empty for the excluded fuzz tooling
  graph;
- for assessed releases, `scripts/validate-release-readiness.sh vX.Y.Z` proves
  that the reviewed implementation commit is an ancestor of the final release
  commit;
- shared readiness rejects modified tracked files and all untracked files;
- the version-specific gate snapshots the clean validated `HEAD`, requires it
  to remain unchanged, and reruns readiness after every check;
- GitHub CI and CodeQL default setup are green on the final release commit;
- tagging has been explicitly requested.

`Reviewed-Commit:` records the implementation commit that was reviewed. If
retest, CodeQL, or another release gate causes release-relevant changes, rerun
the review and update `Reviewed-Commit:` to the latest reviewed commit before
tagging.

Normal implementation CI validates release metadata before the permanent
report exists. The versioned release gate requires the report and enforces the
selected publication stage.
The reviewed implementation commit must be an ancestor of the final release
commit. The permanent report and final release metadata may be committed
together after a green pentest. GitHub validates that complete release commit.
The normal publisher still requires a verifiable signed, annotated `vX.Y.Z`
tag to point at `HEAD` and has no dirty-tree, skipped-check, untagged, or
no-verification bypass flags.

When an intermediate version's implementation criteria are done, stop and say:

```text
vX.Y.Z implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.N.0.
```

For a public checkpoint or production candidate, stop and request the
applicable exact-commit pentest before tagging and publication.

### Pentest Handoff Flow

Use this loop for every release assessment:

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

For an intermediate tagged milestone, complete implementation and release
metadata, run all local and version-specific gates, record the security review
and exact checkpoint deferral, commit, confirm GitHub CI and CodeQL, then create
the normal signed `vX.Y.Z` tag only when explicitly requested. Do not create a
pentest report or invoke crates.io publication.

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
| FIPS configuration flags do not prove certificate, target, module, operating environment, or operational readiness. | Retire the experimental AWS-LC feature after `v0.70.0`, exclude FIPS from 1.0, and permit a later Brynja integration only after the exact module, certificate, operating environment, build, runtime, and review conditions in `docs/FIPS_DEFERMENT.md` are satisfied. |
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

- Rust stable `1.97.1` pinned.
- Rust `1.90.0` through `1.97.1` compatibility policy.
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
v0.12.0 pentest stop passed for this exact commit. Commit only the
permanent report, wait for CI, then run release readiness before tagging.
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

Historical status: implemented for v0.23.0 and retired from the active
workspace after v0.70.0. This section records earlier-tag behavior and is not
part of the 1.0 scope; future work is deferred to Brynja.

Goal: add a fail-closed blocking rustls FIPS-mode transport without weakening
or silently changing the standard blocking transport, while avoiding a
validation claim the current AWS-LC-FIPS 3.x dependency cannot support.

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
v0.30.0 pentest stop passed for this exact commit. Commit the permanent
report, run the clean release gate, and wait for GitHub before tagging.
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
v0.31.0 pentest stop passed for this exact commit before tagging.
Permanent pentest and release evidence remain bound to that commit.
```

## Tier A - Neutral Wire And Isolation Kernel

### v0.32.0 - Extensible Provider And Service Identities

Status: tagged 2026-07-26.

Goal: remove closed Hetzner-shaped core taxonomies.

Deliverables:

- Allocation-free `ProviderId` and `ServiceId` values bounded to 63 bytes.
- Locale-independent lowercase ASCII/digit syntax with canonical single
  internal hyphens; invalid boundaries, repeated separators, Unicode, and
  unbounded values fail closed.
- Open provider-owned `ProviderMarker` and `ServiceMarker` contracts; every
  service names its owning provider and no central registry or catch-all
  service remains.
- Marker-derived `ProviderService` construction plus direct construction from
  already validated IDs.
- Hetzner-owned Cloud, DNS, security, and Console Storage markers and complete
  migration of preparation, checked decoding, examples, and testkit evidence.
- Explicit migration guidance and public API/security review.

Verification:

- Boundary tests for every identifier rejection class and exact maximum sizes.
- External-crate proof that adding a provider requires no core source edit.
- Compile-fail tests for forged IDs and incomplete service ownership.
- Wrong-provider/service decoder rejection and unchanged endpoint binding.
- Default, no_std, all-feature, docs, package, clippy, and MSRV checks.
- `scripts/check_provider_identities.sh`.
- `scripts/release_0_32_gate.sh` after pentest evidence is committed.

Exit criteria: all deliverables and verification above are complete, the public
API review records accepted and rejected designs, documentation and release
metadata are synchronized, and the implementation commit is ready for
independent security review.

Stop gate:

```text
v0.32.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.33.0 - Complete HTTP Method Domain

Status: implementation complete; pentest and final retest passed.

Goal: support provider HTTP methods without repeated core changes.

Deliverables:

- Allocation-free GET, POST, PUT, DELETE, PATCH, HEAD, and origin-form-only
  OPTIONS constants.
- Static provider extension methods bounded to 32 bytes, restricted to
  uppercase canonical HTTP token bytes, and prohibited from aliasing a known
  method.
- Explicit denial of CONNECT and TRACE.
- Continued rejection of `OPTIONS *`; protocol upgrade and tunnelling remain
  outside the transport contract and require a separately reviewed future
  design.
- Blocking and async reqwest mapping for every admitted method.
- Hetzner operation impact, request semantics, and retry eligibility declared
  by provider-owned operation classes rather than inferred from HTTP methods.
- Migration guidance and public API/security review.

Verification:

- Casing, known-alias, token-byte, empty, and exact length-boundary corpora.
- CONNECT, TRACE, `OPTIONS *`, forged method, upgrade, and tunnel rejection
  evidence.
- Exact blocking and async wire tests for PATCH, HEAD, OPTIONS, and PURGE.
- Testkit exact extension-method matching.
- Complete Hetzner prepared-operation migration with preserved metadata tests.
- Default, no_std, all-feature, docs, package, clippy, and MSRV checks.
- `scripts/check_http_method_domain.sh`.
- `scripts/release_0_33_gate.sh` after pentest evidence is committed.

Exit criteria: every admitted method is canonical and bounded, no denied
method or non-origin request target is constructible through safe public APIs,
provider safety metadata has no method-derived helper, documentation and
release metadata are synchronized, and the implementation commit is ready for
independent security review.

Stop gate:

```text
v0.33.0 implementation stop reached. Run pentest for this exact commit.
```

### v0.34.0 - Endpoint Policy Algebra

Status: implementation complete; pentest and final retest passed.

Goal: make credential destinations provider-owned and explicit.

Deliverables: fixed, finite official-set, region-derived, and acknowledged-custom endpoint policies with non-static identities and exact scheme/authority/port/base-path checks. Authority rules canonicalize bracketed IPv6 literals, reject IPv6 zone identifiers, trailing DNS dots, userinfo, percent-encoded hosts, and Unicode host input, and accept only canonical lowercase ASCII DNS/A-label IDNA. Resolved-address and egress filtering remain optional transport/environment policy, never DNS logic in core.

Verification: SSRF, IPv6/zone/trailing-dot/userinfo/percent-host/IDNA normalization, redirect, credential binding, optional egress-hook isolation, compile-fail trust tests, and `scripts/release_0_34_gate.sh`.

Exit criteria: every credential destination is admitted by a provider-owned
policy or explicit trusted-operator acknowledgement; authority and base-path
normalization cannot change policy identity; complete endpoint input is bounded
before allocation; blocking, async, testkit, docs, and migration evidence agree;
and the implementation commit has passed independent security review.

Stop gate: `v0.34.0 implementation stop reached. Run pentest for this exact commit.`

### v0.35.0 - Canonical Path And Query Separation

Status: tagged.

Goal: give every transport the same request-target security meaning.

Deliverables: separate validated path/query components; distinct absent and present-empty query states; stable pair ordering with duplicate-key preservation; distinct missing and empty values; uppercase percent hex; `%20` for query spaces while `+` remains a separate provider-dialect type; rejection of pre-encoded fragments; exact final query bytes reused by signing and request fingerprints; core rejection of malformed percent triplets, encoded separators/controls, dot segments, doubled slashes, fragments, backslashes, non-ASCII, and ambiguous assembly.

Verification: absent/empty/missing/empty-value/duplicate/order/hex-case/space/pre-encoded vectors, signing/fingerprint byte identity, cross-adapter corpus, fuzzing, atomic assembly tests, and `scripts/release_0_35_gate.sh`.

Exit criteria: core is the only request-target validation authority; paths and
queries remain separately inspectable after assembly; exact query bytes,
presence, pair order, duplicates, and value presence survive core, provider,
reqwest, and testkit boundaries; form encoding cannot enter the canonical
query type; malformed or ambiguous targets fail before transport I/O; all
affected crates, docs, fuzzing, and migration evidence agree; and the
implementation commit has passed independent security review.

Stop gate: `v0.35.0 implementation stop reached. Run pentest for this exact commit.`

### v0.36.0 - Bounded HTTP Header Model

Status: implementation complete; pentest and final retest passed.

Goal: represent complete provider requests without adapter-injected policy.

Deliverables: bounded ordered names/values, sensitivity markers, atomic encoding, typed common headers, and bounded response metadata; reserve `Host`, `Content-Length`, `Transfer-Encoding`, hop-by-hop, proxy, and `Authorization` ownership; bind Host/SNI to `EndpointIdentity`; enforce per-header, header-count, and aggregate-byte request/response limits.

Verification: smuggling, conflicting/identical duplicates, reserved ownership, Host/SNI mismatch, controls, redaction, every capacity boundary, adapter conformance, and `scripts/release_0_36_gate.sh`.

Stop gate: `v0.36.0 implementation stop reached. Run pentest for this exact commit.`

### v0.37.0 - Response Buffer Provenance

Status: implementation complete; pentest and final retest passed.

Goal: make it impossible for a transport to return bytes outside the admitted caller buffer.

Deliverables: a cleanup-owning `ResponseBuffer` lends one sealed `ResponseWriter<'buffer>` with exclusive admitted-prefix access. Transports may write bytes and commit only status, bounded metadata, and initialized length; core validates the length, constructs response views internally, and returns a cleanup-owning `CheckedResponseGuard`. The split owner/writer model preserves sequential non-`Sync` asynchronous transports without lending the sanitizer across suspension. Owned decoding clears before return; borrowed decoding is closure-scoped and cannot outlive the guard.

Verification: malicious safe transports attempting static/external/oversized bodies or forged lengths, escaping-borrow compile-fail cases, owned/borrowed decode cleanup, blocking/async parity, and `scripts/release_0_37_gate.sh`.

Exit criteria: safe transports cannot construct or substitute response body
provenance; all blocking, async, reqwest, testkit, prepared execution, and
Hetzner decoding paths use the sealed writer; uncommitted, oversized, duplicate,
and post-commit states fail closed; policy rejection and owned decoding clear
the complete caller storage through its supplied sanitizer; borrowed response
bytes cannot escape their guard; docs and fuzzing match the new contract; and
the implementation commit has passed independent security review.

Stop gate: `v0.37.0 implementation stop reached. Run pentest for this exact commit.`

### v0.38.0 - Bounded Response Cleanup Guarantees

Goal: define and enforce the strongest cleanup guarantee Rust and the platform can actually provide.

Deliverables: all core-owned clearing routes through one audited volatile/non-elidable primitive in the provider-neutral sanitization boundary, never scattered `fill(0)` implementations; additive platform sanitizers remain optional. The `CheckedResponseGuard` from `v0.37.0` owns the complete response workspace, including caller-owned body and header destinations, cursor or provider-link bytes, request identifiers, and decoder scratch, and clears it on normal return, transport/policy/decode error, cancellation, and unwind where supported. Sensitive bytes never reside in movable fixed arrays: response state carries only non-secret locators into stable caller storage. Retained sensitive state copies directly into a distinct caller-owned cleanup destination wrapped by a non-`Copy` type; successful transfer clears the source immediately, and failed or partial transfer clears both source and destination. Strict JSON object keys use capacity-wiping storage, including unknown extension names. Raw content-type access distinguishes absence from malformed syntax or UTF-8, and core rejects every present parse failure under required, optional, and forbidden policy independent of adapter behavior. Provider/operation metadata explicitly classifies whether request identifiers may be retained, must stay protected, or must be discarded. Borrowed decoding remains guard-scoped; a deliberately named explicit escape exists only if required; process abort, `mem::forget`/leaked guards, and unavoidable TLS, allocator, kernel, or device copies are explicit exclusions. Zero read-back is an integrity check, not proof that an additive sanitizer executed.

Verification: primitive-usage enforcement, stable-address response-header and retained-destination tests, absence of movable secret arrays, protected JSON-key allocation cleanup, malformed ASCII and invalid-UTF-8 content-type rejection under optional and forbidden policies, body/header/cursor/request-ID/scratch dirty prefill, successful and failed/partial transfer cleanup, non-`Copy` retained-state compile failures, every request-ID sensitivity policy, no-op additive sanitizer, rejection, cancellation, panic-unwind, success/decode cleanup, documented non-guarantees, Miri where applicable, and `scripts/release_0_38_gate.sh`.

Stop gate: `v0.38.0 implementation stop reached. Run pentest for this exact commit.`

### v0.39.0 - Atomic Encoders And Capacity Profiles

Status: tagged and published.

Goal: eliminate partial writes, state drift, and secret tails.

Deliverables: checked arithmetic inside every `encoded_len`; exact preflight/write equivalence; an immutable snapshot or exact generation/version-checked input across both passes. If digest comparison is required because exact bounded comparison is impossible, it follows the collision-resistant, versioned, domain-separated rules from `v0.46.0` and never uses ordinary `Hash` or another non-cryptographic digest. Include transactional cursors, aggregate caps, exact sensitive subslices, preparation cleanup guards, request-side ownership/cleanup contracts, and embedded/default/large storage profiles with optional alloc convenience.

Verification: arithmetic boundaries, preflight/write differential tests, generation/version and snapshot mismatch, exact bounded comparison evidence, non-cryptographic hash prohibition, mutated/nondeterministic multi-pass input rejection, every undersized capacity, unchanged-storage assertions, secret-tail cleanup, allocation failure, fuzzing, and `scripts/release_0_39_gate.sh`. Digest vectors are required only if exact comparison is impossible; the v0.39 implementation uses exact comparison and no digest.

Stop gate: `v0.39.0 implementation stop reached. Run pentest for this exact commit.`

### v0.40.0 - Raw Bounded HTTP Executor

Status: tagged and published.

Goal: make transports execute complete validated HTTP requests without provider policy.

Deliverables: method/target/header/body execution through mandatory transactional `ResponseAttempt` guards over `ResponseWriter`, with direct writer mutation and commitment private; complete uncommitted body/header cleanup on error, timeout, unwind, cancellation, and reuse; conservative delivery phase (`NotSent`, `PossiblySent`, `ResponseStarted`) with unknown mapped to `PossiblySent` and response-start defined as any observed informational or final head; a bounded informational-response count and final-response selection; rejection of 101; HEAD and forbidden-body rules for 204/304; wire-level header count, encoded-field, and parser-buffer limits before unbounded accumulation; actual streamed-byte enforcement when `Content-Length` is absent, false, or oversized; separate success/error body caps and media policies; duplicate response-head rejection; explicit trailer policy; unknown response headers dropped unless admitted by the operation; cookies and transient sensitive headers redacted and cleared; a first-party reqwest 8 MiB pre-allocation request-copy ceiling explicitly distinguished from the provider-neutral raw traits; documented upper bounds for unavoidable HTTP/TLS buffers and a precise distinction between caller-buffer and complete process-allocation guarantees; adapter-owned request body/header/staging cleanup on every ordinary exit; runtime disabling of implicit auth, JSON Accept, redirects, proxies, content decoding, retries, and cross-origin forwarding; and accurate documentation of proxy/redirect-capable modules compiled transitively by reqwest.

Verification: raw blocking/async/FIPS conformance, send-phase fault injection, immediate informational-limit cancellation, attempt-only compile-fail enforcement, failed-writer reuse plus drop/unwind/cancellation cleanup, exact and limit-plus-one request bodies through both public adapters, post-parse validator/body-budget fuzzing, actual Hyper HTTP/1 wire/state-machine fuzzing through canonical CRLF seeds and below/exact/plus-one encoded-head boundaries, informational-count/101/HEAD/204/304/duplicate/trailer/media cases, hostile header accumulation, missing/false length, streamed overflow, unknown/cookie headers, documented allocation and compiled-module bounds, auth confusion, cleanup, dependency checks, and `scripts/release_0_40_gate.sh`.

Stop gate: `v0.40.0 implementation stop reached. Run pentest for this exact commit.`

### v0.41.0 - Bearer Authentication Policy

Status: tagged.

Goal: provide reusable bearer auth outside neutral execution.

Deliverables: HTTPS and exactly one owner of `Authorization`; immutable bearer authentication scope requires provider, service, and endpoint identity and permits only bounded/redacted provider-owned audience/account/tenant additions so credentials cannot cross same-authority boundaries. The authenticated adapter requires exact `Required` provider/service/endpoint policy and provider/operation-owned `Required`, `Optional`, or `Forbidden` rules for extended fields; configured endpoint mismatch, downgraded base rules, omitted required fields, supplied forbidden fields, and every mismatch fail before header construction. Include mutable/guarded ingestion, strict bearer grammar, atomic rotation, credential generations and store-lineage-bound compare-and-swap refresh so slow or misrouted refresh cannot overwrite another lifecycle or a newer token, in-flight snapshots, executor-neutral snapshot/refresh handoff for expiring credentials, redaction, retired-token and adapter-header cleanup, and no acquisition, clock, executor, or secret store in core.

Verification: every provider/service/endpoint/audience/account/tenant required/optional/forbidden omission, presence, mismatch, and base-rule downgrade case; configured/credential endpoint mismatch; strict bearer padding grammar; rotation/refresh races; foreign-store and stale-generation CAS rejection; poisoned state; cleanup; blocking/async parity; and `scripts/release_0_41_gate.sh`.

Stop gate: `v0.41.0 implementation stop reached. Run pentest for this exact commit.`

### v0.42.0 - Basic And Signing Input Policies

Status: tagged.

Goal: support non-bearer providers without embedding provider signing in transports.

Deliverables: type-separated Basic credentials with the same provider/operation-owned `Required`, `Optional`, or `Forbidden` provider/service/endpoint/audience/account/tenant scope policy and fail-closed omission/presence/mismatch behavior; source-locked username charset/colon rules and an encoded authorization-header cap; v2 bounded canonical signing inputs covering complete provider/service/normalized-endpoint/scope/key/digest-algorithm/signature-algorithm context, tagged canonical DNS/IPv4/IPv6 host identity, selected headers, internally coupled exact-body hashing, nonce, and caller-provided time; equivalent IPv6 spellings produce one canonical representation; retained exact requests and validated cleanup-owning signature output; caller-provided hashing/signing; adapter-auth cleanup; and no implicit clock, randomness, filesystem, or key store. Source-lock a narrow Robot wire fixture covering one read request, one non-executed repeated-form mutation fixture, errors, quotas, maintenance, lockout policy, and empty-body behavior without claiming operation coverage.

Verification: auth ownership and every scope mismatch, Basic colon/charset/encoded-length boundaries, exact v2 context/header/body vectors, canonical IPv6 equivalence, independent changes to every context, request, and freshness field, nonce/time replay, hasher/signer failure and invalid-length rejection, retained request identity, unwind/redaction/cleanup, Robot credential-free conformance and no-publish gates, and `scripts/release_0_42_gate.sh`.

Stop gate: `v0.42.0 implementation stop reached. Run pentest for this exact commit.`

### v0.43.0 - Existing Hetzner Wire Migration

Status: tagged.

Goal: move every current Hetzner operation onto the neutral wire kernel.

Deliverables: all 208 active operations use exact provider/service identities, official endpoint policy, canonical target, bounded request headers, provider-owned authentication scope, status-class raw response policy, authenticated raw execution, conservative delivery phase, response provenance, and cleanup with no compatibility fallback. Cloud, DNS, security, and Storage operations bind their own service marker rather than inheriting a base-URL-wide Cloud identity. Bearer and Basic adapters inject authorization only inside the shared raw Hyper engine after complete scope validation.

Verification: a machine-checked 208-operation zero-fallback migration gate, exact service/auth/raw-policy fixtures for every Hetzner API surface, blocking and async bearer/Basic adapter parity, delivery-phase and cleanup regressions, testkit policy capture, live read-only smoke through prepared authenticated requests, no_std/platform/MSRV/dependency/SBOM checks, and `scripts/release_0_43_gate.sh`.

Stop gate: `v0.43.0 implementation stop reached. Run pentest for this exact commit.`

## Tier B - Control Plane, Execution, And Typed Workflows

### v0.44.0 - Pagination Strategy Family

Status: release candidate; pentest and final retest passed.

Goal: model numbered, cursor, offset, marker, and link pagination separately.

Deliverables: hard budgets, exact bounded snapshot identities, bounded opaque state cleared on drop, and cursor-digest cycle checks where collisions fail closed rather than bypass correctness. `PaginationCursor` and `ValidatedProviderLink` are cleanup-owning non-`Copy` state types populated only through the atomic retained-state transfer contract from `v0.38.0`. A distinct `ValidatedProviderLink` preserves already encoded raw path/query bytes without decode/re-encode, cannot mix with structured query components, preserves duplicate ordering and percent encoding, binds to the original operation's admitted path pattern rather than only the provider base path, and couples endpoint verification to blocking or asynchronous authenticated dispatch through the same `BoundTransport` object without a redirectable callback. Validation and transport failures use one flattened result with redacted transport diagnostics. It preserves the original method and rejects unbound transports, authority, scheme, method, and operation-target changes plus all fragments and userinfo. Snapshot/drift policy and state updates remain explicit and transactional; diagnostics never contain snapshots, cursors, links, or transport error payloads.

Verification: omission/drift/repetition/cycle/digest collision, exact snapshot byte boundaries and transactional commit, structured/link type separation, atomic response-to-state transfer and source/destination cleanup, non-`Copy` state compile failures, raw-byte/duplicate/percent preservation, blocking/async same-object endpoint-check and authenticated-dispatch tests, rejected-executor non-invocation, flattened transport failure classification, redacted `Debug`/`Display`, `core::error::Error`, authority/path-pattern/method/operation/userinfo/fragment/downgrade rejection, budget/cleanup tests, DigitalOcean-style fixtures, dedicated offset/opaque/history/provider-link fuzzing, deterministic positive-seed preflight, and `scripts/release_0_44_gate.sh`.

Stop gate: `v0.44.0 implementation stop reached. Run pentest for this exact commit.`

### v0.45.0 - Provider Quota And Rate-Limit Strategies

Status: release candidate; pentest and final retest passed.

Goal: move quota decoding out of transports.

Deliverables: provider-owned decoders and bounded multiple buckets; type-separated delta durations, absolute wall-clock reset timestamps, and HTTP-date `Retry-After`; explicit invalid/past timestamp handling; caller-imposed maximum delay; deterministic conflict policy between `Retry-After` and provider quota metadata; pure decision outputs only, with sleep and clock acquisition external; duplicate/partial-header policy and preserved informational extensions.

Verification: duration/timestamp/HTTP-date boundaries, invalid/past/rollback inputs, maximum-delay clamping/rejection, conflicting metadata, overflow, duplicate, incomplete, unknown-state, multi-bucket tests, and `scripts/release_0_45_gate.sh`.

Stop gate: `v0.45.0 implementation stop reached. Run pentest for this exact commit.`

### v0.46.0 - Retry And Idempotency Strategies

Status: tagged and published.

Goal: source-lock retry behavior per operation instead of inferring it from methods.

Deliverables: retry/idempotency tables, one explicit retry owner, request-body replayability capability, nonzero attempt accounting, maximum cumulative requested delay, mutation authorization, delivery-phase consumption with unknown treated as possibly sent, replay/mismatch rejection, and caller-owned delay/jitter inputs. Request fingerprints use a versioned domain-separated canonical format over provider, service, operation, admitted endpoint identity, exact query bytes, selected headers, body, and applicable account/scope; comparison uses exact bounded canonical bytes or a collision-resistant caller-supplied digest, never `Hash` or another non-cryptographic digest; temporary canonical inputs are redacted and cleared. Private-field retry subjects bind prepared policy to fingerprint identity, while the controller separately compares every retry-critical prepared policy before state mutation. Each idempotency key begins as fresh caller entropy retained in one borrowed cleanup location and is then bound to the fingerprint, so separate intentional identical operations cannot collide by construction. Wall-clock observations are typed separately from monotonic durations used for delay, timeout, and elapsed budgets; decisions and permit execution advance one controller-owned monotonic state, so rollback cannot extend any budget. The hard elapsed deadline includes requested delay and is rechecked by a one-use permit after sleep. A permit exclusively borrows controller state through direct blocking or executor-neutral async execution, preventing simultaneously outstanding safe-code fan-out without claiming to constrain code that intentionally bypasses the controller. Non-replayable bodies and indeterminate mutations never retry automatically.

Verification: canonical version/domain/field separation, endpoint admission, unrelated-policy rejection, identical-wire full-policy laundering, exact-byte and cryptographic digest vectors, weak/non-cryptographic digest rejection, fingerprint/digest cleanup, borrowed-intent cleanup, fixed-time byte checks, fresh-intent uniqueness, identical-intent binding, policy completeness, zero/maximum attempts, cumulative-delay overflow/exhaustion, endless 429/transient responses, decision-to-permit clock rollback, projected and post-sleep monotonic-budget exhaustion, delivery phases, stale/reused keys, body mismatch, replayable/non-replayable bodies, competing retry owners, compile-fail outstanding-permit fan-out, blocking/async permit execution, and `scripts/release_0_46_gate.sh`.

Stop gate: `v0.46.0 implementation stop reached. Run pentest for this exact commit.`

### v0.47.0 - Local Async Contract

Status: tagged and published.

Goal: support `!Send` browser-WASM, embedded, and single-threaded executors.

Deliverables: `LocalAsyncTransport`, `LocalAsyncAuthenticatedTransport`, and
`LocalAsyncRawHttpExecutor` beside their cross-thread contracts; automatic
local compatibility for existing `Send` implementations; local prepared,
provider-link, and one-use retry-permit execution; one non-committing
`AsyncResponseStaging` and returned `ResponseCompletion` contract for Send and
local futures; SDK-owned drivers that commit only after `Ready(Ok)`; an explicit
possibly-sent cancellation classification; rollback of partial state; a deliberately
`!Sync` no-allocation testkit mock; and no allocator, runtime, task, clock, or
executor ownership.

Verification: genuinely `!Send` compile evidence, cross-thread blanket
adaptation, Send/local partial-secret dropped-future cleanup and compile-fail
proof that implementations cannot commit,
possibly-sent cancellation,
sequential and cooperatively outstanding local futures, prepared request,
provider-link, raw-executor, and retry-permit conformance, browser-WASM,
embedded and complete portable compile matrices, doctests, and
`scripts/release_0_47_gate.sh`.

Stop gate: `v0.47.0 implementation stop reached. Run pentest for this exact commit.`

### v0.48.0 - Streaming Transport Contracts

Status: tagged and published.

Goal: support bounded upload/download/event I/O without changing buffered requests.

Deliverables: separate opt-in streaming boundaries; explicit body replayability; per-operation maximum upload bytes plus hard chunk and observation budgets; checked actual-byte accounting across chunks; declared-length mismatch detection; overflow rejection before accepting the next chunk; rejection or explicitly bounded tolerance of consecutive empty chunks; a progress requirement for finite upload/download streams; explicit unknown-length and executor-owned framing policy; replayability invalidation when a source changes between attempts; transactional versus dirty partial upload/download sink states; partial-I/O and cancellation semantics; deterministic backpressure; no automatic retry for non-replayable mutations; and no implicit buffering/runtime. Long-lived event streams may remain open-ended only with caller-owned cancellation and an observation policy that still bounds zero-progress behavior.

Verification: exact/under/over/unknown declared lengths, every byte/chunk/observation cap boundary, endless empty chunks, alternating empty/data chunks, consecutive-empty tolerance exhaustion, finite-stream zero progress, event cancellation/observation exhaustion, source mutation between attempts, executor framing, chunk boundaries, short I/O, cancellation, transactional/dirty sinks, replay attempts, backpressure, cleanup, platform checks, and `scripts/release_0_48_gate.sh`.

Stop gate: `v0.48.0 implementation stop reached. Run pentest for this exact commit.`

### v0.49.0 - Incremental Provider Decoding

Status: tagged and published.

Goal: decode large lists, metrics, zonefiles, logs, and streams without one large JSON tree.

Deliverables: bounded visitor/state APIs preserving duplicate, nesting, aggregate, string, secret, early-stop, and cleanup protections; explicit total token and field limits; bounded numeric token and exponent lengths; validated partial UTF-8 state across chunk boundaries; fallible protected and structural allocation; panic-poisoned visitor callbacks; panic-safe scratch cleanup; and immediate staging cleanup on stop.

Numeric events retain the buffered decoder's finite-number admission. Input
chunks remain caller-owned, visitor payloads are borrowed and debug-redacted,
and `Stopped` remains structurally distinct from complete-document validation.

Verification: differential fixtures, token/field/numeric/exponent exhaustion, every partial UTF-8 and general chunk boundary, truncation, amplification, early stop, caught visitor panic, fallible-growth tests, independent fuzz validity oracle, deterministic control-prefixed valid/duplicate seed preflight, fuzzing, and `scripts/release_0_49_gate.sh`.

Stop gate: `v0.49.0 implementation stop reached. Run pentest for this exact commit.`

### v0.50.0 - Compile-Time Operation Associations

Status: tagged and published; pentest and release gates passed.

Goal: make endpoint/query/body/response mismatches unrepresentable.

Deliverables: sealed operation traits, `QueryFor<O>`, `BodyFor<O>`, typed `Prepared<O>`, cleanup-owning typed preparation, and clear-before-validation write-free exact policy checking that snapshots one immutable token consumed directly by request assembly; exhaustive associated service/endpoint policy, auth class and authentication-scope requirements, request headers/media, admitted statuses, success/error body and media policies, response caps, pagination/quota/retry strategy, streaming mode, response/error types, and required permit class; plus a strict reviewed classification manifest and forward-compatible enum rules.

Verification: compile-fail mismatch for every association, strict-schema and unknown-classification failures, exhaustive 208-row descriptor coherence, exact prepared-policy equality, guarded cleanup, source-derived binding coverage, semver review, and `scripts/release_0_50_gate.sh`.

Stop gate: `v0.50.0 implementation stop reached. Run pentest for this exact commit.`

### v0.51.0 - Mutation, Destructive, And Cost Permits

Status: tagged internal development milestone; cumulative pentest and crates.io
publication remain deferred to v0.55.0.

Goal: enforce safety intent at execution.

Deliverables: scope/fingerprint/expiry-bound permits are non-`Copy`, and direct permits are non-`Clone`. If an explicitly shareable handle is needed, every clone references the same atomic consumption state, remaining budget, and recovery generation; cloning never creates independent authority, dropping one handle never restores authority consumed by another, and `NotSent` recovery is generation-checked before the shared state becomes usable again. `PossiblySent` or `ResponseStarted` consumes the shared authority into spent/pending-reconciliation state; repetition after uncertain delivery requires the exact idempotency identity and operation-specific reconciliation. Plan-confirm fingerprints use a distinct versioned domain over provider, service, operation, endpoint identity, exact query bytes, selected headers, body, account/tenant, currency, observed price, spending ceiling, and permit scope; use exact bounded comparison or caller-supplied collision-resistant hashing, never non-cryptographic `Hash`; clear/redact canonical inputs. Include no-op rejection, replay policy, and caller-owned time/price/entropy. Authenticated request construction and extraction stay internal, permit attempts expose no reusable prepared request, the exact confirmed endpoint is rechecked at dispatch, and exclusive expiry is sampled from a caller `PermitClock` immediately before blocking transport access or on first async poll. Backward wall-clock observations cannot extend permit validity.

Verification: plan-confirm version/domain/field separation, weak digest rejection, canonical-input cleanup, compile-fail direct copy/clone and authenticated-capability extraction misuse, shared-handle clone/drop interleavings, shared-budget exhaustion, stale-generation `NotSent` recovery rejection, concurrent double-spend, every delivery-state transition, recover/reconcile/repeat rules, exact idempotency mismatch, exact endpoint mismatch inside an admitted official set, exclusive dispatch-time expiry for blocking and delayed async polling, wall-clock rollback, stale/mismatch/replay/budget/price/no-op/redaction tests, Loom where supported, and `scripts/release_0_51_gate.sh`.

Stop gate: `v0.51.0 implementation stop reached. Complete the security review and full release gate for this exact commit; defer cumulative pentest and crates.io publication to v0.55.0.`

### v0.52.0 - Provider-Generic Client Kernel

Status: tagged 2026-08-04 as an internal development milestone; cumulative
pentest and crates.io publication remain deferred to v0.55.0.

Goal: make secure typed execution reusable without publishing extra client crates.

Deliverables: one policy path for blocking, Send-async, and local-async preparation, authentication, send-once execution, checked decoding, and cleanup; caller-owned workspace leases per in-flight request; bounded admission with no hidden unbounded queue; no aliased mutable storage across await points; identical cleanup semantics in every execution mode.

Verification: fake-provider conformance, endpoint/auth mismatch, lease exhaustion/reuse, alias compile-fail cases, cancellation, bounded concurrency, cross-mode cleanup, and `scripts/release_0_52_gate.sh`.

Stop gate: `v0.52.0 implementation stop reached. Complete the security review and full release gate for this exact commit; defer cumulative pentest and crates.io publication to v0.55.0.`

### v0.53.0 - Pager And Action Workflow Drivers

Status: release candidate; local security review and repository gates passed.
Tagged on 2026-08-04 after the clean release gate and green GitHub CI and
CodeQL on the exact commit.

Goal: provide ergonomic workflows without clocks, sleep, or executors in core.

Deliverables: pure next-request/delay drivers, unconditional observation limits, provider progress policies, separate `PollControl` and bounded backoff, redacted policy errors, typed wall-clock observations only for provider timestamps/expiry, and monotonic durations for delay/timeout/elapsed budgets so clock rollback cannot extend execution.

Verification: busy-loop, progress reset/regression, terminal bounds, cancellation, wall-clock rollback, monotonic timeout/budget exhaustion, page/action scenarios, and `scripts/release_0_53_gate.sh`.

Stop gate: `v0.53.0 implementation stop reached. Complete the security review and full release gate for this exact commit; defer cumulative pentest and crates.io publication to v0.55.0.`

### v0.54.0 - Structured Payload-Free Diagnostics

Status: release candidate; local security review and repository gates passed.
Tagging requires the clean release gate and green GitHub CI and CodeQL on the
exact commit.

Goal: make failures actionable without leaking provider or customer data.

Deliverables: bounded provider/service/operation/status/request-ID/retry/error categories with no credentials, cursors, targets, bodies, messages, or generic payload-bearing `Debug` path; request-ID observation follows the provider/operation sensitivity and retention policy established in `v0.38.0`; an opt-in observer receives structured lifecycle events, while core never logs automatically.

Verification: redaction corpus, maximum lengths, observer disabled/enabled behavior, reentrancy/error isolation, downstream error types, snapshots, and `scripts/release_0_54_gate.sh`.

Stop gate: `v0.54.0 implementation stop reached. Complete the security review and full release gate for this exact commit; defer cumulative pentest and crates.io publication to v0.55.0.`

### v0.55.0 - Dynamic Testkit

Goal: test realistic multi-request behavior deterministically.

Deliverables: bounded recording, dynamic responders, fault injection including endless-empty and alternating-empty/data stream sources, pagination/action scripts, cancellation, partial I/O, and provider fixture builders.

Verification: exhaustion, mismatch non-consumption, recording caps, injected failures, no_std checks, and `scripts/release_0_55_gate.sh`.

Stop gate: `v0.55.0 implementation stop reached. Run the cumulative pentest through this exact commit before tagging and crates.io publication.`

### v0.56.0 - Provider-Generic Drift Engine

Goal: source-lock future providers with auditable historical evidence.

Deliverables: manifests/plugins for sources, auth, endpoints, operations, schemas, pagination, headers, retry/idempotency, cost policy, canonical diffs, and alert ownership.

Verification: malicious documents, redirect denial, digest rotation, reproducibility, plugin fixtures, and `scripts/release_0_56_gate.sh`.

Stop gate: `v0.56.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.60.0.`

## Tier C - OVHcloud API v2 Probe And Neutral Freeze

### v0.57.0 - OVHcloud Probe Source Lock

Goal: select and immutably record the unpublished probe surface.

Deliverables: official documents, console schema fingerprints, 5-10 read-only candidates, authorities, token endpoints, schema versions, task/event evidence, threat note, and no-publish gate.

Verification: drift fixtures, operation inventory, source reproducibility, and `scripts/release_0_57_gate.sh`.

Stop gate: `v0.57.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.60.0.`

### v0.58.0 - OVHcloud Authority And OAuth Conformance

Goal: challenge endpoint/auth contracts with geographic API/token pairs and expiring OAuth2.

Deliverables: source-bound EU/CA API-to-token fixtures; allocation-free `RegionalEndpointPair` and `EndpointPairPolicy`; alias, cross-region, duplicate, downgrade, and credentialed-redirect rejection; caller-clock `CredentialLifetime` conversion from `expires_in`; exclusive expiry and time-qualified refresh handoffs; lineage/generation-bound atomic token-and-lifetime replacement across blocking and async clients; mutable-source cleanup; and OVHcloud least-privilege guidance without an SDK support claim.

Verification: source-lock/fixture equality, pair mismatch, alias, duplicate, redirect, expiry, rollback, overflow, refresh-window boundaries, stale/concurrent rotation, mutable-source cleanup, redaction, unchanged no_std/default dependency graphs, and `scripts/release_0_58_gate.sh`.

Stop gate: `v0.58.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.60.0.`

### v0.59.0 - OVHcloud Cursor And Header Conformance

Status: release candidate; pentest and final retest passed. Tagging awaits the
clean release gate and green GitHub CI and CodeQL.

Goal: prove opaque pagination and schema-validation headers need no core exception.

Deliverables: bounded/redacted cursor headers, terminal-page semantics, validation-only `X-Schemas-Version`, reviewed schema-major evidence, and prepared-request-bound metadata decoding and continuation execution.

Verification: cursor cycles/controls/oversize, missing-next, duplicate headers, operation mismatch, method/target/service/endpoint/tenant rebinding prevention, schema drift, and `scripts/release_0_59_gate.sh`.

Stop gate: `v0.59.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.60.0.`

### v0.60.0 - OVHcloud Task And Event Conformance

Goal: prove asynchronous resource models against real source-locked read routes.

Deliverables: actual `/task` or `/event` operation coverage where available, bounded task/progress/error/event models, and generic examples kept as fixtures rather than endpoint claims.

Implementation assignment: source-lock the production authenticated
`GET /notification/contactMean/{contactMeanId}/task` collection and
`GET /notification/contactMean/{contactMeanId}/task/{taskId}` resource from
the official notification schema. Bind all task fields and six statuses.
Retain events as generic fixtures because this reviewed surface exposes no
event route; do not infer or advertise one.

Verification: state/progress/timestamp/link/message adversarial fixtures and `scripts/release_0_60_gate.sh`.

Stop gate: `v0.60.0 implementation stop reached. Run the pentest for this exact commit before tagging and crates.io publication.`

### v0.61.0 - OVHcloud End-To-End Probe

Status: signed internal milestone; pentest, final retest, local release gate,
GitHub CI, and CodeQL passed. Crates.io publication was deferred.

Goal: execute the complete unpublished probe through unchanged neutral contracts.

Deliverables: execute all ten source-locked read-only operations through
blocking, Send-async, local-async, and testkit paths in the exact
`publish = false` `ovhcloud-v2-probe` harness; include credential-free
fixtures, an ignored least-privilege EU `GET /iam/policy` live smoke, secure
token-file cleanup, and zero provider exceptions in reusable core code.

Verification: catalog/source-lock equality; cross-executor result and policy
matrix; live-config adversarial tests; compiled-but-ignored smoke; workspace
dependency/SBOM controls; no-publish/release-plan regression gates;
`scripts/check_ovhcloud_execution_probe.sh`; and
`scripts/release_0_61_gate.sh`.

Stop gate: `v0.61.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.65.0.`

### v0.62.0 - Neutral API Freeze

Status: release candidate; pentest and final retest passed. Local release
evidence is complete; GitHub CI, CodeQL, and the signed internal tag remain.

Goal: freeze provider-neutral contracts only after materially different probes and complete primary-provider vertical slices.

Deliverables: OVHcloud probe-driven changes complete and the `v0.42.0` Robot Basic/form/error/quota/maintenance/empty-body fixture passes unchanged. Implement full-fidelity slices in the real `cloud-sdk-hetzner` provider for: a paginated Cloud read plus one mutation/action; DNS zonefile or TSIG secret output; certificate or SSH-key secret output; a large Storage Box response through incremental decoding; a typed provider error; and an empty/no-content response. Every slice uses complete source fields for its selected operation, typed associations, `CheckedResponseGuard`, secret ownership and cleanup, and executes through blocking, Send-async, local-async, and testkit paths. The remaining 208-operation model/binding/client completion stays in `v0.63.0-v0.73.0`. Finish public API/semver review, compile-fail contract suite, migration guide, threat-model delta, and rejected-abstraction record.

Verification: public API diff; OVHcloud and Robot conformance fixtures; vertical-slice source-field, association, guard, secret, large-response, typed-error, no-content, and cross-executor matrices; downstream fixtures; no_std/platform matrix; and `scripts/release_0_62_gate.sh`.

Stop gate: `v0.62.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.65.0.`

## Tier D - Complete Hetzner Models And Clients

### v0.63.0 - Complete Cloud Resource Models

Status: tagged 2026-08-08 as an internal development milestone.

Goal: complete compute, network, IP, volume, pricing, and catalog fields.

Deliverables: source-derived models for firewalls, floating IPs, images, ISOs,
load balancers and their types, networks, placement groups, primary IPs,
servers and their types, volumes, pricing, and locations. Validate all 535
source-known field paths, required fields, nullability, exact JSON types,
numeric/string/array bounds, and discriminated service/target variants. Retain
bounded unknown fields and enum strings explicitly, and route ordinary Cloud
operations through dedicated `CloudResource` variants with no common-identity
fallback. Bind the generated field table and full fixtures to the exact pinned
Cloud specification through the upstream drift gate. Redact complete metadata
from diagnostics, permit copies only through checked-allocation `try_clone`,
count source string bounds as Unicode scalars under the hard byte ceiling, and
enforce source formats and patterns while failing generation on unsupported
security constraints.

Verification: generator determinism and canonical-schema-equality tests; all
208 operation response fixtures; missing, wrong-type, nullability, future-field,
future-enum, nested-union, and allocation-bound tests; complete Cloud response
fuzz seed; redacted-debug and fallible-copy regressions; RFC 3339, decimal,
integer-format, pattern, and multibyte-string boundaries; live upstream schema
drift; and `scripts/release_0_63_gate.sh`.

Stop gate: `v0.63.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.65.0.`

### v0.64.0 - Cloud Actions, Metrics, And Special Models

Status: implementation stop reached; pentest required.

Goal: complete action, metrics, composite, decimal, and timestamp responses.

Deliverables: add `UtcTimestamp` with calendar validation and canonical uppercase
UTC `T`/`Z` form; add `ExactDecimal` that retains the complete bounded JSON
number token, including integer, fraction, exponent, and negative zero. Replace
lossy metric floats with exact values, enforce positive steps plus per-series
and aggregate point bounds, and expose only fallible copies and redacted
diagnostics. Complete action IDs, source-bounded resource IDs, status,
progress, timestamps, resources, nullable error, exact unknown error-code text,
and protected messages. Preserve composite `action`, `actions`, and
`next_actions` independently and distinguish absent, null, and protected secret
outputs according to each operation's exact source nullability.

Verification: leap-year, leap-second, lowercase, offset, fraction, exponent,
negative-zero, non-finite, oversized-number, positive-step, per-series,
aggregate-point, source-ID, action-state, unknown-error, nullability,
redacted-debug, and fallible-copy tests. Run all 208 minimal operation fixtures,
the dedicated checked special-response fuzz target and seeds, live pinned
schema/operation drift, and `scripts/release_0_64_gate.sh`.

Stop gate: `v0.64.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.65.0.`

### v0.65.0 - Complete DNS Models

Goal: complete source-locked DNS zone, RRSet, zonefile, action, and
secret-bearing response models, then publish the cumulative v0.61-v0.65 crate
train.

Deliverables:

- Extend deterministic Cloud-spec extraction to DNS zones and RRSets,
  including the primary/secondary root discriminator, every required field,
  nullability, numeric bound, text bound, and source-known enum value.
- Return dedicated `Zone` and `DnsRrset` models from singleton, paginated, and
  create-composite responses instead of reducing DNS objects to generic IDs.
- Preserve zone mode, status, TTL, record count, labels, protection, registrar,
  authoritative/delegated nameservers, delegation timestamps/status, RRSet
  ownership, record comments, and nullable inherited TTLs.
- Move returned TSIG keys directly from protected parser storage into
  `SensitiveText`, redact all DNS diagnostics, admit legacy algorithms only as
  response observations, and retain HMAC-SHA256 as the outbound request policy.
- Accept bounded additive future uppercase RR types without assigning them
  source-known request semantics; reject empty or duplicate record sets,
  invalid IDs/TTLs, noncanonical TSIG Base64, incoherent primary-zone transfer
  servers, and oversized provider-omitted collections.
- Keep zonefiles under the global 8 MiB response boundary and incrementally
  prevalidate zone lists, RRSet lists, and exported zonefiles before the
  duplicate-rejecting protected one-shot model parser.
- Publish only changed crates: `cloud-sdk 0.65.0`,
  `cloud-sdk-hetzner 0.40.0`, `cloud-sdk-reqwest 0.34.0`, and
  `cloud-sdk-testkit 0.30.0`; retain unchanged
  `cloud-sdk-sanitization 0.18.0` and exclude the OVHcloud probe.

Verification:

- Regenerate and compare the complete source-derived model table and fixtures;
  reject inconsistent occurrences, unsupported schema composition, or changed
  union branches.
- Test TSIG ownership/redaction/canonical Base64, mode coherence, delegation,
  ID/TTL edges, nullable TTLs, empty/duplicate records, unknown RR types,
  pagination counts, create composites, zonefile boundaries, malformed JSON,
  and allocation/size failures.
- Route named DNS corpus seeds through the checked response fuzzer, run all 208
  minimal operation fixtures, incremental chunk validation, credential-gated
  read-only zone smoke coverage, and pinned live API/model drift checks.
- Verify package contents and cumulative independent publication order, then run
  `scripts/release_0_65_gate.sh` on the clean evidence commit.

Stop gate: `v0.65.0 implementation stop reached. Run the pentest for this exact commit before tagging and cumulative crates.io publication.`

### v0.66.0 - Complete Security Models

Goal: complete certificate and SSH-key typed coverage.

Deliverables:

- Extend deterministic Cloud-spec extraction to certificate and SSH-key roots,
  including every required field, nullability rule, numeric/text bound, and
  source-known state.
- Return dedicated certificate and SSH-key models from singleton, paginated,
  update, and create-composite responses through one typed security-resource
  family.
- Preserve certificate type, chain, validity, domains, fingerprint, labels,
  usage, and managed issuance/renewal detail; preserve SSH-key identity,
  fingerprint, public key, labels, and creation time.
- Keep certificate chains, SSH public keys, and provider failure messages in
  protected owned storage with closure-scoped inspection, redacted
  diagnostics, guarded parser error paths, and sanitizing drop behavior.
- Reject malformed or excessive PEM chains, invalid OpenSSH keys and MD5
  fingerprints, unknown managed states, status/error contradictions, invalid
  timestamps, and uploaded/managed shape confusion.
- Add a credential-gated read-only certificate/SSH-key live probe without
  introducing network, runtime, TLS, or secret-store dependencies into the
  default graph.

Verification: regenerate the exact source field table and fixtures; run all
208 operation fixtures, singleton/page/composite routing, PEM five/six-block
boundaries, key/fingerprint/timestamp/state/coherence failures, redaction and
cleanup checks, vertical execution, ignored live-probe staging, and named fuzz
seeds. Run `scripts/check_security_response_models.sh`, pinned live API/model
drift, and `scripts/release_0_66_gate.sh`.

Stop gate: `v0.66.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.70.0.`

### v0.67.0 - Complete Console Storage Box Models

Goal: replace generic Console success resources with source-complete typed
Storage Box response families while retaining bounded, cleanup-owning decode
behavior.

Deliverables:

- Extend deterministic model evidence from the pinned Cloud and Console
  specifications and fail when either source operation or field contract
  disappears, changes type, changes nullability, or exceeds admitted patterns.
- Add private-field models and read-only accessors for Storage Boxes, Storage
  Box types and prices, snapshots and statistics, subaccounts and access
  settings, pagination, and create-composite references.
- Validate source integer and text bounds, canonical UTC timestamps, status and
  nullable initialization coherence, snapshot character policy, subaccount
  home-directory policy, list limits, and page-size coherence.
- Keep provider-returned names, descriptions, hostnames, system identifiers,
  usernames, labels, and monetary text in cleanup-owning storage with redacted
  aggregate diagnostics and guarded fallible construction.
- Route list, singleton, update, and create-composite operations to dedicated
  `HetznerSuccess` and `StorageBoxResource` variants after contiguous and
  incremental JSON admission; retain shared typed action and folder models.
- Add credential-gated read-only box/type live probes without requiring owned
  inventory or admitting transport dependencies to the provider default graph.

Verification: regenerate the combined source field table and fixtures; run
all-operation minimal envelopes, box/type/snapshot/subaccount singleton and
list routing, create-reference distinctions, timestamp/nullability/status and
character failures, exact collection limits, cross-chunk large responses,
vertical execution, ignored live-probe staging, and named fuzz seeds. Run
`scripts/check_storage_response_models.sh`, pinned live API/model drift, and
`scripts/release_0_67_gate.sh`.

Stop gate: `v0.67.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.70.0.`

### v0.68.0 - Complete Hetzner Typed Binding Gate

Goal: prove exact associations for all 208 active pre-Robot operations.

Deliverables: zero missing request/query/body/response/error/policy bindings and explicit exclusion of all deprecated operations.

Verification: generated/source-derived matrix gate, compile-fail mismatches, and `scripts/release_0_68_gate.sh`.

Stop gate: `v0.68.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.70.0.`

### v0.69.0 - Hetzner Client Foundation

Goal: stabilize official/custom construction and storage lifecycle.

Deliverables: Cloud/DNS/Console endpoint and credential separation, explicit custom trust, caller storage profiles, concurrency, and no implicit retry/runtime policy.

Verification: endpoint/auth confusion, cleanup/cancellation/rotation, examples, and `scripts/release_0_69_gate.sh`.

Stop gate: `v0.69.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.70.0.`

### v0.70.0 - Cloud Client Methods

Status: release candidate; pentest and final retest passed.

Goal: expose every claimed Cloud operation through typed workflows.

Deliverables: complete read/mutation/action/metrics methods, permits, pagination, quotas, decoding, and blocking/async/local-async parity.

Verification: operation-client coverage, scenarios, live read-only smoke, and `scripts/release_0_70_gate.sh`.

Stop gate: `v0.70.0 implementation stop reached. Run the pentest for this exact commit before tagging and crates.io publication.`

### v0.71.0 - DNS Client Methods

Status: release candidate; pentest and final retest passed.

Goal: expose every claimed DNS operation through typed workflows.

Deliverables: zone/RRSet CRUD, actions, zonefiles, TSIG, permits, pagination, and cleanup across all execution modes; retire the experimental AWS-LC FIPS transport and enforce deferment to Brynja without changing ordinary rustls transports.

Verification: client coverage, secret/cancellation scenarios, live read-only smoke, active graph/source/package FIPS-absence checks, and `scripts/release_0_71_gate.sh`.

Stop gate: `v0.71.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.75.0.`

### v0.72.0 - Security Client Methods

Status: release candidate; pentest and final retest passed.

Goal: complete certificate and SSH-key workflows.

Deliverables: typed CRUD/actions, key/private-material lifecycle, rotation, permits, and cleanup across all execution modes.

Verification: client coverage, secret/error/cancellation scenarios, live read-only smoke, and `scripts/release_0_72_gate.sh`.

Stop gate: `v0.72.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.75.0.`

### v0.73.0 - Console Storage Box Client Methods

Status: release candidate; pentest passed with no findings.

Goal: expose every claimed Console Storage Box operation through typed workflows.

Deliverables: boxes/types/snapshots/folders/subaccounts/actions, permits, pagination, secret handling, and streaming where required.

Verification: client coverage, large/secret scenarios, live read-only smoke, and `scripts/release_0_73_gate.sh`.

Stop gate: `v0.73.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.75.0.`

## Tier E - Hetzner Robot

### v0.74.0 - Robot Source Lock And Matrix

Status: release candidate; pentest and final retest passed.

Goal: establish the reproducible Robot source of truth.

Deliverables: active/deprecated inventory, auth/lockout/forms/errors/limits/maintenance semantics, and explicit exclusion of all 16 deprecated Storage Box operations.

Verification: source fixtures, drift fetch, inventory gate, and `scripts/release_0_74_gate.sh`.

Stop gate: `v0.74.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.75.0.`

### v0.75.0 - Robot Form Codec

Status: released; pentest and final retest passed.

Goal: implement bounded atomic form encoding.

Deliverables: repeated fields, percent rules, exact preflight, transactional state, aggregate caps, and secret-tail cleanup.

Verification: every capacity, repeats, controls, fuzzing, and `scripts/release_0_75_gate.sh`.

Stop gate: `v0.75.0 implementation stop reached. Run the pentest for this exact commit before tagging and crates.io publication.`

### v0.76.0 - Robot Credentials And Lockout Policy

Status: released; pentest and final retest passed.

Goal: type-separate Robot Basic credentials and prevent unsafe authentication testing.

Deliverables: protected ingestion/rotation/cleanup, endpoint and Robot-service scope binding, and a lockout-aware credential-attempt generation bound to its exact issuing owner. Keep an allocation-free borrowed core attempt and add an alloc-backed non-hashable owned lineage for task-safe Robot attempts without a per-attempt allocation or credential borrow. Authentication rejection closes that generation for execution; foreign-owner attempts fail before generation comparison; rotation may advance credentials while an older request remains in flight and its response then fails stale; only newly supplied credentials or an explicit caller reconfirmation creates a new generation. No automatic policy, pager, action, or client path can reopen or repeat a rejected generation, and live evidence never intentionally uses invalid credentials.

Verification: auth cross-use, equal-generation foreign-owner rejection, non-hashable owner identities, redaction, rotation with an outstanding owned attempt, stale response after rotation, task-movable attempt shape, rejection-state transition, stale/rejected generation reuse, explicit reconfirmation, concurrent attempts sharing one generation, standalone no-default/`alloc`/`std` production builds, lockout gate, and `scripts/release_0_76_gate.sh`.

Stop gate: `v0.76.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.80.0.`

### v0.77.0 - Robot Error And Quota Protocol

Status: released; pentest and final retest passed.

Goal: type Robot errors, maintenance, invalid input, and quota behavior.

Deliverables: bounded envelopes, payload-free diagnostics, provider quota decoder, and structurally distinct authentication-rejection, quota, maintenance, invalid-input, and transient transport classifications. Authentication rejection is never automatically retryable and cannot be converted into a generic transient category by unknown-code fallback; fixtures bind this rule to the source-locked Robot protocol.

Verification: malformed/unknown/oversized/duplicate/quota tests, auth-versus-quota/maintenance/transient separation, unknown-code fail-closed behavior, authentication retry denial, and `scripts/release_0_77_gate.sh`.

Stop gate: `v0.77.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.80.0.`

### v0.78.0 - Robot Servers

Status: release candidate; pentest and final retest passed.

Goal: complete server list/get/update operations and models.

Deliverables: canonical server identity, capabilities, statuses, nullable subnets, explicit update intent, and no legacy IP aliases. Classified identities, topology, dates, states, and capabilities use fallible stable allocation-backed storage; strict JSON retains no ordinary scalar payload beside protected lexical/fixed storage; Robot parsing uses bounded clear-on-drop scratch and distinguishes malformed values from protected allocation failure; protected Boolean transfers write directly into final storage; request identities remain protected decimal bytes; model moves relocate only allocation metadata; and duplicate checks sort public indices without copied classified keys.

Verification: source coverage, field/conflict/boundary tests, stable-allocation move tests for parser and model values, canonical decimal maximum and path tests, complete IPv4/IPv6/date/subnet parser tests, public-decoder differential fuzzing of complete values against `core::net::IpAddr::from_str`, malformed embedded-IPv4 compression seeds, protected error-class mapping, direct protected Boolean transfer, absence of Robot scalar extraction, index-only duplicate checks, and `scripts/release_0_78_gate.sh`.

Stop gate: `v0.78.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.80.0.`

### v0.79.0 - Robot Cancellations

Status: release candidate; pentest and final retest passed.

Goal: complete cancellation get/create/revoke workflows.

Deliverables: all nine named server/IP/subnet GET, POST, and DELETE requests; canonical protected identities and calendar dates; explicit immediate/date scheduling; bounded redacted server reasons; explicit location-reservation intent; destructive permit metadata with retry denial; request-bound plan, fingerprint, direct/shared permit, and blocking/Send-async/local-async attempt execution; exact JSON success except the documented empty server revoke; exact typed request/response association through authorized execution; complete POST-intent including reservation availability/reserved acknowledgement and inactive revoke checks; shared display-safe reason validation; allocation-aware date decoding; strict identity/date/state/reason/reservation/subnet conflict decoding; and source-locked handling of official field spelling and server-number shape inconsistencies.

Verification: all method/path/form combinations, target-specific response policies, date and canonical-address boundaries, identity mismatch, cross-operation compile failure, direct/shared blocking/Send-async/local-async permit-authorized exact-request response decoding, mandatory POST digest and exact-or-digest DELETE policy, unpolled-attempt cleanup and reconciliation state, mismatched mutation date/reason/reservation availability/reserved/active-state rejection after execution, Unicode control and directional-format rejection, protected allocation-error classification, source fixture/checker regressions, checked-response fuzz seeds, alloc/Serde feature graphs, and `scripts/release_0_79_gate.sh`.

Stop gate: `v0.79.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.80.0.`

### v0.80.0 - Robot IP Management

Goal: complete every active Robot single-IP and separate-MAC operation while
publishing the cumulative v0.76-v0.80 checkpoint.

Deliverables:

- implement `GET /ip`, optional canonical `server_ip` filtering,
  `GET /ip/{ip}`, and `POST /ip/{ip}` with a non-empty partial traffic-warning
  update;
- implement separate-MAC get, generate, and delete with named request types and
  exact method, route, form, operation ID, authentication scope, response, and
  retry policy;
- add canonical protected IP and lowercase EUI-48 identities, bounded
  duplicate-free list/detail/MAC models, assignment, lock, traffic threshold,
  network, and nullable-MAC state;
- retain the exact request through checked decoding and direct/shared
  blocking, Send-async, and local-async mutation or destructive permit
  execution;
- reject identity, filter, network, requested-update, and MAC-state conflicts,
  clear failed preparation and sensitive form storage, and keep automatic
  mutation retry disabled unless metadata explicitly admits it;
- source-lock the six active operations and exact response fields, add a
  direct checked-response fuzz target, and publish the accumulated core,
  provider, transport, sanitization, and testkit versions selected by
  `release-crates.toml`.

Verification:

- `scripts/check_robot_ips.sh` and `scripts/test-robot-ips.py` prove all six
  source rows, methods, paths, query/form fields, operation metadata, response
  shapes, and fuzz-target registration;
- provider tests cover canonical addresses/MACs, list bounds and duplicates,
  assignment filters, network consistency, exact update acknowledgement,
  nullable MAC semantics, request provenance, permit scope, all transport
  modes, cleanup after preparation failure, and unpolled-attempt cleanup;
- `scripts/check_fuzz_harness.sh --build` and `--smoke` exercise the bounded
  Robot IP decoder with deterministic list/detail/MAC/delete selectors;
- `scripts/release_0_80_gate.sh` runs the cumulative public-checkpoint,
  dependency, platform, upstream-drift, SBOM, audit, and release-readiness
  gates.

Stop gate: `v0.80.0 implementation stop reached. Run the pentest for this exact commit before tagging and crates.io publication.`

### v0.81.0 - Robot Subnet Management

Goal: complete the six active Robot subnet and subnet-MAC operations while
preserving the provider's exact route-identity behavior.

Deliverables:

- add named list, detail, traffic-update, MAC-read, explicit MAC-assignment,
  and default-MAC-restoration requests bound to the official Robot endpoint,
  Basic scope, exact methods, operation IDs, response policy, and retry rules;
- add canonical protected subnet route identities, optional IPv4 server
  filtering, non-empty traffic updates, and an explicit canonical selected-MAC
  form without introducing arbitrary route or form construction;
- decode exact bounded subnet models with nullable server assignment, positive
  server identity, failover/lock state, traffic thresholds, family-valid
  integer masks, and same-network gateways;
- preserve documented host-bits-set route identities while exposing derived
  mathematical network and IPv4 broadcast accessors;
- decode the source-specific decimal-string MAC mask and a nonempty map of at
  most 256 canonical address-to-MAC choices, require the current MAC in that
  map, require PUT acknowledgement to match the selected MAC, and derive
  DELETE authority from checked same-resource subnet/MAC snapshots so its
  acknowledgement matches the assigned server's advertised default MAC;
- require a fixed 30-second observation window and a same-resource external
  mutation-lock lease for DELETE, bind all non-wire evidence into digest-only
  authorization fingerprints, reject permits that outlive evidence, and
  recheck evidence with the generic clock sample immediately before dispatch;
- redact and drop-clear non-copyable traffic policy/update aggregates and
  prevalidate every late preparation policy before caller storage is written;
- decode every documented subnet `(status, code)` pair through the exact
  request type, including operation-specific `404` and `500` failures, without
  widening the shared Robot decoder;
- retain exact request association through checked decoding and direct/shared
  blocking, Send-async, and local-async mutation or destructive permits;
- source-lock the official nullable/string and integer/string inconsistencies,
  add direct response fuzzing, redaction and cleanup tests, and keep all crates
  excluded from publication until the v0.85 checkpoint.

Verification:

- `scripts/check_robot_subnets.sh` and `scripts/test-robot-subnets.py` prove all
  six source rows, methods, paths, fields, reviewed inconsistencies, security
  policy, exact quotas and failure pairs, mutation resistance, compiled Rust
  behavior, and fuzz-target compilation;
- provider tests cover source-compatible host bits, IPv4/IPv6 prefix limits,
  derived network/broadcast boundaries, gateway family/membership, nullable
  assignments, list duplicates, response identity, exact update/MAC outcomes,
  request/traffic redaction, failed preparation cleanup, sensitive-evidence
  exact-plan rejection, server/MAC/timestamp/lock fingerprint mismatch,
  observation/lease expiry, validity mismatch, blocking/Send-async/local-async
  dispatch expiry, evidence/algorithm/digest panic cleanup, permit scope, all
  transport modes, and unpolled-attempt cleanup;
- `scripts/check_fuzz_harness.sh --build` and `--smoke` exercise list, detail,
  MAC-read, MAC-set, and MAC-delete checked decoder paths;
- `scripts/release_0_81_gate.sh` runs the cumulative internal-milestone,
  dependency, platform, upstream-drift, SBOM, audit, and release-readiness
  gates and selects no crate for crates.io publication.

Stop gate: `v0.81.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.85.0.`

### v0.82.0 - Robot Reset

Goal: complete the source-locked Robot reset discovery and disruptive
execution boundary without permitting callers to invent unsupported reset
capabilities or bypass destructive authorization.

Deliverables:

- add exact list, detail, and execute request types for the three active Robot
  reset routes, with official endpoint, Basic scope, operation IDs, quotas,
  methods, paths, content types, response bounds, and operation-associated
  provider failures;
- decode bounded duplicate-free capability inventories, canonical IPv4 and
  IPv6 identities, positive server numbers, protected operating status, and
  finite `sw`, `hw`, `power`, `power_long`, and `man` reset types;
- construct execution only from a 30-second authenticated detail observation
  bound to an opaque transport credential lineage and an explicitly selected
  advertised capability; raw decoding remains non-authorizing;
- bind credential lineage, complete server identity, capability, observation,
  and expiry into digest-only evidence, constrain permit validity, and recheck
  credential plus freshness immediately before destructive dispatch;
- encode the exact sensitive form, classify execution as
  destructive/non-idempotent, and deny automatic retry;
- require strong-digest exact-plan confirmation and request-bound direct or
  shared destructive permits across blocking, Send-async, and local-async
  transports; read-only reset requests cannot construct those plans;
- deny generic execute preparation and prepared-request type erasure; retain a
  mandatory authorization-evidence marker in core and reject marked requests
  from generic exact or digest plan builders;
- bind action success to checked IPv4, IPv6 network, optional server number,
  and exact requested reset type while narrowly admitting the official action
  example's omitted `server_number` inconsistency;
- source-lock every active row, field, quota, failure pair, finite capability,
  reviewed inconsistency, and security policy; add mutation tests and direct
  list/detail/action checked-response fuzzing; use separate 2 MiB, 4 KiB, and
  2 KiB success limits; publish no crate before v0.85.

Verification:

- `scripts/check_robot_resets.sh` verifies the immutable source fixture,
  mutation resistance, compiled capability/association/failure/cleanup/permit
  tests, and reset fuzz-target compilation;
- provider tests cover exact wire preparation, unsupported capability
  rejection, duplicate/unknown capability rejection, unknown/missing fields,
  identity mismatch, the documented optional action number, exact action type,
  failed preparation cleanup, sensitive-body exact-plan rejection, and
  request-bound direct/shared execution, authenticated preflight minting,
  foreign-credential and stale-evidence rejection before network access,
  evidence/permit lifetime coupling, and exact 4,095/4,096/4,097 list bounds;
- compile-fail tests prove execute requests implement neither generic
  `PrepareOperation` nor `as_untyped`; core tests prove generic plan builders
  reject marked requests and clear caller storage;
- `scripts/check_fuzz_harness.sh --build` and `--smoke` exercise all three
  checked response paths;
- `scripts/release_0_82_gate.sh` runs cumulative dependency, platform,
  upstream-drift, SBOM, audit, and release-readiness checks and selects no
  package for crates.io publication.

Stop gate: `v0.82.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.85.0.`

### v0.83.0 - Robot Failover

Goal: complete the source-locked Robot failover discovery and route-transition
boundary without widening network identities, silently accepting contradictory
acknowledgements, or automatically replaying ambiguous mutations.

Deliverables:

- add exact list, detail, reroute, and route-delete request types for all four
  active Robot failover routes, with official endpoint, Basic scope, operation
  IDs, quotas, methods, paths, content types, response bounds, and
  operation-associated provider failures;
- retain canonical failover, owner IPv4, owner IPv6-network, and active-target
  addresses in protected allocation-backed storage with redacted diagnostics;
- decode only the six reviewed failover fields, require positive server
  numbers, enforce route/netmask family agreement, reject noncontiguous masks
  and host bits in canonical route identities, require active-target family
  agreement, and reject duplicate list routes;
- encode reroute as the exact sensitive `active_server_ip` form and classify it
  as non-idempotent mutation; classify route deletion as non-idempotent
  destructive intent; deny automatic retries for both transition operations;
- require request-bound direct or shared mutation/destructive permits across
  blocking, Send-async, and local-async authenticated transports, with
  sensitive reroutes requiring strong plan digests;
- bind every checked response to its exact request type and route identity;
  require reroute success to echo the requested destination and delete success
  to return the official JSON failover object with `active_server_ip: null`;
  explicitly reject the earlier roadmap assumption of a no-content response;
- source-lock all active rows, fields, quotas, status/code pairs, the nullable
  source inconsistency, exact response policies, and security decisions in a
  bounded immutable fixture with mutation tests;
- add deterministic IPv4, IPv6, null-route, mask, family, duplicate, identity,
  conflict, cleanup, permit, and operation-association tests plus a four-path
  checked-response fuzz target; publish no crate before v0.85.

Verification:

- `scripts/check_robot_failovers.sh` verifies the immutable source fixture,
  mutation resistance, implementation-policy tokens, compiled failover tests,
  and fuzz-target compilation;
- provider tests cover exact wire preparation, sensitive form handling,
  separate response limits, route-family and contiguous-mask validation,
  canonical network identity, nullable active routes, strict fields, duplicate
  rejection, request identity, exact reroute/delete outcomes, failure-code
  narrowing, complete preparation cleanup, digest-only reroute confirmation,
  permit scope separation, and direct/shared execution;
- compile-fail documentation proves checked responses remain associated with
  their exact failover request type;
- `scripts/check_fuzz_harness.sh --build` and `--smoke` exercise list, detail,
  reroute, and delete checked-response paths;
- `scripts/release_0_83_gate.sh` runs cumulative dependency, platform,
  upstream-drift, SBOM, audit, and release-readiness checks and selects no
  package for crates.io publication.

Stop gate: `v0.83.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.85.0.`

### v0.84.0 - Robot Wake-On-LAN

Goal: complete both source-locked Robot Wake-on-LAN operations without
admitting the deprecated IP-address route, treating raw decoded data as
execution authority, or replaying an ambiguous packet send.

Deliverables:

- add exact `GET /wol/{server-number}` discovery and
  `POST /wol/{server-number}` send request types with official endpoint,
  Basic-auth scope, operation IDs, methods, quotas, empty-form media policy,
  checked JSON response policy, and operation-associated provider failures;
- accept only canonical positive `RobotServerNumber` path identity and expose
  no constructor or path encoder for the deprecated `{server-ip}` alias;
- decode exactly `server_ip`, `server_ipv6_net`, and `server_number`, require
  canonical fixed-family addresses and exact request-number association,
  reject duplicate/unknown/missing fields, and independently enforce the
  16 KiB operation body limit;
- make wake execution an explicit `RobotWolIntent::Send` that can only be
  constructed from a successful authenticated discovery response; bind its
  checked identity, credential lineage, observation, and 30-second expiry into
  non-forgeable authorization evidence;
- classify sending as non-idempotent mutation with retry eligibility `Never`,
  require request-bound direct/shared mutation permits and a strong plan
  digest, and recheck credential lineage plus evidence freshness immediately
  before blocking, Send-async, or local-async dispatch;
- require the send acknowledgement to preserve the exact server number and
  strict identity shape; source-lock `SERVER_NOT_FOUND`, `WOL_NOT_AVAILABLE`,
  and send-only `WOL_FAILED` without widening failures across operations;
- record both active rows, quotas, errors, exact response fields, empty-form
  source note, deprecated alias exclusion, and security decisions in a bounded
  immutable fixture with mutation tests; publish no crate before v0.85.

Verification:

- `scripts/check_robot_wol.sh` verifies the immutable source fixture,
  mutation resistance, implementation-policy tokens, and focused compiled
  WOL tests;
- provider tests cover exact GET/POST targets and metadata, canonical address
  families, unknown-field and alias rejection, request identity mismatch,
  complete failed-preparation cleanup, authenticated capability minting,
  evidence-only digest construction, dispatch-time credential/freshness
  checks, direct/shared permit state, and exact send acknowledgement;
- compile-fail documentation proves WOL execution cannot use generic operation
  preparation or erase its typed request association;
- the generic Robot API drift checker re-fetches the official documentation
  and proves exactly two active WOL rows remain assigned to this milestone;
- `scripts/release_0_84_gate.sh` runs cumulative dependency, platform,
  upstream-drift, SBOM, audit, and release-readiness checks and selects no
  package for crates.io publication.

Stop gate: `v0.84.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.85.0.`

### v0.85.0 - Robot Boot Configuration

Goal: complete the active Robot rescue, Linux, VNC, and Windows boot API and
publish the cumulative v0.81-v0.85 checkpoint without admitting deprecated
address aliases or architecture request fields.

Deliverables:

- implement the exact 15 active overview, get, activate, deactivate, and last
  operations under `cloud_sdk_hetzner::robot::boot` with canonical positive
  server-number paths and the documented 500-request/hour quota;
- provide bounded boot selectors, keyboard layouts, language values, and up
  to 64 unique authorized-key fingerprints with atomic form preparation;
- classify every activation/deactivation as non-idempotent with automatic
  retry disabled, and classify Linux, VNC, and Windows activation as
  destructive because rebooting into an installer can erase server data;
- decode strict identity-bound family and overview envelopes, accepting only
  the explicitly source-locked deprecated response fields while never
  exposing them in public models;
- retain generated passwords, authorized keys, and host keys only in
  redacted, cleanup-owning protected storage and require closure-scoped secret
  access;
- bind every checked response to its exact request type, enforce activation
  selector/language acknowledgement, and require deactivation to return an
  inactive password-free state;
- source-lock the official operation, quota, field, error, and deprecation
  inventory and add deterministic response seeds plus a bounded direct
  decoder fuzz target; and
- publish `cloud-sdk 0.85.0` and `cloud-sdk-hetzner 0.44.0`, plus dependency-
  only `cloud-sdk-reqwest 0.35.2` and `cloud-sdk-testkit 0.30.4`; keep the
  unchanged sanitization package unselected.

Verification:

- `scripts/check_robot_boot.sh` verifies the immutable source fixture,
  mutation resistance, implementation-policy tokens, and focused compiled
  tests for all four boot families;
- provider tests cover exact methods, paths, forms, quota metadata,
  destructive classification, no-retry behavior, malformed and duplicate
  values, unknown fields, identity mismatch, protected secrets, mutation
  outcome mismatch, and complete preparation cleanup;
- compile-fail documentation proves checked responses cannot cross operation
  types, while the fuzz harness exercises overview and family responses at
  the exact 1 MiB boundary;
- the generic Robot API drift checker re-fetches the official documentation
  and proves exactly 15 active boot rows remain assigned to this milestone;
  and
- `scripts/release_0_85_gate.sh` runs cumulative dependency, platform,
  upstream-drift, SBOM, audit, pentest-readiness, and package-selection checks
  for the public checkpoint.

Stop gate: `v0.85.0 implementation stop reached. Run the pentest for this exact commit before tagging and crates.io publication.`

### v0.86.0 - Robot Reverse DNS

Goal: complete reverse-DNS operations.

Deliverables: canonical addresses, bounded DNS names, forms, conflicts, permits, and exact models.

Verification: DNS/address/source tests and `scripts/release_0_86_gate.sh`.

Stop gate: `v0.86.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.90.0.`

### v0.87.0 - Robot Traffic

Goal: complete traffic queries and large response handling.

Deliverables: bounded ranges/intervals/repeated addresses/numeric limits and incremental decoding.

Verification: date/range/repeat/stream/source tests and `scripts/release_0_87_gate.sh`.

Stop gate: `v0.87.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.90.0.`

### v0.88.0 - Robot SSH Keys

Goal: complete SSH-key operations and protected material handling.

Deliverables: algorithms, fingerprints, names, keys, atomic forms, redaction, and cleanup.

Verification: key/form/secret/source tests and `scripts/release_0_88_gate.sh`.

Stop gate: `v0.88.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.90.0.`

### v0.89.0 - Robot Firewalls And Templates

Goal: complete firewall and template operations.

Deliverables: bounded ordered rules, CIDRs, ports, protocols, replacement intent, conflicts, and permits.

Verification: ordering/duplicate/rule/form/source tests and `scripts/release_0_89_gate.sh`.

Stop gate: `v0.89.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.90.0.`

### v0.90.0 - Robot vSwitches

Goal: complete vSwitch membership and cancellation operations.

Deliverables: VLANs, server lists, attach/detach/cancel intent, conflicts, repeated forms, and permits.

Verification: VLAN/membership/form/source tests and `scripts/release_0_90_gate.sh`.

Stop gate: `v0.90.0 implementation stop reached. Run the pentest for this exact commit before tagging and crates.io publication.`

### v0.91.0 - Robot Ordering Catalogs

Goal: complete read-only products, auctions, prices, currencies, and addons.

Deliverables: exact decimals, locations, distributions, limits, current-price warnings, and typed plan inputs without purchase execution.

Verification: catalog/price/decimal/source tests and `scripts/release_0_91_gate.sh`.

Stop gate: `v0.91.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.95.0.`

### v0.92.0 - Robot Transactions

Goal: complete transaction and per-server addon read models.

Deliverables: all states, identifiers, prices, timestamps, nullability, pagination, and read-only workflows.

Verification: state/decimal/date/source tests and `scripts/release_0_92_gate.sh`.

Stop gate: `v0.92.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.95.0.`

### v0.93.0 - Robot Ordering Mutations

Goal: gate every billable server, auction, and addon order.

Deliverables: cost permits and plan-confirm fingerprints bound to product, observed price, currency, quantity, account, expiry input, and replay policy; delivery-phase-aware indeterminate-send handling; mandatory transaction reconciliation before any repeat after a possibly sent order; CI cannot purchase.

Verification: stale-price/mismatch/replay/budget, not-sent/possibly-sent/response-started faults, reconciliation-before-repeat, non-execution/source tests, and `scripts/release_0_93_gate.sh`.

Stop gate: `v0.93.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.95.0.`

### v0.94.0 - Robot Client Integration

Goal: expose every active Robot operation through typed clients.

Deliverables: blocking, Send-async, local-async, pager/action workflows, endpoint/auth separation, permits, cleanup, and complete mock scenarios. All layers delegate retry ownership to the `v0.46.0` policy, propagate Robot authentication rejection without repetition, and require a newly supplied or explicitly reconfirmed credential-attempt generation before another call.

Verification: client coverage, authentication rejection through direct/pager/action/workflow paths with exactly one wire attempt, rejected-generation reuse denial, explicit reconfirmation, lockout/cancellation/concurrency scenarios, and `scripts/release_0_94_gate.sh`.

Stop gate: `v0.94.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v0.95.0.`

### v0.95.0 - Robot Live Evidence

Goal: validate least-privilege read-only Robot behavior without lockout or cost risk.

Deliverables: credential-free staging, ignored operator harness, private token files, no invalid credentials, mutations, orders, or destructive calls.

Verification: staging/runner tests, explicit operator smoke, source drift, and `scripts/release_0_95_gate.sh`.

Stop gate: `v0.95.0 implementation stop reached. Run the pentest for this exact commit before tagging and crates.io publication.`

## Tier F - Whole-Platform Qualification

### v0.96.0 - Complete Adversarial And Fuzz Qualification

Goal: close the full wire/auth/decoder/permit/cleanup/Robot adversarial matrix.

Deliverables: zero unclassified claimed operations, maintained corpora, cross-adapter differential tests, and current fuzz evidence.

Verification: all corpora/fuzz smoke/matrices/SBOM/deny/audit and `scripts/release_0_96_gate.sh`.

Stop gate: `v0.96.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v1.0.0.`

### v0.97.0 - Platform And MSRV Qualification

Goal: produce current evidence for every supported target, compiler, and active feature graph while keeping FIPS deferred to Brynja.

Deliverables: complete portable/native target evidence, unsupported-target rejection, exact compiler and feature-graph records, native dependency/build review, and a verified absence of FIPS features, dependencies, package content, and compliance claims.

Verification: full platform/MSRV matrix, packaged active-feature tests, FIPS deferment gate, native-build review, dependency freshness, and `scripts/release_0_97_gate.sh`.

Stop gate: `v0.97.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v1.0.0.`

### v0.98.0 - Provenance And Governance Review

Goal: make release trust and independent-review claims exact.

Deliverables: signer rotation/revocation, branch/release protection, trusted-publishing evaluation, reproducible packages/SBOMs, recovery procedures, and explicit independence disclosure without report-signing burden.

Verification: runbook/signer/provenance/reproducibility tests and `scripts/release_0_98_gate.sh`.

Stop gate: `v0.98.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v1.0.0.`

### v0.99.0 - Controlled Mutation Release Candidate

Goal: finish real mutation evidence and freeze the exact 1.0 candidate.

Deliverables: manual-only disposable project, approval, spending ceilings, unique prefixes, cleanup ledger, empty-inventory verification, final API/docs/migration review, and no CI mutation capability.

Verification: fake-provider dry runs, approved manual evidence when available, every release gate, and `scripts/release_0_99_gate.sh`.

Stop gate: `v0.99.0 implementation stop reached. Complete the pentest and full release gate for this exact commit; defer crates.io publication to v1.0.0.`

### v1.0.0 - Full Hetzner Production SDK

Goal: release the qualified candidate without adding features.

Deliverables: complete non-deprecated Hetzner Cloud, DNS, security, Console Storage Box, and Robot typed SDK; frozen neutral contracts; current docs, provenance, platform, SBOM, audit, fuzz, mutation, independent-review disclosure, and pentest evidence.

Verification: exact `v0.99.0` candidate ancestry, `scripts/checks.sh`, all source locks/matrices, `scripts/release_1_0_gate.sh`, and green GitHub/CodeQL.

Stop gate: `v1.0.0 implementation stop reached. Run pentest for this exact commit.`

## Post-1.0 Provider Blueprint

Provider crates start their own pre-1.0 package histories even when the
workspace facade is stable. Every release below requires its own source lock,
threat-model delta, release notes, release gate, and exact-commit pentest stop.

### v1.1.0 - Scaleway Source Lock And Scope

Goal: define a finite Scaleway product and stable-GA API scope before publishing
provider code.

Deliverables: exact official source revisions and retrieval evidence; a bounded
product/version/operation inventory; provider threat model, endpoint and
authentication policy, API matrix, deprecation policy, live-test policy, and
explicit alpha, beta, adjacent-product, and version exclusions. Later releases
cannot widen the inventory without a separately reviewed roadmap change.

Verification: reproducible source digests, redirect and source-origin checks,
zero unclassified inventory rows, documentation-link validation, and
`scripts/release_1_1_gate.sh`.

Stop gate: `v1.1.0 implementation stop reached. Run pentest for this exact commit.`

### v1.2.0 - Scaleway Provider Foundation

Goal: publish the initial focused `cloud-sdk-scaleway` preview without changing
frozen neutral contracts unnecessarily.

Deliverables: one primary provider crate; no_std request and response
foundations; source-locked regional and zonal endpoint derivation;
`X-Auth-Token` scope and rotation policy; typed errors; official/custom endpoint
separation; and no provider-specific transport, sanitization, or testkit crate.

Verification: default/no_std/platform builds, provider-crate policy, endpoint
and credential-confusion tests, redaction and cleanup tests, source fixtures,
package verification, and `scripts/release_1_2_gate.sh`.

Stop gate: `v1.2.0 implementation stop reached. Run pentest for this exact commit.`

### v1.3.0 - Scaleway Compute And Catalog Reads

Goal: complete the read-only compute and catalog rows admitted by `v1.1.0`.

Deliverables: complete typed models, request bindings, pagination, quota
metadata, checked response decoding, and blocking, Send-async, and local-async
client methods for every selected compute and catalog read operation.

Verification: zero-missing selected rows, golden/adversarial fixtures,
pagination and quota boundaries, cross-executor testkit scenarios, read-only
live smoke, and `scripts/release_1_3_gate.sh`.

Stop gate: `v1.3.0 implementation stop reached. Run pentest for this exact commit.`

### v1.4.0 - Scaleway Network And Storage Reads

Goal: complete the read-only network and storage rows admitted by `v1.1.0`.

Deliverables: complete typed models, regional/zonal bindings, pagination,
large-response handling, checked decoding, and all execution-mode client methods
for every selected network and storage read operation.

Verification: zero-missing selected rows, address and region/zone boundaries,
large and malformed response fixtures, cross-executor scenarios, read-only live
smoke, and `scripts/release_1_4_gate.sh`.

Stop gate: `v1.4.0 implementation stop reached. Run pentest for this exact commit.`

### v1.5.0 - Scaleway Mutations And Actions

Goal: complete only the mutation and action rows explicitly admitted by
`v1.1.0`.

Deliverables: typed request/response bindings, provider pagination variants,
source-locked retry and idempotency policy, mutation/destructive/cost permits,
delivery-phase handling, action workflows, reconciliation rules, and no
implicit retry or billable execution.

Verification: operation coverage, permit and plan-confirm mismatch tests,
not-sent/possibly-sent/response-started faults, idempotency and reconciliation
scenarios, non-executing live staging, and `scripts/release_1_5_gate.sh`.

Stop gate: `v1.5.0 implementation stop reached. Run pentest for this exact commit.`

### v1.6.0 - Scaleway Scope Stabilization

Goal: stabilize and qualify only the finite Scaleway scope selected in
`v1.1.0`.

Deliverables: zero unclassified selected rows; complete clients and examples;
current source drift, threat model, migration notes, API support matrix,
platform/SBOM/fuzz evidence, and explicit continued exclusion of every
unselected, alpha, beta, or differently versioned product.

Verification: full selected-inventory matrix, package and public-API review,
default/no_std/platform/transport-compatible boundary checks, live read-only smoke,
all Scaleway release gates, and `scripts/release_1_6_gate.sh`.

Stop gate: `v1.6.0 implementation stop reached. Run pentest for this exact commit.`

### v1.7.0 - DigitalOcean Source Lock And Scope

Goal: define a finite DigitalOcean product and operation scope from exact
official OpenAPI revisions before publishing provider code.

Deliverables: reproducible source records; bounded product/operation inventory;
provider threat model, `/v2` endpoint and bearer-auth policy, API matrix,
pagination/rate-limit inventory, live-test policy, and explicit exclusions for
Spaces, metadata, OAuth applications, AI, and every adjacent service.

Verification: source digest and origin checks, schema/operation inventory,
zero unclassified selected rows, exclusion regression tests, and
`scripts/release_1_7_gate.sh`.

Stop gate: `v1.7.0 implementation stop reached. Run pentest for this exact commit.`

### v1.8.0 - DigitalOcean Provider Foundation

Goal: publish the initial focused `cloud-sdk-digitalocean` preview on frozen
neutral contracts.

Deliverables: one primary provider crate; no_std models and request foundations;
bearer scope and rotation; canonical `/v2` endpoint binding; raw
same-authority `ValidatedProviderLink` pagination; typed errors; and no nested
provider transport, sanitization, or testkit crates.

Verification: default/no_std/platform builds, endpoint/auth and pagination-link
confinement tests, redaction/cleanup, source fixtures, package boundaries, and
`scripts/release_1_8_gate.sh`.

Stop gate: `v1.8.0 implementation stop reached. Run pentest for this exact commit.`

### v1.9.0 - DigitalOcean Read Operations

Goal: complete all read-only rows selected in `v1.7.0`.

Deliverables: complete typed read models and clients; checked decoding; bounded
link pagination; provider rate-limit buckets; `Retry-After` conflict policy;
and blocking, Send-async, and local-async parity.

Verification: zero-missing selected reads, raw-link preservation and confinement,
quota/time/conflict boundaries, malformed and oversized fixtures,
cross-executor scenarios, read-only live smoke, and
`scripts/release_1_9_gate.sh`.

Stop gate: `v1.9.0 implementation stop reached. Run pentest for this exact commit.`

### v1.10.0 - DigitalOcean Mutations And Actions

Goal: complete only the mutation and action rows admitted by `v1.7.0`.

Deliverables: typed mutation/action clients, source-locked retry and idempotency
classification, fresh-intent keys, permits, delivery-phase handling, action
workflows, and operation-specific reconciliation without implicit execution or
retry.

Verification: operation coverage, fingerprint/idempotency vectors, permit and
replay denial, delivery-phase fault injection, reconciliation, non-executing
live staging, and `scripts/release_1_10_gate.sh`.

Stop gate: `v1.10.0 implementation stop reached. Run pentest for this exact commit.`

### v1.11.0 - DigitalOcean Scope Stabilization

Goal: stabilize and qualify only the finite DigitalOcean scope selected in
`v1.7.0`.

Deliverables: zero unclassified selected rows; complete clients, examples,
source drift, threat model, migration notes, API support matrix,
platform/SBOM/fuzz evidence, and explicit continued exclusion of Spaces,
metadata, OAuth applications, AI, and every unselected product.

Verification: complete selected-inventory and public-API review, package and
platform checks, adversarial pagination/auth/retry evidence, live read-only
smoke, all DigitalOcean release gates, and `scripts/release_1_11_gate.sh`.

Stop gate: `v1.11.0 implementation stop reached. Run pentest for this exact commit.`

### v1.12.0 - Three-Provider Neutral Conformance

Goal: prove the frozen provider-neutral contracts against Hetzner, Scaleway,
and DigitalOcean before planning full OVHcloud publication.

Deliverables: one cross-provider matrix for identities, endpoints,
authentication, paths/queries, structured and link pagination, quota/retry,
streaming, decoding, permits, cleanup, diagnostics, and all execution modes;
documented provider-owned differences; no compatibility fallback or premature
`cloud-sdk-ovhcloud` package.

Verification: shared conformance suites across all three provider crates,
compile-fail association tests, cross-adapter differential scenarios, public-API
and semver review, all provider drift gates, and
`scripts/release_1_12_gate.sh`.

Stop gate: `v1.12.0 implementation stop reached. Run pentest for this exact commit.`

Full `cloud-sdk-ovhcloud` publication receives a separate version plan after
`v1.12.0`; the unpublished pre-1.0 probe never becomes its package history. The
one-primary-crate-per-provider rule remains mandatory.
