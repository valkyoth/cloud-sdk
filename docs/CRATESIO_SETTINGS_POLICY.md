# crates.io Settings Mutations

Commit 15 implements the two public PATCH operations in `settings`, with
`alloc` for models/permits and `blocking` for the trusted credential-adapter
callback. No dependency, default feature or transport capability is added.

## Source Contract

The [public schema](https://crates.io/api/openapi.json) and pinned controllers
at upstream `9ae7f769cea32f38ebc2ea9ec2ce455b47641511` define the contract:
[crate settings](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/krate/update.rs)
and [version settings](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/version/update.rs).

| Operation | Request | Required postcondition |
| --- | --- | --- |
| `update_crate` | PATCH `/api/v1/crates/{name}` | 200 JSON, crate ID/name and requested `trustpub_only` |
| `update_version` | PATCH `/api/v1/crates/{name}/{version}` | 200 JSON, crate, exact version including build suffix, requested yank state and exact message |

`scripts/generate_cratesio_settings.py` verifies request fields, nullable shapes,
authority, response schema and exact success status in CI and the release gate.
Version PATCH and GET use the same source schema and checked decoder. Pinned
controller bytes are verified by `scripts/check_cratesio_request_policy.py --fetch`.

## Patch Semantics

- Crate settings require an explicit boolean. No empty patch or null boolean is
  emitted. The upstream server checks owner authority and trusted-publishing
  endpoint scope and rejects legacy tokens; local consent is not that proof.
- Version settings accept `Some(bool)` or omitted `yanked` via `None`.
  The message is always an explicit `YankMessage::Set` or `Clear`.
- Upstream omission and null both clear `yank_message`, even if only the yank
  state changes. The SDK never labels either form as preserving the message.
- Set with unyank is rejected locally. Message-only Set requires an already
  yanked version upstream and a yanked postcondition. Empty Set remains an empty
  string, not null. Clear emits explicit null and can accompany an omitted state.
- Message input is at most 4096 UTF-8 bytes; controls other than CR/LF/tab are
  rejected and JSON is escaped. Scratch is bounded to 25,000 bytes and cleared.
  Borrowed source text and any caller-created copies remain caller-owned and
  require caller cleanup if treated as sensitive.
- Descriptions, URLs and archived metadata are not writable through these
  endpoints. Response metadata uses existing bounded crate/version validation;
  inert links never authorize fetches or receive credentials.

## Authority And Failure

`SettingsRequest::confirm` consumes the exact immutable request and borrows an
API token into a non-cloneable permit. This explicit confirmation includes
potentially destructive yanking and weakening trusted-publishing policy.
Execution consumes the permit, binds official production/staging origin,
method and target, and shares the process-wide admission gate. There is no
automatic retry, sleep, read-before-write, hidden preflight or mutation chaining.
Token scope/ownership is server-enforced, not inferred from response metadata.

The callback is a trusted integration boundary: it must send exactly once to
the bound executor, add sensitive Authorization exactly once, enforce the raw
policy and body limits, and disable cookies, redirects and retries. Content-Type,
Content-Encoding and Retry-After are retained for provider validation. All four
caller scratch buffers clear on every exit. Bundled authenticated and async
integration remains Commit 20, not claimed complete here.

The API exposes no revision/ETag compare-and-swap contract. Desired-state
postconditions describe only the returned snapshot; another writer can change
state immediately afterwards. Concurrent changes, ambiguous transport errors,
or rejected postconditions require out-of-band reconciliation, not replay.
The version controller can update the database before audit/index job work
fails. Even a server error may follow an applied mutation. Success does not
mean background index synchronization has completed. An already matching state
can succeed without a new upstream action; it is not proof of a fresh change.

## Verification And Stop

Tests cover exact paths/body omission/null/escaping, bounded storage, conflicts,
empty strings, no-op snapshots, mismatched crate/version/message/state, archived
metadata retention, response field bounds, status and provider errors, origin
and storage rejection, cleanup and single-send behavior. Compile-fail tests
cover permit duplication/replay. Generator regressions reject schema/auth/status
drift and stale artifacts. No live mutation is part of testing.

Incremental pentest baseline: `fae5b5a1`. Stop before Commit 16; no tag or
publication is authorized. Known Hetzner drift remains a final-1.1.0 task.

## Local Checkpoint Evidence

On 2026-09-25, `scripts/checks.sh` passed, including package verification,
workspace tests/doctests, warning-denied Clippy and security/documentation gates.
Provider all-feature tests passed (169 unit tests, two integration tests and
32 doctests); alloc-only passed (130 unit tests, two integration tests and
28 doctests), and default-feature tests passed. Rust 1.92.0 all-feature
compilation and all four SBOM freshness checks passed.

Live crates.io drift is clean. The settings generator and its adversarial
regressions passed, as did all 27 pinned implementation-source digest checks
and the settings feature-boundary regressions. No dependency manifest, lockfile,
default feature or neutral transport API changed. These are local verification
results, not independent pentest acceptance or release authorization.
