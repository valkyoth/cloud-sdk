# crates.io Yank And Unyank Contract

Commit 17 of the unreleased 1.1.0 train. Incremental pentest baseline:
`9fabe832`. Stop before Commit 18; no tag or publication is authorized.

## Source Contract

The [Cargo Web API](https://doc.rust-lang.org/cargo/reference/registry-web-api.html#yank)
and pinned OpenAPI describe bodyless DELETE `/api/v1/crates/{name}/{version}/yank`
and PUT `/api/v1/crates/{name}/{version}/unyank`. Both require authorization and
Accept JSON. The admitted crates.io success status is exactly 200 with boolean
`ok: true`, not an echoed resource or version snapshot. Cargo error envelopes
take precedence even when the status is 200.

The [yank controller](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/version/yank.rs)
and shared [version update implementation](https://github.com/rust-lang/crates.io/blob/9ae7f769cea32f38ebc2ea9ec2ce455b47641511/src/controllers/version/update.rs)
source-lock the semantics. The controller invokes the shared update with the
desired yank flag and no message: **both Cargo operations clear any existing
yank message**, including a repeated yank. They do not delete an archive or
invalidate existing lockfiles. Use `settings::SettingsRequest` for explicit
yank-message edits instead.

`python3 scripts/check_cratesio_yank.py` checks source digests, both verbs,
identities, authentication, parameters, absence of bodies, status and response
schemas. It runs in local and release gates with offline rejection regressions.
The request-policy source inventory now includes the yank controller. Existing
drift checking remains responsible for discovering changes to live contracts.

## Authority And Replay

`YankRequest` binds validated crate and exact SemVer, including build metadata.
Atomic percent encoding preserves the caller's buffer on insufficient capacity.
`confirm` creates a non-cloneable `YankPermit` borrowing one API token. This is
caller consent, not upstream ownership or token-scope verification. The server
enforces crate ownership and yank permission. No cookie authentication is added.

`YankClient` uses the official production/staging authority and shared request
gate. Wrong-origin credentials fail before send. The trusted callback must
honor the raw response policy, attach sensitive Authorization exactly once,
send through the supplied bound executor, and disable redirects/cookies/retries.
Credential, response-body and header scratch are cleared on every exit. No
request-body scratch is needed because these Cargo requests are bodyless.
Bundled authenticated and async integration remains Commit 20.

Both operations are state-convergent only in the absence of intervening writes.
Neither has compare-and-swap semantics, an idempotency key, or automatic retry.
Database changes can precede audit/index-job failures. Timeouts, malformed
acknowledgements and errors therefore leave mutation outcome uncertain. Obtain
fresh evidence and fresh consent before any new write, rather than undoing
another operator's intervening choice with a blind replay.

## Acknowledgement And Observation

An acknowledgement carries the original target and authority but no credential.
It does not claim a changed state (a repeated action can already be satisfied),
index propagation, or durable exclusivity. No response IDs are echoed, so its
association to the request depends on the trusted exchange.

The caller explicitly sends `verification_request()` to
`verification_endpoint()`, anonymously and subject to the shared admission gate.
Pass the wire-admitted exact-200 JSON result to `decode_observed_state()`.
The existing version decoder checks crate and exact version identities;
`YankObservation::observed_yanked()` exposes the actual snapshot value and
`matches_requested_state()` exposes disagreement rather than silently claiming
confirmation. Stale reads or concurrent writes can explain disagreement.
Even a matching snapshot does not prove index synchronization or future state.
No hidden polling, sleeps, retries, index reads or live mutations are introduced.

## Verification And Stop

Tests cover both exact Cargo verbs/paths, body and content-type absence,
prerelease/build metadata, malformed versions, atomic target bounds, consumed
permits and credential borrows, repeated intents, contradictory and duplicate
`ok`, 200-with-errors, unexpected status/media/encoding, uncommitted and partial
responses, timeout-after-send behavior, origin/gate/capacity rejection, cleanup,
wrong crate/version read-back, and both matching and stale state observations.
No dependency, feature or neutral transport contract changes are needed.

Independent incremental pentest and GitHub acceptance are required before
Commit 18. This implementation does not constitute complete 1.1.0 qualification.

## Local Checkpoint Evidence

On 2026-09-25, `scripts/checks.sh` passed, including package verification,
workspace tests/doctests, warning-denied Clippy and isolated provider feature
checks. The provider all-feature suite passed 180 unit tests, two integration
tests and 41 doctests; alloc-only passed 137 unit tests, two integration tests
and 33 doctests. Rust 1.92.0 all-feature compilation passed.

All four SBOM graphs remained fresh. Live crates.io drift was clean, all 28
pinned request-policy implementation sources verified, and the dedicated yank
Cargo/OpenAPI/controller checker passed. Offline schema, feature-boundary,
documentation-link and formatting checks passed. These are implementation-agent
results, not independent pentest acceptance. No live mutation was performed.

## Commit 17 Remediation

The incremental review found a credential-path mismatch for percent-encoded
build metadata and contradictory release-note stop boundaries. Both the policy
regression and full authenticated execution regression failed on `97bf914e`
before remediation. Version segments now decode only canonical `%2B` into a
bounded buffer and pass the existing SemVer validator. Other percent escapes,
double encoding, invalid versions and decoded lengths above 150 bytes fail;
non-version segment authorization is unchanged.

Tests cover PATCH settings authorization as well as DELETE yank and PUT unyank,
maximum/oversized versions, and build-metadata execution through every existing
success/failure scenario with exactly one callback and scratch cleanup. Release
notes now have only the `9fabe832` baseline and Commit 18 stop for this increment;
the yank regression script rejects the contradictory historical text.
Independent remediation retest is required before Commit 18.

Remediation verification passed: full `scripts/checks.sh`, warning-denied
provider Clippy, all-feature provider tests (181 unit tests, two integration
tests and 41 doctests), alloc-only tests (138 unit tests, two integration tests
and 33 doctests), and Rust 1.92.0 all-feature compilation. The full gate also
confirmed isolated feature checks and package verification after replacing the
new helper's cursor increment with checked arithmetic.
