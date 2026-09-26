# crates.io Public Accounts And Owners

Status: unreleased `1.1.0`, logical Commit 12 pentest and GitHub passed.
Baseline: `5c925018` (accepted Commit 11 evidence).
The separate Cargo owner-list addition belongs to Commit 20, still in progress
and not yet accepted by pentest; the historical qualification below is Commit 12.

## Executable Scope

| Operation | Request | Response |
| --- | --- | --- |
| `find_user` | `AccountRequest::user` with optional `include=linked_accounts` | `PublicUser` |
| `get_user_stats` | `AccountRequest::user_stats` with positive numeric user ID | aggregate download count |
| `find_team` | `AccountRequest::team` with qualified team login | `AccountRecord` |
| `list_owners` | `AccountRequest::owners` | mixed `OwnerList` |
| `get_user_owners` | `AccountRequest::user_owners` | user-only `OwnerList` |
| `get_team_owners` | `AccountRequest::team_owners` | team-only `OwnerList` |

All six are anonymous, read-only GETs. No body, token, cookie, mutation permit,
redirect, hidden lookup, retry, sleep or bulk crawl is introduced. The
`AccountClient` reuses the same checked GET runner as discovery, catalog,
versions and downloads: official origin and identifying user-agent binding,
shared process-wide API rate admission, exact 200/JSON success, bounded buffers,
typed provider failures, and cleanup on errors and cancellation. Blocking,
local async and Send async have the same policy. Requests remain available
without allocation; response models require `alloc`, execution is opt-in.

## Identity And Data Boundaries

The rules in this section describe the website/OpenAPI account operations.
The separately selected Cargo profile below does not relax those models.

Registry users are not linked OAuth identities. The pinned database lookup
normalizes ASCII case and maps `-` to `_`; response correlation and duplicate
user-login detection use that rule without changing stored or emitted text.
The team controller compares the exact qualified login. Team path colons are
percent-encoded; user and team routes cannot be interchanged.

IDs are positive signed-32-bit values inside separate user/team namespaces.
The same numeric ID is valid once in each namespace. Owner records must carry
the schema's exact `kind`; filtered owner endpoints reject the wrong kind.
Duplicate IDs or equivalent logins inside a kind fail rather than deduplicate.
Team duplicate login comparison is ASCII case-insensitive. Order is preserved.
The combined endpoint really uses a `users` array containing both kinds;
`owner_team` uses `teams` instead. No response implies authorization to mutate.

The statistics controller can return zero for an unknown ID; the SDK does not
interpret this as proof of existence. Counts are bounded to the source OpenAPI
nonnegative signed-64-bit domain, even though the server implementation uses
a saturating unsigned count. Count and ownership envelopes do not echo the
requested user ID/crate name. They rely on the bound transport exchange for
that association; the decoder cannot infer it from JSON alone.

Display names, avatars and profile URLs preserve permitted null values. User
creation timestamps preserve null for older/deleted linked accounts. Unknown
fields pass the same bounded parser; retained record fields expose scoped,
protected metadata access, not transport or permission capabilities. URLs and
display names must not be rendered as trusted HTML or used as fetch targets.
Diagnostic strings do not print protected text. Caller-created copies remain
the caller's cleanup responsibility.

## Bounds

- Owner lists: at most 256 entries, with bounded pairwise duplicate checks.
  Larger lists fail; they are never silently truncated or labeled complete.
- Linked accounts: at most 16, present exactly when explicitly requested;
  omitted, empty and null are distinct. Null is not a valid linked-account list.
  The admitted provider is GitHub; account IDs and case-insensitive logins
  must be unique. Linked IDs remain opaque nonempty strings, at most 256 bytes.
- Login grammar and limits reuse `UserLogin`/`TeamLogin`; names are at most
  256 bytes and avatar/profile URLs at most 4096 bytes. These fields reject
  control characters. Response URLs need not be trusted HTTPS authorities
  because they are inert metadata, not request inputs.
- All input shares the existing JSON/body, nesting, key, text, node and array
  bounds. Allocation errors remain typed. Response and header storage clears
  on success, rejection, cancellation and unpolled future drop.

## Source And Drift

### Cargo Owner-List Profile (Commit 20)

