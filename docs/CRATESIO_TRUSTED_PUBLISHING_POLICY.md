# crates.io Trusted Publishing Policy

Commit 19 implements all eight source-locked operations: GitHub and GitLab
configuration list/create/delete, OIDC exchange and temporary-token revocation.
This is an unreleased 1.1.0 checkpoint, not a completed unified client.

## Source And Execution

`scripts/generate_cratesio_trusted_publishing.py` checks the pinned public
OpenAPI and 15 controller, validation, claim, workflow and token source digests
at upstream commit `9ae7f769cea32f38ebc2ea9ec2ce455b47641511`. It produces exact
request contracts, response schema nodes and non-secret configuration fixtures.
Both repository and candidate gates run it and its mutation regressions.
Live upstream drift remains the separate [source-lock process](CRATESIO_SOURCE_LOCK.md).

The default graph stays no_std and allocation-free. Models, protected JSON and
credentials require `alloc`; the single-attempt client requires `blocking`.
The existing trusted callback must enforce TLS, fixed origin, identifying user
agent, raw response bounds/media, deadlines and no cookies/redirects/retries.
Commit 20 adds unified blocking/local/Send execution for these permits, preserving
OIDC preflight and strict empty acknowledgements. Guards precede future creation;
owned temporary credentials drop with the future. Complete bundled qualification
remains in the [implementation ledger](CRATESIO_UNIFIED_CLIENT.md). CI uses fixtures,
never real assertion exchange, configuration mutation, publication or revocation.

Config routes use an API token (no cookie support). Exchange is anonymous JSON:
the assertion is in `jwt`, never Authorization. Revoke uses the owned temporary
token as Bearer authorization. Production and staging credentials cannot cross.
Every action consumes a permit; there is no automatic retry or replay after
ambiguous transport failure. Response staging and all credential/body scratch
are erased on success, error and unwind, not process abort.

## Configuration Intent

Create binds crate, provider, owner/namespace, repository/project, workflow and
optional environment. Positive provider-qualified IDs require explicit delete
confirmation. Delete has no crate echo; upstream verifies ownership. A 204 with
any body, Content-Type or Content-Encoding is rejected. Creation responses must
match the intent (GitHub owner normalization permits ASCII case differences).
Read metadata grants no mutation permission. List requires exactly one crate or
user filter, with seek-only pagination, at most 100 entries and checked links
preserving all filters. User-filter ownership is server-checked, not echoed.
Duplicate IDs or identical configurations fail; page totals are snapshots.

Names and paths are limited to 255 bytes. The conservative local profile rejects
wildcards, templates, controls, dot traversal and empty path components. GitHub
workflow filenames cannot contain a slash; GitLab workflow paths must be relative.
GitLab namespace components are individually checked. This is stricter than
some upstream validation: unusual Unicode filenames/environment folding and
template-like names are not supported. `environment: None` is explicitly broad:
upstream permits any environment, not only absence of an environment.

## Unverified Assertion Preflight

Protected compact JWT ingress is capped at 16,384 bytes. Decoding uses the exact
already-admitted base64-ng constant-work scalar API with one reused 12,288-byte,
fallibly allocated `SecretBoxBytes` scratch buffer. It is cleared before each
decode and volatile-cleared on drop, including errors and unwinding. No JWT-sized
stack array is used. Scratch allocation failure returns `Allocation`. Together
with protected JSON allocations this still requires an allocator; no_std does
not imply that arbitrary embedded stack or heap budgets are sufficient.
Decoded parts undergo bounded duplicate-rejecting JSON parsing. This does not make JSON parsing or the
entire preflight constant-time. Passing preflight does NOT authenticate a JWT.

Preflight requires RS256, a key ID, exact caller-selected audience and the fixed
GitHub Actions or GitLab.com issuer; issued/expiry/not-before times; required
run/namespace/repository IDs, replay ID and commit; and matching repository,
workflow and optional environment. Unsupported critical/JWS encoding headers
are rejected. GitHub pull_request_target and workflow_run are rejected. The
workflow profile requires the configured repository/path, excluding external
reusable workflows. GitLab self-hosted issuers are not admitted.

Only crates.io validates signatures, key provenance, issuer authority, replay,
repository/namespace numeric identity, ownership and scope. Local names and IDs
in unsigned claims are not proof. No JWKS fetching or JWT verification is added.
The trusted clock comes from the application for model-only calls, and system
wall time immediately before/after exchange for the blocking client. Incorrect
host time remains an integration risk; local preflight is stricter than upstream
clock leeway and can reject otherwise valid tokens near boundaries.

## Temporary Credentials

The upstream response contains only `token`. The server currently sets a
30-minute lifetime and may authorize every crate matching its configurations.
The SDK does not invent authenticated expiry/scope fields. A local lifetime of
1..1800 seconds begins when ExchangePolicy is created, not on delayed receipt;
receipt outside that window or before the starting time fails. TemporaryToken
retains a local intended-crate binding and moves the protected JSON allocation
into credential storage without making another ordinary String copy. Token
prefix/length/checksum validation is format checking, not cryptographic proof.

`confirm_publish` requires matching metadata and fresh trusted Unix time. The
returned low-level publish permit has no clock: dispatch immediately and let the
registry enforce actual expiry. Do not queue it past the local deadline. The
existing raw TrustedPublishingToken/PublishRequest APIs remain lower-level
adapter facilities; their users must enforce time and intended scope themselves.

TemporaryToken has no Clone or raw getter. Explicit clear erases locally only.
Revocation consumes it and erases local storage even if the response fails;
remote failure therefore needs operator reconciliation, not an automatic retry.
Revocation is idempotent upstream but 204 does not prove prior existence.
Callers must clear their original source storage and qualify transport cleanup,
resource limits, clocks and process-abort handling. See [the threat model](threat-model.md).

## Checkpoint Verification

The checkpoint has 16 regular regression tests and one optimized stack test,
including the eight wire operations,
GitLab exchange/publish/revoke composition, unverified claim confusion,
configuration binding and duplicate rejection, checked continuation, expiry,
wrong origins, short buffers, redaction, failure/unwind cleanup and shared-gate
deferral. Tests use synthetic assertions, not valid signed tokens or network
mutation. Eight source-contract regression groups reject changed authority,
media/status, inherited parameters and stale request evidence.

Commit 19 pentest remediation replaces per-part stack arrays with the shared
protected allocation. Both check gates run optimized preflight on a requested
32 KiB thread stack for GitHub/GitLab, including large claims and near-limit
signatures. This is host regression evidence, not universal stack qualification.
An injected scratch-allocation failure must return Allocation. The duplicate-key
regression now duplicates a valid issuer in a completely valid claims object:
both the original and an independent last-value parser's result pass the control,
while the duplicate document must fail specifically at JSON validation.

`scripts/checks.sh` passes with full workspace tests, doctests, feature-isolated
Clippy and package verification. MSRV 1.92.0 all-feature compilation and alloc
regressions, `thumbv7em-none-eabi` alloc compilation, advisory/license checks and
all four SBOM completeness/freshness checks also pass. The known verifier
freshness follow-up remains explicitly assigned to Commit 20. Incremental
pentest is still required against `ddb12f74`; stop before Commit 20.
