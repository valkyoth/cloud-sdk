# crates.io Unified Client Work

Status: Commit 20 **in progress**, not an accepted implementation checkpoint.
Baseline: `f49b7712` (Commit 19, user-confirmed pentest and GitHub pass).
No tag, publication, Commit 21 authorization or full-provider coverage claim.

## Implemented Foundation

- `client::RegistryClient` binds production or staging and delegates blocking
  typed reads and consumed mutation permits to their existing checked runners.
  Request assembly and authorization dispatch no longer need a caller callback
  on these paths. Separate streaming `publish`, `publish_local` and `publish_async`
  methods now accept a consumed permit and borrowed archive source.
- `execute_local` and `execute_async` now cover those same read and permit
  families. API-token catalog requests have corresponding explicit methods.
  Local execution accepts non-Send transports; Send execution has compile-checked
  Send futures. There is no blocking bridge inside production async execution.
- Credential staging is crate-private and borrows cleanup-owned scratch across
  suspension. Public credential access remains callback-scoped. All guards are
  installed before the future is returned, so even unpolled drops clear scratch.
  Consumed temporary credentials remain owned by the future until completion/drop.
- Email confirmation and token-based invitation acceptance now execute in all
  three modes, consuming protected credentials without an Authorization header.
  Raw adapters stage URI paths in sanitization-owned Bytes; non-secret authority
  storage is separate so retained origin keys cannot hold the token allocation.
  Custom executors must not log targets or retain unprotected URI copies.
  HTTP/TLS wire buffers and server/proxy logs remain deployment boundaries.
- All four `RegistryBuffers` regions clear on errors and successful completion,
  including admission and endpoint-construction failures. Rate limiting remains
  process-wide and never sleeps or retries implicitly.
- Core authorized raw contracts keep credentials separate from anonymous raw
  requests. The neutral bundled adapters compare the expected destination
  before credential copying/I/O and preserve the complete header value without
  adding a Bearer prefix. Owned header staging uses sanitization.
- Bundled production/staging factories accept identifying user agent and
  deadlines, never a caller-controlled URL or implicit credential.
- `bundled::ArtifactTransport` wraps a separate fixed static-CDN adapter.
  Blocking and async bodies are live sources, not whole-archive buffers. Async
  sources also implement local-async execution. The source enforces a finite
  byte limit, 4,096 upstream frame observations and the original total deadline.
  It accepts only status 200 and identity coding, rejects duplicate headers and
  declared/observed trailers, and never follows redirects. Dropping a pending
  read invalidates the source and closes its unpooled body.
- `downloads::Sha256Checksum` uses the already admitted `sha2 0.11.0` with
  defaults disabled and a single-use state. Expected hashes must originate from
  trusted registry metadata; hashing alone does not authenticate that metadata.
- Optional `blocking-rustls`, `async-rustls` and `artifact-sha256` features do
  not change the default, alloc-only or Serde-only dependency boundary.
- Bundled publication now streams Cargo framing through destination-bound raw
  upload execution. A one-chunk queue, sanitized owned frames, declared-length
  and progress checks, total deadline, and checked response admission prevent
  whole-archive buffering or implicit replay. Early final responses before
  source completion fail closed. Local sources need not be Send. Source storage
  and non-cooperative source execution remain caller responsibilities.
- Platform-verifier 0.7.1 and its Android helper 0.2.0 are recorded in all
  affected locks and the dependency-review digest. Android bundled transport
  support is not added by this dependency refresh.
- Artifact execution has real Unix host-filesystem sink tests, using the
  opt-in SHA-256 implementation and an independent known-answer digest.
  Blocking/local/Send paths qualify partial writes, hidden tentative output,
  checksum/length/I/O failures, non-overwriting publication and cancellation.
  This is a test reference sink, not a bundled filesystem adapter, crash-safety
  guarantee, secure disk-erasure claim or Windows qualification.

## Remaining Commit 20 Gates

1. Retain unified execution and bundled adapter qualification in the final
   checkpoint. All 51 facade rows now have successful three-mode witnesses,
   including publication with both API and trusted-publishing credentials.
   This does not establish live mutation or per-operation TLS qualification.
2. Retain the live authenticated streaming-upload qualification in the final
   gate. Transport loopback tests, Cargo framing tests and facade preflight/
   cancellation checks now exist alongside integrated publication fixtures.
3. Retain the passing generated execution gate against all 51 operation rows.
   Trait implementations alone do not prove each variant executes correctly.
   Retain the secret-path URI ownership, exact-wire and cancellation evidence
   in that generated qualification.
4. Complete higher-level Cargo publish/owner/yank/unyank/search workflows and
   independently verify all seven stable wire contracts byte-for-byte. Add the
   explicit Cargo owner profile described below in the final integrated matrix.
