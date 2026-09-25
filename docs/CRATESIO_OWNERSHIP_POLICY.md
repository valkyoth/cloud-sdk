# crates.io Ownership Mutations

Commit 16 implements `add_owners` (PUT) and `remove_owners` (DELETE) at
`/api/v1/crates/{name}/owners`. Both send JSON, require an API-token permit and
accept only exact 200 with `ok: true` and a bounded string `msg`. HTTP 200 with
a Cargo error envelope fails, as do false, missing or malformed success fields.
The API does not echo crate identity or per-owner results; the trusted transport
exchange establishes association, not parsing the provider's human-readable text.

## Source Evidence

The [public OpenAPI](https://crates.io/api/openapi.json) describes `owners`;
the [Cargo contract](https://doc.rust-lang.org/cargo/reference/registry-web-api.html#owners)
uses `users`. The SDK emits only `users`, an explicitly supported controller
alias, never both fields. `scripts/check_cratesio_ownership.py` checks both
locked sources and a supplemental
[controller pin](https://github.com/rust-lang/crates.io/blob/8ee3e10d68792af6a8caf362f1ab19cd1a5e7f26/src/controllers/krate/owners.rs).

The OpenAPI already documents `crates.io:user` and `github:user`; the older
`9ae7f769` controller source does not. The supplemental `8ee3e10d` pin is checked
by exact SHA-256 and size in CI/release checks and supplies current namespace
and ambiguity semantics. It does not silently rewrite unrelated baseline pins.
The new checker rejects changed request, response, status or authentication
contracts. Offline regressions verify it and its required gate integration.

## Identity And Bounds

`OwnerSelector` distinguishes unqualified usernames, explicit registry users,
GitHub users and GitHub teams. Exact validated text is serialized; prefixes are
not guessed. Unqualified names are resolved by the server and may be rejected
as ambiguous. Explicit prefixes are preferable when the intended namespace is
known. Team membership and endpoint/crate token scopes remain server-enforced.

Batches contain 1-10 selectors; ten is the source addition ceiling and a local
conservative removal ceiling. Identity lengths reuse bounded user/team types.
Duplicate team spellings are rejected case-insensitively. User spellings are
conservatively compared case-insensitively with hyphen/underscore folding,
including across user namespaces. This may reject two different accounts with
similar spellings; it does not prove that different aliases cannot resolve to
the same upstream account. No automatic splitting, retry or deduplication occurs.
JSON storage is at most 4096 bytes; messages are at most 8192 UTF-8 bytes.

## Consent And Preflight

Requests are immutable and bind one exact crate and owner batch. `confirm_add`
cannot authorize a removal. `confirm_removal` explicitly acknowledges destructive
intent. Both produce non-cloneable, consumed permits borrowing one credential.
Fixed production/staging endpoints, credential binding, process-wide admission,
header retention and bounded response admission reuse the existing client path.
All four caller scratch buffers clear on every exit. Borrowed input identities
and caller-created copies remain caller-owned.

`confirm_removal_after_preflight` additionally checks a caller-supplied
`RemovalSnapshot::from_complete_active_list`, the acting registry username and
explicit `SelfRemoval` policy. It rejects wrong-crate snapshots, unknown targets,
duplicate identities, absent actors, removal of all remaining individual owners
and self-removal unless explicitly allowed. Teams and pending invitations never
substitute for remaining individual owners.

Snapshots must come from a complete active owner list associated with the trusted
list exchange for that crate. Construction is a caller assertion, not a fabricated
provenance guarantee. User entries mean registry usernames. Preflight refuses
unqualified/GitHub user selectors because this snapshot lacks linked-account
resolution evidence; callers can use explicit registry selectors or separately
review and confirm removal without this optional preflight. Snapshots may be
stale, and no revision/CAS API is provided. The server must recheck permissions
and the last-individual-owner rule; preflight neither authorizes nor locks state.

## Acknowledgements And Failure

`AdditionAcknowledged` may describe newly invited users, already-pending invites
or immediate team addition. It never claims that a user accepted, that email was
delivered, or that every message fragment is a machine-readable outcome.
`RemovalAcknowledged` is not a refreshed owner list. Messages remain protected,
redacted and accessible through scoped callbacks as inert untrusted prose.

The pinned controller uses a database transaction for each batch and checks for
at least one individual owner after removal. Email is sent best-effort after
addition commits; external GitHub/notification work is not a global transaction.
Transport errors can follow committed changes. The SDK never retries, chains a
follow-up mutation or infers rollback; reconcile the entire batch out of band
before issuing fresh consent after an ambiguous result.

The blocking callback remains an explicitly trusted credential adapter: enforce
the supplied policy, send once to the bound executor, add Authorization exactly
once as sensitive, and disable redirects/cookies/retries. Bundled authenticated
and async integration remains Commit 20. No live owner mutation is performed by
the test suite.

## Verification And Stop

Regressions cover exact Cargo wire bodies, both methods, namespace grammar,
empty/duplicate/oversized batches, permission-kind confusion, self/last-owner
preflight, wrong-crate binding, pending-invite prose, malformed acknowledgements,
200-with-errors, media/status rejection, partial writes, gate/origin/capacity
rejection and scratch cleanup. Compile-fail tests prohibit permit cloning,
replay and unconfirmed execution.

Incremental pentest baseline: `42e534bd`. Stop before Commit 17; no tag or
publication is authorized. Known Hetzner drift remains final-1.1.0 work.

## Local Checkpoint Evidence

On 2026-09-25, `scripts/checks.sh` passed, including package verification,
workspace tests/doctests, warning-denied Clippy and security/documentation gates.
Provider all-feature tests passed (175 unit tests, two integration tests and
36 doctests); alloc-only passed (134 unit tests, two integration tests and
30 doctests). Default-feature tests and Rust 1.92.0 all-feature compilation
also passed. All four SBOM freshness checks passed.

Live crates.io drift was clean, the 27 existing pinned implementation sources
verified, and the supplemental ownership-controller digest and OpenAPI/Cargo
contract check passed. Offline schema/gate and feature-boundary regressions
passed. No dependency manifest, lockfile, feature or neutral transport API
changed. These are implementation-agent results, not independent pentest
acceptance or release authorization.
