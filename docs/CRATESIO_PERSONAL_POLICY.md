# crates.io Personal Workflow Contract

Commit 13 covers eight public operations, through `accounts::personal` in the
existing provider crate. Models and scoped serialization require `alloc`; the
one-attempt callback runner requires `blocking`. No dependency or runtime is
added to the default provider graph.

## Authority And Execution

`PersonalRequest` has action-specific constructors and immutable private data.
`confirm` consumes that exact request and borrows one API credential into a
non-cloneable permit. Execution takes only the permit, not another request or
credential. Unfollow and invitation decline require the same explicit step.
Email confirmation and tokenized invitation acceptance consume their distinct
protected credential types. No API Authorization is attached to path-token
operations. Production and staging remain isolated.

The blocking adapter callback is a trusted boundary, as with authenticated
catalog execution. It must enforce the bound origin, exact request, sensitive
authorization, response policy, TLS, deadlines, no cookies, no redirects and
no retries. Commit 20 adds unified blocking/local/Send execution for all eight
personal operations, with guards installed before future creation. The two
secret-path operations consume protected tokens without an Authorization header.
Bundled raw URI staging uses sanitization-owned bytes, not a String/URL copy.
Custom executors must never log targets or retain unprotected URI copies.
HTTP/TLS wire buffers and server/proxy access logs remain deployment boundaries;
disable sensitive wire logging and configure server-side token redaction. See the
[implementation ledger](CRATESIO_UNIFIED_CLIENT.md) for remaining gates.
All four scratch buffers are cleared on every exit, including pre-dispatch
rejections and unwinding. Caller-owned email input, adapter copies, process
abort and deliberately leaked allocations remain outside cleanup guarantees.

Every attempt uses the shared process API admission gate. Rate-limit response
delays are bounded to 24 hours; no call sleeps or retries automatically.
Transport failure can follow a completed upstream mutation. Reconciliation and
any new permit are caller decisions. Follow/unfollow are state-idempotent in
the pinned controller, not blanket retry-safe under concurrent user intent.

## Source Semantics

The eight response and two OpenAPI-documented body projections are verified by
`python3 scripts/generate_cratesio_personal.py`. Five additional controllers at
the reviewed crates.io source commit are hash-checked by
`python3 scripts/check_cratesio_request_policy.py --fetch`.

- API tokens identify the current user upstream. SDK syntax and a supplied user
  ID do not prove identity, scopes, expiry or account ownership. User update and
  verification resend are rejected upstream when the path ID differs.
- Invitation handling uses the JSON crate ID upstream. The SDK writes the same
  typed ID into path and body and requires that ID and decision in the response.
- A secret invitation token itself selects the upstream crate. The expected
  crate ID is a response consistency check, not preflight authorization; it
  cannot undo the upstream action. Obtain both from the same trusted invitation.
- Repeated/expired invitation failures remain provider errors. Email confirmation
  may succeed repeatedly in the pinned controller; the SDK makes no claim that
  provider tokens are universally single-use or locally verifiable.
- Email updates can follow notification updates in the upstream controller,
  including when the email update then fails. The SDK deliberately exposes one
  user setting per request, with no combined update or null/no-op constructor.
  An acknowledgement means neither email delivery nor verified-email state.
- The deprecated notification endpoint's public OpenAPI omits its required body.
  The pinned controller accepts `[{"id":42,"email_notifications":true}]` and
  ignores crates not owned by the authenticated user. SDK batches contain 1-64
  unique crate IDs; success cannot prove every requested row changed. This
  experimental feature never became a complete notification service upstream.

JSON is bounded, duplicate keys and error envelopes fail closed, `ok` must be
true, and invitation decisions must match. Protected response metadata is never
logged. Inert callback data must not become a URL or new credential authority.

## Verification And Stop

Tests cover exact verbs/paths/bodies, schema projections, wrong crate/decision,
wrong-origin refusal, server wrong-user/expired/repeated-use errors, conflicting
notification batches, email escaping/redaction, acknowledgement types, transport
failure, scratch exhaustion, unwinding and response cleanup. No live mutation is
run. Independent incremental pentest and green GitHub are required before
Commit 14. This is not a release or whole-service production qualification.

Both `scripts/checks.sh` (including CI) and the final release gate run the real
personal-schema generator against the digest-locked OpenAPI source. A stale
committed table fails without rewriting it. Offline regression fixtures also
exercise stale/weakened tables and verify both gate entry points.

Dependency review requires separate rows for each change in all four lockfiles;
evidence from another graph cannot satisfy a row. Rows bind complete package
records using SHA-256, including source, checksum and dependency edges even
when the package version is unchanged. Duplicate identities fail closed.
Rustix and policy-checker
admission versions/checksums are checked against exact manifest pins and locks.
These controls address the Commit 13 pentest findings and the follow-up
same-version dependency-identity finding; independent retest
is still required before advancing.
Remediation verification on 2026-09-24 passed the complete `scripts/checks.sh`
suite and all four SBOM freshness checks. No runtime Rust, dependency manifest,
or lockfile changed during this remediation.

### Local Checkpoint Evidence

On 2026-09-24 the full `scripts/checks.sh` suite passed, including workspace
default/all-feature tests, doctests, warning-denied Clippy, packages, documentation,
source boundaries and release-train checks. Provider tests include 153 unit
tests, two integration tests and 22 doctests with all features; the alloc-only
configuration passed 123 unit tests, two integration tests and 22 doctests.
The 12-version Rust matrix (1.92.0 through 1.98.1), ten portable targets and
host-native checks passed. All 35 fuzz targets passed 64-run smoke campaigns
on nightly-2026-09-24; these campaigns are not exhaustive security evidence.

All four final SBOMs match their lockfiles. Fresh RustSec scans passed for all
four workspaces. Cargo Deny passed the root policy and the CI-specified
auxiliary advisory/license/source checks; the feature-unification fixture's
known Apple core-foundation duplication is not a newly admitted root exception.
Direct dependency freshness now covers the root and three isolated workspaces.
Rust stable 1.98.1, rustup 1.29.1, checkout 7.0.1 (including its exact SHA), and
the pinned Cargo security tools were verified current against upstream sources.

Live crates.io drift and 24 pinned implementation-source checks passed. Robot
remains clean at 89 active and 16 deprecated operations. Hetzner Cloud still
fails its live drift check: 209 active operations versus 208 admitted, including
the network-members endpoint. Its current Cloud digest is
`592b22eb5a71b960d4d4b9cd13a026f8948828bf7ca7c5d88ea73ce9259b0fd5`.
The changelog also differs (latest entry: 2026-09-23 primary-IP `unassigned`);
that assignment behavior already has model validation and regression coverage.
The Hetzner baselines were not advanced to conceal the missing operation.

No live mutation, tag, publication, or independent pentest is claimed by this
local evidence. The incremental review baseline is `7d4c147b`.
