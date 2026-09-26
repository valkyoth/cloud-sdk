# crates.io Token Management

Commit 14 implements the three public API-token operations under
`accounts::tokens`, using `alloc` for protected models and `blocking` for the
trusted credential-adapter execution callback. No dependency or feature is added.

## Source Contract

The OpenAPI projection is checked by `scripts/generate_cratesio_tokens.py`
in both CI's local check gate and the final release gate. It binds operation
names, methods, paths, success statuses and body contracts. The pinned
`src/controllers/token.rs` at upstream commit
`9ae7f769cea32f38ebc2ea9ec2ce455b47641511` is verified by
`scripts/check_cratesio_request_policy.py --fetch`.

| Operation | Request | Success |
| --- | --- | --- |
| `find_api_token` | GET `/api/v1/me/tokens/{id}` | 200 JSON, `api_token.id` matches the request |
| `revoke_api_token` | DELETE `/api/v1/me/tokens/{id}` | 200 empty JSON object |
| `revoke_current_api_token` | DELETE `/api/v1/tokens/current` | 204, no body or content-type/content-encoding |

The revoke-by-ID controller does not check its affected-row count. An
acknowledgement therefore does not prove that the ID existed, belonged to the
caller, or was previously active. Lookup does not expose revocation state and
does not prove usability. The server alone checks ownership and token authority.
Internal browser-cookie token creation and listing are deliberately excluded;
the public token API must not be expanded by guessing undocumented routes.

## Authority And Execution

`TokenPermit::inspect`, `confirm_revoke`, and `confirm_revoke_current` bind one
operation and its optional ID to one borrowed API credential. Execution consumes
the permit; it cannot be cloned, replayed or retained across credential rotation.
The latter two constructors are explicit destructive confirmations. Creating a
new permit is an explicit new intent, not a retry policy. Trusted-publishing
temporary tokens are a different credential type and cannot enter these APIs.

Production and staging remain fixed, separate origins. Execution shares the
process-wide admission gate with other clients, dispatches at most once and
never sleeps, retries, rotates or revokes implicitly. Provider errors and
Retry-After metadata use the existing bounded policy. Self-revocation checks
its exact empty success contract separately; other statuses cannot become
successful revocation. The lower-level JSON decoder cannot accept self-revocation.
Both the shared JSON policy and the empty-204 policy explicitly retain
`Content-Type` and `Content-Encoding`, so a policy-honoring adapter cannot discard
metadata needed by the validators. JSON accepts only absent or identity encoding; self-revocation
rejects any encoding header, including identity.

The callback is trusted to add sensitive Authorization exactly once, enforce
raw framing/body limits, send only to the bound executor, and disable redirects,
cookies and retries. Commit 20 adds unified blocking/local/Send execution for
all three token operations, with identical JSON/empty response policies. Async
guards also clear unpolled/cancelled futures. Final bundled integration
qualification remains tracked in the [implementation ledger](CRATESIO_UNIFIED_CLIENT.md).
Credential and response scratch are cleared on success, errors and unwinding.
Process abort and caller-made copies remain outside that cleanup guarantee.

## Protected Metadata

Token names, crate patterns, timestamps and unknown fields retain protected,
redacted storage with scoped text access. Caller-made copies are caller-owned.
The source schema requires all seven metadata fields, including nullable fields;
omission never silently becomes unrestricted access. Null endpoint scopes mean
the provider's legacy scope; null crate scopes mean unrestricted crates. Empty
lists remain distinct. Scope metadata is never promoted to local authorization.

Known endpoint scopes have typed variants. Unknown variants fail for source
review. Crate patterns remain inert bounded strings, not a local authorization
matcher. Each scope list is capped at 128 entries, names at 1024 bytes and crate
patterns at 256 bytes. Oversize data fails without truncation. RFC3339 timestamps
use the existing calendar validator; no current-time or expiry decision is
inferred. Exact input spelling is preserved, including offsets and null expiry.

## Rotation Guidance

1. Provision a replacement through the provider-supported external flow.
2. Protect and validate it against the intended origin and application workload.
3. Switch callers deliberately, accounting for concurrent in-flight requests.
4. Confirm revocation using the old credential or the known old token ID.
5. Clear locally owned credentials and caller buffers when no longer needed.

A timeout or malformed response may occur after the server revoked the token.
Do not automatically retry or roll back to that credential. Reconcile through
an independently authorized channel. These steps are not an atomic transaction,
and a lookup alone cannot establish that the replacement has sufficient scopes.

## Verification And Stop

Tests cover source contracts, exact verbs/targets, ID binding, required scope
fields, invalid dates, bounds, null/empty distinctions, malformed acknowledgements,
redaction, missing destructive permits, replay, credential rotation borrows,
wrong origins, storage limits, provider errors, no-retry behavior and cleanup.
No live authenticated read or destructive request is used in these tests.

The accepted incremental baseline is `9a1f2020`. Stop for an independent pentest
of this complete increment, then wait for GitHub before Commit 15. No tag or
publication is authorized. Known Hetzner drift remains a separate final-1.1.0
qualification requirement, not a silently accepted coverage baseline.

### Local Checkpoint Evidence

On 2026-09-25, `scripts/checks.sh` passed, including workspace tests, doctests,
warning-denied Clippy, package verification and security/documentation gates.
The provider passed 161 unit tests, two integration tests and 27 doctests with
all features; alloc-only passed 126 unit tests, two integration tests and 25
doctests. All twelve supported Rust versions (1.92.0 through 1.98.1), ten
portable targets and native transport checks passed. The clock/delay helper
stays in the existing std-gated shared execution boundary.

Live crates.io drift was clean; all 25 pinned implementation sources and the
token schema/status generator verified. All four SBOMs remain fresh. No
dependency manifest or lockfile changed. These are implementation-agent checks,
not independent pentest acceptance or release authorization.

The incremental pentest found a missing header-retention instruction. The
remediation retains encoding metadata in both policies and adds a policy-honoring
adapter regression for all three token operations, case-insensitive header names,
valid controls, rejection paths and scratch cleanup. Shared discovery executor
tests also assert retention across blocking and async execution. The regression
failed before the fix; independent remediation retest is required.

Remediation verification passed: `scripts/checks.sh`, all-feature provider tests
(162 unit tests, two integration tests and 27 doctests), Rust 1.92.0 all-feature
compilation, documentation links and all four SBOM freshness checks.

A follow-up review found that `Content-Type` also needed explicit retention:
transport media validation does not imply retaining the header for provider
validation. Both policies now retain it, including JSON error responses to
self-revocation; the forbidden media policy for successful 204 responses is
unchanged. The regression adapter now filters both media headers and covers
provider-error classification for every token operation, as well as success,
encoding rejection and cleanup. This regression failed before the fix.
Independent remediation retest is required.

Follow-up verification passed: the complete `scripts/checks.sh` suite,
all-feature provider tests, Rust 1.92.0 all-feature compilation and all four
SBOM freshness checks. No dependencies or public API changed.