5. Retain the real-filesystem qualification in the final integration gate.
   Unix host tests now cover cancellation, checksum failure and commit failure;
   other storage adapters and platforms still require their own qualification.
6. Complete adapter/provider integration, compile-checked examples, complete
   repository/MSRV/platform/dependency/SBOM checks, and the incremental pentest.

Do not mark Commit 20 complete or advance the accepted baseline until all six
items and the original commit-plan exit criteria have executable evidence.

## Foundation Verification

Checked locally on 2026-09-26, before the foundation commit:

- `scripts/checks.sh`: passed, including workspace all-feature tests,
  doctests, warning-denied Clippy, isolated provider feature checks, packaging,
  default dependency boundary, fuzz metadata and fixture regressions.
- Rust 1.92.0 checks for the crates.io provider and neutral reqwest adapter,
  with all features: passed. This is not a complete platform-matrix claim.
- SHA-256-only provider tests and the AST fail-closed test policy: passed.
- Fresh `cargo audit` and `cargo deny check`: passed.
- `scripts/check_sbom_freshness.sh`: all four locked graphs passed.
- Formatting, documentation links, file-length policy and `git diff --check`:
  passed. Secret-path rejection tests also prove zero dispatch, untouched rate
  admission and cleanup of all four scratch buffers.

These are implementation checks, not a pentest or acceptance of Commit 20.

## Async Increment Verification

The continuation after `97c8e0cb` adds unified local/Send execution for the
currently enabled reads and permits. Checked locally on 2026-09-26:

- Full `scripts/checks.sh`, including workspace tests, warning-denied Clippy,
  isolated features, packaging, doctests and fuzz metadata: passed.
- Rust 1.92.0 provider tests with `blocking,async`, including compile-fail
  credential/permit tests: passed. Async-only Clippy also passed.
- All 25 anonymous fixtures, six permit families, and explicit token catalog
  execution have facade tests. Send futures are statically checked; the local
  fixture holds `Rc` and cannot satisfy the Send executor contract.
- Unpolled/in-flight cancellation, shared admission, wrong/changed origins,
  cleared credentials, short scratch, strict empty acknowledgements, malformed
  replies and oversized delays fail closed and clear supplied scratch.
- Documentation links, source-length/modularity policy, AST fail-closed test
  policy, response cleanup, SBOM freshness and whitespace checks: passed.

No manifests or lockfiles changed in this increment. These tests do not replace
the outstanding generated matrix, bundled integration, storage and pentest gates.

## Storage Increment Verification

Checked locally on 2026-09-26 after the async increment:

- Full `scripts/checks.sh`: passed, including all four new filesystem test
  groups in the all-feature provider suite.
- Rust 1.92.0 filesystem tests with `blocking,async,artifact-sha256`: passed.
- Warning-denied provider Clippy, the AST fail-closed test checker, exact crate
  source inventory, formatting, file-length/modularity policies, documentation
  links and whitespace checks: passed.
- SBOM freshness: all four graphs passed; no manifests or lockfiles changed.

The tests cover 38 transfer cases using actual host files: three successful
mode variants, 21 error cases, six publication collisions and eight cancellation
cases. This evidence qualifies the test sink on this Unix host, not arbitrary
caller storage or crash recovery. The full Commit 20 checkpoint remains open.

## Upload Increment Verification

Checked locally on 2026-09-26 after the storage increment:

- Full `scripts/checks.sh`: passed, including workspace tests, Clippy, feature
  isolation, packaging, doctests and the new bundled publish README example.
- Rust 1.92.0: all eight neutral upload test groups and all 14 provider publish
  test groups passed with all features. Provider async-only and async-rustls-only
  warning-denied Clippy passed as well.
- The loopback source waits for server receipt before producing its next chunk,
  proving the adapter does not preload the body. Tests cover exact wire bytes,
  explicit authorization, early responses, deadlines, short/long sources,
  incoherent policies, response media/size/framing checks and cancellation.
- Cargo framing is compared with independently assembled bytes across chunk
  sizes and short/long archives. Non-Send local/blocking sources are supported;
  Send futures are compile-checked. Both credential kinds fail origin mismatch
  without consuming archive input, and unpolled facade futures clear all scratch.
- The fail-closed AST policy, crate/source inventory, feature-guard regression
  tests, publish source/schema checks, formatting, documentation links,
  file-length/modularity checks and all four SBOM graphs passed.

No manifests or lockfiles changed. No live registry mutation was performed;
HTTP upload tests use loopback fixtures. This increment is not acceptance of
Commit 20 or a substitute for the final integrated matrix and pentest.

## Secret-Path Increment Verification