`accounts::cargo::CargoOwnersRequest` requires an explicit `ApiToken` and typed
crate name. The unified client's blocking/local/Send execution uses the same
official-origin, identifying-user-agent, rate-admission, bounded-response and
cleanup controls. The token is sent without a Bearer prefix. No query, body,
cookies, redirect, retry or extra lookup is introduced. An origin mismatch or
cleared token fails before dispatch and admission. Anonymous `AccountRequest`
behavior and the generic `CredentialContext::api` allowlist are unchanged.

This profile follows the separately admitted
[Cargo owner-list contract](https://doc.rust-lang.org/cargo/reference/registry-web-api.html#owners-list),
not the website schema: `users` contains unsigned 32-bit IDs (including zero),
required nonempty logins, and optional or null names. Login and name strings
are limited to 256 UTF-8 bytes and reject control characters. All records and
extensions share the bounded protected JSON parser; at most 256 owners are
accepted. Missing and null names remain distinguishable. Unknown fields are
inert metadata, never credentials, URLs to fetch, or authority to mutate.

Cargo does not define a user/team discriminator or login canonicalization.
No kind is guessed and no website identity is constructed from these records.
Exact duplicate logins fail. Numeric IDs alone are not de-duplicated because
crates.io user/team namespaces can overlap. Case/separator equivalence and
ownership authority must be resolved through the strict website models before
any optional ownership-change preflight; this snapshot grants no capability.
The response does not echo the crate: association relies on the bound exchange.

Independent minimal-wire tests cover all three execution modes, exact raw
authorization, credential/origin failures, media/status/encoding rejection,
short scratch, malformed JSON, exact bounds, duplicates, and cancellation.
The Cargo source lock remains separate from generated website schema fixtures.

### Website Sources

Sources: [OpenAPI](https://crates.io/api/openapi.json) and the pinned
[user controller](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/user/other.rs),
[team controller](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/team.rs),
[owner controller](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/krate/owners.rs)
and database username canonicalization tests. Their exact hashes are checked
by `scripts/check_cratesio_request_policy.py --fetch`.

`scripts/generate_cratesio_accounts.py` verifies six source-derived schema
projections and fixtures. The statistics minimum is admitted only when it is
exactly integer zero and the type/format remain integer/int64; the runtime
integer node already enforces that lower bound. Unknown constraints fail
generation. Owner fixture kinds come from the source's enum discriminants.

Live crates.io drift was clean during implementation. The previously reported
Hetzner network-members endpoint still changes the active operation count from
208 to 209. That unrelated coverage gap remains fail-closed and is not accepted
by a baseline refresh in this checkpoint.

## Verification And Stop

Regression tests cover every route and execution mode, atomic short buffers,
wrong namespaces and response identities, case/separator equivalence, filtered
owner kinds, duplicate IDs/logins, nullable and unknown fields, exact/oversized
lists and metadata, explicit include correlation, invalid statistics, provider
errors, media/status rejection, shared rate admission and cancellation cleanup.
Offline generator tests cover source identity, constraints, enum fixtures,
determinism and allocation/client feature guards.

### Local Qualification (2026-09-15)

- Full `scripts/checks.sh`: passed, including package verification, default
  and all-feature workspace tests, doctests and warning-denied Clippy.
- Provider all features: 139 unit tests, two integration tests and 19 doctests;
  alloc-only: 116 unit tests, two integration tests and 19 doctests.
- All twelve supported compiler checks, Rust 1.92.0 through 1.98.1: passed.
- All ten configured portable targets and native transport checks: passed.
  The smoke harness's 17 offline tests passed; five live tests stayed ignored.
- All four SBOM dependency graphs remain fresh. No manifests, lockfiles,
  publication flags, GitHub tooling or other provider implementation changed.
- Six account schema projections and fixtures, 19 pinned controller/source
  hashes, README regressions, documentation links, file limits and plan checks:
  passed. Live crates.io and Robot checks were clean. Hetzner Cloud/changelog
  drift remains the explicit outstanding item above, not a passing live check.

These are local implementation checks, not an independent pentest. No
credentialed probe, live mutation, tag or publication ran.

The [incremental pentest](../security/pentest/cratesio-commit-12.md) passed for
`5c925018..7d98a087`, including generator and boundary policy changes.
After that review, the full repository, compiler/platform matrices, alloc-only
tests, SBOM, audit, deny, source and documentation checks were repeated.
All passed except the explicitly outstanding Hetzner Cloud/changelog drift.
Release validation confirms candidate status and all six publication flags false.
Stop before Commit 13 until the user confirms GitHub is green.
No tag or publication is authorized; the full `1.1.0` train remains pending.
