# crates.io Commit 21 Qualification

Status: implementation stop reached; incremental pentest required.
Commit 20 was accepted after user-confirmed pentest and
GitHub green at `df5892e712281ac5d27aeae900d89b754aa60a49`.
No Commit 21 pentest acceptance, tag, workspace publication or Commit 22 authorization.

## Read Harness Increment

The [live-read harness](CRATESIO_LIVE_READ.md) adds an explicitly opted-in,
fixed-origin, fixed-route, read-only production probe. Normal tests remain
offline. Token input uses guarded mutable stdin storage, never arguments or
credential environment variables. CI refusal is exercised through the actual
ignored-test entry point, in addition to ordinary configuration regressions.
Normal CI publication/ownership/yank/token/trusted-publishing tests remain
mock-only. No new production SDK API or dependency is introduced.

Anonymous production metadata and a one-entry keyword page passed locally on
2026-09-26. Token scope cannot be verified locally. Authenticated reads require
an explicit isolated-account acknowledgement, not an unsupported assertion that
the token itself is read-only. Registry deletion requires browser authentication;
API-token cleanup is unavailable. The operator separately authorized a single
disposable publication with manual website deletion, outside the read harness.

## Operator Live Results (2026-09-26)

- Anonymous site metadata and keyword reads passed.
- The token-backed following read returned HTTP 403. It is not qualified as a
  successful authenticated read; selecting all token scopes does not establish
  access to endpoints that use the upstream legacy-token authentication path.
- A standalone SDK driver locally packaged and verified
  `cloud-sdk-qualification-20260926-15a061b1c143` version `0.0.0` (1,089 bytes).
  It restricted mutations to that exact name and required an anonymous 404
  preflight before each explicitly authorized publication attempt.
- The initial publication returned HTTP 400. An independent version read returned
  HTTP 404. Mutations stopped without automatic retry; the redacted error did not
  establish the cause.
- After the operator verified the account email and explicitly authorized another
  attempt, publication succeeded through the SDK. An independent version read
  confirmed `yanked=false`.
- Yank succeeded and an independent read confirmed `yanked=true`. Unyank then
  succeeded and an independent read confirmed `yanked=false`. A final yank and
  independent read confirmed `yanked=true`, the state left for manual cleanup.
- Each of the four successful state readbacks also decoded the dependency and
  owner responses. Request scratch clearing checks passed after every successful
  driver invocation. This is evidence for this fixture, not all mutation APIs.
- Token input used guarded stdin storage, not command arguments, logs, package
  contents or credential environment variables. The temporary driver and archive
  are not release artifacts. No workspace crate was published.

The operator confirmed manual website deletion of the disposable crate on
2026-09-26. Cleanup is closed on that confirmation, not an SDK deletion call or
independent verification of registry storage removal.

The operator subsequently authorized inviting `eldryoth` as an additional owner
of this exact disposable crate. The SDK owner-add request succeeded with
`AdditionAcknowledged`, and scratch clearing checks passed. This does not prove
invitation delivery or acceptance. The original test-account owner was retained;
no owner removals, token revocation, trusted-publishing configuration or account
settings changes were performed. Invitation acceptance was not tested; the
operator deleted the crate without completing that step.

## Qualification Implementation

- Four isolated crates.io fuzz targets cover paths/query encoding, credential
  source erasure, metadata/JSON and framing-length arithmetic, continuation
  binding/progress, and anonymous download redirect provenance. Shared exercise
  functions have deterministic success, rejection and boundary tests.
- All 39 repository fuzz targets built and completed 64-run smoke campaigns.
  This is smoke evidence, not exhaustive exploration. Actual upload framing and
  streaming cancellation remain covered by the existing provider/adapter tests.
- A reproducible OpenAPI campaign detected 128 parameter, status,
  authentication and schema mutations. These are deterministic semantic probes,
  not a coverage-guided Python fuzzer.
- Archive verification built the provider package twice in independent target
  directories and compared complete archive hashes and validated members.
  Seven negative/positive archive-inspection cases test the checker itself.
  This does not substitute for Commit 22's two-clean-clone reproduction.
- Rust 1.92.0 through 1.98.1 (all 12 admitted toolchains), ten portable targets,
  native transport checks and all six package graph verifications passed.
- Fresh RustSec scans across four lockfiles and cargo-deny passed. The audit
  tool reported an index-lock warning during concurrent builds; advisory scans
  completed successfully. Live crates.io source and semantic drift checks passed.
- Full repository checks passed, including workspace default/all-feature tests,
  doctests, warning-denied Clippy, source-locked fixtures, no_std boundaries,
  line-length/security policies, release tooling and packaging.
- Documentation links, archive-checker regressions and refreshed complete SBOM
  freshness passed. The original fuzz Clippy run used `--tests`, not
  `--all-targets`; the broader claim was corrected by the remediation below.
  No production Rust source changed.

## Commit 21 Pentest Remediation

The review of `bd80a77d` identified two Low assurance findings:

- Replaced truncated text extraction with structural TOML validation of the
  complete ordered binary inventory, including exact names, paths and target
  flags. Automatic Cargo binary discovery is disabled. The validated inventory
  is also the list used by the smoke runner, avoiding separate unchecked lists.
- Fixed the existing fuzz-target Clippy warnings and made warning-denied
  `cargo clippy --locked --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings`
  mandatory in every fuzz gate mode, including normal CI's metadata mode.

Regression tests inject extra targets before/within/after the reviewed list,
remap paths, remove/reorder targets, alter flags and exercise malformed TOML.
Shell-level probes prove inventory rejection precedes Cargo, exact Clippy
arguments are used, and Clippy failure stops the gate before tests.
The review's warning fixes are confined to fuzz harnesses; SDK behavior is
unchanged. Verification passed: full `scripts/checks.sh`, all-target fuzz
Clippy, all 39 fuzz smoke campaigns, inventory/shell regressions, SBOM freshness,
documentation links, formatting and shell syntax. User retest acceptance remains
pending.

Repeat the credential-free checkpoint with:

```sh
scripts/check_cratesio_qualification.sh
```

This command never performs registry mutations. Opt-in live reads remain a
separate operator activity. Stop for incremental pentest and GitHub acceptance
before Commit 22; no tag or workspace publication is authorized.

## Hetzner Release Blocker

The extra 2026-09-26 live check failed closed: Cloud active operation count is
209 versus the locked 208. The changelog also changed, including the new
[Network members endpoint and Primary IP assignee semantics](https://docs.hetzner.cloud/changelog).
The observed Cloud schema SHA-256 is
`592b22eb5a71b960d4d4b9cd13a026f8948828bf7ca7c5d88ea73ce9259b0fd5`.
Do not refresh fingerprints alone or claim that Hetzner is current. Commit 22
must reconcile live schemas, new operations, models, deprecations, tests and
documentation before any 1.1.0 release decision.