Checked locally on 2026-09-26 after the upload increment:

- Full `scripts/checks.sh`: passed, including workspace tests, feature isolation,
  doctests, warning-denied Clippy, packaging and existing transport regressions.
- Rust 1.92.0: provider secret-path tests, five URI ownership/composition tests,
  and the loopback wire test passed. Async-rustls-only provider Clippy passed.
- Both personal path-token operations have exact-method/target/no-authorization
  fixtures for blocking/local/Send execution. Wrong origins, short scratch,
  cleared tokens, malformed acknowledgements and invitation identity mismatches
  fail closed. Unpolled/in-flight cancellation clears scratch without replay.
- The pinned `http 1.5.0` PathAndQuery and `hyper-util 0.1.21` origin-form paths
  retain shared Bytes storage. Tests verify pointer retention, zeroed bytes at
  owner destruction, clone/rejection lifetimes, and authority independence.
  Canonical, form and provider-link query composition is compared with the old
  URL behavior, including empty queries and rejected raw query apostrophes.
- Actual loopback requests preserve percent encoding and present-empty queries,
  omit authorization, and keep public request diagnostics redacted. No live
  email confirmation, invitation acceptance or registry mutation was performed.
- All four SBOM graphs remain fresh; no manifests or lockfiles changed.

These guarantees cover SDK-owned URI staging, not external HTTP/TLS wire
buffers, process-abort cleanup or remote access logs. Custom executors must
implement the documented secret-target storage/logging policy. Final integrated
operation coverage and the Commit 20 pentest stop remain outstanding.

## Measured Execution Coverage

`python3 scripts/check_cratesio_execution_coverage.py` is the strict completion
gate, also invoked by `scripts/release_1_1_gate.sh`. It regenerates the expected
route inventory in memory, rejects stale bindings, runs the provider's actual
all-feature test suite, and accepts only successful fixture runs. A witness is
emitted after all three facade calls, decoded results, exact wire assertions,
call counts and scratch-cleanup assertions pass. Only public operation IDs and
execution modes are emitted, never targets, tokens or payloads.

Current measured result: **51/51** operation rows have blocking/local/Send
witnesses. Publication now executes through the same provider runners used by
the bundled adapters, with independent Cargo framing bytes, exact raw/Bearer
authorization checks, partial writes, checked crate/warning responses and
scratch cleanup. Both credential kinds pass all three modes and three chunk
sizes before the publication witness is recorded. The strict checker passes;
`--report` remains progress-only and is not used by the release gate.

This is measured facade fixture coverage, not proof of every parameter variant,
all bundled TLS exchanges, every authentication mode or Cargo compatibility.
Those qualifications remain separate requirements.

## Cargo Compatibility Profiles

The independent minimal Cargo search fixture executes through all three modes
with literal expected query bytes, not a target generated by the request under
test. Parameter order is the SDK's canonical order; Cargo's contract does not
prescribe that order.

Owner-list compatibility has a distinct, explicitly selected profile:

- Cargo documents Authorization for owner listing. The current public crates.io
  `AccountRequest::owners` deliberately executes anonymously.
- `accounts::cargo::CargoOwnersRequest::new(name, token)` executes a read-only
  GET with explicit origin-bound raw API-token authorization in blocking,
  local-async and Send-async modes. It sends neither query nor request body.
- The minimal Cargo owner response has `id`, `login` and optional `name`.
  The website decoder additionally requires provider fields including `kind`,
  `avatar` and `url`; it correctly rejects the minimal profile today.
- Its separate bounded `CargoOwners` decoder preserves absent/null names,
  unsigned 32-bit IDs and protected login text. It does not infer user/team
  kinds or mutation authority. Exact duplicate logins fail; numeric IDs may
  overlap between service namespaces. See the [account policy](CRATESIO_ACCOUNT_POLICY.md).
- Website decoding and the generic credential-context allowlist are unchanged.
  The Cargo authorization exception constructs only the typed owner-list path.

