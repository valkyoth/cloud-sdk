# crates.io Unified Client Work

Status: Commit 20 **in progress**, not an accepted implementation checkpoint.
Baseline: `f49b7712` (Commit 19, user-confirmed pentest and GitHub pass).
No tag, publication, Commit 21 authorization or full-provider coverage claim.

## Implemented Foundation

- `client::RegistryClient` binds production or staging and delegates blocking
  typed reads and consumed mutation permits to their existing checked runners.
  Request assembly and authorization dispatch no longer need a caller callback
  on these paths. Separate bundled `publish`, `publish_local` and `publish_async`
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

1. Qualify unified async execution with the bundled adapters across the final
   operation matrix. Mock parity and cleanup tests pass for the currently
   enabled families, including path-secret operations; this does not substitute
   for final integrated publish-operation evidence.
2. Retain the live authenticated streaming-upload qualification in the final
   gate. Transport loopback tests, Cargo framing tests and facade preflight/
   cancellation checks now exist; complete the final integrated operation matrix.
3. Generate exact execution coverage against all 51 operation matrix rows;
   trait implementations alone do not prove each variant executes correctly.
   Retain the secret-path URI ownership, exact-wire and cancellation evidence
   in that generated qualification.
4. Complete higher-level Cargo publish/owner/yank/unyank/search workflows and
   independently verify all seven stable wire contracts byte-for-byte.
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