The characterization test preserves this distinction alongside the Cargo tests.
Successful website owner tests must not be reported as exact Cargo owner-list
compatibility. The source is the
[Cargo Registry Web API](https://doc.rust-lang.org/cargo/reference/registry-web-api.html).

Publication's binary framing and both authorization schemes are independently
verified. Its response model still deliberately follows the pinned crates.io
schema, requiring `crate` and `warnings`, whereas generic Cargo permits a
minimal success with no warnings. A three-mode characterization test confirms
that the strict decoder rejects both `{}` and warnings-only success. Complete
the final seven-contract qualification with this distinction explicit; do not
label the current response model a generic registry fallback. A separate Cargo
response profile, if required, must not silently relax the strict model.

## Neutral Upload Integration

Core `AuthorizedUpload` and blocking/local/Send upload executor traits define
the trusted, destination-bound finite upload contract without dependencies or
allocation. The neutral reqwest adapter delegates these traits to its existing
bounded live upload implementation. Provider publication is now generic over
these contracts; `blocking`/`async` suffice for custom upload adapters, while
bundled network/TLS/runtime features remain explicitly opt-in. No test-only
endpoint override or credential-routing bypass was introduced.

The integrated facade tests compare metadata-length/metadata/archive-length/
archive bytes against an independently assembled Cargo frame. They cover both
credential schemes, non-Send local/blocking sources, Send futures, partial
writes, short/long archives, preflight rejection, provider/protocol/transport
errors, no retry, and unpolled/in-flight cancellation. Real loopback upload
tests now exercise the neutral traits as well as the underlying adapter.

Custom executors remain trusted: the contract requires complete source framing
and response admission before commitment, deadlines, no redirects/cookies or
retries, sensitive authorization handling and cleanup. Provider tests do not
prove that an arbitrary external executor honors these requirements.

### Integration Verification

Checked locally on 2026-09-26:

- Full `scripts/checks.sh`: passed, including workspace tests, package
  verification, doctests, feature-isolated checks and warning-denied Clippy.
- Strict generated execution coverage: **51/51**, all three modes; passed
  without the progress-only flag or weakening the expected inventory.
- Four provider publication test groups passed, covering 18 successful
  credential/mode/chunk combinations, 27 failure combinations, four future-drop
  cases and six minimal-Cargo-response characterization cases.
- The provider publication groups and all eight neutral loopback upload groups
  passed on Rust 1.92.0. Allocation-only blocking/async publication tests passed
  on the development compiler without enabling bundled networking features.
- Core no-default-feature and provider allocation-only checks passed for
  `thumbv7em-none-eabi`. This is compile evidence, not a TLS/platform claim.
- AST fail-closed checks across core/provider/adapter, source/feature guard
  regressions, documentation links, formatting, file-length/modularity policy,
  whitespace and all four SBOM graphs passed.

No manifest or lockfile changed, and no live mutation or publication ran.
Commit 20 still needs final Cargo compatibility qualification and its pentest;
this is a tested implementation increment, not acceptance of the checkpoint.

## Cargo Owner Increment Verification

Checked locally on 2026-09-26:

- Full `scripts/checks.sh`: passed, including workspace tests, all-feature
  doctests, warning-denied Clippy, isolated feature checks and packaging.
- The minimal Cargo profile executes in blocking, non-Send local and Send
  modes. Tests assert exact authorization bytes, method/path/body, decoded
  field semantics, call counts and complete supplied-scratch cleanup.
- Negative cases cover wrong/changed origins, cleared tokens, short credential
  scratch, malformed records, status/media/encoding rejection and transport
  failure. Preflight failures preserve rate admission. Pending and unpolled
  cancellation clear storage; the generic website credential route stays closed.
- Model tests cover unsigned-ID boundaries, nullable/absent names, malformed
  and duplicate fields, exact/over-limit owner and string bounds, redaction,
  inert extensions and separate user/team numeric-ID namespaces.
- Cargo-focused tests passed on Rust 1.92.0 with all features and on the
  development compiler with allocation-only blocking/async features. The new
  README example is compile-checked. Feature-guard regression tests passed.
- Live source-lock verification passed for 51 operations and seven Cargo
  overlaps. No source refresh, dependency or manifest changes were needed.
- AST fail-closed checks, mandatory cleanup policy, documentation links,
  file-length/modularity policies, whitespace and all four SBOM graphs passed.
- Progress coverage remains 50/51. Publication still lacks the three integrated
  witnesses; this is not acceptance of Commit 20 or a green release gate.

## Coverage Increment Verification

Checked locally on 2026-09-26:

- Full `scripts/checks.sh`, including packaging, workspace tests, feature
  isolation, doctests and warning-denied Clippy: passed.
- All 15 unified-client test groups passed on Rust 1.92.0 with all features,
  and with allocation-only blocking/async features on the development compiler.
- Coverage checker regressions passed for missing/duplicate-mode/unknown
  witnesses, stale routes, added operations, failed Cargo runs, exact command
  arguments and strict release-gate wiring.
- The real strict coverage run returned failure for the one pending publication
  row; explicit `--report` returned the same 50/51 result as progress only.
  This expected incomplete result is not a green release qualification.
- AST fail-closed checks, source inventory, documentation links, formatting,
  file length/modularity, whitespace and all four SBOM graphs passed.

No production API implementation, manifests or lockfiles changed in this
increment. No live registry mutation or new pentest acceptance is claimed.
